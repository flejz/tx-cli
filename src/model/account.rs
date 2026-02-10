use core::f64;

use serde::{Serialize, Serializer, ser::SerializeStruct};

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
    pub frozen: bool,

    pub(crate) transactions: Vec<Transaction>,
}

impl Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Account", 5)?;
        state.serialize_field("client", &self.client)?;
        state.serialize_field("available", &self.available)?;
        state.serialize_field("held", &self.held)?;
        state.serialize_field("total", &self.total())?;
        state.serialize_field("locked", &self.frozen)?;
        state.end()
    }
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

    pub fn find_transaction(&self, tx_id: u32, tx_type: TransactionType) -> Option<&Transaction> {
        self.transactions
            .iter()
            .find(move |tx| tx.tx == tx_id && tx.r#type == tx_type)
    }

    pub fn process_transaction(&mut self, tx: Transaction) -> Result<(), AccountError> {
        if self.client != tx.client {
            return Err(AccountError::MismatchingAccounts(self.client, tx.client));
        }

        match &tx.r#type {
            TransactionType::Deposit => rules::deposit(self, &tx),
            TransactionType::Withdrawal => rules::withdrawal(self, &tx),
            TransactionType::Dispute => rules::dispute(self, &tx),
            TransactionType::Resolve => rules::resolve(self, &tx),
            TransactionType::Chargeback => rules::chargeback(self, &tx),
        }
        .expect("failed to process transaction");

        // TODO: implement rules
        self.transactions.push(tx);

        Ok(())
    }
}
