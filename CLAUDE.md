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
cargo test rules::deposit

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
2. **Processing**: Each transaction is deserialized and routed to the appropriate rule handler
3. **Output**: Account states serialized to CSV on stdout

### Module Structure

- **`model/`** - Data types
  - `Transaction` - Input transaction with type, client ID, transaction ID, and amount
  - `Account` - Client account state with available/held balances and frozen flag
  - Uses `rust_decimal::Decimal` for precise financial calculations

- **`rules/`** - Transaction processing logic (one file per transaction type)
  - `deposit` - Adds to available balance
  - `withdrawal` - Subtracts from available balance (requires sufficient funds)
  - `dispute` - Moves deposited amount from available to held
  - `resolve` - Moves disputed amount from held back to available
  - `chargeback` - Removes held amount and freezes account permanently

### Key Patterns

- Each rule function validates transaction type at runtime (panics on mismatch)
- All rules check for frozen accounts first and return `RuleError::AccountFrozen`
- Dispute/resolve/chargeback reference prior transactions by tx ID
- Account maintains a transaction history in `transactions: Vec<Transaction>`
- Custom `Serialize` implementation on `Account` normalizes decimal output and maps `frozen` to `locked`

### Transaction Types

| Type | Requires Amount | References Prior TX |
|------|-----------------|---------------------|
| deposit | Yes | No |
| withdrawal | Yes | No |
| dispute | No | Yes (deposit) |
| resolve | No | Yes (deposit + dispute) |
| chargeback | No | Yes (deposit + dispute) |
