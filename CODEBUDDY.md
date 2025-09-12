# CODEBUDDY.md

## Project Overview
**Repository:** qmx_backend_lib  
**Type:** Rust library (v1.2.0, edition 2024)  
**Purpose:** Backend library for QMX student and cash management system

## Common Commands
- **Build:** `cargo build`
- **Release build:** `cargo build --release`
- **Run tests:** `cargo test`
- **Run specific test:** `cargo test <test_name>`
- **Format code:** `cargo fmt --all`
- **Lint code:** `cargo clippy --all-targets --all-features -- -D warnings`
- **Generate docs:** `cargo doc --open`

## Project Structure

### Source Modules (`src/`)
- **`lib.rs`** - Library entry point, exports all modules and re-exports stats functionality
- **`common.rs`** - ✨NEW✨ Generic Database<T> trait and HasUid trait for unified database operations
- **`student.rs`** - Student management with UID system, profiles, database operations, and membership management
- **`cash.rs`** - Cash/financial records with installment support and database operations  
- **`database.rs`** - Combined database wrapper for student and cash data
- **`stats.rs`** - Dashboard statistics computation and aggregation
- **`init.rs`** - System initialization utilities (now uses anyhow::Result)
- **`save.rs`** - Data persistence utilities (now uses anyhow::Result)

### Test Suite (`tests/`)
- **Total tests:** 116 comprehensive tests across all modules
- **`student_tests.rs`** - Student functionality, UID management, database operations
- **`cash_tests.rs`** - Cash records, installments, database operations
- **`database_tests.rs`** - Combined database functionality
- **`stats_tests.rs`** - Statistics computation and edge cases
- **`integration_tests.rs`** - Cross-module integration testing

## Data Architecture

### Student System
- **Student struct:** UID, age, name, phone, lesson_left, class, subject, rings (scores), notes, membership dates
- **Enums:** Class (TenTry, Month, Year, Others), Subject (Shooting, Archery, Others)
- **StudentDatabase:** BTreeMap-based storage with CRUD operations, batch processing, JSON persistence
- **UID Management:** Atomic counter with file persistence (`./data/uid_counter`)

### Cash System  
- **Cash struct:** Own UID, optional student_id, amount, note, optional Installment data
- **Installment:** Plan details, status, frequency, due dates
- **CashDatabase:** Similar to StudentDatabase with specialized installment operations
- **Features:** Plan grouping, overdue detection, next installment generation, plan cancellation

### Database Integration
- **Database struct:** Combines StudentDatabase and CashDatabase
- **Initialization:** Creates `./data` directory, loads or creates sub-databases
- **Persistence:** JSON files in `./data/` directory

### Statistics
- **DashboardStats:** Aggregated metrics from both student and cash data
- **Metrics:** Student totals, revenue/expense, score averages, active courses

## Data Storage
- **Directory:** `./data/` (auto-created on init)
- **Student data:** `./data/student_database.json`
- **Cash data:** `./data/cash_database.json`  
- **UID counters:** `./data/uid_counter`, `./data/cash_uid_counter`

## Dependencies
- **Runtime:** anyhow, chrono, log, serde, serde_json
- **Development:** tempfile (for test isolation)

## Development Notes
- **Error handling:** Uses `anyhow` crate with rich context throughout
- **Logging:** Integrated via `log` crate (requires logger setup in consuming applications)
- **Testing:** Comprehensive coverage with isolated test environments
- **Concurrency:** Atomic operations for UID management

## Recent Changes (v1.2.0)
- ✨ **Major Code Quality Improvements**: Created generic Database<T> trait, eliminated 200+ lines of duplicate code
- 🎯 **Unified Error Handling**: Migrated all modules to use anyhow::Result for consistent error management
- 🛠️ **API Design Enhancements**: rings() method now returns &[f64] instead of &Vec<f64>, following Rust best practices
- 👥 **Membership Management**: Added comprehensive membership system with date tracking and status validation
- 🚀 **Performance Optimizations**: Reduced codebase by 71 lines while adding functionality
- ✅ **Test Improvements**: All 116 tests passing, fixed test isolation issues
- 📚 **Documentation Overhaul**: Completely updated API.md and README.md with current examples and best practices
- 🔒 **Backward Compatibility**: All existing code continues to work without modifications

## API Reference
- See `README.md` for quick start guide and basic usage examples
- See `API.md` for comprehensive API documentation with up-to-date examples
- All documentation is now synchronized with v1.2.0 codebase