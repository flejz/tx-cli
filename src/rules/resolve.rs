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
/// - Returns [`RuleError::TrasactionNotOnDispute`] if no dispute with id `tx.tx` exists on the account.
/// - Returns [`RuleError::TrasactionNotFound`] if no deposit with id `tx.tx` exists on the account.
///
/// # Panics
///
/// Panics if `tx.type` is not [`TransactionType::Resolve`].
pub fn resolve(account: &mut Account, tx: &Transaction) -> Result<(), RuleError> {
    if !matches!(tx.r#type, TransactionType::Resolve) {
        panic!("failed to resolve transaction: {tx:?}");
    }

    account
        .find_transaction(tx.tx, TransactionType::Dispute)
        .ok_or(RuleError::TrasactionNotOnDispute(tx.tx))?;

    let amount = account
        .find_transaction(tx.tx, TransactionType::Deposit)
        .ok_or(RuleError::TrasactionNotFound(tx.tx))?
        .amount;

    account.held -= amount;
    account.available += amount;

    Ok(())
}
