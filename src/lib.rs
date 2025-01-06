use std::{
    env,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
    process,
};

use anyhow::{Context, Result};
use reqwest::{header::HeaderMap, redirect::Policy};
use scraper::{Html, Selector};
use tracing::{error, warn};

pub const AOC_URL: &str = "https://adventofcode.com";
pub const AUTH_VAR: &str = "AOC_AUTH_TOKEN";
pub const CACHE_PATH: &str = ".cache/aoc";

/// A `(year, day)` pair to identify a puzzle.
pub type PuzzleId = (u16, u8);

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

    /// Scrape a puzzle and store in cache.
    pub fn download_puzzle(&self, id: &PuzzleId) -> Result<Puzzle> {
        let puzzle = self.scrape_puzzle(id)?;
        self.cache.insert(id, &puzzle);
        Ok(puzzle)
    }

    /// Get the puzzle's input from cache or by requesting the server.
    pub fn get_input(&self, id: &PuzzleId) -> Result<String> {
        if let Some(input) = self.cache.get_input(id) {
            return Ok(input);
        }
        self.download_input(id)
    }

    /// Retrieve the puzzle's input from the server and cache it.
    pub fn download_input(&self, id: &PuzzleId) -> Result<String> {
        let input = self
            .http
            .get(format!("{}/input", self.mkurl(id)))
            .send()?
            .error_for_status()?
            .text()?;
        self.cache.insert_input(id, &input);
        Ok(input)
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

    /// Submit a puzzle's answer for a specific part.
    pub fn submit(
        &self,
        id: &PuzzleId,
        part: Option<u8>,
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
            .body(format!("level={}&answer={}", part, answer.as_ref()))
            .send()?
            .error_for_status()?
            .text()?;

        match self.submission_outcome(&html) {
            Submit::Correct => {
                println!("Correct!");
                return Ok(Some(self.download_puzzle(id)?));
            }
            Submit::Incorrect => println!("Incorrect!"),
            Submit::Wait => println!("Wait!"),
            Submit::Error => println!("Unknown response"),
        };
        Ok(None)
    }

    fn submission_outcome(&self, response: &str) -> Submit {
        if response.contains("That's the right answer") {
            Submit::Correct
        } else if response.contains("That's not the right answer") {
            Submit::Incorrect
        } else if response.contains("You gave an answer too recently") {
            Submit::Wait
        } else {
            Submit::Error
        }
    }

    fn mkurl(&self, (y, d): &PuzzleId) -> String {
        format!("{AOC_URL}/{y}/day/{d}")
    }
}

/// The outcome of a puzzle submission.
pub enum Submit {
    Correct,
    Incorrect,
    Wait,
    Error,
}

/// File system cache to store downloaded puzzles.
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
        let path = self.mkpath(id).join("in");
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
        fs::write(self.mkpath(id).join("in"), input)
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
            if show_answers {
                if let Some(a1) = &self.a1 {
                    let _ = writeln!(&mut buf, "**Answer**: `{a1}`.");
                }
            }
        }
        if let Some(q2) = &self.q2 {
            let _ = writeln!(&mut buf, "\n{q2}");
            if show_answers {
                if let Some(a2) = &self.a2 {
                    let _ = writeln!(&mut buf, "**Answer**: `{a2}`.");
                }
            }
        }
        buf
    }

    pub fn write_view(&self, path: impl AsRef<Path>) -> Result<()> {
        Ok(fs::write(path, self.view(true))?)
    }
}

fn home_dir() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap_or_else(|e| {
        error!(cause = %e, "HOME");
        process::exit(1);
    }))
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

    fn derive_id_from_path(path: impl AsRef<Path>) -> Result<(Option<u16>, Option<u8>)> {
        for parent in path.as_ref().ancestors() {
            let mut buf = String::new();
            let mut chars = parent
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .chars()
                .peekable();

            while let Some(c) = chars.next() {
                if c.is_ascii_digit() {
                    buf.push(c);
                    if !chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                        break;
                    }
                }
            }
        }
        todo!()
    }

    #[test]
    fn from_path() {
        let cases = vec![
            ("/Users/j0rdi/aoc/2015/d01", Some((2015, 1))),
            ("/home/j0rdi/aoc/2024/25", Some((2024, 25))),
            ("/Users/j0rdi/aoc/2017/other/d8", Some((2017, 8))),
            ("/home/j0rdi/aoc/2017/other/08/sub", Some((2017, 8))),
        ];

        for (path, expected) in cases {
            assert_eq!(puzzle_id_from_path(path), expected)
        }

        assert_eq!(puzzle_id_from_path("/invalid/path"), None)
    }
}

// struct Id(u32, u32);
//
// impl TryFrom<&Path> for Id {
//     type Error = &'static str;
//
//     fn try_from(path: &Path) -> Result<Self, Self::Error> {
//         match puzzle_id_from_path(path) {
//             Some((y, d)) => Ok(Id(y, d)),
//             None => Err("could not determine puzzle id from path"),
//         }
//     }
// }

/* impl<P: AsRef<Path>> From<P> for Id {
    fn from(value: P) -> Self {
        todo!()
    }
} */

// impl From<(u32, u32)> for Id {
//     fn from((y, d): (u32, u32)) -> Self {
//         Id(y, d)
//     }
// }
