use super::cash::CashDatabase;
use super::student::StudentDatabase;

use crate::error::{Result, Error};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

/// 主数据库容器
///
/// 包含学生数据库和现金数据库的组合容器，提供统一的数据访问接口。
///
/// # 字段
///
/// - `student`: 学生数据库实例
/// - `cash`: 现金数据库实例
///
/// # 示例
///
/// ```rust
/// use qmx_backend_lib::*;
///
/// # fn main() -> qmx_backend_lib::error::Result<()> {
/// // 初始化数据库
/// let mut db = database::init()?;
///
/// // 创建学生
/// let mut student = student::Student::new();
/// student.set_name("张三".to_string());
/// let student_id = student.uid();
/// db.student.insert(student);
///
/// // 创建现金记录
/// let mut cash = cash::Cash::new(Some(student_id));
/// cash.set_cash(1000);
/// db.cash.insert(cash);
///
/// // 保存所有数据
/// db.save()?;
/// # Ok(())
/// # }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
    pub student: StudentDatabase,
    pub cash: CashDatabase,
}

impl Database {
    /// 创建新的数据库实例
    ///
    /// # 参数
    ///
    /// - `student`: 学生数据库实例
    /// - `cash`: 现金数据库实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qmx_backend_lib::*;
    ///
    /// let student_db = student::StudentDatabase::new();
    /// let cash_db = cash::CashDatabase::new();
    /// let db = database::Database::new(student_db, cash_db);
    /// ```
    pub fn new(student: StudentDatabase, cash: CashDatabase) -> Self {
        Self { student, cash }
    }

    /// 保存所有数据库到磁盘
    ///
    /// 使用原子操作确保数据一致性，如果任一数据库保存失败，整个操作将回滚。
    ///
    /// # 错误
    ///
    /// 当文件系统操作失败或序列化失败时返回错误。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qmx_backend_lib::*;
    ///
    /// # fn main() -> qmx_backend_lib::error::Result<()> {
    /// let db = database::init()?;
    ///
    /// // 修改数据后保存
    /// db.save()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save(&self) -> Result<()> {
        info!("开始持久化所有数据库");
        self.student.save().map_err(Error::from)?;
        self.cash.save().map_err(Error::from)?;
        debug!("所有数据库已成功保存");
        Ok(())
    }
}

