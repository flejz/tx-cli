use std::collections::{HashMap, HashSet};

use rust_decimal::Decimal;
use serde::{Serialize, Serializer, ser::SerializeStruct};

use super::{Transaction, TransactionType};
use crate::rules::{self, RuleError};

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("account do not match. actual = {0}, incoming = {1} ")]
    MismatchingAccounts(u16, u16),

    #[error(transparent)]
    RuleViolation(#[from] RuleError),
}

#[derive(Debug, Default)]
pub struct Account {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub frozen: bool,

    pub(crate) deposits: HashMap<u32, Decimal>,
    pub(crate) disputes: HashSet<u32>,
}

impl Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Account", 5)?;
        state.serialize_field("client", &self.client)?;
        state.serialize_field("available", &self.available.normalize().to_string())?;
        state.serialize_field("held", &self.held.normalize().to_string())?;
        state.serialize_field("total", &self.total().normalize().to_string())?;
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

    /// Account available + held amounts
    pub fn total(&self) -> Decimal {
        self.available + self.held
    }

    /// Return deposit amount if found
    pub fn find_deposit(&self, tx_id: &u32) -> Option<&Decimal> {
        self.deposits.get(tx_id)
    }

    /// Return dispute transaction when found
    pub fn has_dispute(&self, tx_id: &u32) -> Option<&u32> {
        self.disputes.get(tx_id)
    }

    /// Increases the available balance by the given amount.
    fn deposit(&mut self, tx: &Transaction) -> Result<(), RuleError> {
        let amount = rules::require_amount(tx.tx, tx.amount)?;
        self.available += amount;
        self.deposits.insert(tx.tx, amount);
        Ok(())
    }

    /// Decreases the available balance by the given amount.
    fn withdrawal(&mut self, tx: &Transaction) -> Result<(), RuleError> {
        let amount = rules::require_amount(tx.tx, tx.amount)?;
        rules::check_sufficient_funds(self, amount)?;
        self.available -= amount;
        Ok(())
    }

    /// Moves funds from available to held for a disputed transaction.
    fn dispute(&mut self, tx: &Transaction) -> Result<(), RuleError> {
        let amount = *rules::get_deposit_amount(self, &tx.tx)?;
        self.available -= amount;
        self.held += amount;
        self.disputes.insert(tx.tx);
        Ok(())
    }

    /// Moves funds from held back to available, resolving a dispute.
    fn resolve(&mut self, tx: &Transaction) -> Result<(), RuleError> {
        let amount = *rules::get_deposit_amount(self, &tx.tx)?;
        rules::check_dispute_exists(self, &tx.tx)?;
        self.held -= amount;
        self.available += amount;
        self.disputes.remove(&tx.tx);
        Ok(())
    }

    /// Removes held funds and freezes the account permanently.
    fn chargeback(&mut self, tx: &Transaction) -> Result<(), RuleError> {
        let amount = *rules::get_deposit_amount(self, &tx.tx)?;
        rules::check_dispute_exists(self, &tx.tx)?;
        self.held -= amount;
        self.frozen = true;
        self.disputes.remove(&tx.tx);
        Ok(())
    }

    pub fn process_transaction(&mut self, tx: Transaction) -> Result<(), AccountError> {
        if self.client != tx.client {
            return Err(AccountError::MismatchingAccounts(self.client, tx.client));
        }
        rules::check_not_frozen(self)?;

        match &tx.r#type {
            TransactionType::Deposit => {
                self.deposit(&tx)?;
            }
            TransactionType::Withdrawal => {
                self.withdrawal(&tx)?;
            }
            TransactionType::Dispute => {
                self.dispute(&tx)?;
            }
            TransactionType::Resolve => {
                self.resolve(&tx)?;
            }
            TransactionType::Chargeback => {
                self.chargeback(&tx)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tx(
        r#type: TransactionType,
        client: u16,
        tx: u32,
        amount: Option<Decimal>,
    ) -> Transaction {
        Transaction {
            r#type,
            client,
            tx,
            amount,
        }
    }

    fn make_deposit(client: u16, tx: u32, amount: Decimal) -> Transaction {
        make_tx(TransactionType::Deposit, client, tx, Some(amount))
    }

    fn make_withdrawal(client: u16, tx: u32, amount: Decimal) -> Transaction {
        make_tx(TransactionType::Withdrawal, client, tx, Some(amount))
    }

    fn make_dispute(client: u16, tx: u32) -> Transaction {
        make_tx(TransactionType::Dispute, client, tx, None)
    }

    fn make_resolve(client: u16, tx: u32) -> Transaction {
        make_tx(TransactionType::Resolve, client, tx, None)
    }

    fn make_chargeback(client: u16, tx: u32) -> Transaction {
        make_tx(TransactionType::Chargeback, client, tx, None)
    }

    mod deposit_tests {
        use super::*;

        #[test]
        fn deposit_increases_available() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            assert_eq!(account.available, Decimal::from(100));
        }

        #[test]
        fn deposit_does_not_affect_held() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(50)))
                .unwrap();
            assert_eq!(account.held, Decimal::ZERO);
        }

        #[test]
        fn deposit_total_equals_available_when_no_held() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(75)))
                .unwrap();
            assert_eq!(account.total(), account.available);
        }

        #[test]
        fn multiple_deposits_accumulate() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(1)))
                .unwrap();
            account
                .process_transaction(make_deposit(1, 2, Decimal::from(2)))
                .unwrap();
            account
                .process_transaction(make_deposit(1, 3, Decimal::from(3)))
                .unwrap();
            assert_eq!(account.available, Decimal::from(6));
        }

        #[test]
        fn deposit_on_frozen_account_returns_error() {
            let mut account = Account::new(1);
            account.frozen = true;
            let result = account.process_transaction(make_deposit(1, 1, Decimal::from(100)));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(RuleError::AccountFrozen))
            ));
            assert_eq!(account.available, Decimal::ZERO);
        }

        #[test]
        fn deposit_missing_amount_returns_error() {
            let mut account = Account::new(1);
            let result =
                account.process_transaction(make_tx(TransactionType::Deposit, 1, 1, None));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(RuleError::MissingAmount(1)))
            ));
            assert_eq!(account.available, Decimal::ZERO);
        }
    }

    mod withdrawal_tests {
        use super::*;

        #[test]
        fn withdrawal_decreases_available() {
            let mut account = Account::new(1);
            account.available = Decimal::from(100);
            account
                .process_transaction(make_withdrawal(1, 1, Decimal::from(40)))
                .unwrap();
            assert_eq!(account.available, Decimal::from(60));
            assert_eq!(account.held, Decimal::ZERO);
        }

        #[test]
        fn withdrawal_exact_balance_succeeds() {
            let mut account = Account::new(1);
            account.available = Decimal::from(50);
            account
                .process_transaction(make_withdrawal(1, 1, Decimal::from(50)))
                .unwrap();
            assert_eq!(account.available, Decimal::ZERO);
        }

        #[test]
        fn withdrawal_insufficient_funds_returns_error_and_does_not_modify_account() {
            let mut account = Account::new(1);
            account.available = Decimal::from(10);
            let result = account.process_transaction(make_withdrawal(1, 1, Decimal::from(20)));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(RuleError::InsuficientFunds))
            ));
            assert_eq!(account.available, Decimal::from(10));
        }

        #[test]
        fn withdrawal_on_frozen_account_returns_error() {
            let mut account = Account::new(1);
            account.available = Decimal::from(100);
            account.frozen = true;
            let result = account.process_transaction(make_withdrawal(1, 1, Decimal::from(40)));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(RuleError::AccountFrozen))
            ));
            assert_eq!(account.available, Decimal::from(100));
        }

        #[test]
        fn withdrawal_missing_amount_returns_error() {
            let mut account = Account::new(1);
            account.available = Decimal::from(100);
            let result =
                account.process_transaction(make_tx(TransactionType::Withdrawal, 1, 1, None));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(RuleError::MissingAmount(1)))
            ));
            assert_eq!(account.available, Decimal::from(100));
        }
    }

    mod dispute_tests {
        use super::*;

        #[test]
        fn dispute_moves_amount_from_available_to_held() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            let total_before = account.total();
            account.process_transaction(make_dispute(1, 1)).unwrap();
            assert_eq!(account.available, Decimal::ZERO);
            assert_eq!(account.held, Decimal::from(100));
            assert_eq!(account.total(), total_before);
        }

        #[test]
        fn dispute_unknown_tx_returns_error() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            let result = account.process_transaction(make_dispute(1, 99));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(RuleError::DepositNotFound(99)))
            ));
            assert_eq!(account.available, Decimal::from(100));
            assert_eq!(account.held, Decimal::ZERO);
        }

        #[test]
        fn dispute_on_frozen_account_returns_error() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            account.frozen = true;
            let result = account.process_transaction(make_dispute(1, 1));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(RuleError::AccountFrozen))
            ));
            assert_eq!(account.available, Decimal::from(100));
            assert_eq!(account.held, Decimal::ZERO);
        }
    }

    mod resolve_tests {
        use super::*;

        #[test]
        fn resolve_moves_amount_from_held_to_available() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            account.process_transaction(make_dispute(1, 1)).unwrap();
            let total_before = account.total();
            account.process_transaction(make_resolve(1, 1)).unwrap();
            assert_eq!(account.available, Decimal::from(100));
            assert_eq!(account.held, Decimal::ZERO);
            assert_eq!(account.total(), total_before);
        }

        #[test]
        fn resolve_without_dispute_returns_error() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            let result = account.process_transaction(make_resolve(1, 1));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(
                    RuleError::TrasactionNotOnDispute(1)
                ))
            ));
            assert_eq!(account.available, Decimal::from(100));
            assert_eq!(account.held, Decimal::ZERO);
        }

        #[test]
        fn resolve_deposit_not_found_returns_error() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            account.process_transaction(make_dispute(1, 1)).unwrap();
            let result = account.process_transaction(make_resolve(1, 99));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(RuleError::DepositNotFound(99)))
            ));
        }

        #[test]
        fn resolve_on_frozen_account_returns_error() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            account.process_transaction(make_dispute(1, 1)).unwrap();
            account.frozen = true;
            let result = account.process_transaction(make_resolve(1, 1));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(RuleError::AccountFrozen))
            ));
            assert_eq!(account.held, Decimal::from(100));
            assert_eq!(account.available, Decimal::ZERO);
        }
    }

    mod chargeback_tests {
        use super::*;

        #[test]
        fn chargeback_removes_held_and_freezes_account() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            account.process_transaction(make_dispute(1, 1)).unwrap();
            account.process_transaction(make_chargeback(1, 1)).unwrap();
            assert_eq!(account.held, Decimal::ZERO);
            assert_eq!(account.available, Decimal::ZERO);
            assert!(account.frozen);
        }

        #[test]
        fn chargeback_decreases_total() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            account.process_transaction(make_dispute(1, 1)).unwrap();
            let total_before = account.total();
            account.process_transaction(make_chargeback(1, 1)).unwrap();
            assert_eq!(account.total(), total_before - Decimal::from(100));
            assert!(account.frozen);
        }

        #[test]
        fn chargeback_deposit_not_found_returns_error() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            account.process_transaction(make_dispute(1, 1)).unwrap();
            let result = account.process_transaction(make_chargeback(1, 99));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(RuleError::DepositNotFound(99)))
            ));
        }

        #[test]
        fn chargeback_deposit_not_found_does_not_modify_account() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            account.process_transaction(make_dispute(1, 1)).unwrap();
            let _ = account.process_transaction(make_chargeback(1, 99));
            assert_eq!(account.held, Decimal::from(100));
            assert!(!account.frozen);
        }

        #[test]
        fn chargeback_without_dispute_returns_error() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            let result = account.process_transaction(make_chargeback(1, 1));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(
                    RuleError::TrasactionNotOnDispute(1)
                ))
            ));
        }

        #[test]
        fn chargeback_without_dispute_does_not_modify_account() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            let _ = account.process_transaction(make_chargeback(1, 1));
            assert_eq!(account.available, Decimal::from(100));
            assert_eq!(account.held, Decimal::ZERO);
            assert!(!account.frozen);
        }

        #[test]
        fn chargeback_on_frozen_account_returns_error() {
            let mut account = Account::new(1);
            account
                .process_transaction(make_deposit(1, 1, Decimal::from(100)))
                .unwrap();
            account.process_transaction(make_dispute(1, 1)).unwrap();
            account.frozen = true;
            let result = account.process_transaction(make_chargeback(1, 1));
            assert!(matches!(
                result,
                Err(AccountError::RuleViolation(RuleError::AccountFrozen))
            ));
            assert_eq!(account.held, Decimal::from(100));
            assert!(account.frozen);
        }
    }

    mod account_error_tests {
        use super::*;

        #[test]
        fn mismatching_accounts_returns_error() {
            let mut account = Account::new(1);
            let result = account.process_transaction(make_deposit(2, 1, Decimal::from(100)));
            assert!(matches!(
                result,
                Err(AccountError::MismatchingAccounts(1, 2))
            ));
        }
    }
}
