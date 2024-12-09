use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

use anyhow::{ensure, Result};
use clap::Parser;

use aoc::{Args, Client, Command, Puzzle, PuzzleId, Submit};
use tracing::{debug, error};

fn id_to_path((y, d): (u32, u32)) -> PathBuf {
    Path::new(&y.to_string()).join(d.to_string())
}

const AUTH_VAR: &str = "AOC_AUTH_COOKIE";

fn main() -> Result<()> {
    let args = Args::parse();
    setup_logging()?;

    let auth = env::var(AUTH_VAR).unwrap_or_else(|e| {
        error!(cause = %e, "Environment variable `{AUTH_VAR}` not found");
        process::exit(1);
    });

    let client = Client::new(&auth)?;
    let cache = Path::new(&env::var("HOME").unwrap()).join("dev/comp/aocli/cache");

    match args.command {
        Command::Get { year, day, output } => {
            let id = puzzle_id(year, day)?;
            let puzzle_path = cache.join(id_to_path(id));
            let puzzle = match Puzzle::read(&puzzle_path, id) {
                Some(puzzle) => {
                    debug!("puzzle from cache");
                    if !puzzle.a1.is_empty() && puzzle.q2.is_empty() {
                        debug!("next part");
                        let next_part = client.get_puzzle(&id)?;
                        next_part.write(&puzzle_path)?;
                        next_part
                    } else {
                        puzzle
                    }
                }
                None => {
                    let puzzle = client.get_puzzle(&id)?;
                    puzzle.write(&puzzle_path)?;
                    puzzle
                }
            };
            let input_path = puzzle_path.join("input");
            let input = if input_path.exists() {
                debug!("input from cache");
                fs::read_to_string(&input_path)?
            } else {
                let input = client.get_input(&id)?;
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
            let id = puzzle_id(year, day)?;
            let puzzle_path = cache.join(id_to_path(id));
            let part = part.unwrap_or_else(|| {
                if fs::metadata(puzzle_path.join("answer1")).is_ok_and(|m| m.len() > 0) {
                    2
                } else {
                    1
                }
            });
            match client.submit(&id, part, &answer)? {
                Submit::Correct(msg) => {
                    debug!("{msg}");
                    if part == 1 {
                        let puzzle = client.get_puzzle(&id)?;
                        puzzle.write(&puzzle_path)?;
                    }
                    fs::write(puzzle_path.join(format!("answer{part}")), &answer)?;
                }
                any => debug!("{any:?}"),
            }
        }

        Command::View { year, day } => {
            let id = puzzle_id(year, day)?;
            let puzzle_path = cache.join(id_to_path(id));
            let puzzle = match Puzzle::read(&puzzle_path, id) {
                Some(puzzle) => {
                    if !puzzle.a1.is_empty() && puzzle.q2.is_empty() {
                        let next_part = client.get_puzzle(&id)?;
                        next_part.write(&puzzle_path)?;
                        next_part
                    } else {
                        puzzle
                    }
                }
                None => {
                    let puzzle = client.get_puzzle(&id)?;
                    puzzle.write(&puzzle_path)?;
                    puzzle
                }
            };
            let view = format!("{}\n{}", puzzle.q1, puzzle.q2);
            println!("{view}");
        }
    }

    Ok(())
}

fn puzzle_id(year: Option<u32>, day: Option<u32>) -> Result<PuzzleId> {
    validate_puzzle_id(match (year, day) {
        (Some(y), Some(d)) => (y, d),
        _ => find_current_puzzle_id().unwrap_or_else(|| {
            error!("Could not determine puzzle from current directory");
            process::exit(1);
        }),
    })
}

fn validate_puzzle_id((year, day): PuzzleId) -> Result<PuzzleId> {
    ensure!(2015 <= year && year < 2025, "Invalid year: {year}");
    ensure!(1 <= day && day < 25, "Invalid day: {day}");
    Ok((year, day))
}

fn find_current_puzzle_id() -> Option<PuzzleId> {
    let mut day = 0xff;
    let mut year = 0;
    for parent in env::current_dir().unwrap().ancestors() {
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

fn setup_logging() -> Result<()> {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()?
        .add_directive("hyper::proto=info".parse()?);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();

    Ok(())
}
