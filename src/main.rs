use std::{fs, path::Path};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
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
    },
}

fn parse(html: &str) -> Result<String> {
    let doc = Html::parse_document(html);
    let selector = Selector::parse("article.day-desc").expect("slector");
    let article = doc.select(&selector).next().context("select")?;
    html2text::from_read(article.inner_html().as_bytes(), 80).context("html2text")
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Get { year, day } => {
            let dir = Path::new("cache")
                .join(year.to_string())
                .join(day.to_string());

            let question_path = dir.join("question.txt");

            let question = if !question_path.exists() {
                let url = format!("https://adventofcode.com/{year}/day/{day}");
                let html = reqwest::blocking::get(&url).unwrap().text().unwrap();
                let text = parse(&html)?;
                fs::create_dir_all(&dir)?;
                fs::write(&question_path, text.as_bytes())?;
                text
            } else {
                fs::read_to_string(&question_path)?
            };

            println!("{question}");
        }
    }

    Ok(())
}
