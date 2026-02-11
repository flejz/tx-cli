use rust_decimal::Decimal;

use crate::model::{Account, TransactionType};

#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    #[error("account is frozen")]
    AccountFrozen,

    #[error("insufficient funds")]
    InsuficientFunds,

    #[error("deposit not found: {0}")]
    DepositNotFound(u32),

    #[error("transaction not being disputed: {0}")]
    TrasactionNotOnDispute(u32),
}

/// Checks that the account is not frozen.
///
/// # Errors
///
/// Returns [`RuleError::AccountFrozen`] if the account is frozen.
pub fn check_not_frozen(account: &Account) -> Result<(), RuleError> {
    if account.frozen {
        return Err(RuleError::AccountFrozen);
    }
    Ok(())
}

/// Checks that the account has sufficient available funds for the given amount.
///
/// # Errors
///
/// Returns [`RuleError::InsuficientFunds`] if `account.available` is less than `amount`.
pub fn check_sufficient_funds(account: &Account, amount: Decimal) -> Result<(), RuleError> {
    if account.available < amount {
        return Err(RuleError::InsuficientFunds);
    }
    Ok(())
}

/// Finds a deposit transaction by ID and returns its amount.
///
/// # Errors
///
/// Returns [`RuleError::DepositNotFound`] if no deposit with the given `tx_id` exists.
pub fn get_deposit_amount(account: &Account, tx_id: u32) -> Result<Decimal, RuleError> {
    account
        .find_transaction(tx_id, TransactionType::Deposit)
        .map(|tx| tx.amount)
        .ok_or(RuleError::DepositNotFound(tx_id))
}

/// Checks that a dispute exists for the given transaction ID.
///
/// # Errors
///
/// Returns [`RuleError::TrasactionNotOnDispute`] if no dispute with the given `tx_id` exists.
pub fn check_dispute_exists(account: &Account, tx_id: u32) -> Result<(), RuleError> {
    let _ = get_deposit_amount(account, tx_id)?;
    account
        .find_transaction(tx_id, TransactionType::Dispute)
        .ok_or(RuleError::TrasactionNotOnDispute(tx_id))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Transaction;

    fn make_tx(r#type: TransactionType, client: u16, tx: u32, amount: Decimal) -> Transaction {
        Transaction {
            r#type,
            client,
            tx,
            amount,
        }
    }

    mod check_not_frozen_tests {
        use super::*;

        #[test]
        fn active_account_passes() {
            let account = Account::new(1);
            assert!(check_not_frozen(&account).is_ok());
        }

        #[test]
        fn frozen_account_returns_error() {
            let mut account = Account::new(1);
            account.frozen = true;
            assert!(matches!(
                check_not_frozen(&account),
                Err(RuleError::AccountFrozen)
            ));
        }
    }

    mod check_sufficient_funds_tests {
        use super::*;

        #[test]
        fn sufficient_funds_passes() {
            let mut account = Account::new(1);
            account.available = Decimal::from(100);
            assert!(check_sufficient_funds(&account, Decimal::from(50)).is_ok());
        }

        #[test]
        fn exact_funds_passes() {
            let mut account = Account::new(1);
            account.available = Decimal::from(100);
            assert!(check_sufficient_funds(&account, Decimal::from(100)).is_ok());
        }

        #[test]
        fn insufficient_funds_returns_error() {
            let mut account = Account::new(1);
            account.available = Decimal::from(50);
            assert!(matches!(
                check_sufficient_funds(&account, Decimal::from(100)),
                Err(RuleError::InsuficientFunds)
            ));
        }
    }

    mod get_deposit_amount_tests {
        use super::*;

        #[test]
        fn deposit_found_returns_amount() {
            let mut account = Account::new(1);
            account
                .transactions
                .push(make_tx(TransactionType::Deposit, 1, 1, Decimal::from(100)));
            assert_eq!(get_deposit_amount(&account, 1).unwrap(), Decimal::from(100));
        }

        #[test]
        fn deposit_not_found_returns_error() {
            let account = Account::new(1);
            assert!(matches!(
                get_deposit_amount(&account, 99),
                Err(RuleError::DepositNotFound(99))
            ));
        }

        #[test]
        fn non_deposit_tx_not_found() {
            let mut account = Account::new(1);
            account.transactions.push(make_tx(
                TransactionType::Withdrawal,
                1,
                1,
                Decimal::from(100),
            ));
            assert!(matches!(
                get_deposit_amount(&account, 1),
                Err(RuleError::DepositNotFound(1))
            ));
        }
    }

    mod check_dispute_exists_tests {
        use super::*;

        #[test]
        fn dispute_exists_passes() {
            let mut account = Account::new(1);
            account
                .transactions
                .push(make_tx(TransactionType::Deposit, 1, 1, Decimal::from(100)));
            account
                .transactions
                .push(make_tx(TransactionType::Dispute, 1, 1, Decimal::ZERO));
            assert!(check_dispute_exists(&account, 1).is_ok());
        }

        #[test]
        fn deposit_not_found_returns_error() {
            let account = Account::new(1);
            assert!(matches!(
                check_dispute_exists(&account, 1),
                Err(RuleError::DepositNotFound(1))
            ));
        }

        #[test]
        fn dispute_missing_returns_error() {
            let mut account = Account::new(1);
            account
                .transactions
                .push(make_tx(TransactionType::Deposit, 1, 1, Decimal::from(100)));
            assert!(matches!(
                check_dispute_exists(&account, 1),
                Err(RuleError::TrasactionNotOnDispute(1))
            ));
        }
    }
}
