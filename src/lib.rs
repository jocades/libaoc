use std::{
    env,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
    process,
};

use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use reqwest::redirect::Policy;
use scraper::{Html, Selector};
use tracing::{error, info, warn};

pub const AOC_URL: &str = "https://adventofcode.com";
pub const AUTH_VAR: &str = "AOC_AUTH_TOKEN";
pub const CACHE_PATH: &str = ".cache/aoc";

/// A `(year, day)` pair to identify a puzzle.
pub type PuzzleId = (u32, u32);

fn home_dir() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap_or_else(|e| {
        error!(cause = %e, "HOME");
        process::exit(1);
    }))
}

/// The `Advent of Code` client handles puzzle retrieval and cache.
pub struct Client {
    http: reqwest::blocking::Client,
    cache: Cache,
}

impl Client {
    pub fn new() -> Result<Self> {
        let token = env::var(AUTH_VAR).unwrap_or_else(|e| {
            error!(cause = %e, AUTH_VAR);
            process::exit(1);
        });

        let mut headers = HeaderMap::new();
        headers.insert("cookie", format!("session={token}").parse()?);
        Ok(Self {
            http: reqwest::blocking::Client::builder()
                .user_agent("libaoc.rs")
                .default_headers(headers)
                .redirect(Policy::none())
                .build()?,
            cache: Cache::new(home_dir().join(CACHE_PATH))?,
        })
    }

    /// Get a puzzle from cache or by scraping the website if not found.
    pub fn get_puzzle(&self, id: &PuzzleId) -> Result<Puzzle> {
        if let Some(puzzle) = self.cache.get(id) {
            return Ok(puzzle);
        }
        self.download_puzzle(id)
    }

    /// Scrape a puzzle's questions and answers.
    pub fn scrape_puzzle(&self, id: &PuzzleId) -> Result<Puzzle> {
        let html = self
            .http
            .get(self.mkurl(id))
            .send()?
            .error_for_status()?
            .text()?;

        let doc = Html::parse_document(&html);
        let query = Selector::parse("article.day-desc").unwrap();
        let mut questions = doc.select(&query);
        let q1 = questions
            .next()
            .and_then(|el| html2text::from_read(el.inner_html().as_bytes(), 80).ok());
        let q2 = questions
            .next()
            .and_then(|el| html2text::from_read(el.inner_html().as_bytes(), 80).ok());

        let query = Selector::parse("article.day-desc + p code").unwrap();
        let mut answers = doc.select(&query);
        let a1 = answers.next().map(|el| el.text().collect::<String>());
        let a2 = answers.next().map(|el| el.text().collect::<String>());

        Ok(Puzzle {
            id: *id,
            q1,
            q2,
            a1,
            a2,
        })
    }

    /// Scrape a puzzle and save it in cache.
    pub fn download_puzzle(&self, id: &PuzzleId) -> Result<Puzzle> {
        let puzzle = self.scrape_puzzle(id)?;
        self.cache.insert(id, &puzzle);
        Ok(puzzle)
    }

    /// Get the puzzle's input from cache or by requesting the server.
    pub fn get_input(self, id: &PuzzleId) -> Result<String> {
        if let Some(input) = self.cache.get_input(id) {
            return Ok(input);
        }
        let input = self
            .http
            .get(format!("{}/input", self.mkurl(id)))
            .send()?
            .error_for_status()?
            .text()?;
        self.cache.insert_input(id, &input);
        Ok(input)
    }

    /// Submit a puzzle's answer for a specific part.
    pub fn submit(
        &self,
        id: &PuzzleId,
        part: Option<u32>,
        answer: impl AsRef<str>,
    ) -> Result<Option<Puzzle>> {
        // TODO: Check for answers in cache to be able to submit once the puzzle
        // is finished.
        let path = self.cache.mkpath(id);
        let part = part.unwrap_or_else(|| {
            if fs::metadata(path.join("a1")).is_ok_and(|m| m.len() > 0) {
                2
            } else {
                1
            }
        });

        let html = self
            .http
            .post(format!("{}/answer", self.mkurl(id)))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(format!("level={part}&answer={}", answer.as_ref()))
            .send()?
            .error_for_status()?
            .text()?;

        Ok(if html.contains("That's the right answer") {
            info!("Correct!");
            Some(self.download_puzzle(id)?)
        } else if html.contains("That's not the right answer") {
            error!("Incorrect!");
            None
        } else if html.contains("You gave an answer too recently") {
            warn!("Wait!");
            None
        } else {
            error!("Unknown response");
            None
        })
    }

    fn mkurl(&self, (y, d): &PuzzleId) -> String {
        format!("{AOC_URL}/{y}/day/{d}")
    }
}

/// A file system cache for the puzzles.
struct Cache {
    path: PathBuf,
}

