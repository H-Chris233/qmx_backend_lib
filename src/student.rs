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
    rings: Vec<Vec<f64>>,
    note: String,
    cash: (i32, i32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    fn add_ring(&mut self, ring: Vec<f64>) -> &mut Self;
    fn set_note(&mut self, note: String) -> &mut Self;
    fn set_cash(&mut self, cash: (i32, i32)) -> &mut Self;
    unsafe fn set_id(&mut self, id: u64) -> &mut Self;

    fn uid(&self) -> u64;
    fn age(&self) -> u8;
    fn name(&self) -> &str;
    fn lesson_left(&self) -> Option<u32>;
    fn class(&self) -> &Class;
    fn rings(&self) -> &Vec<Vec<f64>>;
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

    fn add_ring(&mut self, ring: Vec<f64>) -> &mut Self {
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

    fn rings(&self) -> &Vec<Vec<f64>> {
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
}
