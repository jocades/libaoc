use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::{Parser, Subcommand};

use aoc::{Client, Puzzle, PuzzleId};

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
        year: Option<u32>,
        #[arg(long, short, value_parser = clap::value_parser!(u32).range(1..=24))]
        day: Option<u32>,
        #[arg(long, short, value_parser = clap::value_parser!(u32).range(1..=2))]
        part: Option<u32>,
    },
}

fn id_to_path((y, d): (u32, u32)) -> PathBuf {
    Path::new(&y.to_string()).join(&d.to_string())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::new()?;

    let cache = Path::new("cache");

    match args.command {
        Command::Get { year, day, output } => {
            let id = (year, day);
            let puzzle_path = cache.join(id_to_path(id));
            let puzzle = match Puzzle::read(&puzzle_path, year, day) {
                Some(puzzle) => {
                    println!("input from cache");
                    if !puzzle.a1.is_empty() && puzzle.q2.is_empty() {
                        println!("next part");
                        let puzzle = client.get_puzzle(id)?;
                        puzzle.write(&puzzle_path)?;
                        puzzle
                    } else {
                        puzzle
                    }
                }
                None => {
                    let puzzle = client.get_puzzle(id)?;
                    puzzle.write(&puzzle_path)?;
                    puzzle
                }
            };
            let input_path = puzzle_path.join("input");
            let input = if input_path.exists() {
                println!("puzzle from cache");
                fs::read_to_string(&input_path)?
            } else {
                let input = client.get_input(id)?;
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
            let id = match (year, day) {
                (Some(y), Some(d)) => (y, d),
                _ => find_current_puzzle_id().unwrap_or_else(|| {
                    eprintln!("Could not determine current puzzle from cwd");
                    std::process::exit(1)
                }),
            };

            let part = part.unwrap_or(1);
            match client.submit(id, part, answer)? {
                any => println!("{any:?}"),
            }
        }
    }

    Ok(())
}

fn find_current_puzzle_id() -> Option<PuzzleId> {
    let mut day = 0;
    let mut year = 0;
    for parent in env::current_dir().unwrap().ancestors() {
        let mut chars = parent.file_name().unwrap().to_str().unwrap().chars();
        let mut s = String::new();
        while let Some(c) = chars.next() {
            if c.is_ascii_digit() {
                s.push(c);
                while let Some(c) = chars.next() {
                    if !c.is_ascii_digit() {
                        break;
                    }
                    s.push(c);
                }
            }
        }
        if !s.is_empty() {
            if day == 0 {
                day = s.parse().unwrap();
            } else {
                year = s.parse().unwrap();
            }
        }
        if year > 0 {
            return Some((year, day));
        }
    }
    None
}
