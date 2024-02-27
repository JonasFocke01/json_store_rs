use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Display, fs::OpenOptions, io::Write, path::PathBuf};

#[derive(Debug)]
pub enum JsonStoreError {
    FileNotFound,
    PathNotValid,
    FilecontentNotValid,
    #[allow(non_camel_case_types)]
    FilecontentNotValid_CreatedBackupfile,
    #[allow(non_camel_case_types)]
    FilecontentNotValid_CouldNotCreateBackupfile,
}

impl Display for JsonStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for JsonStoreError {
    fn description(&self) -> &str {
        "Json Store Error"
    }
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonStore<T> {
    data: T,
    #[serde(skip)]
    db_file_path: PathBuf,
    #[serde(skip)]
    unsaved_writes: usize,
}

impl<T> JsonStore<T>
where
    T: Serialize + DeserializeOwned + Default + Clone,
{
    pub fn load(db_file_path: PathBuf) -> Result<Self, JsonStoreError> {
        match Self::read_and_deserialize_file(db_file_path.clone()) {
            Err(JsonStoreError::FilecontentNotValid) => match Self::backup_db_file(db_file_path) {
                Ok(_) => Err(JsonStoreError::FilecontentNotValid_CreatedBackupfile),
                Err(_) => Err(JsonStoreError::FilecontentNotValid_CouldNotCreateBackupfile),
            },
            Err(e) => Err(e),
            Ok(s) => Ok(s),
        }
    }
    pub fn setup(db_file_path: PathBuf) -> Result<Self> {
        let json_store = Self {
            data: T::default(),
            db_file_path: db_file_path.clone(),
            unsaved_writes: 0,
        };

        json_store.serialize_and_write_file()?;

        Ok(json_store)
    }

    // TODO: migrate funtion

    pub fn data(&self) -> &T {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut T {
        self.unsaved_writes = self.unsaved_writes.saturating_add(1);
        &mut self.data
    }
    pub fn update(&mut self, update_fn: fn(T) -> T) -> Result<()> {
        self.data = update_fn(self.data.clone());
        self.serialize_and_write_file()?;
        self.unsaved_writes = 0;
        Ok(())
    }

    pub fn write(&mut self) -> Result<bool> {
        if self.unsaved_writes > 0 {
            self.serialize_and_write_file()?;
            self.unsaved_writes = 0;
            return Ok(true);
        }
        Ok(false)
    }
}

impl<T> JsonStore<T>
where
    T: Serialize + DeserializeOwned,
{
    fn backup_db_file(db_file_path: PathBuf) -> Result<u64> {
        let mut backup_file_path = db_file_path.clone();
        backup_file_path.pop();
        backup_file_path.push("backup.json");
        Ok(std::fs::copy(db_file_path.clone(), backup_file_path).unwrap())
    }
    fn serialize_and_write_file(&self) -> Result<()> {
        let serialized_data = serde_json::to_string_pretty(&self)?;
        let Ok(path_as_string) = self.db_file_path.clone().into_os_string().into_string() else {
            return Err(JsonStoreError::PathNotValid.into());
        };

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path_as_string)
            .unwrap();

        write!(file, "{serialized_data}")?;

        Ok(())
    }

    fn read_and_deserialize_file(db_file_path: PathBuf) -> Result<Self, JsonStoreError> {
        let Ok(parsed_db_file_path) = db_file_path.clone().into_os_string().into_string() else {
            return Err(JsonStoreError::PathNotValid);
        };
        let Ok(data_str) = std::fs::read_to_string(parsed_db_file_path.as_str()) else {
            return Err(JsonStoreError::FileNotFound);
        };
        let Ok(data) = serde_json::from_str(&data_str) else {
            return Err(JsonStoreError::FilecontentNotValid);
        };
        Ok(Self {
            data,
            db_file_path,
            unsaved_writes: 0,
        })
    }
}
