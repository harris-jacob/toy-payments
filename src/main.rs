mod account;
mod state;
mod transaction;
use std::{env, fs::File};

use csv::Trim;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use transaction::*;

use crate::state::{Engine, InMemoryState};

#[derive(Debug, Deserialize)]
struct InputRecord {
    r#type: String,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
}

#[derive(Debug, Serialize)]
struct OutputRecord {
    client_id: u16,
    available: Decimal,
    held: Decimal,
    locked: bool,
}

impl InputRecord {
    pub fn transaction(self) -> anyhow::Result<transaction::Transaction> {
        Ok(transaction::Transaction {
            r#type: TransactionType::new(self.r#type)?,
            id: TransactionId::new(self.tx),
            client_id: ClientId::new(self.client),
            amount: Amount::new(self.amount.unwrap_or_default()),
        })
    }
}

fn main() -> anyhow::Result<()> {
    let filename = env::args().nth(1).expect("csv file required as arg");

    let file = File::open(filename)?;
    let mut state = InMemoryState::new();

    let mut reader = csv::ReaderBuilder::new().trim(Trim::All).from_reader(file);
    let mut writer = csv::WriterBuilder::new().from_writer(std::io::stdout());

    for result in reader.deserialize() {
        let record: InputRecord = result?;

        let transaction = record.transaction()?;
        state.apply_transaction(transaction);
    }

    for (client_id, account) in state.accounts() {
        let _ = writer.serialize(OutputRecord {
            client_id: client_id.inner(),
            available: account.available.display(),
            held: account.held.display(),
            locked: account.frozen,
        });
    }

    Ok(())
}