impl Cache {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path).context("mkdir cache")?;
        }
        Ok(Self { path: path.into() })
    }

    pub fn get(&self, id: &PuzzleId) -> Option<Puzzle> {
        let path = self.mkpath(id);
        path.exists().then(|| Puzzle::read(path, id))
    }

    pub fn get_input(&self, id: &PuzzleId) -> Option<String> {
        let path = self.mkpath(id).join("input");
        path.exists().then(|| fs::read_to_string(path).unwrap())
    }

    #[allow(dead_code)]
    pub fn get_answers(&self, id: &PuzzleId) -> (Option<String>, Option<String>) {
        let path = self.mkpath(id);
        (
            fs::read_to_string(path.join("a1")).ok(),
            fs::read_to_string(path.join("a2")).ok(),
        )
    }

    pub fn insert(&self, id: &PuzzleId, puzzle: &Puzzle) {
        puzzle
            .write(self.mkpath(id))
            .unwrap_or_else(|_| warn!("failed to insert puzzle"));
    }

    pub fn insert_input(&self, id: &PuzzleId, input: &str) {
        fs::write(self.mkpath(id).join("input"), input)
            .unwrap_or_else(|_| warn!("failed to insert input"));
    }

    #[allow(dead_code)]
    pub fn update_answer(&self, id: &PuzzleId, part: u32, answer: &str) {
        fs::write(self.mkpath(id).join(format!("a{part}")), answer)
            .unwrap_or_else(|_| warn!("failed to update answer"));
    }

    fn mkpath(&self, (y, d): &PuzzleId) -> PathBuf {
        self.path.join(format!("{y}/{d}"))
    }
}

#[derive(Debug, Default, Clone)]
pub struct Puzzle {
    pub id: PuzzleId,
    pub q1: Option<String>,
    pub q2: Option<String>,
    pub a1: Option<String>,
    pub a2: Option<String>,
}

impl Puzzle {
    pub fn read(path: impl AsRef<Path>, id: &PuzzleId) -> Puzzle {
        let path = path.as_ref();
        Puzzle {
            id: *id,
            q1: fs::read_to_string(path.join("q1")).ok(),
            q2: fs::read_to_string(path.join("q2")).ok(),
            a1: fs::read_to_string(path.join("a1")).ok(),
            a2: fs::read_to_string(path.join("a2")).ok(),
        }
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        fs::create_dir_all(path)?;
        if let Some(q) = &self.q1 {
            fs::write(path.join("q1"), q.as_bytes())?;
        }
        if let Some(q) = &self.q2 {
            fs::write(path.join("q2"), q.as_bytes())?;
        }
        if let Some(a) = &self.a1 {
            fs::write(path.join("a1"), a.as_bytes())?;
        }
        if let Some(a) = &self.a2 {
            fs::write(path.join("a2"), a.as_bytes())?;
        }
        Ok(())
    }

    pub fn view(&self, show_answers: bool) -> String {
        let mut buf = String::new();
        if let Some(q1) = &self.q1 {
            let _ = writeln!(&mut buf, "{q1}");
        }
        if show_answers {
            if let Some(a1) = &self.a1 {
                let _ = writeln!(&mut buf, "Answer: {a1}.");
            }
        }
        if let Some(q2) = &self.q2 {
            let _ = writeln!(&mut buf, "\n{q2}");
        }
        if show_answers {
            if let Some(a2) = &self.a2 {
                let _ = writeln!(&mut buf, "Answer: {a2}.");
            }
        }
        buf
    }

    pub fn write_view(&self, path: impl AsRef<Path>) -> Result<()> {
        Ok(fs::write(path, self.view(true))?)
    }
}

/// Determine the puzzle's year and day from a path.
pub fn puzzle_id_from_path(path: impl AsRef<Path>) -> Option<PuzzleId> {
    let mut day = 0xff;
    let mut year = 0;
    for parent in path.as_ref().ancestors() {
        let mut chars = parent
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .peekable();
        let mut buf = String::new();
        while let Some(c) = chars.next() {
            if c.is_ascii_digit() {
                buf.push(c);
                if !chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                    break;
                }
            }
        }
        if !buf.is_empty() {
            if day == 0xff {
                day = buf.parse().unwrap();
            } else {
                year = buf.parse().unwrap();
            }
        }
        if year > 0 {
            return Some((year, day));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_path() {
        assert_eq!(
            puzzle_id_from_path("/Users/j0rdi/aoc/2015/d01"),
            Some((2015, 1))
        );
        assert_eq!(
            puzzle_id_from_path("/home/j0rdi/aoc/2024/25"),
            Some((2024, 25))
        );
        assert_eq!(
            puzzle_id_from_path("/Users/j0rdi/aoc/2017/other/d8"),
            Some((2017, 8))
        );
        assert_eq!(
            puzzle_id_from_path("/home/j0rdi/aoc/2017/other/08/sub"),
            Some((2017, 8))
        );
    }
}
