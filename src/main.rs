use std::{env, fs, path::Path};

use anyhow::Result;
use clap::{Parser, Subcommand};
use reqwest::header::HeaderMap;
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

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";

fn main() -> Result<()> {
    let args = Args::parse();
    let auth = env::var("AOC_AUTH_COOKIE")
        .map(|token| format!("session={token}"))
        .unwrap_or_else(|_| {
            eprintln!("Must provide auth cookie.");
            std::process::exit(1)
        });

    let headers = HeaderMap::from_iter([("cookie".parse()?, auth.parse()?)]);

    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .default_headers(headers)
        .build()?;

    match args.command {
        Command::Get { year, day } => {
            let dir = Path::new("cache")
                .join(year.to_string())
                .join(day.to_string());

            if !dir.exists() {
                let url = format!("https://adventofcode.com/{year}/day/{day}");
                let html = client.get(&url).send().unwrap().text().unwrap();
                let doc = Html::parse_document(&html);
                let selector = Selector::parse("article.day-desc").unwrap();
                fs::create_dir_all(&dir)?;
                for (i, article) in doc.select(&selector).enumerate() {
                    if i > 1 {
                        eprintln!("found more than 2 articles");
                        break;
                    }
                    let text = html2text::from_read(article.inner_html().as_bytes(), 80)?;
                    let path = dir.join(format!("part{}q", i + 1));
                    fs::write(&path, text.as_bytes())?;
                }
            }
        }
    }

    Ok(())
}
