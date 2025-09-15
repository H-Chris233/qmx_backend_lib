//! # QMX Backend Library
//!
//! 一个用 Rust 编写的高性能学生管理和财务管理库，专为教育培训机构设计。
//! 提供完整的学生信息管理、现金流记录、分期付款处理、会员管理和统计分析功能。
//!
//! ## 特性
//!
//! - 🎓 **学生管理** - 完整的学生档案管理，包含课程、成绩、会员信息
//! - 💰 **财务管理** - 现金流跟踪、收支记录、财务统计
//! - 📅 **分期付款** - 灵活的分期付款计划管理，支持多种付款频率
//! - 👥 **会员系统** - 会员期限管理、状态检查、到期提醒
//! - 📊 **统计分析** - 仪表板统计、收入分析、学员表现统计
//! - 💾 **数据持久化** - 原子性文件操作，数据安全可靠
//! - 🔒 **类型安全** - 完全利用 Rust 的类型系统，编译时错误检查
//! - ⚡ **高性能** - 零成本抽象，内存安全，无垃圾回收
//!
//! ## 快速开始
//!
//! ### 基本使用 (v1 API)
//!
//! ```rust
//! use qmx_backend_lib::*;
//! use chrono::{Utc, Duration};
//!
//! # fn main() -> anyhow::Result<()> {
//! // 初始化系统
//! let mut db = init::init()?;
//!
//! // 创建学生
//! let mut student = student::Student::new();
//! student
//!     .set_name("张三".to_string())
//!     .set_age(18)
//!     .set_class(student::Class::TenTry)
//!     .set_subject(student::Subject::Shooting)
//!     .add_ring(9.5);
//!
//! // 设置会员期限
//! student.set_membership_dates(
//!     Some(Utc::now()),
//!     Some(Utc::now() + Duration::days(365))
//! );
//!
//! let student_id = student.uid();
//! db.student.insert(student);
//!
//! // 添加收入记录
//! let mut cash = cash::Cash::new(Some(student_id));
//! cash.set_cash(1000);
//! cash.set_note(Some("学费收入".to_string()));
//! db.cash.insert(cash);
//!
//! // 生成统计报告
//! let stats = get_dashboard_stats(&db.student, &db.cash)?;
//! println!("学生总数: {}", stats.total_students);
//! println!("总收入: {}", stats.total_revenue);
//!
//! // 保存数据
//! save::save(db)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 现代化 API (v2 API)
//!
//! ```rust
//! use qmx_backend_lib::*;
//! use chrono::{Utc, Duration};
//!
//! # fn main() -> anyhow::Result<()> {
//! // 初始化管理器
//! let manager = QmxManager::new(true)?; // auto_save = true
//!
//! // 使用 Builder 模式创建学生
//! let student_builder = StudentBuilder::new("李四".to_string(), 20)
//!     .class(student::Class::Month)
//!     .subject(student::Subject::Archery);
//!
//! let student_id = manager.create_student(student_builder)?;
//!
//! // 添加成绩
//! let updater = StudentUpdater::new()
//!     .add_ring(8.5)
//!     .add_ring(9.0);
//! manager.update_student(student_id, updater)?;
//!
//! // 添加收入记录
//! let cash_builder = CashBuilder::new(1500)
//!     .student_id(student_id)
//!     .note("月卡费用".to_string());
//! manager.record_cash(cash_builder)?;
//!
//! // 查询和统计
//! let query = StudentQuery::new();
//! let students = manager.search_students(query)?;
//!
//! let stats = manager.get_dashboard_stats()?;
//! println!("学生数: {}", students.len());
//! println!("总收入: {}", stats.total_revenue);
//!
//! // 保存数据
//! manager.save()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 模块结构
//!
//! - [`student`] - 学生管理和会员系统
//! - [`cash`] - 现金流和分期付款管理
//! - [`database`] - 数据库初始化和持久化
//! - [`stats`] - 统计分析功能
//! - [`manager`] - 现代化统一 API (v2)
//! - [`common`] - 通用数据库 trait 和工具

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
