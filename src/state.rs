use std::collections::HashMap;

use crate::{
    account::Account,
    transaction::{self, Amount, ClientId, Transaction, TransactionId},
};

struct TransactionRecord {
    client_id: ClientId,
    amount: Amount,
    disputed: bool,
}

pub struct InMemoryState {
    transactions: HashMap<TransactionId, TransactionRecord>,
    accounts: HashMap<ClientId, Account>,
}

impl InMemoryState {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            accounts: HashMap::new(),
        }
    }
}

pub trait Engine {
    // Applys transaction, updating internal engine state.
    fn apply_transaction(&mut self, transaction: Transaction);

    fn accounts(&self) -> impl Iterator<Item = (&ClientId, &Account)>;
}

impl Engine for InMemoryState {
    fn apply_transaction(&mut self, transaction: Transaction) {
        let account = self
            .accounts
            .entry(transaction.client_id)
            .or_insert_with(Account::new);

        // The assumption here is that no operation can be performed on a frozen
        // account and manual intervention is required to unfreeze.
        if account.frozen {
            return;
        }

        match transaction.r#type {
            transaction::TransactionType::Deposit => {
                // Assuming that only deposits are disputed
                // so they are the only transactions we keep a reference of.
                self.transactions.insert(
                    transaction.id,
                    TransactionRecord {
                        client_id: transaction.client_id,
                        amount: transaction.amount,
                        disputed: false,
                    },
                );
                account.deposit(transaction.amount);
            }
            transaction::TransactionType::Withdrawal => {
                // We could opt to warn about failed transactions here.
                let _ = account.withdraw(transaction.amount);
            }
            transaction::TransactionType::Dispute => {
                // Non existent transactions are ignored
                if let Some(record) = self.transactions.get_mut(&transaction.id) {
                    // A transaction already disputed should not be disputed again.
                    if record.disputed {
                        return;
                    }

                    // If the recorded transaction is not for the same client
                    // as the disputed transaction, it is assumed this in 
                    // error on the partner's side and ignored
                    if record.client_id != transaction.client_id {
                        return;
                    }

                    record.disputed = true;
                    account.hold(record.amount);
                }
            }
            transaction::TransactionType::Resolve => {
                if let Some(record) = self.transactions.get_mut(&transaction.id) {
                    // Transactions that are not disputed cannot be resolved.
                    if !record.disputed {
                        return;
                    }
                    // If the recorded (disputed) transaction is not for the same client
                    // as the resolve transaction, it is assumed this in 
                    // error on the partner's side and ignored
                    if record.client_id != transaction.client_id {
                        return;
                    }

                    record.disputed = false;
                    account.release(record.amount);
                }
            }
            transaction::TransactionType::Chargeback => {
                if let Some(record) = self.transactions.get_mut(&transaction.id) {
                    // Transactions that are not disputed cannot be charged back.
                    if !record.disputed {
                        return;
                    }
                    
                    // If the recorded (disputed) transaction is not for the same client
                    // as the chargeback transaction, it is assumed this in 
                    // error on the partner's side and ignored
                    if record.client_id != transaction.client_id {
                        return;
                    }

                    record.disputed = false;
                    account.chargeback(record.amount);
                }
            }
        }
    }

    fn accounts(&self) -> impl Iterator<Item = (&ClientId, &Account)> {
        self.accounts.iter()
    }
}
