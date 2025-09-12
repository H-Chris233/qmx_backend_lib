use super::{cash, database::Database, student};

use anyhow::{Context, Result};
use log::{debug, info};

/// 保存数据库并更新 UID 计数器
///
/// 该函数负责：
/// 1. 保存学生和现金模块的 UID 计数器到文件
/// 2. 保存数据库到磁盘
///
/// 成功时返回 Ok(())，失败时返回错误信息
pub fn save(database: Database) -> Result<()> {
    debug!("开始数据库保存操作");

    // 保存 UID 计数器
    student::save_uid()
        .context("无法将学生UID计数器持久化到存储")?;

    cash::save_uid()
        .context("无法将现金UID计数器持久化到存储")?;

    // 保存数据库内容
    database
        .save()
        .context("无法将数据库持久化到存储")?;

    info!("成功完成数据库持久化操作");
    Ok(())
}
