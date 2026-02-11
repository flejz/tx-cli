use crate::{
    model::{Account, Transaction, TransactionType},
    rules::RuleError,
};

/// Applies a dispute to an account, moving the amount of the referenced deposit from
/// available funds to held funds.
///
/// The disputed transaction is looked up by `tx.tx` and must be a prior deposit.
///
/// # Errors
///
/// - Returns [`RuleError::AccountFrozen`] if the account is frozen.
/// - Returns [`RuleError::DepositNotFound`] if no deposit with id `tx.tx` exists on the account.
///
/// # Panics
///
/// Panics if `tx.type` is not [`TransactionType::Dispute`].
pub fn dispute(account: &mut Account, tx: &Transaction) -> Result<(), RuleError> {
    if !matches!(tx.r#type, TransactionType::Dispute) {
        panic!("failed to dispute transaction: {tx:?}");
    }

    if account.frozen {
        return Err(RuleError::AccountFrozen);
    }

    let amount = account
        .find_transaction(tx.tx, TransactionType::Deposit)
        .ok_or(RuleError::DepositNotFound(tx.tx))?
        .amount;

    account.available -= amount;
    account.held += amount;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;

    fn make_tx(r#type: TransactionType, client: u16, tx: u32, amount: Decimal) -> Transaction {
        Transaction {
            r#type,
            client,
            tx,
            amount,
        }
    }

    /// Returns an account that has a deposit of `amount` with tx id `tx_id` already applied.
    fn account_with_deposit(client: u16, tx_id: u32, amount: Decimal) -> Account {
        let mut account = Account::new(client);
        account.available = amount;
        account
            .transactions
            .push(make_tx(TransactionType::Deposit, client, tx_id, amount));
        account
    }

    #[test]
    fn dispute_moves_amount_from_available_to_held() {
        let mut account = account_with_deposit(1, 1, Decimal::from(100));
        let total_before = account.total();
        let tx = make_tx(TransactionType::Dispute, 1, 1, Decimal::ZERO);
        dispute(&mut account, &tx).unwrap();
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.held, Decimal::from(100));
        assert_eq!(account.total(), total_before);
    }

    #[test]
    fn dispute_unknown_tx_returns_error() {
        let mut account = account_with_deposit(1, 1, Decimal::from(100));
        let tx = make_tx(TransactionType::Dispute, 1, 99, Decimal::ZERO);
        let result = dispute(&mut account, &tx);
        assert!(matches!(result, Err(RuleError::DepositNotFound(99))));
        assert_eq!(account.available, Decimal::from(100));
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn dispute_on_frozen_account_returns_error() {
        let mut account = account_with_deposit(1, 1, Decimal::from(100));
        account.frozen = true;
        let tx = make_tx(TransactionType::Dispute, 1, 1, Decimal::ZERO);
        let result = dispute(&mut account, &tx);
        assert!(matches!(result, Err(RuleError::AccountFrozen)));
        assert_eq!(account.available, Decimal::from(100));
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    #[should_panic]
    fn dispute_panics_on_wrong_type() {
        let mut account = account_with_deposit(1, 1, Decimal::from(100));
        let tx = make_tx(TransactionType::Deposit, 1, 1, Decimal::ZERO);
        dispute(&mut account, &tx).unwrap();
    }
}
