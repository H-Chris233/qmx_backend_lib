use super::{cash, database::Database, student};

use anyhow::{Context, Result};
use log::{debug, info};

/// 保存数据库并更新 UID 计数器
///
/// 该函数负责：
/// 1. 保存 UID 计数器到文件
/// 2. 保存数据库到磁盘
///
/// 成功时返回 Ok(())，失败时返回包含上下文信息的错误字符串
pub fn save(database: Database) -> Result<(), String> {
    // 记录调试日志：开始保存操作
    debug!("开始数据库保存操作");

    // 保存 UID 计数器并转换错误类型为字符串
    student::save_uid()
        .context("无法将学生UID计数器持久化到存储")
        .map_err(|e| e.to_string())?;

    cash::save_uid()
        .context("无法将现金UID计数器持久化到存储")
        .map_err(|e| e.to_string())?;

    // 保存数据库内容并转换错误类型为字符串
    database
        .save()
        .context("无法将学生数据库持久化到存储")
        .map_err(|e| e.to_string())?;

    // 记录信息日志：保存成功
    info!("成功完成数据库持久化操作");
    Ok(())
}
