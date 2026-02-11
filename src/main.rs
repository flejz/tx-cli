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

    /// Sort the output per account number ascending
    #[arg(short, long, default_value_t = false)]
    sort: bool,
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

        if let Err(err) = account.process_transaction(tx) {
            // print to stderr so on stdout redirection (>) does not include the error
            eprintln!("{err}");
        }
    }

    let mut csv_writer = csv::WriterBuilder::new().from_writer(std::io::stdout());

    // README:
    // We are collecting here just for the sake of sorting for comparison between the output
    // and the accounts.csv base file
    // This allocation however just allocates pointer references, it does not clone account values
    let mut accounts: Vec<&Account> = accounts.values().collect();
    if cli.sort {
        accounts.sort_by_key(|account| account.client);
    };

    accounts.iter().for_each(|account| {
        csv_writer
            .serialize(account)
            .expect("failed to serialize account")
    });

    Ok(())
}
