use std::{env, fs, process};

use anyhow::{ensure, Result};
use clap::Parser;
use tracing::error;

use libaoc::{Args, Client, Command, PuzzleId, AUTH_VAR};

fn main() -> Result<()> {
    let args = Args::parse();
    setup_logging(args.verbose)?;

    let token = env::var(AUTH_VAR).unwrap_or_else(|e| {
        error!(cause = %e, "Environment variable `{AUTH_VAR}` not found");
        process::exit(1);
    });

    let client = Client::new(&token)?;

    match args.command {
        Command::Get {
            year,
            day,
            output,
            build,
        } => {
            let id = puzzle_id(year, day)?;
            let puzzle = client.get_puzzle(&id)?;
            let input = client.get_input(&id)?;

            let dest = if build {
                let mut path = format!("{}/d", id.0);
                if id.1 < 10 {
                    path.push('0');
                }
                path.push_str(&id.1.to_string());
                std::path::PathBuf::from(path)
            } else {
                output.unwrap_or("./".into())
            };
            fs::create_dir_all(&dest)?;
            fs::write(
                dest.join("puzzle.md"),
                format!("{}\n{}", puzzle.q1, puzzle.q2),
            )?;
            fs::write(dest.join("input"), &input)?;
        }

        Command::Submit {
            answer,
            year,
            day,
            part,
        } => {
            let id = puzzle_id(year, day)?;
            client.submit(&id, part, &answer)?;
        }

        Command::View { year, day } => {
            let id = puzzle_id(year, day)?;
            let puzzle = client.get_puzzle(&id)?;
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

fn setup_logging(verbose: bool) -> Result<()> {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::builder()
        .with_default_directive(if verbose {
            LevelFilter::INFO.into()
        } else {
            LevelFilter::ERROR.into()
        })
        .from_env()?
        .add_directive("hyper::proto=info".parse()?)
        .add_directive("libaoc=debug".parse()?);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();

    Ok(())
}
