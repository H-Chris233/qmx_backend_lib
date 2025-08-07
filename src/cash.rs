use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::BTreeMap;

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

pub static CASH_UID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// 独立的 Cash 结构体，包含自己的 UID 和关联的学生 ID
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cash {
    /// Cash 自己的唯一标识符
    pub uid: u64,
    /// 关联的学生 UID
    pub student_id: Option<u64>,
    /// 当前金额
    pub cash: i32,
}

impl Cash {
    pub fn new(student_id: Option<u64>) -> Self {
        let uid = CASH_UID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let new_cash = Self {
            uid,
            student_id,
            cash: 0,
        };
        info!("Created new Cash record with UID: {}", new_cash.uid);
        new_cash
    }

    pub fn add(&mut self, num: i32) {
        self.cash += num;
    }

    pub fn set_cash(&mut self, num: i32) {
        self.cash = num;
    }

    pub fn set_id(&mut self, id: u64) {
        if id == 0 {
            self.student_id = None;
        } else {
            self.student_id = Some(id)
        }
    }
}

/// Cash 数据库结构，支持持久化存储
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CashDatabase {
    pub cash_data: BTreeMap<u64, Cash>,
}

impl Default for CashDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl CashDatabase {
    pub fn new() -> Self {
        Self {
            cash_data: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, cash: Cash) {
        info!("Inserting cash record with UID: {}", cash.uid);
        self.cash_data.insert(cash.uid, cash);
    }

    pub fn get(&self, index: &u64) -> Option<&Cash> {
        self.cash_data.get(index)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to("./data/cash_database.json")
    }

    pub fn save_to(&self, path: &str) -> Result<()> {
        info!("Saving cash database to {}", path);
        let file = File::create(path).with_context(|| format!("Failed to create file at '{}'", path))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)
            .with_context(|| format!("Failed to serialize and write cash database to '{}'", path))
    }

    pub fn read_from(path: &str) -> Result<Self> {
        info!("Loading cash database from {}", path);
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader)
                    .with_context(|| format!("Failed to deserialize cash database from '{}'", path))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                warn!("Cash database file not found at '{}', creating new database", path);
                Ok(Self::new())
            }
            Err(e) => Err(e).with_context(|| format!("Failed to open file at '{}'", path)),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u64, &Cash)> + '_ {
        self.cash_data.iter()
    }

    pub fn len(&self) -> usize {
        self.cash_data.len()
    }

    /// 检查数据库是否为空
    pub fn is_empty(&self) -> bool {
        self.cash_data.is_empty()
    }

    /// 删除指定UID的现金记录
    /// 
    /// # 参数
    /// * `uid` - 要删除的现金记录UID
    /// 
    /// # 返回值
    /// 返回被删除的现金记录，如果不存在则返回None
    pub fn remove(&mut self, uid: &u64) -> Option<Cash> {
        info!("Removing cash record with UID: {}", uid);
        let removed = self.cash_data.remove(uid);
        if removed.is_some() {
            info!("Successfully removed cash record with UID: {}", uid);
        } else {
            warn!("Attempted to remove non-existent cash record with UID: {}", uid);
        }
        removed
    }

    /// 批量删除现金记录
    /// 
    /// # 参数
    /// * `uids` - 要删除的现金记录UID列表
    /// 
    /// # 返回值
    /// 返回成功删除的现金记录数量
    pub fn remove_batch(&mut self, uids: &[u64]) -> usize {
        let mut removed_count = 0;
        for &uid in uids {
            if self.cash_data.remove(&uid).is_some() {
                removed_count += 1;
            }
        }
        info!("Batch removed {} cash records", removed_count);
        removed_count
    }
}

/// 加载已保存的 Cash UID 计数器
pub fn load_saved_cash_uid() -> Result<u64> {
    let path = "./data/cash_uid_counter";
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let result = content
                .trim()
                .parse::<u64>()
                .with_context(|| format!("Failed to parse CASH UID from file '{}'", path));
            match &result {
                Ok(uid) => {
                    info!("Successfully loaded CASH UID: {}", uid);
                    Ok(*uid)
                }
                Err(e) => {
                    error!("Failed to parse CASH UID: {:?}", e);
                    Err(result.unwrap_err())
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("No existing CASH UID file found. Starting from default value 1");
            Ok(1)
        }
        Err(e) => {
            error!("Failed to read CASH UID file: {}", e);
            Err(e).with_context(|| format!("Failed to read CASH UID file '{}'", path))
        }
    }
}

/// 保存 Cash UID 计数器
pub fn save_uid() -> Result<()> {
    let uid = CASH_UID_COUNTER.load(Ordering::Relaxed);
    let path = "./data/cash_uid_counter";
    let mut file = File::create(path).with_context(|| format!("Failed to create file '{}'", path))?;

    file.write_all(uid.to_string().as_bytes())
        .with_context(|| format!("Failed to write CASH UID to file '{}'", path))?;

    debug!("Successfully saved CASH UID: {} to file", uid);
    Ok(())
}

/// Cash 模块初始化函数
pub fn init() -> Result<()> {
    let saved_uid = load_saved_cash_uid().context("Failed to load saved CASH UID during initialization")?;
    CASH_UID_COUNTER.store(saved_uid, Ordering::Relaxed);
    info!("CASH UID counter initialized to {}", saved_uid);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cash_creation() {
        let cash = Cash::new(Some(123));
        assert!(cash.uid > 0);
        assert_eq!(cash.student_id, Some(123));
        assert_eq!(cash.cash, 0);
    }

    #[test]
    fn test_cash_modification() {
        let mut cash = Cash::new(None);
        
        cash.add(100);
        assert_eq!(cash.cash, 100);
        
        cash.set_cash(200);
        assert_eq!(cash.cash, 200);
        
        cash.set_id(456);
        assert_eq!(cash.student_id, Some(456));
    }

    #[test]
    fn test_cash_database_crud() {
        let mut db = CashDatabase::new();
        
        // 测试插入
        let cash1 = Cash::new(Some(1));
        let cash2 = Cash::new(Some(2));
        let uid1 = cash1.uid;
        let uid2 = cash2.uid;
        
        db.insert(cash1);
        db.insert(cash2);
        
        assert_eq!(db.len(), 2);
        assert!(!db.is_empty());
        
        // 测试查询
        assert!(db.get(&uid1).is_some());
        assert!(db.get(&uid2).is_some());
        assert!(db.get(&999).is_none());
        
        // 测试删除
        let removed = db.remove(&uid1);
        assert!(removed.is_some());
        assert_eq!(db.len(), 1);
        assert!(db.get(&uid1).is_none());
        
        // 测试批量删除
        let count = db.remove_batch(&[uid2, 999]);
        assert_eq!(count, 1);
        assert!(db.is_empty());
    }

    #[test]
    fn test_cash_json_operations() {
        let mut db = CashDatabase::new();
        let cash = Cash::new(Some(123));
        db.insert(cash);
        
        let json = serde_json::to_string(&db).unwrap();
        assert!(!json.is_empty());
        
        let deserialized: CashDatabase = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 1);
    }
}
