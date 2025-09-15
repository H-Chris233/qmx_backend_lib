use super::{cash, database::Database, student};

use crate::error::Result;
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
    student::save_uid()?;

    cash::save_uid()?;

    // 保存数据库内容
    database.save()?;

    info!("成功完成数据库持久化操作");
    Ok(())
}
