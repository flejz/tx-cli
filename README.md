# tx-cli

A Rust CLI tool for processing financial transactions from CSV files.

## Overview

`tx-cli` reads a CSV file containing financial transactions (deposits, withdrawals, disputes, resolves, and chargebacks), processes them against client accounts, and outputs the final account states to stdout.

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/tx-cli`.

## Usage

```bash
tx-cli <input.csv>
```

### Input Format

The input CSV file should have the following columns:

| Column | Type | Description |
|--------|------|-------------|
| type | string | Transaction type: `deposit`, `withdrawal`, `dispute`, `resolve`, `chargeback` |
| client | u16 | Client ID |
| tx | u32 | Transaction ID |
| amount | decimal | Amount (required for deposit/withdrawal, ignored for others) |

Example input:
```csv
type,client,tx,amount
deposit,1,1,100.0
deposit,2,2,50.0
withdrawal,1,3,25.0
dispute,1,1,
resolve,1,1,
deposit,1,4,75.5
dispute,1,4,
chargeback,1,4,
```

### Output Format

The output is a CSV written to stdout with the following columns:

| Column | Type | Description |
|--------|------|-------------|
| client | u16 | Client ID |
| available | decimal | Available funds |
| held | decimal | Held funds (under dispute) |
| total | decimal | Total funds (available + held) |
| locked | bool | Whether the account is frozen |

Example output:
```csv
client,available,held,total,locked
1,75,0,75,true
2,50,0,50,false
```

## Test Run
To test run, use provided ai-generated `transactions.csv`.

```bash
# run program
cargo run -- transactions.csv

# run without stderr output
cargo run -- transactions.csv 2>/dev/null
```

The `accounts_expected.csv` file has also been provided with the expected output. Run the following command to make sure tx-cli output matches `accounts_expected.csv`.

```bash
# validate correctness
diff -u accounts_expected.csv <(cargo run -- transactions.csv --sort 2>/dev/null)
```

## Transaction Types

### Deposit
Adds funds to the client's available balance.
- Requires: `amount`
- Fails if: account is frozen, amount is missing

### Withdrawal
Removes funds from the client's available balance.
- Requires: `amount`
- Fails if: account is frozen, insufficient funds, amount is missing

### Dispute
Places a prior deposit under dispute, moving its amount from available to held.
- References: a prior deposit by `tx` ID
- Fails if: account is frozen, deposit not found

### Resolve
Resolves a dispute, moving the held amount back to available.
- References: a prior deposit that is under dispute
- Fails if: account is frozen, deposit not found, transaction not under dispute

### Chargeback
Finalizes a dispute by removing the held funds and permanently freezing the account.
- References: a prior deposit that is under dispute
- Fails if: account is frozen, deposit not found, transaction not under dispute

## Design

- **Precision**: Uses `rust_decimal::Decimal` with 4 decimal places for financial calculations
- **Validation**: Pure validator functions in `rules.rs` separate business logic from state mutations
- **Storage**: Optimized to store only deposits (`HashMap<tx_id, amount>`) and disputes (`HashSet<tx_id>`)
- **Error Handling**: Comprehensive error types for all failure modes

## Development

```bash
# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```

## License

See LICENSE file for details.
