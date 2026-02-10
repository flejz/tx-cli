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
/// Returns [`RuleError::TrasactionNotFound`] if no deposit with id `tx.tx` exists on the account.
///
/// # Panics
///
/// Panics if `tx.type` is not [`TransactionType::Dispute`].
pub fn dispute(account: &mut Account, tx: &Transaction) -> Result<(), RuleError> {
    if !matches!(tx.r#type, TransactionType::Dispute) {
        panic!("failed to dispute transaction: {tx:?}");
    }

    let amount = account
        .find_transaction(tx.tx, TransactionType::Deposit)
        .ok_or(RuleError::TrasactionNotFound(tx.tx))?
        .amount;

    account.available -= amount;
    account.held += amount;

    Ok(())
}
