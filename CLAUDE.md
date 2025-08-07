# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust backend library (`qmx_backend_lib`) for managing student and cash records. The library provides functionality for:
- Student management with personal information, class enrollment, and lesson tracking
- Cash transaction management with student associations
- Persistent storage using JSON files
- UID generation and management for both students and cash records

## Build System

This project uses Rust with Cargo:
- Build: `cargo build`
- Run tests: `cargo test`
- Check code: `cargo check`
- Format code: `cargo fmt`
- Lint code: `cargo clippy`

## Architecture

### Core Modules
- **`lib.rs`**: Main library entry point that exports all modules
- **`database.rs`**: Central database container that manages both student and cash databases
- **`init.rs`**: System initialization coordinator that initializes all modules in proper order
- **`save.rs`**: Persistence coordinator that saves all data and UID counters
- **`student.rs`**: Student management with Person struct, Student trait, and StudentDatabase
- **`cash.rs`**: Cash management with Cash struct and CashDatabase

### Key Components

**Person/Student System** (`src/student.rs`):
- `Person` struct: Main student entity with UID, age, name, class, lessons, rings (measurements), notes, and cash
- `Student` trait: Builder-style interface for modifying student data
- `StudentDatabase`: BTreeMap-based container for persistent student storage
- Atomic UID counter with file-based persistence
- **Default trait**: Implemented for Person and StudentDatabase
- **Delete operations**: `remove()` for single record deletion and `remove_batch()` for batch deletion
- **Utility methods**: `is_empty()` for checking if database is empty

**Cash System** (`src/cash.rs`):
- `Cash` struct: Individual cash record with own UID and optional student association
- `CashDatabase`: BTreeMap-based container for persistent cash storage
- Separate UID counter for cash records
- **Default trait**: Implemented for CashDatabase
- **Delete operations**: `remove()` for single record deletion and `remove_batch()` for batch deletion
- **Utility methods**: `is_empty()` for checking if database is empty

**Database Container** (`src/database.rs`):
- `Database` struct: Runtime container holding both student and cash databases
- Provides unified save/load interface
- Loads from JSON files in `./data/` directory

### Data Persistence
- All data stored as JSON files in `./data/` directory
- Student database: `./data/student_database.json`
- Cash database: `./data/cash_database.json`
- UID counters: `./data/uid_counter` and `./data/cash_uid_counter`
- Automatic file creation if missing

### Initialization Sequence
1. Initialize student module (load UID counter)
2. Initialize cash module (load UID counter)
3. Initialize database container (load both databases from JSON)

## Dependencies
- `anyhow`: Error handling with context
- `log`: Logging infrastructure
- `serde` + `serde_json`: JSON serialization/deserialization

## File Paths
The application expects data files in `./data/` relative to the working directory:
- `./data/student_database.json`
- `./data/cash_database.json`
- `./data/uid_counter`
- `./data/cash_uid_counter`

## Code Patterns
- Extensive use of `with_context()` for error context
- Builder-style method chaining for data modification
- Atomic counters for UID generation
- BTreeMap for ordered data storage
- Separation of concerns between runtime containers and persistent storage
- **Safety documentation**: Comprehensive documentation for unsafe functions
- **Trait implementations**: Default trait for key structs
- **Error handling**: Proper error propagation and logging

## Testing
- **Unit tests**: 8 comprehensive unit tests covering all major functionality
- **Test coverage**: Includes creation, modification, deletion, and JSON operations
- **Test commands**: `cargo test` to run all tests
- **Code quality**: `cargo clippy` for linting, all warnings resolved