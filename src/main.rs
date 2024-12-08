use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reqwest::header::HeaderMap;
use reqwest::{
    blocking::{multipart, Client},
    redirect::Policy,
};
use scraper::{Html, Selector};

#[derive(Parser)]
#[command(version, author, propagate_version = true)]
struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
enum Command {
    Get {
        #[arg(long, short)]
        year: u32,
        #[arg(long, short)]
        day: u32,
        /// The output directory.
        output: Option<PathBuf>,
    },

    Submit {
        answer: String,
        #[arg(long, short, value_parser = clap::value_parser!(u32).range(2015..=2024))]
        year: u32,
        #[arg(long, short, value_parser = clap::value_parser!(u32).range(1..=24))]
        day: u32,
        #[arg(long, short, value_parser = clap::value_parser!(u32).range(1..=2))]
        part: u32,
    },
}

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";

fn aoc_url(year: u32, day: u32) -> String {
    format!("https://adventofcode.com/{year}/day/{day}")
}

#[derive(Debug, Default)]
struct Puzzle {
    id: (u32, u32),
    q1: String,
    a1: String,
    q2: String,
    a2: String,
}

fn id_to_path((y, d): (u32, u32)) -> PathBuf {
    Path::new(&y.to_string()).join(&d.to_string())
}

impl Puzzle {
    fn read(path: &Path, year: u32, day: u32) -> Option<Puzzle> {
        path.exists().then(|| Puzzle {
            id: (year, day),
            q1: fs::read_to_string(path.join("question1")).unwrap_or_default(),
            q2: fs::read_to_string(path.join("question2")).unwrap_or_default(),
            a1: fs::read_to_string(path.join("answer1")).unwrap_or_default(),
            a2: fs::read_to_string(path.join("answer2")).unwrap_or_default(),
        })
    }

    fn write(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        fs::create_dir_all(&path)?;
        fs::write(path.join("question1"), self.q1.as_bytes())?;
        fs::write(path.join("question2"), self.q2.as_bytes())?;
        fs::write(path.join("answer1"), self.a1.as_bytes())?;
        fs::write(path.join("answer2"), self.a2.as_bytes())?;
        Ok(())
    }

    fn scrape(client: &Client, year: u32, day: u32) -> Result<Puzzle> {
        let url = aoc_url(year, day);
        let html = client.get(&url).send().context("get view")?.text()?;
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

        let puzzle = Puzzle {
            id: (year, day),
            q1: q1.unwrap_or_default(),
            q2: q2.unwrap_or_default(),
            a1: a1.unwrap_or_default(),
            a2: a2.unwrap_or_default(),
        };

        Ok(puzzle)
    }
}

fn scrape_input(client: &Client, year: u32, day: u32) -> Result<String> {
    let url = format!("{}/input", aoc_url(year, day));
    let data = client.get(&url).send()?.text()?;
    Ok(data)
}

// struct AdventClient {
//
// }

fn main() -> Result<()> {
    let args = Args::parse();
    let auth = env::var("AOC_AUTH_COOKIE")
        .map(|token| format!("session={token}"))
        .unwrap_or_else(|_| {
            eprintln!("Must provide auth cookie.");
            std::process::exit(1)
        });

    let headers = HeaderMap::from_iter([("cookie".parse()?, auth.parse()?)]);
    let client = Client::builder()
        .user_agent("")
        .default_headers(headers)
        .redirect(Policy::none())
        .build()?;

    let cache = Path::new("cache");

    match args.command {
        Command::Get { year, day, output } => {
            let puzzle_path = cache.join(id_to_path((year, day)));
            let puzzle = match Puzzle::read(&puzzle_path, year, day) {
                Some(puzzle) => {
                    println!("input from cache");
                    puzzle
                }
                None => {
                    let puzzle = Puzzle::scrape(&client, year, day)?;
                    puzzle.write(&puzzle_path)?;
                    puzzle
                }
            };
            let input_path = puzzle_path.join("input");
            let input = if input_path.exists() {
                println!("puzzle from cache");
                fs::read_to_string(&input_path)?
            } else {
                let input = scrape_input(&client, year, day)?;
                fs::write(&input_path, &input)?;
                input
            };

            let out = output.unwrap_or("./".into());
            fs::create_dir_all(&out)?;
            fs::write(
                out.join("puzzle.md"),
                format!("{}\n{}", puzzle.q1, puzzle.q2),
            )?;
            fs::write(out.join("input"), &input)?;
        }

        Command::Submit {
            answer,
            year,
            day,
            part,
        } => {
            let aoc = aoc_url(year, day);
            let url = format!("{aoc}/answer");
            println!("day={day} year={year} answer={answer} url={url}");

            let resp = client
                .post(&url)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(format!("level={part}&answer={answer}"))
                .send()?;
            println!("{resp:?}");
            let text = resp.text()?;
            println!("{text}");
        }
    }

    Ok(())
}
