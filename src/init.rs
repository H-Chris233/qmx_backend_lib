use super::{database, database::Database, student};

use anyhow::{Context, Result};
use log::{debug, info, warn};

/// 初始化系统核心组件
///
/// 执行以下关键初始化步骤：
/// 1. 学生模块初始化（包含 UID 计数器）
/// 2. 数据库模块初始化（从持久化存储加载）
///
/// 成功返回完整数据库实例，失败返回带上下文的错误字符串
pub fn init() -> Result<Database, String> {
    // 记录调试日志：初始化开始
    debug!("Starting system initialization sequence");

    // 初始化学生模块（包含 UID 持久化机制）
    student::init()
        .context("Student module initialization failed during system startup")
        .map_err(|e| {
            warn!("Student module initialization failed: {:?}", e);
            e.to_string()
        })?;

    // 初始化数据库模块
    let database = database::init()
        .context("Database module initialization failed during startup")
        .map_err(|e| {
            warn!("Database initialization failed: {:?}", e);
            e.to_string()
        })?;

    // 记录信息日志：系统就绪
    info!("System initialization completed successfully");
    Ok(database)
}
