use std::{env, fs, path::PathBuf, process};

use anyhow::{ensure, Result};
use clap::{value_parser, Parser, Subcommand};
use tracing::error;

use libaoc::{Client, PuzzleId, AUTH_VAR};

#[derive(Parser)]
#[command(version, author, propagate_version = true)]
struct Args {
    #[arg(long, short)]
    pub verbose: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Args)]
struct YearDay {
    #[arg(long, short, value_parser = value_parser!(u32).range(2015..=2025))]
    year: Option<u32>,
    #[arg(long, short, value_parser = value_parser!(u32).range(1..=24))]
    day: Option<u32>,
}

#[derive(Subcommand)]
enum Command {
    Get {
        #[command(flatten)]
        yd: YearDay,
        /// The output directory, default: `.`
        output: Option<PathBuf>,
        /// Build the directories `./year/day/`
        #[arg(long, short)]
        build: bool,
    },
    Submit {
        #[command(flatten)]
        yd: YearDay,
        #[arg(long, short, value_parser = value_parser!(u32).range(1..=2))]
        part: Option<u32>,
        answer: String,
    },
    View {
        #[command(flatten)]
        yd: YearDay,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    setup_logging(args.verbose)?;

    let token = env::var(AUTH_VAR).unwrap_or_else(|e| {
        error!(cause = %e, AUTH_VAR);
        process::exit(1);
    });

    let client = Client::new(&token)?;
    let dest = destination();

    match args.command {
        Command::Get { yd, output, build } => {
            let id = puzzle_id(yd.year, yd.day)?;
            let puzzle = client.get_puzzle(&id)?;
            let input = client.get_input(&id)?;

            let dest = if build {
                let mut path = format!("{}/d", id.0);
                if id.1 < 10 {
                    path.push('0');
                }
                path.push_str(&id.1.to_string());
                PathBuf::from(path)
            } else {
                output.unwrap_or("./".into())
            };
            fs::create_dir_all(&dest)?;
            fs::write(dest.join("puzzle.md"), puzzle.view())?;
            fs::write(dest.join("input"), &input)?;
        }

        Command::Submit { answer, yd, part } => {
            let id = puzzle_id(yd.year, yd.day)?;
            if let Some(puzzle) = client.submit(&id, part, &answer)? {
                puzzle.write_view(dest.join("puzzle.md"))?;
            }
        }

        Command::View { yd } => {
            let id = puzzle_id(yd.year, yd.day)?;
            let puzzle = client.get_puzzle(&id)?;
            println!("{}", puzzle.view());
        }
    }

    Ok(())
}

fn destination() -> PathBuf {
    env::var("__NVIM_AOC")
        .ok()
        .map(PathBuf::from)
        .unwrap_or(env::current_dir().unwrap())
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
    ensure!((2015..=2025).contains(&year), "Invalid year: {year}");
    ensure!((1..=25).contains(&day), "Invalid day: {day}");
    Ok((year, day))
}

fn find_current_puzzle_id() -> Option<PuzzleId> {
    libaoc::puzzle_id_from_path(env::current_dir().unwrap())
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
