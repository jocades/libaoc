use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

use anyhow::{ensure, Result};
use clap::{value_parser, Parser, Subcommand};
use tracing::{error, info};

use libaoc::{Client, PuzzleId};

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
    /// The puzzle's year
    #[arg(long, short, value_parser = value_parser!(u16).range(2015..=2024))]
    year: Option<u16>,
    /// The puzzle's day
    #[arg(long, short, value_parser = value_parser!(u8).range(1..=25))]
    day: Option<u8>,
}

#[derive(Subcommand)]
enum Command {
    Get {
        #[command(flatten)]
        id: YearDay,
        /// The output directory, default: `.`
        output: Option<PathBuf>,
        /// Build the directories `./year/day/`
        #[arg(long, short)]
        build: bool,
    },
    Submit {
        #[command(flatten)]
        id: YearDay,
        #[arg(long, short, value_parser = value_parser!(u8).range(1..=2))]
        part: Option<u8>,
        answer: String,
    },
    View {
        #[command(flatten)]
        id: YearDay,
        /// Wether to show the answers for each part.
        #[arg(long, short)]
        answers: bool,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    setup_logging(args.verbose)?;

    let client = Client::new()?;
    let cwd = env::current_dir()?;

    match args.command {
        Command::Get { id, output, build } => {
            let id = derive_id(id, &cwd)?;
            let puzzle = client.get_puzzle(&id)?;
            let input = client.get_input(&id)?;
            let dest = build_path(&id, build).unwrap_or_else(|| output.unwrap_or(cwd));
            fs::create_dir_all(&dest)?;
            let puzzle_path = dest.join("puzzle.md");
            fs::write(&puzzle_path, puzzle.view(true))?;
            info!("{} written", puzzle_path.display());
            let input_path = dest.join("input");
            fs::write(&input_path, &input)?;
            info!("{} written", input_path.display());
        }
        Command::Submit { id, part, answer } => {
            let id = derive_id(id, &cwd)?;
            if let Some(puzzle) = client.submit(&id, part, &answer)? {
                puzzle.write_view(cwd.join("puzzle.md"))?;
            }
        }
        Command::View { id, answers } => {
            let id = derive_id(id, &cwd)?;
            let puzzle = client.get_puzzle(&id)?;
            println!("{}", puzzle.view(answers));
        }
    }

    Ok(())
}

fn derive_id(id: YearDay, cwd: impl AsRef<Path>) -> Result<PuzzleId> {
    validate_puzzle_id(match (id.year, id.day) {
        (Some(y), Some(d)) => (y, d),
        _ => libaoc::puzzle_id_from_path(cwd).unwrap_or_else(|| {
            error!("Could not determine puzzle from current directory");
            process::exit(1);
        }),
    })
}

fn validate_puzzle_id((year, day): PuzzleId) -> Result<PuzzleId> {
    ensure!((2015..=2024).contains(&year), "Invalid year: {year}");
    ensure!((1..=25).contains(&day), "Invalid day: {day}");
    Ok((year, day))
}

fn build_path((y, d): &PuzzleId, build: bool) -> Option<PathBuf> {
    build.then(|| {
        let mut path = format!("{}/d", y);
        if *d < 10 {
            path.push('0');
        }
        path.push_str(&d.to_string());
        path.into()
    })
}

fn setup_logging(verbose: bool) -> Result<()> {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::builder()
        .with_default_directive(if verbose {
            LevelFilter::DEBUG.into()
        } else {
            LevelFilter::ERROR.into()
        })
        .from_env()?
        .add_directive("hyper::proto=info".parse()?)
        .add_directive("libaoc=debug".parse()?);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .compact()
        .init();

    Ok(())
}
