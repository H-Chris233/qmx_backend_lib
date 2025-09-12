use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::common::{Database, HasUid};

pub static STUDENT_UID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Student {
    uid: u64,
    age: u8,
    name: String,
    phone: String,
    lesson_left: Option<u32>,
    class: Class,
    subject: Subject,
    rings: Vec<f64>,
    note: String,
    // 会员相关字段
    membership_start_date: Option<DateTime<Utc>>,
    membership_end_date: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Class {
    TenTry,
    Month,
    Year,
    Others,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Subject {
    Shooting,
    Archery,
    Others,
}

impl Student {
    pub fn new() -> Self {
        let uid = STUDENT_UID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let new_student = Self {
            uid,
            age: 0,
            name: "未填写".to_string(),
            phone: "未填写".to_string(),
            lesson_left: None,
            class: Class::Others,
            subject: Subject::Others,
            rings: Vec::new(),
            note: String::new(),
            membership_start_date: None,
            membership_end_date: None,
        };
        info!("创建新用户，UID: {}", new_student.uid);
        new_student
    }

    pub fn set_age(&mut self, age: u8) -> &mut Self {
        let old_age = self.age;
        self.age = age;
        debug!("年龄从 {} 更新到 {}，对象: {}", old_age, age, self.name);
        self
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        let old_name = self.name.clone();
        self.name = name;
        info!("名称从 '{}' 改为 '{}'", old_name, self.name);
        self
    }

    pub fn set_class(&mut self, class: Class) -> &mut Self {
        let old_class = self.class.clone();
        self.lesson_left = match class {
            Class::TenTry => Some(10),
            _ => None,
        };
        self.class = class;
        debug!("班级从 {:?} 改为 {:?}", old_class, self.class);
        self
    }

    pub fn set_lesson_left(&mut self, lesson: u32) -> &mut Self {
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

    pub fn add_ring(&mut self, ring: f64) -> &mut Self {
        info!("为 {} 添加新的环形数据", self.name);
        self.rings.push(ring);
        self
    }

    pub fn set_note(&mut self, note: String) -> &mut Self {
        let old_note = self.note.clone();
        self.note = note;
        debug!("备注已更新: {}. 旧长度: {} 字符", self.name, old_note.len());
        self
    }

    /// # Safety
    /// 调用方必须确保提供的新 UID 在整个数据库中唯一，且不会与任何现有记录冲突；
    /// 同时必须保证后续持久化流程会同步更新，避免造成数据不一致或悬挂引用。
    pub unsafe fn set_id(&mut self, id: u64) -> &mut Self {
        warn!("强制更改 UID 从 {} 到 {}", self.uid, id);
        self.uid = id;
        self
    }

    pub fn set_phone(&mut self, phone: String) -> &mut Self {
        let old_phone = self.phone.clone();
        self.phone = phone;
        info!("电话号码从 '{}' 改为 '{}'", old_phone, self.phone);
        self
    }

    pub fn set_subject(&mut self, subject: Subject) -> &mut Self {
        let old_subject = self.subject.clone();
        self.subject = subject;
        debug!(
            "Subject changed from {:?} to {:?} for {}",
            old_subject, self.subject, self.name
        );
        self
    }

    pub fn set_membership_dates(&mut self, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> &mut Self {
        self.membership_start_date = start_date;
        self.membership_end_date = end_date;
        
        match (&start_date, &end_date) {
            (Some(start), Some(end)) => {
                info!("设置{}的会员期限: {} 到 {}", self.name, start.format("%Y-%m-%d"), end.format("%Y-%m-%d"));
            }
            (Some(start), None) => {
                info!("设置{}的会员开始时间: {}", self.name, start.format("%Y-%m-%d"));
            }
            (None, Some(end)) => {
                info!("设置{}的会员结束时间: {}", self.name, end.format("%Y-%m-%d"));
            }
            (None, None) => {
                info!("清除{}的会员时间", self.name);
            }
        }
        self
    }

    pub fn set_membership_start_date(&mut self, start_date: DateTime<Utc>) -> &mut Self {
        self.membership_start_date = Some(start_date);
        info!("设置{}的会员开始时间: {}", self.name, start_date.format("%Y-%m-%d"));
        self
    }

    pub fn set_membership_end_date(&mut self, end_date: DateTime<Utc>) -> &mut Self {
        self.membership_end_date = Some(end_date);
        info!("设置{}的会员结束时间: {}", self.name, end_date.format("%Y-%m-%d"));
        self
    }

    pub fn clear_membership(&mut self) -> &mut Self {
        self.membership_start_date = None;
        self.membership_end_date = None;
        info!("清除{}的会员信息", self.name);
        self
    }

    /// 检查会员是否有效（当前时间在会员期内）
    pub fn is_membership_active(&self) -> bool {
        let now = Utc::now();
        match (&self.membership_start_date, &self.membership_end_date) {
            (Some(start), Some(end)) => now >= *start && now <= *end,
            (Some(start), None) => now >= *start,  // 只有开始时间，认为永久有效
            (None, Some(end)) => now <= *end,      // 只有结束时间
            (None, None) => false,                 // 没有会员信息
        }
    }

    /// 获取会员剩余天数
    pub fn membership_days_remaining(&self) -> Option<i64> {
        if let Some(end_date) = self.membership_end_date {
            let now = Utc::now();
            if now <= end_date {
                Some((end_date - now).num_days())
            } else {
                Some(0) // 已过期
            }
        } else {
            None // 没有结束时间
        }
    }

    pub fn uid(&self) -> u64 {
        self.uid
    }
    pub fn age(&self) -> u8 {
        self.age
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn lesson_left(&self) -> Option<u32> {
        self.lesson_left
    }
    pub fn class(&self) -> &Class {
        &self.class
    }
    pub fn rings(&self) -> &[f64] {
        &self.rings
    }
    pub fn note(&self) -> &str {
        &self.note
    }
    pub fn phone(&self) -> &str {
        self.phone.as_str()
    }
    pub fn subject(&self) -> &Subject {
        &self.subject
    }
    
    pub fn membership_start_date(&self) -> Option<DateTime<Utc>> {
        self.membership_start_date
    }
    
    pub fn membership_end_date(&self) -> Option<DateTime<Utc>> {
        self.membership_end_date
    }
}

impl Default for Student {
    fn default() -> Self {
        Self::new()
    }
}

impl HasUid for Student {
    fn uid(&self) -> u64 {
        self.uid
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
    let uid = STUDENT_UID_COUNTER.load(Ordering::SeqCst);
    let path = "./data/uid_counter";
    let mut file = File::create(path).with_context(|| format!("无法创建文件 '{}'", path))?;
    file.write_all(uid.to_string().as_bytes())
        .with_context(|| format!("写入UID到文件 '{}' 失败", path))?;
    debug!("成功将UID: {} 保存到文件", uid);
    Ok(())
}

pub fn init() -> Result<()> {
    std::fs::create_dir_all("./data").with_context(|| "无法创建data目录")?;
    let saved_uid = load_saved_uid().context("初始化期间加载已保存的UID失败")?;
    STUDENT_UID_COUNTER.store(saved_uid, Ordering::SeqCst);
    info!("UID计数器初始化为 {}", saved_uid);
    save_uid().context("初始化期间保存初始UID失败")?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StudentDatabase {
    pub student_data: BTreeMap<u64, Student>,
}

impl Default for StudentDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl Database<Student> for StudentDatabase {
    fn data(&self) -> &BTreeMap<u64, Student> {
        &self.student_data
    }
    
    fn data_mut(&mut self) -> &mut BTreeMap<u64, Student> {
        &mut self.student_data
    }
    
    fn default_path(&self) -> &'static str {
        "./data/student_database.json"
    }
    
    fn type_name(&self) -> &'static str {
        "学生"
    }
    
    fn static_type_name() -> &'static str {
        "学生"
    }
    
    fn new() -> Self {
        Self {
            student_data: BTreeMap::new(),
        }
    }
}

impl StudentDatabase {
    pub fn from_json(json: &str) -> Result<Self> {
        let deserialized =
            serde_json::from_str(json).with_context(|| "反序列化JSON到学生数据库失败")?;
        debug!("反序列化结果: {:?}", &deserialized);
        Ok(deserialized)
    }

    // 向后兼容性方法 - 直接委托给trait实现
    pub fn new() -> Self {
        <Self as Database<Student>>::new()
    }

    pub fn insert(&mut self, student: Student) {
        <Self as Database<Student>>::insert(self, student)
    }

    pub fn insert_batch(&mut self, students: Vec<Student>) -> usize {
        <Self as Database<Student>>::insert_batch(self, students)
    }

    pub fn update_batch<F>(&mut self, uids: &[u64], update_fn: F) -> usize
    where
        F: FnMut(&mut Student) -> bool,
    {
        <Self as Database<Student>>::update_batch(self, uids, update_fn)
    }

    pub fn json(&self) -> String {
        <Self as Database<Student>>::json(self)
    }

    pub fn get(&self, index: &u64) -> Option<&Student> {
        <Self as Database<Student>>::get(self, index)
    }

    pub fn save(&self) -> Result<()> {
        <Self as Database<Student>>::save(self)
    }

    pub fn save_to(&self, path: &str) -> Result<()> {
        <Self as Database<Student>>::save_to(self, path)
    }

    pub fn read_from(path: &str) -> Result<Self> {
        <Self as Database<Student>>::read_from(path)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u64, &Student)> + '_ {
        <Self as Database<Student>>::iter(self)
    }

    pub fn len(&self) -> usize {
        <Self as Database<Student>>::len(self)
    }

    pub fn is_empty(&self) -> bool {
        <Self as Database<Student>>::is_empty(self)
    }

    pub fn remove(&mut self, uid: &u64) -> Option<Student> {
        <Self as Database<Student>>::remove(self, uid)
    }

    pub fn remove_batch(&mut self, uids: &[u64]) -> usize {
        <Self as Database<Student>>::remove_batch(self, uids)
    }
}
