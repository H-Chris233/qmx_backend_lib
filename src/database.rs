use super::student::{Person, Student};

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Debug)]
pub struct Database {
    data: BTreeMap<u64, Person>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let deserialized = serde_json::from_str(json)
            .with_context(|| "Failed to deserialize JSON into Database")?;
        debug!("Deserialized: {:?}", &deserialized);
        Ok(deserialized)
    }

    pub fn insert(&mut self, person: Person) {
        info!("Inserting person with UID: {}", person.uid());
        self.data.insert(person.uid(), person);
    }

    pub fn json(&self) -> String {
        serde_json::to_string(self)
            .expect("Failed to serialize database to JSON. This should never happen.")
    }

    pub fn get(&self, index: &u64) -> Option<&Person> {
        self.data.get(index)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to("./data/database.json")
    }

    pub fn save_to(&self, path: &str) -> Result<()> {
        info!("Saving database to {}", path);
        let file =
            File::create(path).with_context(|| format!("Failed to create file at '{}'", path))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)
            .with_context(|| format!("Failed to serialize and write database to '{}'", path))
    }

    pub fn read_from(path: &str) -> Result<Self> {
        info!("Loading database from {}", path);
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader)
                    .with_context(|| format!("Failed to deserialize database from '{}'", path))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                warn!(
                    "Database file not found at '{}', creating new database",
                    path
                );
                Ok(Self::new())
            }
            Err(e) => Err(e).with_context(|| format!("Failed to open file at '{}'", path)),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u64, &Person)> + '_ {
        self.data.iter()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

pub fn init() -> Result<Database> {
    Database::read_from("./data/database.json")
}

pub fn get_data_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let mut path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| anyhow!("Failed to resolve app data directory: {}", e))?;

    path.push("database.json");

    Ok(path)
}

#[tauri::command]
pub async fn get_database_path(app_handle: AppHandle) -> Result<String, String> {
    let path = get_data_path(&app_handle)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .into_owned();

    Ok(path)
}

