use core::f64;

use super::{Transaction, TransactionType};
use crate::rules::{self};

#[derive(Debug, thiserror::Error)]
pub enum AccountError {}

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

    pub fn push_transaction(&mut self, tx: Transaction) -> Result<(), AccountError> {
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
