use super::{database::Database, student};

use anyhow::{Context, Result};
use log::{debug, info};

/// 保存数据库并更新 UID 计数器
///
/// 该函数负责：
/// 1. 保存 UID 计数器到文件
/// 2. 保存数据库到磁盘
///
/// 成功时返回 Ok(())，失败时返回包含上下文信息的错误字符串
#[tauri::command]
pub fn save(database: Database) -> Result<(), String> {
    // 记录调试日志：开始保存操作
    debug!("Starting database save operation");

    // 保存 UID 计数器并转换错误类型为字符串
    student::save_uid()
        .context("Failed to persist UID counter to storage")
        .map_err(|e| e.to_string())?;

    // 保存数据库内容并转换错误类型为字符串
    database
        .save()
        .context("Failed to persist student database to storage")
        .map_err(|e| e.to_string())?;

    // 记录信息日志：保存成功
    info!("Successfully completed database persistence operation");
    Ok(())
}
