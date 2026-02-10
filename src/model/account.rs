use core::f64;

use super::{Transaction, TransactionType};
use crate::rules::{self};

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("account do not match. actual = {0}, incoming = {1} ")]
    MismatchingAccounts(u16, u16),
}

#[derive(Debug, Default)]
pub struct Account {
    pub client: u16,
    pub available: f64,
    pub held: f64,

    transactions: Vec<Transaction>,
}

impl Account {
    pub fn new(client: u16) -> Self {
        Self {
            client,
            ..Default::default()
        }
    }

    pub fn total(&self) -> f64 {
        self.available + self.held
    }

    pub fn is_locked(&self) -> bool {
        unimplemented!()
    }

    pub fn find_transaction(&self, tx_id: u32, tx_type: TransactionType) -> Option<&Transaction> {
        self.transactions
            .iter()
            .find(move |tx| tx.tx == tx_id && tx.r#type == tx_type)
    }

    pub fn push_transaction(&mut self, tx: Transaction) -> Result<(), AccountError> {
        if self.client != tx.client {
            return Err(AccountError::MismatchingAccounts(self.client, tx.client));
        }

        match &tx.r#type {
            TransactionType::Deposit => rules::deposit(self, &tx),
            TransactionType::Withdrawal => (),
            TransactionType::Dispute => (),
            TransactionType::Resolve => (),
            TransactionType::Chargeback => (),
        }

        // TODO: implement rules
        self.transactions.push(tx);

        Ok(())
    }
}