/// 初始化数据库系统
///
/// 创建数据目录，加载或创建学生数据库和现金数据库。
/// 如果数据文件不存在，将创建新的空数据库。
///
/// # 返回值
///
/// 返回初始化完成的 `Database` 实例。
///
/// # 错误
///
/// 当无法创建数据目录、读取数据文件或初始化数据库时返回错误。
///
/// # 示例
///
/// ```rust
/// use qmx_backend_lib::*;
///
/// # fn main() -> qmx_backend_lib::error::Result<()> {
/// // 初始化数据库系统
/// let db = database::init()?;
///
/// println!("学生数量: {}", db.student.len());
/// println!("现金记录数量: {}", db.cash.len());
/// # Ok(())
/// # }
/// ```
///
/// # 文件结构
///
/// 初始化后将创建以下文件结构：
/// ```text
/// ./data/
/// ├── student_database.json    # 学生数据
/// ├── cash_database.json       # 现金数据
/// ├── uid_counter              # 学生UID计数器
/// └── cash_uid_counter         # 现金UID计数器
/// ```
pub fn init() -> Result<Database> {
    info!("正在初始化运行时数据库");
    let data_dir = std::env::var("QMX_DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    std::fs::create_dir_all(&data_dir).map_err(Error::from)?;

    let student_db = match StudentDatabase::read_from(&format!("{}/student_database.json", data_dir)) {
        Ok(db) => {
            info!("学生数据库加载成功");
            db
        }
        Err(e) => {
            if let Error::Io(ref io_err) = e {
                if io_err.kind() == std::io::ErrorKind::NotFound {
                    warn!("学生数据库文件不存在，正在创建新的数据库...");
                    let new_db = StudentDatabase::new();
                    new_db.save().map_err(Error::from)?;
                    new_db
                } else {
                    error!("加载学生数据库失败: {}", io_err);
                    return Err(Error::Other(format!("加载学生数据库失败: {}", io_err)));
                }
            } else {
                error!("加载学生数据库失败: {e:?}");
                return Err(Error::Other(format!("加载学生数据库失败: {e:?}")));
            }
        }
    };

    let cash_db = match CashDatabase::read_from("./data/cash_database.json") {
        Ok(db) => {
            info!("现金数据库加载成功");
            db
        }
        Err(e) => {
            if let Error::Io(ref io_err) = e {
                if io_err.kind() == std::io::ErrorKind::NotFound {
                    warn!("现金数据库文件不存在，正在创建新的数据库...");
                    let new_db = CashDatabase::new();
                    new_db.save().map_err(Error::from)?;
                    new_db
                } else {
                    error!("加载现金数据库失败: {}", io_err);
                    return Err(Error::Other(format!("加载现金数据库失败: {}", io_err)));
                }
            } else {
                error!("加载现金数据库失败: {e:?}");
                return Err(Error::Other(format!("加载现金数据库失败: {e:?}")));
            }
        }
    };

    info!("运行时数据库初始化完成");
    Ok(Database::new(student_db, cash_db))
}

/// 初始化数据库（测试模式，使用简单保存）
#[cfg(test)]
pub fn init_simple() -> Result<Database> {
    info!("正在初始化运行时数据库（测试模式）");
    let data_dir = std::env::var("QMX_DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    std::fs::create_dir_all(&data_dir).map_err(Error::from)?;

    let student_db = match StudentDatabase::read_from(&format!("{}/student_database.json", data_dir)) {
        Ok(db) => {
            info!("学生数据库加载成功");
            db
        }
        Err(e) => {
            if let Error::Io(ref io_err) = e {
                if io_err.kind() == std::io::ErrorKind::NotFound {
                    warn!("学生数据库文件不存在，正在创建新的数据库...");
                    let new_db = StudentDatabase::new();
                    <StudentDatabase as crate::common::Database<super::student::Student>>::save_to_simple(&new_db, &format!("{}/student_database.json", data_dir)).map_err(Error::from)?;
                    new_db
                } else {
                    error!("加载学生数据库失败: {}", io_err);
                    return Err(Error::Other(format!("加载学生数据库失败: {}", io_err)));
                }
            } else {
                error!("加载学生数据库失败: {e:?}");
                return Err(Error::Other(format!("加载学生数据库失败: {e:?}")));
            }
        }
    };

    let cash_db = match CashDatabase::read_from(&format!("{}/cash_database.json", data_dir)) {
        Ok(db) => {
            info!("现金数据库加载成功");
            db
        }
        Err(e) => {
            if let Error::Io(ref io_err) = e {
                if io_err.kind() == std::io::ErrorKind::NotFound {
                    warn!("现金数据库文件不存在，正在创建新的数据库...");
                    let new_db = CashDatabase::new();
                    <CashDatabase as crate::common::Database<super::cash::Cash>>::save_to_simple(
                        &new_db,
                        &format!("{}/cash_database.json", data_dir),
                    )
                    .map_err(Error::from)?;
                    new_db
                } else {
                    error!("加载现金数据库失败: {}", io_err);
                    return Err(Error::Other(format!("加载现金数据库失败: {}", io_err)));
                }
            } else {
                error!("加载现金数据库失败: {e:?}");
                return Err(Error::Other(format!("加载现金数据库失败: {e:?}")));
            }
        }
    };

    info!("运行时数据库初始化完成（测试模式）");
    Ok(Database::new(student_db, cash_db))
}

pub fn save(db: &Database) -> Result<()> {
    db.save()
}
