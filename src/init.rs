use super::{cash, database, database::Database, student};

use anyhow::{Context, Result};
use log::{debug, info};

/// 初始化系统核心组件
///
/// 执行以下关键初始化步骤：
/// 1. 学生模块初始化（包含 UID 计数器）
/// 2. 现金模块初始化（包含 UID 计数器）
/// 3. 数据库模块初始化（从持久化存储加载）
///
/// 成功返回完整数据库实例，失败返回错误信息
pub fn init() -> Result<Database> {
    debug!("开始系统初始化流程");

    // 初始化学生模块（包含 UID 持久化机制）
    student::init().context("学生模块初始化失败（系统启动期间）")?;

    // 初始化现金模块（包含 UID 持久化机制）
    cash::init().context("现金模块初始化失败（系统启动期间）")?;

    // 初始化数据库模块
    let database = database::init().context("数据库模块初始化失败（启动期间）")?;

    info!("系统初始化成功完成");
    Ok(database)
}
