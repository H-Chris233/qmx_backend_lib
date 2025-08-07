use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

pub static STUDENT_UID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Person {
    uid: u64,
    age: u8,
    name: String,
    lesson_left: Option<u32>,
    class: Class,
    rings: Vec<f64>,
    note: String,
    cash: (i32, i32),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Class {
    TenTry,
    Month,
    Year,
    Others,
}

pub trait Student {
    fn set_age(&mut self, age: u8) -> &mut Self;
    fn set_name(&mut self, name: String) -> &mut Self;
    fn set_class(&mut self, class: Class) -> &mut Self;
    fn set_lesson_left(&mut self, lesson: u32) -> &mut Self;
    fn add_ring(&mut self, ring: f64) -> &mut Self;
    fn set_note(&mut self, note: String) -> &mut Self;
    fn set_cash(&mut self, cash: (i32, i32)) -> &mut Self;
    /// 强制设置UID，这可能会破坏数据一致性
    /// 
    /// # Safety
    /// 此函数是unsafe的，因为它可能导致UID冲突，破坏数据一致性。
    /// 调用者必须确保新的UID是唯一的且不会与现有记录冲突。
    unsafe fn set_id(&mut self, id: u64) -> &mut Self;

    fn uid(&self) -> u64;
    fn age(&self) -> u8;
    fn name(&self) -> &str;
    fn lesson_left(&self) -> Option<u32>;
    fn class(&self) -> &Class;
    fn rings(&self) -> &Vec<f64>;
    fn note(&self) -> &str;
    fn cash(&self) -> (i32, i32);
}

impl Student for Person {
    fn set_age(&mut self, age: u8) -> &mut Self {
        let old_age = self.age;
        self.age = age;
        debug!("年龄从 {} 更新到 {}，对象: {}", old_age, age, self.name);
        self
    }

    fn set_name(&mut self, name: String) -> &mut Self {
        let old_name = self.name.clone();
        self.name = name;
        info!("名称从 '{}' 改为 '{}'", old_name, self.name);
        self
    }

    fn set_class(&mut self, class: Class) -> &mut Self {
        let old_class = self.class.clone();
        self.lesson_left = match class {
            Class::TenTry => Some(10),
            _ => None,
        };
        self.class = class;
        debug!("班级从 {:?} 改为 {:?}", old_class, self.class);
        self
    }

    fn set_lesson_left(&mut self, lesson: u32) -> &mut Self {
        if self.lesson_left.is_none() {
            warn!("尝试为非TenTry班级设置剩余课时: {}", self.name);
            return self;
        }
        let old_value = self.lesson_left.unwrap_or(0);
        self.lesson_left = Some(lesson);
        info!(
            "剩余课时从 {} 改为 {}，对象: {}",
            old_value, lesson, self.name
        );
        self
    }

    fn add_ring(&mut self, ring: f64) -> &mut Self {
        info!("为 {} 添加新的环形数据", self.name);
        self.rings.push(ring);
        self
    }

    fn set_note(&mut self, note: String) -> &mut Self {
        let old_note = self.note.clone();
        self.note = note;
        debug!("备注已更新: {}. 旧长度: {} 字符", self.name, old_note.len());
        self
    }

    /// 强制设置UID，这可能会破坏数据一致性
    /// 
    /// # Safety
    /// 此函数是unsafe的，因为它可能导致UID冲突，破坏数据一致性。
    /// 调用者必须确保新的UID是唯一的且不会与现有记录冲突。
    unsafe fn set_id(&mut self, id: u64) -> &mut Self {
        info!("强制更改 UID 从 {} 到 {}", self.uid, id);
        self.uid = id;
        self
    }

    fn set_cash(&mut self, cash: (i32, i32)) -> &mut Self {
        let old_cash = self.cash;
        self.cash = cash;
        debug!(
            "现金从 {:?} 更新到 {:?}，对象: {}",
            old_cash, self.cash, self.name
        );
        self
    }

    fn uid(&self) -> u64 {
        self.uid
    }

    fn age(&self) -> u8 {
        self.age
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn lesson_left(&self) -> Option<u32> {
        self.lesson_left
    }

    fn class(&self) -> &Class {
        &self.class
    }

    fn rings(&self) -> &Vec<f64> {
        &self.rings
    }

    fn note(&self) -> &str {
        &self.note
    }

    fn cash(&self) -> (i32, i32) {
        self.cash
    }
}

impl Default for Person {
    fn default() -> Self {
        Self::new()
    }
}

impl Person {
    pub fn new() -> Self {
        let uid = STUDENT_UID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let new_person = Self {
            uid,
            age: 0,
            name: "Default".to_string(),
            lesson_left: None,
            class: Class::Others,
            rings: Vec::new(),
            note: String::new(),
            cash: (0, 0),
        };
        info!("创建新用户，UID: {}", new_person.uid);
        new_person
    }
}

pub fn load_saved_uid() -> Result<u64> {
    let path = "./data/uid_counter";
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let result = content
                .trim()
                .parse::<u64>()
                .with_context(|| format!("解析路径为 '{}' 的UID文件失败", path));
            match &result {
                Ok(uid) => {
                    info!("成功加载UID: {}", uid);
                    Ok(*uid)
                }
                Err(e) => {
                    error!("解析UID失败: {:?}", e);
                    Err(result.unwrap_err())
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("未找到现有UID文件，从默认值1开始");
            Ok(1)
        }
        Err(e) => {
            error!("读取UID文件失败: {}", e);
            Err(e).with_context(|| format!("读取路径为 '{}' 的UID文件失败", path))
        }
    }
}

pub fn save_uid() -> Result<()> {
    let uid = STUDENT_UID_COUNTER.load(Ordering::Relaxed);
    let path = "./data/uid_counter";
    let mut file = File::create(path).with_context(|| format!("无法创建文件 '{}'", path))?;

    file.write_all(uid.to_string().as_bytes())
        .with_context(|| format!("写入UID到文件 '{}' 失败", path))?;

    debug!("成功将UID: {} 保存到文件", uid);
    Ok(())
}

pub fn init() -> Result<()> {
    let saved_uid = load_saved_uid().context("初始化期间加载已保存的UID失败")?;

    STUDENT_UID_COUNTER.store(saved_uid, Ordering::Relaxed);
    info!("UID计数器初始化为 {}", saved_uid);

    save_uid().context("初始化期间保存初始UID失败")?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StudentDatabase {
    pub student_data: BTreeMap<u64, Person>,
}

impl Default for StudentDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl StudentDatabase {
    pub fn new() -> Self {
        Self {
            student_data: BTreeMap::new(),
        }
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let deserialized =
            serde_json::from_str(json).with_context(|| "反序列化JSON到学生数据库失败")?;
        debug!("反序列化结果: {:?}", &deserialized);
        Ok(deserialized)
    }

    pub fn insert(&mut self, person: Person) {
        info!("插入用户，UID: {}", person.uid());
        self.student_data.insert(person.uid(), person);
    }

    pub fn insert_batch(&mut self, persons: Vec<Person>) -> usize {
        let mut inserted_count = 0;
        for person in persons {
            let uid = person.uid();
            info!("批量插入用户，UID: {}", uid);
            self.student_data.insert(uid, person);
            inserted_count += 1;
        }
        info!("批量插入 {} 个学生记录", inserted_count);
        inserted_count
    }

    pub fn update_batch<F>(&mut self, uids: &[u64], mut update_fn: F) -> usize
    where
        F: FnMut(&mut Person) -> bool,
    {
        let mut updated_count = 0;
        for &uid in uids {
            if let Some(person) = self.student_data.get_mut(&uid) {
                if update_fn(person) {
                    info!("批量更新学生记录，UID: {}", uid);
                    updated_count += 1;
                }
            }
        }
        info!("批量更新 {} 个学生记录", updated_count);
        updated_count
    }

    pub fn json(&self) -> String {
        serde_json::to_string(self).expect("将学生数据库序列化为JSON失败（此错误不应发生）")
    }

    pub fn get(&self, index: &u64) -> Option<&Person> {
        self.student_data.get(index)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to("./data/student_database.json")
    }

    pub fn save_to(&self, path: &str) -> Result<()> {
        info!("正在保存学生数据库到 {}", path);
        let file =
            File::create(path).with_context(|| format!("无法创建路径为 '{}' 的文件", path))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)
            .with_context(|| format!("序列化并写入学生数据库到 '{}' 失败", path))
    }

    pub fn read_from(path: &str) -> Result<Self> {
        info!("从 {} 加载学生数据库", path);
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader)
                    .with_context(|| format!("反序列化路径为 '{}' 的学生数据库失败", path))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                warn!("未找到学生数据库文件 '{}'，创建新数据库", path);
                Ok(Self::new())
            }
            Err(e) => Err(e).with_context(|| format!("打开路径为 '{}' 的文件失败", path)),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u64, &Person)> + '_ {
        self.student_data.iter()
    }

    pub fn len(&self) -> usize {
        self.student_data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.student_data.is_empty()
    }

    /// 删除指定UID的学生记录
    /// 
    /// # 参数
    /// * `uid` - 要删除的学生UID
    /// 
    /// # 返回值
    /// 返回被删除的学生记录，如果不存在则返回None
    pub fn remove(&mut self, uid: &u64) -> Option<Person> {
        info!("Removing student with UID: {}", uid);
        let removed = self.student_data.remove(uid);
        if removed.is_some() {
            info!("Successfully removed student with UID: {}", uid);
        } else {
            warn!("Attempted to remove non-existent student with UID: {}", uid);
        }
        removed
    }

    /// 批量删除学生记录
    /// 
    /// # 参数
    /// * `uids` - 要删除的学生UID列表
    /// 
    /// # 返回值
    /// 返回成功删除的学生数量
    pub fn remove_batch(&mut self, uids: &[u64]) -> usize {
        let mut removed_count = 0;
        for &uid in uids {
            if self.student_data.remove(&uid).is_some() {
                removed_count += 1;
            }
        }
        info!("Batch removed {} student records", removed_count);
        removed_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_creation() {
        let person = Person::new();
        assert!(person.uid() > 0);
        assert_eq!(person.age(), 0);
        assert_eq!(person.name(), "Default");
        assert_eq!(person.class(), &Class::Others);
        assert!(person.rings().is_empty());
        assert_eq!(person.note(), "");
        assert_eq!(person.cash(), (0, 0));
    }

    #[test]
    fn test_person_modification() {
        let mut person = Person::new();
        
        person.set_age(25)
            .set_name("张三".to_string())
            .set_class(Class::TenTry)
            .set_note("测试学生".to_string());
        
        assert_eq!(person.age(), 25);
        assert_eq!(person.name(), "张三");
        assert_eq!(person.class(), &Class::TenTry);
        assert_eq!(person.lesson_left(), Some(10));
        assert_eq!(person.note(), "测试学生");
    }

    #[test]
    fn test_student_database_crud() {
        let mut db = StudentDatabase::new();
        
        // 测试插入
        let person1 = Person::new();
        let person2 = Person::new();
        let uid1 = person1.uid();
        let uid2 = person2.uid();
        
        db.insert(person1);
        db.insert(person2);
        
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
    fn test_json_serialization() {
        let mut db = StudentDatabase::new();
        let person = Person::new();
        db.insert(person);
        
        let json = db.json();
        assert!(!json.is_empty());
        
        let deserialized = StudentDatabase::from_json(&json).unwrap();
        assert_eq!(deserialized.len(), 1);
    }

    #[test]
    fn test_batch_operations() {
        let mut db = StudentDatabase::new();
        
        // 测试批量插入
        let persons = vec![Person::new(), Person::new(), Person::new()];
        let inserted_count = db.insert_batch(persons);
        assert_eq!(inserted_count, 3);
        assert_eq!(db.len(), 3);
        
        // 收集所有UID用于批量更新测试
        let uids: Vec<u64> = db.iter().map(|(&uid, _)| uid).collect();
        
        // 测试批量更新 - 更新所有学生的年龄
        let updated_count = db.update_batch(&uids, |person| {
            person.set_age(25);
            true
        });
        assert_eq!(updated_count, 3);
        
        // 验证更新结果
        for &uid in &uids {
            let person = db.get(&uid).unwrap();
            assert_eq!(person.age(), 25);
        }
        
        // 测试批量更新 - 只更新部分学生（条件更新）
        let updated_count = db.update_batch(&uids, |person| {
            if person.age() == 25 {
                person.set_name("BatchUpdated".to_string());
                true
            } else {
                false
            }
        });
        assert_eq!(updated_count, 3);
        
        // 验证条件更新结果
        for &uid in &uids {
            let person = db.get(&uid).unwrap();
            assert_eq!(person.name(), "BatchUpdated");
        }
    }
}
