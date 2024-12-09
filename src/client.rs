use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use reqwest::redirect::Policy;
use scraper::{Html, Selector};

use crate::{Puzzle, PuzzleId, AOC_URL};

pub struct Client {
    http: reqwest::blocking::Client,
}

#[derive(Debug)]
pub enum Submit {
    Correct(String),
    Incorrect(String),
    Wait(String),
    Error(String),
}

impl Client {
    pub fn new(token: &str) -> Result<Self> {
        let auth = format!("session={token}");
        let headers = HeaderMap::from_iter([("cookie".parse()?, auth.parse()?)]);
        let http = reqwest::blocking::Client::builder()
            .user_agent("aocli.rs")
            .default_headers(headers)
            .redirect(Policy::none())
            .build()?;

        Ok(Self { http })
    }

    pub fn get_puzzle(&self, id: &PuzzleId) -> Result<Puzzle> {
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
            .map(|el| html2text::from_read(el.inner_html().as_bytes(), 80).unwrap());
        let q2 = questions
            .next()
            .map(|el| html2text::from_read(el.inner_html().as_bytes(), 80).unwrap());

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
        Ok(self
            .http
            .get(format!("{}/input", self.mkurl(&id)))
            .send()
            .context("get input")?
            .text()?)
    }

    pub fn submit(&self, id: &PuzzleId, part: u32, answer: impl AsRef<str>) -> Result<Submit> {
        let html = self
            .http
            .post(format!("{}/answer", self.mkurl(id)))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(format!("level={part}&answer={}", answer.as_ref()))
            .send()?
            .text()?;

        Ok(if html.contains("That's the right answer") {
            Submit::Correct("Correct!".into())
        } else if html.contains("That's not the right answer") {
            Submit::Incorrect("Incorrect!".into())
        } else if html.contains("You gave an answer too recently") {
            Submit::Wait("Wait!".into())
        } else {
            Submit::Error("Unknown response".into())
        })
    }

    fn mkurl(&self, (year, day): &PuzzleId) -> String {
        format!("{AOC_URL}/{year}/day/{day}")
    }
}
