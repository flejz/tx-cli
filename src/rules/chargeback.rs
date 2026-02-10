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
