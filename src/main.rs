use clap::Parser;
use model::Transaction;
use std::path::PathBuf;

mod model;

/// Transaction CLI tool
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    input: PathBuf,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    CSVError(#[from] csv::Error),
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    if !cli.input.is_file() {
        eprintln!("Error: '{}' is not a valid file", cli.input.display());
        std::process::exit(1);
    }

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(cli.input)
        .expect("failed to read from CSV");

    for tx in reader.deserialize::<Transaction>() {
        dbg!(tx?);
    }

    Ok(())
}
