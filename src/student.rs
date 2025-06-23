use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::BTreeMap;

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

pub static UID_COUNTER: AtomicU64 = AtomicU64::new(1);

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
        debug!("Updated age from {} to {} for {}", old_age, age, self.name);
        self
    }

    fn set_name(&mut self, name: String) -> &mut Self {
        let old_name = self.name.clone();
        self.name = name;
        info!("Changed name from '{}' to '{}'", old_name, self.name);
        self
    }

    fn set_class(&mut self, class: Class) -> &mut Self {
        let old_class = self.class.clone();
        self.lesson_left = match class {
            Class::TenTry => Some(10),
            _ => None,
        };
        self.class = class;
        debug!("Class changed from {:?} to {:?}", old_class, self.class);
        self
    }

    fn set_lesson_left(&mut self, lesson: u32) -> &mut Self {
        if !self.lesson_left.is_some() {
            warn!(
                "Attempted to set remaining lessons for non-TenTry class: {}",
                self.name
            );
            return self;
        }
        let old_value = self.lesson_left.unwrap_or(0);
        self.lesson_left = Some(lesson);
        info!(
            "Lessons left changed from {} to {} for {}",
            old_value, lesson, self.name
        );
        self
    }

    fn add_ring(&mut self, ring: Vec<f64>) -> &mut Self {
        info!("Added new ring measurement for {}", self.name);
        self.rings.push(ring);
        self
    }

    fn set_note(&mut self, note: String) -> &mut Self {
        let old_note = self.note.clone();
        self.note = note;
        debug!(
            "Note updated for {}. Previous length: {} chars",
            self.name,
            old_note.len()
        );
        self
    }

    unsafe fn set_id(&mut self, id: u64) -> &mut Self {
        info!("Forcibly changing UID from {} to {}", self.uid, id);
        self.uid = id;
        self
    }
    
    fn set_cash(&mut self, cash: (i32, i32)) -> &mut Self {
        let old_cash = self.cash;
        self.cash = cash;
        debug!(
            "Updated cash from {:?} to {:?} for {}",
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

impl Person {
    pub fn new() -> Self {
        let uid = UID_COUNTER.fetch_add(1, Ordering::Relaxed);
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
        info!("Created new Person with UID: {}", new_person.uid);
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
                .with_context(|| format!("Failed to parse UID from file '{}'", path));
            match &result {
                Ok(uid) => {
                    info!("Successfully loaded UID: {}", uid);
                    Ok(*uid)
                }
                Err(e) => {
                    error!("Failed to parse UID: {:?}", e);
                    Err(result.unwrap_err())
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("No existing UID file found. Starting from default value 1");
            Ok(1)
        }
        Err(e) => {
            error!("Failed to read UID file: {}", e);
            Err(e).with_context(|| format!("Failed to read UID file '{}'", path))
        }
    }
}

pub fn save_uid() -> Result<()> {
    let uid = UID_COUNTER.load(Ordering::Relaxed);
    let path = "./data/uid_counter";
    let mut file =
        File::create(path).with_context(|| format!("Failed to create file '{}'", path))?;

    file.write_all(uid.to_string().as_bytes())
        .with_context(|| format!("Failed to write UID to file '{}'", path))?;

    debug!("Successfully saved UID: {} to file", uid);
    Ok(())
}

pub fn init() -> Result<()> {
    let saved_uid = load_saved_uid().context("Failed to load saved UID during initialization")?;

    UID_COUNTER.store(saved_uid, Ordering::Relaxed);
    info!("UID counter initialized to {}", saved_uid);

    save_uid().context("Failed to save initial UID to file")?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StudentDatabase {
    pub student_data: BTreeMap<u64, Person>,
}

impl StudentDatabase {
    pub fn new() -> Self {
        Self {
            student_data: BTreeMap::new(),
        }
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let deserialized = serde_json::from_str(json)
            .with_context(|| "Failed to deserialize JSON into StudentDatabase")?;
        debug!("Deserialized: {:?}", &deserialized);
        Ok(deserialized)
    }

    pub fn insert(&mut self, person: Person) {
        info!("Inserting person with UID: {}", person.uid());
        self.student_data.insert(person.uid(), person);
    }

    pub fn json(&self) -> String {
        serde_json::to_string(self)
            .expect("Failed to serialize student database to JSON. This should never happen.")
    }

    pub fn get(&self, index: &u64) -> Option<&Person> {
        self.student_data.get(index)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to("./data/student_database.json")
    }

    pub fn save_to(&self, path: &str) -> Result<()> {
        info!("Saving student database to {}", path);
        let file =
            File::create(path).with_context(|| format!("Failed to create file at '{}'", path))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)
            .with_context(|| format!("Failed to serialize and write student database to '{}'", path))
    }

    pub fn read_from(path: &str) -> Result<Self> {
        info!("Loading student database from {}", path);
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader)
                    .with_context(|| format!("Failed to deserialize student database from '{}'", path))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                warn!(
                    "Student database file not found at '{}', creating new database",
                    path
                );
                Ok(Self::new())
            }
            Err(e) => Err(e).with_context(|| format!("Failed to open file at '{}'", path)),
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

