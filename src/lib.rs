mod client;
pub use client::{Client, Submit};

mod puzzle;
pub use puzzle::{Puzzle, PuzzleId};

pub const AOC_URL: &str = "https://adventofcode.com";

#[derive(clap::Parser)]
#[command(version, author, propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    Get {
        #[arg(long, short)]
        year: Option<u32>,
        #[arg(long, short)]
        day: Option<u32>,
        /// The output directory.
        output: Option<std::path::PathBuf>,
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
    View {
        #[arg(long, short)]
        year: Option<u32>,
        #[arg(long, short)]
        day: Option<u32>,
    },
}
