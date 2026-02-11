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
/// - Returns [`RuleError::AccountFrozen`] if the account is frozen.
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

    if account.frozen {
        return Err(RuleError::AccountFrozen);
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

    /// Returns an account with a deposit and a dispute for `tx_id` already applied,
    /// with balances set to reflect the disputed state (funds in held).
    fn account_with_dispute(client: u16, tx_id: u32, amount: Decimal) -> Account {
        let mut account = Account::new(client);
        account.held = amount;
        account
            .transactions
            .push(make_tx(TransactionType::Deposit, client, tx_id, amount));
        account.transactions.push(make_tx(
            TransactionType::Dispute,
            client,
            tx_id,
            Decimal::ZERO,
        ));
        account
    }

    #[test]
    fn resolve_moves_amount_from_held_to_available() {
        let mut account = account_with_dispute(1, 1, Decimal::from(100));
        let total_before = account.total();
        resolve(
            &mut account,
            &make_tx(TransactionType::Resolve, 1, 1, Decimal::ZERO),
        )
        .unwrap();
        assert_eq!(account.available, Decimal::from(100));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total(), total_before);
    }

    #[test]
    fn resolve_without_dispute_returns_error() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account
            .transactions
            .push(make_tx(TransactionType::Deposit, 1, 1, Decimal::from(100)));
        let result = resolve(
            &mut account,
            &make_tx(TransactionType::Resolve, 1, 1, Decimal::ZERO),
        );
        assert!(matches!(result, Err(RuleError::TrasactionNotOnDispute(1))));
        assert_eq!(account.available, Decimal::from(100));
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn resolve_deposit_not_found_returns_error() {
        let mut account = account_with_dispute(1, 1, Decimal::from(100));
        let result = resolve(
            &mut account,
            &make_tx(TransactionType::Resolve, 1, 99, Decimal::ZERO),
        );
        assert!(matches!(result, Err(RuleError::DepositNotFound(99))));
    }

    #[test]
    fn resolve_on_frozen_account_returns_error() {
        let mut account = account_with_dispute(1, 1, Decimal::from(100));
        account.frozen = true;
        let result = resolve(
            &mut account,
            &make_tx(TransactionType::Resolve, 1, 1, Decimal::ZERO),
        );
        assert!(matches!(result, Err(RuleError::AccountFrozen)));
        assert_eq!(account.held, Decimal::from(100));
        assert_eq!(account.available, Decimal::ZERO);
    }

    #[test]
    #[should_panic]
    fn resolve_panics_on_wrong_type() {
        let mut account = account_with_dispute(1, 1, Decimal::from(100));
        resolve(
            &mut account,
            &make_tx(TransactionType::Deposit, 1, 1, Decimal::ZERO),
        )
        .unwrap();
    }
}
