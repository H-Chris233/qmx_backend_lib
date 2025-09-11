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
    pub fn rings(&self) -> &Vec<f64> {
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
}

impl Default for Student {
    fn default() -> Self {
        Self::new()
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

    pub fn insert(&mut self, student: Student) {
        info!("插入用户，UID: {}", student.uid());
        self.student_data.insert(student.uid(), student);
    }

    pub fn insert_batch(&mut self, students: Vec<Student>) -> usize {
        let mut inserted_count = 0;
        for student in students {
            let uid = student.uid();
            info!("批量插入用户，UID: {}", uid);
            self.student_data.insert(uid, student);
            inserted_count += 1;
        }
        info!("批量插入 {} 个学生记录", inserted_count);
        inserted_count
    }

    pub fn update_batch<F>(&mut self, uids: &[u64], mut update_fn: F) -> usize
    where
        F: FnMut(&mut Student) -> bool,
    {
        let mut updated_count = 0;
        for &uid in uids {
            if let Some(student) = self.student_data.get_mut(&uid) {
                if update_fn(student) {
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

    pub fn get(&self, index: &u64) -> Option<&Student> {
        self.student_data.get(index)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to("./data/student_database.json")
    }

    pub fn save_to(&self, path: &str) -> Result<()> {
        info!("正在保存学生数据库到 {}", path);
        let tmp_path = format!("{}.tmp", path);
        let file =
            File::create(&tmp_path).with_context(|| format!("无法创建临时文件 '{}'", tmp_path))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, self)
            .with_context(|| format!("序列化并写入学生数据库到临时文件 '{}' 失败", tmp_path))?;
        writer
            .flush()
            .with_context(|| format!("刷新写入到临时文件 '{}' 失败", tmp_path))?;
        writer
            .get_ref()
            .sync_all()
            .with_context(|| format!("同步临时文件 '{}' 到磁盘失败", tmp_path))?;
        std::fs::rename(&tmp_path, path)
            .with_context(|| format!("原子替换目标文件 '{}' 失败", path))?;
        Ok(())
    }

    pub fn read_from(path: &str) -> Result<Self> {
        info!("从 {} 加载学生数据库", path);
        let file = File::open(path).with_context(|| format!("打开路径为 '{}' 的文件失败", path))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .with_context(|| format!("反序列化路径为 '{}' 的学生数据库失败", path))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u64, &Student)> + '_ {
        self.student_data.iter()
    }
    pub fn len(&self) -> usize {
        self.student_data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.student_data.is_empty()
    }

    pub fn remove(&mut self, uid: &u64) -> Option<Student> {
        info!("Removing student with UID: {}", uid);
        let removed = self.student_data.remove(uid);
        if removed.is_some() {
            info!("Successfully removed student with UID: {}", uid);
        } else {
            warn!("Attempted to remove non-existent student with UID: {}", uid);
        }
        removed
    }

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
