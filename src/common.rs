use anyhow::{Context, Result};
use log::{debug, info};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

/// 通用数据库trait，定义所有数据库的公共操作
pub trait Database<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    /// 获取数据存储的引用
    fn data(&self) -> &BTreeMap<u64, T>;

    /// 获取数据存储的可变引用
    fn data_mut(&mut self) -> &mut BTreeMap<u64, T>;

    /// 获取默认保存路径
    fn default_path(&self) -> &'static str;

    /// 获取数据库类型名称（用于日志）
    fn type_name(&self) -> &'static str;

    /// 创建新的空数据库
    fn new() -> Self;

    /// 插入记录
    fn insert(&mut self, item: T)
    where
        T: HasUid,
    {
        let uid = item.uid();
        info!("插入{}记录，UID: {}", self.type_name(), uid);
        self.data_mut().insert(uid, item);
    }

    /// 批量插入记录
    fn insert_batch(&mut self, items: Vec<T>) -> usize
    where
        T: HasUid,
    {
        let mut inserted_count = 0;
        for item in items {
            let uid = item.uid();
            info!("批量插入{}记录，UID: {}", self.type_name(), uid);
            self.data_mut().insert(uid, item);
            inserted_count += 1;
        }
        info!("批量插入 {} 个{}记录", inserted_count, self.type_name());
        inserted_count
    }

    /// 批量更新记录
    fn update_batch<F>(&mut self, uids: &[u64], mut update_fn: F) -> usize
    where
        F: FnMut(&mut T) -> bool,
    {
        let mut updated_count = 0;
        for &uid in uids {
            if let Some(item) = self.data_mut().get_mut(&uid) {
                if update_fn(item) {
                    info!("批量更新{}记录，UID: {}", self.type_name(), uid);
                    updated_count += 1;
                }
            }
        }
        info!("批量更新 {} 个{}记录", updated_count, self.type_name());
        updated_count
    }

    /// 获取记录
    fn get(&self, uid: &u64) -> Option<&T> {
        self.data().get(uid)
    }

    /// 删除记录
    fn remove(&mut self, uid: &u64) -> Option<T> {
        let removed = self.data_mut().remove(uid);
        if removed.is_some() {
            info!("成功删除{}记录，UID: {}", self.type_name(), uid);
        } else {
            log::warn!("尝试删除不存在的{}记录，UID: {}", self.type_name(), uid);
        }
        removed
    }

    /// 批量删除记录
    fn remove_batch(&mut self, uids: &[u64]) -> usize {
        let mut removed_count = 0;
        for &uid in uids {
            if self.data_mut().remove(&uid).is_some() {
                removed_count += 1;
            }
        }
        info!("批量删除 {} 个{}记录", removed_count, self.type_name());
        removed_count
    }

    /// 迭代器
    fn iter(&self) -> impl Iterator<Item = (&u64, &T)> + '_
    where
        T: 'static,
    {
        self.data().iter()
    }

    /// 记录数量
    fn len(&self) -> usize {
        self.data().len()
    }

    /// 是否为空
    fn is_empty(&self) -> bool {
        self.data().is_empty()
    }

    /// 序列化为JSON字符串
    fn json(&self) -> String
    where
        Self: Serialize,
    {
        match serde_json::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                log::error!("序列化{}数据库失败: {}", self.type_name(), e);
                String::new()
            }
        }
    }

    /// 从JSON字符串反序列化
    fn from_json(json: &str) -> Result<Self>
    where
        Self: DeserializeOwned,
    {
        serde_json::from_str(json)
            .with_context(|| format!("从JSON反序列化{}数据库失败", Self::static_type_name()))
    }

    /// 获取静态类型名称（用于错误信息）
    fn static_type_name() -> &'static str;

    /// 保存到默认路径
    fn save(&self) -> Result<()>
    where
        Self: Serialize,
    {
        self.save_to(self.default_path())
    }

    /// 非原子保存到指定路径（用于测试环境）
    fn save_to_simple(&self, path: &str) -> Result<()>
    where
        Self: Serialize,
    {
        info!("正在简单保存{}数据库到 {}", self.type_name(), path);

        // 确保父目录存在
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("无法创建父目录 '{}'", parent.display()))?;
            }
        }

        let file = File::create(path).with_context(|| format!("无法创建文件 '{}'", path))?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, self).with_context(|| {
            format!(
                "序列化并写入{}数据库到文件 '{}' 失败",
                self.type_name(),
                path
            )
        })?;

        debug!("成功简单保存{}数据库到 {}", self.type_name(), path);
        Ok(())
    }

    /// 保存到指定路径（原子操作）
    fn save_to(&self, path: &str) -> Result<()>
    where
        Self: Serialize,
    {
        info!("正在保存{}数据库到 {}", self.type_name(), path);

        // 确保父目录存在
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("无法创建父目录 '{}'", parent.display()))?;
            }
        }

        let mut tmpfile = tempfile::NamedTempFile::new_in(
            std::path::Path::new(path)
                .parent()
                .ok_or_else(|| anyhow::anyhow!("无效的保存路径: {}", path))?,
        )?;

        serde_json::to_writer(&mut tmpfile, self).with_context(|| {
            format!(
                "序列化并写入{}数据库到临时文件失败",
                self.type_name()
            )
        })?;

        tmpfile.flush()?;
        tmpfile.as_file().sync_all()?;

        let target_path = std::path::Path::new(path);
        if let Some(dir) = target_path.parent() {
            if let Ok(dir_fd) = std::fs::File::open(dir) {
                let _ = dir_fd.sync_all();
            }
        }

        tmpfile
            .persist(path)
            .map_err(|e| anyhow::anyhow!("持久化临时文件失败: {}", e.error))?;

        debug!("成功原子保存{}数据库到 {}", self.type_name(), path);

        Ok(())
    }

    /// 从指定路径读取
    fn read_from(path: &str) -> Result<Self>
    where
        Self: DeserializeOwned,
    {
        info!("从 {} 加载{}数据库", path, Self::static_type_name());
        let file = File::open(path).with_context(|| format!("打开路径为 '{}' 的文件失败", path))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).with_context(|| {
            format!(
                "反序列化路径为 '{}' 的{}数据库失败",
                path,
                Self::static_type_name()
            )
        })
    }
}

/// 用于获取UID的trait
pub trait HasUid {
    fn uid(&self) -> u64;
}
