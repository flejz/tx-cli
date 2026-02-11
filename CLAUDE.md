# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Run tests for a specific module
cargo test rules::check_not_frozen

# Run the CLI
cargo run -- <input.csv>

# Check code without building
cargo check

# Format code
cargo fmt

# Lint with clippy
cargo clippy
```

## Architecture

This is a Rust CLI tool that processes financial transactions from CSV files and outputs account states.

### Core Flow

1. **Input**: CSV file with transactions (type, client, tx, amount)
2. **Processing**: Each transaction is validated by rule functions and applied via Account methods
3. **Output**: Account states serialized to CSV on stdout

### Module Structure

- **`model/`** - Data types
  - `Transaction` - Input transaction with type, client ID, transaction ID, and optional amount
  - `Account` - Client account state with available/held balances, frozen flag, and optimized storage
  - `AccountError` - Error type wrapping rule violations
  - Uses `rust_decimal::Decimal` for precise financial calculations

- **`rules.rs`** - Pure validator functions
  - `check_not_frozen` - Validates account is not frozen
  - `check_sufficient_funds` - Validates sufficient available balance
  - `require_amount` - Validates transaction has an amount (for deposit/withdrawal)
  - `get_deposit_amount` - Finds deposit by tx ID and returns its amount
  - `check_dispute_exists` - Validates a dispute exists for the given tx ID

### Key Patterns

- **Separation of concerns**: Rules are pure validators, Account has private operation methods
- **Optimized storage**: Account stores `deposits: HashMap<u32, Decimal>` and `disputes: HashSet<u32>` instead of full transaction history
- **Optional amount**: `Transaction.amount` is `Option<Decimal>` since only deposit/withdrawal require it
- All validators return `Result<T, RuleError>` for composable error handling
- Custom `Serialize` implementation on `Account` normalizes decimal output and maps `frozen` to `locked`

### Account Operations (private methods)

| Method | Description |
|--------|-------------|
| `deposit` | Increases available balance, stores in deposits map |
| `withdrawal` | Decreases available balance (validates sufficient funds) |
| `dispute` | Moves amount from available to held, records in disputes set |
| `resolve` | Moves amount from held back to available, removes from disputes |
| `chargeback` | Removes held amount, freezes account, removes from disputes |

### Transaction Types

| Type | Requires Amount | References Prior TX |
|------|-----------------|---------------------|
| deposit | Yes | No |
| withdrawal | Yes | No |
| dispute | No | Yes (deposit) |
| resolve | No | Yes (deposit + dispute) |
| chargeback | No | Yes (deposit + dispute) |

### Error Types

| Error | Description |
|-------|-------------|
| `AccountFrozen` | Operation attempted on frozen account |
| `InsuficientFunds` | Withdrawal exceeds available balance |
| `MissingAmount` | Deposit/withdrawal missing required amount |
| `DepositNotFound` | Dispute/resolve/chargeback references non-existent deposit |
| `TrasactionNotOnDispute` | Resolve/chargeback on non-disputed transaction |
