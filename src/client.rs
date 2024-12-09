use std::env;

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
    pub fn new() -> Result<Self> {
        let auth = env::var("AOC_AUTH_COOKIE")
            .map(|token| format!("session={token}"))
            .context("Must provide auth cookie")?;

        let headers = HeaderMap::from_iter([("cookie".parse()?, auth.parse()?)]);
        let http = reqwest::blocking::Client::builder()
            .user_agent("aocli.rs")
            .default_headers(headers)
            .redirect(Policy::none())
            .build()?;

        Ok(Self { http })
    }

    pub fn get_puzzle(&self, id: PuzzleId) -> Result<Puzzle> {
        let html = self
            .http
            .get(&self.mkurl(&id))
            .send()
            .context("get puzzle")?
            .text()?;
        let doc = Html::parse_document(&html);
        let selector = Selector::parse("article.day-desc").unwrap();
        let mut select = doc.select(&selector);
        let q1 = select
            .next()
            .map(|el| html2text::from_read(el.inner_html().as_bytes(), 80).unwrap());
        let q2 = select
            .next()
            .map(|el| html2text::from_read(el.inner_html().as_bytes(), 80).unwrap());

        let selector = Selector::parse("article.day-desc + p code").unwrap();
        let mut select = doc.select(&selector);
        let a1 = select.next().map(|el| el.text().collect::<String>());
        let a2 = select.next().map(|el| el.text().collect::<String>());

        Ok(Puzzle {
            id,
            q1: q1.unwrap_or_default(),
            q2: q2.unwrap_or_default(),
            a1: a1.unwrap_or_default(),
            a2: a2.unwrap_or_default(),
        })
    }

    pub fn get_input(self, id: PuzzleId) -> Result<String> {
        let url = format!("{}/input", self.mkurl(&id));
        let data = self.http.get(&url).send().context("get input")?.text()?;
        Ok(data)
    }

    pub fn submit(&self, id: PuzzleId, part: u32, answer: impl AsRef<str>) -> Result<Submit> {
        let url = format!("{}/answer", self.mkurl(&id));
        let html = self
            .http
            .post(&url)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(format!("level={part}&answer={}", answer.as_ref()))
            .send()?
            .text()?;

        Ok(if html.contains("That's the right answer") {
            Submit::Correct("Correct!".into())
            // println!("Correct!");
            // if part == 1 {
            //     // let puzzle = Puzzle::scrape(&client, year, day)?;
            //     let puzzle = self.scrape_puzzle(id);
            //     // let puzzle_path = cache.join(id_to_path((year, day)));
            //     // puzzle.write(&puzzle_path)?;
            // }
            // fs::write(
            //     cache.join(id_to_path((year, day)).join(format!("answer{part}"))),
            //     &answer,
            // )?;
        } else if html.contains("That's not the right answer") {
            Submit::Incorrect("Incorrect!".into())
            // println!("Incorrect!");
        } else if html.contains("You gave an answer too recently") {
            // println!("Wait!");
            Submit::Wait("Wait!".into())
        } else {
            Submit::Error("Unkwon response".into())
            // eprintln!("error: unknown response");
            // std::process::exit(1);
        })
    }

    fn mkurl(&self, (year, day): &PuzzleId) -> String {
        format!("{AOC_URL}/{year}/day/{day}")
    }
}
