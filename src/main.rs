use clap::Parser;
use model::{Account, Transaction};
use std::{collections::HashMap, path::PathBuf};

use crate::model::AccountError;

mod model;
mod rules;
/// Transaction CLI tool
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    input: PathBuf,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    AccountError(#[from] AccountError),

    #[error(transparent)]
    CSVError(#[from] csv::Error),
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    if !cli.input.is_file() {
        eprintln!("Error: '{}' is not a valid file", cli.input.display());
        std::process::exit(1);
    }

    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(cli.input)
        .expect("failed to read from CSV");

    let mut accounts: HashMap<u16, Account> = HashMap::new();

    for tx in csv_reader.deserialize::<Transaction>() {
        let tx = tx.expect("the transaction is not valid!");

        let account = accounts
            .entry(tx.client)
            .or_insert_with(|| Account::new(tx.client));

        account.process_transaction(tx)?;
    }

    let mut csv_writer = csv::WriterBuilder::new().from_writer(std::io::stdout());

    accounts.values().for_each(|account| {
        csv_writer
            .serialize(account)
            .expect("failed to serialize account")
    });

    Ok(())
}
