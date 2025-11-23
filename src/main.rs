use std::{env, fs::File};

use csv::{Error as CsvError, Reader, ReaderBuilder, Trim};
use thiserror::Error;

use accounting_demo::account::{Account, AccountError};
use accounting_demo::account_manager::{process_transaction, AccountManager};
use accounting_demo::types::{ClientId, Transaction};

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error{"0"}]
    Account(#[from] AccountError),

    #[error{"0"}]
    CsvReader(#[from] CsvError),

    #[error("Usage: cargo run -- <TRANSACTIONS_CSV>")]
    InvalidArgs,
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;

fn read_csv_path() -> ApplicationResult<String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(ApplicationError::InvalidArgs);
    }

    Ok(args[1].trim().to_string())
}

fn get_csv_reader(path: &str) -> ApplicationResult<Reader<File>> {
    Ok(ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_path(path)?)
}

fn write_accounts(accounts: Vec<(ClientId, Account)>) {
    println!("client,available,held,total,locked");
    accounts.iter().for_each(|(id, account)| {
        println!(
            "{},{:.4},{:.4},{:.4},{}",
            id,
            account.available(),
            account.disputed(),
            account.total(),
            account.locked()
        )
    });
}

fn main() -> ApplicationResult<()> {
    let csv_path = read_csv_path()?;
    let mut csv_reader = get_csv_reader(&csv_path)?;

    let mut account_manager = AccountManager::new();
    for result in csv_reader.deserialize() {
        let tx: Transaction = result?;
        let _ = process_transaction(&mut account_manager, tx);
    }

    let accounts = account_manager.accounts();
    write_accounts(accounts);

    Ok(())
}
