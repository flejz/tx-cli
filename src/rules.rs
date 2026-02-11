use rust_decimal::Decimal;

use crate::model::Account;

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

    #[error("missing amount for transaction: {0}")]
    MissingAmount(u32),
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

/// Checks that the transaction has an amount and returns it.
///
/// # Errors
///
/// Returns [`RuleError::MissingAmount`] if the amount is `None`.
pub fn require_amount(tx_id: u32, amount: Option<Decimal>) -> Result<Decimal, RuleError> {
    amount.ok_or(RuleError::MissingAmount(tx_id))
}

/// Finds a deposit transaction by ID and returns its amount.
///
/// # Errors
///
/// Returns [`RuleError::DepositNotFound`] if no deposit with the given `tx_id` exists.
pub fn get_deposit_amount<'a>(
    account: &'a Account,
    tx_id: &'a u32,
) -> Result<&'a Decimal, RuleError> {
    account
        .find_deposit(tx_id)
        .ok_or(RuleError::DepositNotFound(*tx_id))
}

/// Checks that a dispute exists for the given transaction ID.
///
/// # Errors
///
/// Returns [`RuleError::TrasactionNotOnDispute`] if no dispute with the given `tx_id` exists.
pub fn check_dispute_exists(account: &Account, tx_id: &u32) -> Result<(), RuleError> {
    let _ = get_deposit_amount(account, tx_id)?;
    account
        .has_dispute(tx_id)
        .ok_or(RuleError::TrasactionNotOnDispute(*tx_id))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    mod require_amount_tests {
        use super::*;

        #[test]
        fn amount_present_returns_value() {
            let result = require_amount(1, Some(Decimal::from(100)));
            assert_eq!(result.unwrap(), Decimal::from(100));
        }

        #[test]
        fn amount_missing_returns_error() {
            let result = require_amount(1, None);
            assert!(matches!(result, Err(RuleError::MissingAmount(1))));
        }
    }

    mod get_deposit_amount_tests {
        use super::*;

        #[test]
        fn deposit_found_returns_amount() {
            let mut account = Account::new(1);
            account.deposits.insert(1, Decimal::from(100));
            assert_eq!(
                get_deposit_amount(&account, &1).unwrap(),
                &Decimal::from(100)
            );
        }

        #[test]
        fn deposit_not_found_returns_error() {
            let account = Account::new(1);
            assert!(matches!(
                get_deposit_amount(&account, &99),
                Err(RuleError::DepositNotFound(99))
            ));
        }
    }

    mod check_dispute_exists_tests {
        use super::*;

        #[test]
        fn dispute_exists_passes() {
            let mut account = Account::new(1);
            account.deposits.insert(1, Decimal::from(100));
            account.disputes.insert(1);
            assert!(check_dispute_exists(&account, &1).is_ok());
        }

        #[test]
        fn deposit_not_found_returns_error() {
            let account = Account::new(1);
            assert!(matches!(
                check_dispute_exists(&account, &1),
                Err(RuleError::DepositNotFound(1))
            ));
        }

        #[test]
        fn dispute_missing_returns_error() {
            let mut account = Account::new(1);
            account.deposits.insert(1, Decimal::from(100));
            assert!(matches!(
                check_dispute_exists(&account, &1),
                Err(RuleError::TrasactionNotOnDispute(1))
            ));
        }
    }
}
