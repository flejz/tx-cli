use crate::{
    model::{Account, Transaction, TransactionType},
    rules::RuleError,
};

/// Resolves a disputed transaction, moving the held amount back to available funds.
///
/// Both a dispute and a deposit with id `tx.tx` must exist on the account.
///
/// # Errors
///
/// - Returns [`RuleError::DepositNotFound`] if no deposit with id `tx.tx` exists on the account.
/// - Returns [`RuleError::TrasactionNotOnDispute`] if no dispute with id `tx.tx` exists on the account.
///
/// # Panics
///
/// Panics if `tx.type` is not [`TransactionType::Resolve`].
pub fn resolve(account: &mut Account, tx: &Transaction) -> Result<(), RuleError> {
    if !matches!(tx.r#type, TransactionType::Resolve) {
        panic!("failed to resolve transaction: {tx:?}");
    }

    let amount = account
        .find_transaction(tx.tx, TransactionType::Deposit)
        .ok_or(RuleError::DepositNotFound(tx.tx))?
        .amount;

    account
        .find_transaction(tx.tx, TransactionType::Dispute)
        .ok_or(RuleError::TrasactionNotOnDispute(tx.tx))?;

    account.held -= amount;
    account.available += amount;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tx(r#type: TransactionType, client: u16, tx: u32, amount: f64) -> Transaction {
        Transaction {
            r#type,
            client,
            tx,
            amount,
        }
    }

    /// Returns an account with a deposit and a dispute for `tx_id` already applied,
    /// with balances set to reflect the disputed state (funds in held).
    fn account_with_dispute(client: u16, tx_id: u32, amount: f64) -> Account {
        let mut account = Account::new(client);
        account.held = amount;
        account
            .transactions
            .push(make_tx(TransactionType::Deposit, client, tx_id, amount));
        account
            .transactions
            .push(make_tx(TransactionType::Dispute, client, tx_id, 0.0));
        account
    }

    #[test]
    fn resolve_moves_amount_from_held_to_available() {
        let mut account = account_with_dispute(1, 1, 100.0);
        let total_before = account.total();
        resolve(&mut account, &make_tx(TransactionType::Resolve, 1, 1, 0.0)).unwrap();
        assert_eq!(account.available, 100.0);
        assert_eq!(account.held, 0.0);
        assert_eq!(account.total(), total_before);
    }

    #[test]
    fn resolve_without_dispute_returns_error() {
        let mut account = Account::new(1);
        account.available = 100.0;
        account
            .transactions
            .push(make_tx(TransactionType::Deposit, 1, 1, 100.0));
        let result = resolve(&mut account, &make_tx(TransactionType::Resolve, 1, 1, 0.0));
        assert!(matches!(result, Err(RuleError::TrasactionNotOnDispute(1))));
        assert_eq!(account.available, 100.0);
        assert_eq!(account.held, 0.0);
    }

    #[test]
    fn resolve_deposit_not_found_returns_error() {
        let mut account = account_with_dispute(1, 1, 100.0);
        let result = resolve(&mut account, &make_tx(TransactionType::Resolve, 1, 99, 0.0));
        assert!(matches!(result, Err(RuleError::DepositNotFound(99))));
    }

    #[test]
    #[should_panic]
    fn resolve_panics_on_wrong_type() {
        let mut account = account_with_dispute(1, 1, 100.0);
        resolve(&mut account, &make_tx(TransactionType::Deposit, 1, 1, 0.0)).unwrap();
    }
}
