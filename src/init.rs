use super::{cash, database, database::Database, student};

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

    simple_logger::SimpleLogger::new().env().init().unwrap();

    // 记录调试日志：初始化开始
    debug!("开始系统初始化流程");

    // 初始化学生模块（包含 UID 持久化机制）
    student::init()
        .context("学生模块初始化失败（系统启动期间）")
        .map_err(|e| {
            warn!("学生模块初始化失败: {:?}", e);
            e.to_string()
        })?;

    cash::init()
        .context("现金模块初始化失败（系统启动期间）")
        .map_err(|e| {
            warn!("现金模块初始化失败: {:?}", e);
            e.to_string()
        })?;

    // 初始化数据库模块
    let database = database::init()
        .context("数据库模块初始化失败（启动期间）")
        .map_err(|e| {
            warn!("数据库初始化失败: {:?}", e);
            e.to_string()
        })?;

    // 记录信息日志：系统就绪
    info!("系统初始化成功完成");
    Ok(database)
}
