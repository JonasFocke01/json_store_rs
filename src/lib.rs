// TODO: lazy_load function
// TODO: not only one file, but file tree

use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub enum JsonStoreError {
    FileNotFound,
    PathNotValid,
    FilecontentNotValid,
}

pub struct JsonStore<T: Serialize + DeserializeOwned> {
    data: T,
    writes: usize,
    filepath: PathBuf,
}

impl<T> JsonStore<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn load(filepath: PathBuf) -> Result<Self, JsonStoreError> {
        let Ok(parsed_filepath) = filepath.clone().into_os_string().into_string() else {
            return Err(JsonStoreError::PathNotValid);
        };
        let Ok(data_str) = std::fs::read_to_string(parsed_filepath.as_str()) else {
            return Err(JsonStoreError::FileNotFound);
        };
        let Ok(data) = serde_json::from_str(&data_str) else {
            return Err(JsonStoreError::FilecontentNotValid);
        };
        Ok(Self {
            data,
            writes: 0,
            filepath,
        })
    }
    pub fn write(&mut self) {
        self.writes = self.writes.saturating_add(1);
        let data = serde_json::to_string(&self.data).unwrap();
        let _ = std::fs::write(&self.filepath, data);
    }
    pub fn data(&self) -> &T {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}
