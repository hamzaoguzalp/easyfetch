use std::io::{Result, Error, ErrorKind};
use std::fs::{write, read_to_string};
use std::path::Path;

pub struct FileService {
    file_path: String,
    file_content: String,
}

impl FileService {
    pub fn new(file_path: String) -> Self {
        let file_content = read_to_string(&file_path).unwrap_or_else(|_| String::new());
        FileService { file_path, file_content }
    }
    pub fn read_file(&self) -> Result<String> {
        read_to_string(&self.file_path)
    }
    pub fn write_file(&mut self, content: &str) -> Result<()> {
        write(&self.file_path, content)?;
        self.file_content = content.to_string();
        Ok(())
    }
    pub fn get_content(&self) -> &str {
        &self.file_content
    }
}
