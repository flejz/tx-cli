use crate::{
    model::{Account, Transaction, TransactionType},
    rules::RuleError,
};

/// Applies a withdrawal transaction to an account, decreasing its available funds.
///
/// # Errors
///
/// - Returns [`RuleError::AccountFrozen`] if the account is frozen.
/// - Returns [`RuleError::InsuficientFunds`] if `account.available` is less than `tx.amount`.
///
/// # Panics
///
/// Panics if `tx.type` is not [`TransactionType::Withdrawal`].
pub fn withdrawal(account: &mut Account, tx: &Transaction) -> Result<(), RuleError> {
    if !matches!(tx.r#type, TransactionType::Withdrawal) {
        panic!("failed to withdraw transaction: {tx:?}");
    }

    if account.frozen {
        return Err(RuleError::AccountFrozen);
    }

    if account.available < tx.amount {
        return Err(RuleError::InsuficientFunds);
    }

    account.available -= tx.amount;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;

    fn make_withdrawal(client: u16, tx: u32, amount: Decimal) -> Transaction {
        Transaction {
            r#type: TransactionType::Withdrawal,
            client,
            tx,
            amount,
        }
    }

    fn account_with_funds(client: u16, available: Decimal) -> Account {
        let mut account = Account::new(client);
        account.available = available;
        account
    }

    #[test]
    fn withdrawal_decreases_available() {
        let mut account = account_with_funds(1, Decimal::from(100));
        withdrawal(&mut account, &make_withdrawal(1, 1, Decimal::from(40))).unwrap();
        assert_eq!(account.available, Decimal::from(60));
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn withdrawal_exact_balance_succeeds() {
        let mut account = account_with_funds(1, Decimal::from(50));
        withdrawal(&mut account, &make_withdrawal(1, 1, Decimal::from(50))).unwrap();
        assert_eq!(account.available, Decimal::ZERO);
    }

    #[test]
    fn withdrawal_insufficient_funds_returns_error_and_does_not_modify_account() {
        let mut account = account_with_funds(1, Decimal::from(10));
        let result = withdrawal(&mut account, &make_withdrawal(1, 1, Decimal::from(20)));
        assert!(matches!(result, Err(RuleError::InsuficientFunds)));
        assert_eq!(account.available, Decimal::from(10));
    }

    #[test]
    fn withdrawal_on_frozen_account_returns_error() {
        let mut account = account_with_funds(1, Decimal::from(100));
        account.frozen = true;
        let result = withdrawal(&mut account, &make_withdrawal(1, 1, Decimal::from(40)));
        assert!(matches!(result, Err(RuleError::AccountFrozen)));
        assert_eq!(account.available, Decimal::from(100));
    }

    #[test]
    #[should_panic]
    fn withdrawal_panics_on_wrong_type() {
        let mut account = account_with_funds(1, Decimal::from(100));
        let tx = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Decimal::from(10),
        };
        withdrawal(&mut account, &tx).unwrap();
    }
}
