
use super::student::StudentDatabase;
use super::cash::CashDatabase;

use anyhow::{Context, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};

/// 运行时数据库容器，不持久化到磁盘
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
    pub student: StudentDatabase,
    pub cash: CashDatabase,
}

impl Database {
    /// 仅运行时构造方法
    pub fn new(student: StudentDatabase, cash: CashDatabase) -> Self {
        Self { student, cash }
    }

    /// 显式保存两个子数据库
    pub fn save(&self) -> Result<()> {
        info!("开始持久化所有数据库");
        
        // 分别保存两个子数据库
        self.student.save()
            .with_context(|| "学生数据库持久化失败")?;
        self.cash.save()
            .with_context(|| "现金数据库持久化失败")?;
        
        debug!("所有数据库已成功保存");
        Ok(())
    }
}

/// 初始化运行时数据库容器
pub fn init() -> Result<Database> {
    info!("正在初始化运行时数据库");

    // 分别加载两个子数据库
    let student_db = StudentDatabase::read_from("./data/student_database.json")
        .with_context(|| "加载学生数据库失败")?;
    
    let cash_db = CashDatabase::read_from("./data/cash_database.json")
        .with_context(|| "加载现金数据库失败")?;

    info!("运行时数据库初始化完成");
    Ok(Database::new(student_db, cash_db))
}

pub fn save(db: &Database) -> Result<()> {
    db.save()
}
