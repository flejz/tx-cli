use crate::{
    model::{Account, Transaction, TransactionType},
    rules::RuleError,
};

/// Applies a withdrawal transaction to an account, decreasing its available funds.
///
/// # Errors
///
/// Returns [`RuleError::InsuficientFunds`] if `account.available` is less than `tx.amount`.
///
/// # Panics
///
/// Panics if `tx.type` is not [`TransactionType::Withdrawal`].
pub fn withdrawal(account: &mut Account, tx: &Transaction) -> Result<(), RuleError> {
    if !matches!(tx.r#type, TransactionType::Withdrawal) {
        panic!("failed to withdraw transaction: {tx:?}");
    }

    if account.available < tx.amount {
        return Err(RuleError::InsuficientFunds);
    }

    account.available -= tx.amount;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_withdrawal(client: u16, tx: u32, amount: f64) -> Transaction {
        Transaction {
            r#type: TransactionType::Withdrawal,
            client,
            tx,
            amount,
        }
    }

    fn account_with_funds(client: u16, available: f64) -> Account {
        let mut account = Account::new(client);
        account.available = available;
        account
    }

    #[test]
    fn withdrawal_decreases_available() {
        let mut account = account_with_funds(1, 100.0);
        withdrawal(&mut account, &make_withdrawal(1, 1, 40.0)).unwrap();
        assert_eq!(account.available, 60.0);
        assert_eq!(account.held, 0.0);
    }

    #[test]
    fn withdrawal_exact_balance_succeeds() {
        let mut account = account_with_funds(1, 50.0);
        withdrawal(&mut account, &make_withdrawal(1, 1, 50.0)).unwrap();
        assert_eq!(account.available, 0.0);
    }

    #[test]
    fn withdrawal_insufficient_funds_returns_error_and_does_not_modify_account() {
        let mut account = account_with_funds(1, 10.0);
        let result = withdrawal(&mut account, &make_withdrawal(1, 1, 20.0));
        assert!(matches!(result, Err(RuleError::InsuficientFunds)));
        assert_eq!(account.available, 10.0);
    }

    #[test]
    #[should_panic]
    fn withdrawal_panics_on_wrong_type() {
        let mut account = account_with_funds(1, 100.0);
        let tx = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 10.0,
        };
        withdrawal(&mut account, &tx).unwrap();
    }
}
