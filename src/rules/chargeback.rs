use crate::{
    model::{Account, Transaction, TransactionType},
    rules::RuleError,
};

/// Applies a chargeback on a disputed deposit, removing the held funds and freezing the account.
///
/// Both a deposit and a dispute with id `tx.tx` must exist on the account.
/// Unlike a resolve, a chargeback is irreversible — the held amount is permanently removed
/// and the account is frozen from further activity.
///
/// # Errors
///
/// - Returns [`RuleError::AccountFrozen`] if the account is frozen.
/// - Returns [`RuleError::DepositNotFound`] if no deposit with id `tx.tx` exists on the account.
/// - Returns [`RuleError::TrasactionNotOnDispute`] if no dispute with id `tx.tx` exists on the account.
///
/// # Panics
///
/// Panics if `tx.type` is not [`TransactionType::Chargeback`].
pub fn chargeback(account: &mut Account, tx: &Transaction) -> Result<(), RuleError> {
    if !matches!(tx.r#type, TransactionType::Chargeback) {
        panic!("failed to chargeback transaction: {tx:?}");
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
    account.frozen = true;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal::Decimal;

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
    fn chargeback_removes_held_and_freezes_account() {
        let mut account = account_with_dispute(1, 1, Decimal::from(100));
        chargeback(
            &mut account,
            &make_tx(TransactionType::Chargeback, 1, 1, Decimal::ZERO),
        )
        .unwrap();
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.available, Decimal::ZERO);
        assert!(account.frozen);
    }

    #[test]
    fn chargeback_decreases_total() {
        let mut account = account_with_dispute(1, 1, Decimal::from(100));
        let total_before = account.total();
        chargeback(
            &mut account,
            &make_tx(TransactionType::Chargeback, 1, 1, Decimal::ZERO),
        )
        .unwrap();
        assert_eq!(account.total(), total_before - Decimal::from(100));
        assert!(account.frozen);
    }

    #[test]
    fn chargeback_deposit_not_found_returns_error() {
        let mut account = account_with_dispute(1, 1, Decimal::from(100));
        let result = chargeback(
            &mut account,
            &make_tx(TransactionType::Chargeback, 1, 99, Decimal::ZERO),
        );
        assert!(matches!(result, Err(RuleError::DepositNotFound(99))));
    }

    #[test]
    fn chargeback_deposit_not_found_does_not_modify_account() {
        let mut account = account_with_dispute(1, 1, Decimal::from(100));
        chargeback(
            &mut account,
            &make_tx(TransactionType::Chargeback, 1, 99, Decimal::ZERO),
        )
        .unwrap_err();
        assert_eq!(account.held, Decimal::from(100));
        assert!(!account.frozen);
    }

    #[test]
    fn chargeback_without_dispute_returns_error() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account
            .transactions
            .push(make_tx(TransactionType::Deposit, 1, 1, Decimal::from(100)));
        let result = chargeback(
            &mut account,
            &make_tx(TransactionType::Chargeback, 1, 1, Decimal::ZERO),
        );
        assert!(matches!(result, Err(RuleError::TrasactionNotOnDispute(1))));
    }

    #[test]
    fn chargeback_without_dispute_does_not_modify_account() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account
            .transactions
            .push(make_tx(TransactionType::Deposit, 1, 1, Decimal::from(100)));
        chargeback(
            &mut account,
            &make_tx(TransactionType::Chargeback, 1, 1, Decimal::ZERO),
        )
        .unwrap_err();
        assert_eq!(account.available, Decimal::from(100));
        assert_eq!(account.held, Decimal::ZERO);
        assert!(!account.frozen);
    }

    #[test]
    fn chargeback_on_frozen_account_returns_error() {
        let mut account = account_with_dispute(1, 1, Decimal::from(100));
        account.frozen = true;
        let result = chargeback(
            &mut account,
            &make_tx(TransactionType::Chargeback, 1, 1, Decimal::ZERO),
        );
        assert!(matches!(result, Err(RuleError::AccountFrozen)));
        assert_eq!(account.held, Decimal::from(100));
        assert!(account.frozen);
    }

    #[test]
    #[should_panic]
    fn chargeback_panics_on_wrong_type() {
        let mut account = account_with_dispute(1, 1, Decimal::from(100));
        chargeback(
            &mut account,
            &make_tx(TransactionType::Deposit, 1, 1, Decimal::ZERO),
        )
        .unwrap();
    }
}
