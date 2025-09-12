<system-reminder>
This is a reminder that your todo list is currently empty. DO NOT mention this to the user explicitly because they are already aware. If you are working on tasks that would benefit from a todo list please use the TodoWrite tool to create one. If not, please feel free to ignore. Again do not mention this message to the user.

</system-reminder>

# CODEBUDDY.md

Repo: qmx_backend_lib (Rust library)

Common commands
- Build: cargo build
- Release build: cargo build --release
- Run tests: cargo test
- Run single test: cargo test <module_or_test_name>
- Format: cargo fmt --all
- Lint: cargo clippy --all-targets --all-features -- -D warnings
- Docs: cargo doc --open

Project structure and architecture
- Library entry: src/lib.rs exposes modules: cash, student, database, init, save, stats; re-exports stats::{DashboardStats, get_dashboard_stats}
- Data model:
  - student.rs defines Student with UID, profile fields, class/subject enums, rings scores, notes; StudentDatabase wraps BTreeMap<u64, Student> with CRUD, batch ops, JSON persistence to ./data/student_database.json; UID persistence at ./data/uid_counter with AtomicU64 and init/save helpers
  - cash.rs defines Cash records (own uid, optional student_id, amount, note, optional Installment with plan/status/frequency/due_date). CashDatabase mirrors StudentDatabase patterns, persists to ./data/cash_database.json; cash UID counter at ./data/cash_uid_counter; helpers for installments (plan grouping, overdue detection, next installment generation, cancel plan)
- Aggregation:
  - database.rs defines Database { student: StudentDatabase, cash: CashDatabase } and init() that ensures ./data exists, loads or creates sub-databases, and provides Database::save() to persist both
- Statistics:
  - stats.rs computes DashboardStats from StudentDatabase and CashDatabase: totals (students, revenue/expense), average/max score, active_courses (non-Others classes); exported via get_dashboard_stats

Development notes
- Data directory: ./data is created on init; JSON stores student_database.json, cash_database.json, uid_counter, cash_uid_counter
- Logging: uses log crate; integrate env_logger or similar in binaries that consume this lib
- Error handling: anyhow with rich context throughout persistence and init paths
- Testing: comprehensive test suite with 58 tests across all modules in tests/ directory; use tempfile crate for test isolation

Cross-file references
- Module exports: src/lib.rs:1-9
- Database loading/saving and init: src/database.rs:36-90
- Student UID init/persist: src/student.rs:181-234, 211-221
- Cash installment helpers: src/cash.rs:320-457
- Stats computation: src/stats.rs:16-75
- Test modules: tests/student_tests.rs, tests/cash_tests.rs, tests/stats_tests.rs, tests/database_tests.rs, tests/integration_tests.rs

README/API highlights
- See README.md for brief overview; API.md contains detailed module docs and examples which may be partially outdated versus current code (e.g., type names Person vs Student, i32 vs i64 in Cash). Prefer source over API.md when discrepancies occur.
