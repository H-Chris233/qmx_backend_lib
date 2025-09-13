pub mod cash;
pub mod common;
pub mod database;
pub mod init;
pub mod manager;
pub mod save;
pub mod stats;
pub mod student;

// 新的统一API入口
pub use manager::{
    CashBuilder, CashQuery, CashUpdater, FinancialStats, MembershipStatus, QmxManager,
    StudentBuilder, StudentQuery, StudentStats, StudentUpdater, TimePeriod,
};

// 原有API（保持向后兼容）
pub use common::{Database, HasUid};
pub use stats::{DashboardStats, get_dashboard_stats};
