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

pub trait JsonStore: Serialize + DeserializeOwned + Default + Clone {
    fn db_file_path() -> PathBuf;
    fn load() -> Result<Self, JsonStoreError> {
        match Self::read_and_deserialize_file() {
            Err(JsonStoreError::FilecontentNotValid) => match Self::backup_db_file() {
                Ok(_) => Err(JsonStoreError::FilecontentNotValid_CreatedBackupfile),
                Err(_) => Err(JsonStoreError::FilecontentNotValid_CouldNotCreateBackupfile),
            },
            Err(e) => Err(e),
            Ok(s) => Ok(s),
        }
    }
    fn setup() -> Result<Self> {
        let t = Self::default();
        t.serialize_and_write_file()?;

        Ok(t)
    }

    // TODO: migrate funtion

    fn write(&mut self) -> Result<bool> {
        self.serialize_and_write_file()?;
        Ok(true)
    }

    fn backup_db_file() -> Result<u64> {
        let mut backup_file_path = Self::db_file_path().clone();
        backup_file_path.pop();
        backup_file_path.push("backup.json");
        Ok(std::fs::copy(Self::db_file_path().clone(), backup_file_path).unwrap())
    }
    fn serialize_and_write_file(&self) -> Result<()> {
        let serialized_data = serde_json::to_string_pretty(&self)?;
        let Ok(path_as_string) = Self::db_file_path().clone().into_os_string().into_string() else {
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

    fn read_and_deserialize_file() -> Result<Self, JsonStoreError> {
        let Ok(parsed_db_file_path) = Self::db_file_path().clone().into_os_string().into_string()
        else {
            return Err(JsonStoreError::PathNotValid);
        };
        let Ok(data_str) = std::fs::read_to_string(parsed_db_file_path.as_str()) else {
            return Err(JsonStoreError::FileNotFound);
        };
        let Ok(data) = serde_json::from_str(&data_str) else {
            return Err(JsonStoreError::FilecontentNotValid);
        };
        Ok(data)
    }
}
