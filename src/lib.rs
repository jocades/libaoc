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
pub const AUTH_VAR: &str = "AOC_AUTH_COOKIE";
pub const CACHE_PATH: &str = "dev/comp/aocli/cache";

pub type PuzzleId = (u32, u32);

#[derive(clap::Parser)]
#[command(version, author, propagate_version = true)]
pub struct Args {
    #[arg(long, short)]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    Get {
        #[arg(long, short)]
        year: Option<u32>,
        #[arg(long, short)]
        day: Option<u32>,
        /// The output directory.
        output: Option<std::path::PathBuf>,
    },
    Submit {
        answer: String,
        #[arg(long, short, value_parser = clap::value_parser!(u32).range(2015..=2024))]
        year: Option<u32>,
        #[arg(long, short, value_parser = clap::value_parser!(u32).range(1..=24))]
        day: Option<u32>,
        #[arg(long, short, value_parser = clap::value_parser!(u32).range(1..=2))]
        part: Option<u32>,
    },
    View {
        #[arg(long, short)]
        year: Option<u32>,
        #[arg(long, short)]
        day: Option<u32>,
    },
}

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
        headers.insert("cookie", format!("sesison={token}").parse()?);
        Ok(Self {
            http: reqwest::blocking::Client::builder()
                .user_agent("aocli.rs")
                .default_headers(headers)
                .redirect(Policy::none())
                .build()?,
            cache: Cache::new(home_dir().join(CACHE_PATH)),
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

    fn scrape_puzzle(&self, id: &PuzzleId) -> Result<Puzzle> {
        let html = self
            .http
            .get(self.mkurl(id))
            .send()
            .context("get puzzle")?
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
            .send()?
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
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().into(),
        }
    }

    pub fn get(&self, id: &PuzzleId) -> Option<Puzzle> {
        Puzzle::read(self.mkpath(id), id)
    }

    pub fn get_input(&self, id: &PuzzleId) -> Option<String> {
        let path = self.mkpath(id).join("input");
        path.exists()
            .then(|| fs::read_to_string(self.mkpath(id).join("input")).unwrap())
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
    pub fn read(path: impl AsRef<Path>, id: &PuzzleId) -> Option<Puzzle> {
        let path = path.as_ref();
        path.exists().then(|| Puzzle {
            id: id.clone(),
            q1: fs::read_to_string(path.join("question1")).unwrap_or_default(),
            q2: fs::read_to_string(path.join("question2")).unwrap_or_default(),
            a1: fs::read_to_string(path.join("answer1")).unwrap_or_default(),
            a2: fs::read_to_string(path.join("answer2")).unwrap_or_default(),
        })
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        fs::create_dir_all(path)?;
        fs::write(path.join("question1"), self.q1.as_bytes())?;
        fs::write(path.join("question2"), self.q2.as_bytes())?;
        fs::write(path.join("answer1"), self.a1.as_bytes())?;
        fs::write(path.join("answer2"), self.a2.as_bytes())?;
        Ok(())
    }
}
