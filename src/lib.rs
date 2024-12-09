use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use reqwest::redirect::Policy;
use scraper::{Html, Selector};
use tracing::{debug, error, info, warn};

pub const AOC_URL: &str = "https://adventofcode.com";
pub const AUTH_VAR: &str = "AOC_AUTH_TOKEN";
pub const CACHE_PATH: &str = ".cache/aoc";

pub type PuzzleId = (u32, u32);

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| {
        error!("Environment variable`HOME` not found");
        std::process::exit(1);
    }))
}

pub struct Client {
    http: reqwest::blocking::Client,
    cache: Cache,
}

impl Client {
    pub fn new(token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", format!("session={token}").parse()?);
        Ok(Self {
            http: reqwest::blocking::Client::builder()
                .user_agent("aocli.rs")
                .default_headers(headers)
                .redirect(Policy::none())
                .build()?,
            cache: Cache::new(home_dir().join(CACHE_PATH))?,
        })
    }

    pub fn get_puzzle(&self, id: &PuzzleId) -> Result<Puzzle> {
        if let Some(puzzle) = self.cache.get(id) {
            debug!("puzzle from cache");
            return Ok(puzzle);
        }
        debug!("new puzzle");
        let puzzle = self.scrape_puzzle(id)?;
        self.cache.insert(id, &puzzle);
        Ok(puzzle)
    }

    pub fn scrape_puzzle(&self, id: &PuzzleId) -> Result<Puzzle> {
        let html = self
            .http
            .get(self.mkurl(id))
            .send()
            .context("get puzzle")?
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
            id: id.clone(),
            q1: q1.unwrap_or_default(),
            q2: q2.unwrap_or_default(),
            a1: a1.unwrap_or_default(),
            a2: a2.unwrap_or_default(),
        })
    }

    pub fn get_input(self, id: &PuzzleId) -> Result<String> {
        if let Some(input) = self.cache.get_input(id) {
            debug!("input from cache");
            return Ok(input);
        }
        debug!("new input");
        let input = self
            .http
            .get(format!("{}/input", self.mkurl(&id)))
            .send()
            .context("get input")?
            .error_for_status()?
            .text()?;
        self.cache.insert_input(id, &input);
        Ok(input)
    }

    pub fn submit(
        &self,
        id: &PuzzleId,
        part: Option<u32>,
        answer: impl AsRef<str>,
    ) -> Result<Submit> {
        let part = part.unwrap_or_else(|| {
            if fs::metadata(self.cache.mkpath(id).join("answer1")).is_ok_and(|m| m.len() > 0) {
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
            .send()
            .context("submit puzzle")?
            .error_for_status()?
            .text()?;

        Ok(if html.contains("That's the right answer") {
            info!("Correct!");
            self.refresh_puzzle(id)?;
            Submit::Correct
        } else if html.contains("That's not the right answer") {
            error!("Incorrect!");
            Submit::Incorrect
        } else if html.contains("You gave an answer too recently") {
            warn!("Wait!");
            Submit::Wait
        } else {
            error!("Unknown response");
            Submit::Error
        })
    }

    pub fn refresh_puzzle(&self, id: &PuzzleId) -> Result<Puzzle> {
        debug!("refresh puzzle");
        let puzzle = self.scrape_puzzle(id)?;
        self.cache.insert(id, &puzzle);
        Ok(puzzle)
    }

    fn mkurl(&self, (y, d): &PuzzleId) -> String {
        format!("{AOC_URL}/{y}/day/{d}")
    }
}

struct Cache {
    path: PathBuf,
}

impl Cache {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(&path).context("mkdir cache")?;
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

    pub fn insert(&self, id: &PuzzleId, puzzle: &Puzzle) {
        puzzle
            .write(self.mkpath(&id))
            .unwrap_or_else(|_| warn!(cause = "failed to insert puzzle", "cache error"));
    }

    pub fn insert_input(&self, id: &PuzzleId, input: &str) {
        fs::write(self.mkpath(id).join("input"), input)
            .unwrap_or_else(|_| warn!(cause = "failed to insert input", "cache error"));
    }

    #[allow(dead_code)]
    pub fn update_answer(&self, id: &PuzzleId, part: u32, answer: &str) {
        fs::write(self.mkpath(id).join(format!("answer{part}")), answer)
            .unwrap_or_else(|_| warn!(cause = "failed to update answer", "cache error"));
    }

    fn mkpath(&self, (y, d): &PuzzleId) -> PathBuf {
        self.path.join(format!("{y}/{d}"))
    }
}

pub enum Submit {
    Correct,
    Incorrect,
    Wait,
    Error,
}

#[derive(Debug, Default, Clone)]
pub struct Puzzle {
    pub id: PuzzleId,
    pub q1: String,
    pub q2: String,
    pub a1: String,
    pub a2: String,
}

impl Puzzle {
    pub fn read(path: impl AsRef<Path>, id: &PuzzleId) -> Puzzle {
        let path = path.as_ref();
        Puzzle {
            id: id.clone(),
            q1: fs::read_to_string(path.join("q1")).unwrap_or_default(),
            q2: fs::read_to_string(path.join("q2")).unwrap_or_default(),
            a1: fs::read_to_string(path.join("a1")).unwrap_or_default(),
            a2: fs::read_to_string(path.join("a2")).unwrap_or_default(),
        }
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        fs::create_dir_all(path)?;
        fs::write(path.join("q1"), self.q1.as_bytes())?;
        fs::write(path.join("q2"), self.q2.as_bytes())?;
        fs::write(path.join("a1"), self.a1.as_bytes())?;
        fs::write(path.join("a2"), self.a2.as_bytes())?;
        Ok(())
    }
}

pub fn puzzle_id_from_path(path: &Path) -> Option<PuzzleId> {
    let mut day = 0xff;
    let mut year = 0;
    for parent in path.ancestors() {
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
        let path = Path::new("/Users/j0rdi/aoc/2015/d01");
        assert_eq!(puzzle_id_from_path(&path), Some((2015, 1)));

        let path = Path::new("/Users/j0rdi/aoc/2024/25");
        assert_eq!(puzzle_id_from_path(&path), Some((2024, 25)));

        let path = Path::new("/Users/j0rdi/aoc/2017/other/d8");
        assert_eq!(puzzle_id_from_path(&path), Some((2017, 8)));

        let path = Path::new("/Users/j0rdi/aoc/2017/other/08/sub");
        assert_eq!(puzzle_id_from_path(&path), Some((2017, 8)));
    }
}
