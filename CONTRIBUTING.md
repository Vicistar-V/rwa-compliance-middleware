# Contributing to ARCM

## Prerequisites

- Rust (stable) with `wasm32-unknown-unknown` target
- Install via: `rustup target add wasm32-unknown-unknown`

## Getting Started

```bash
git clone <repo>
cd arcm-compliance-middleware
cargo build
cargo test
```

## Project Structure

- `shared/types` — Core data structures and enums
- `contracts/` — Soroban smart contracts:
  - `gateway` — SEP-0008 entry point
  - `jurisdiction` — Rule engine
  - `kyc_oracle` — KYC/AML oracle
  - `enforcement` — Lock/clawback engine
  - `audit` — Compliance ledger
  - `governance` — Rule governance
  - `credentials` — Credential registry
  - `geo` — Country resolver

## Common Commands

| Command | Description |
|---------|-------------|
| `make build` | Build all contracts |
| `make test` | Run all tests |
| `make lint` | Run clippy |
| `make fmt` | Format code |
| `make doc` | Build docs |
| `make clean` | Clean build artifacts |
| `make all` | Build, lint, test, doc |

## Guidelines

- All tests must pass before merging
- Follow existing code patterns (no_std, Soroban SDK patterns)
- Add tests for new functionality
- Run `cargo clippy` and `cargo fmt` before committing
