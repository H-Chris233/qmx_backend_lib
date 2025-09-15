# CODEBUDDY.md

## Project Overview
**Repository:** qmx_backend_lib  
**Type:** Rust library (v2.2.0, edition 2024)  
**Purpose:** Backend library for QMX student and cash management system

## Common Commands
- **Build:** `cargo build`
- **Release build:** `cargo build --release`
- **Run tests:** `cargo test -- --test-threads=1`
- **Format code:** `cargo fmt`
- **Lint code:** `cargo clippy`
- **Generate docs:** `cargo doc --open`

## Project Structure

### Source Modules (`src/`)
- **`lib.rs`** - Library entry point
- **`common.rs`** - Generic Database<T> trait and HasUid trait
- **`student.rs`** - Student management with membership system
- **`cash.rs`** - Cash/financial records with installment support  
- **`database.rs`** - Combined database wrapper
- **`stats.rs`** - Dashboard statistics computation
- **`init.rs`** - System initialization utilities
- **`save.rs`** - Data persistence utilities

### Test Suite (`tests/`)
- **Total tests:** 171 comprehensive tests
- **`student_tests.rs`** - Student functionality and database operations
- **`cash_tests.rs`** - Cash records and installments
- **`database_tests.rs`** - Combined database functionality
- **`stats_tests.rs`** - Statistics computation
- **`integration_tests.rs`** - Cross-module integration testing

## Data Storage
- **Directory:** `./data/` (auto-created on init)
- **Student data:** `./data/student_database.json`
- **Cash data:** `./data/cash_database.json`  
- **UID counters:** `./data/uid_counter`, `./data/cash_uid_counter`

## Dependencies
- **Runtime:** anyhow, chrono, log, serde, serde_json
- **Development:** tempfile (for test isolation)