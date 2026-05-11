use std::{ops::Add, path::PathBuf};

use anyhow::Result;

#[derive(Debug)]
pub enum BufSource {
    Scratch,
    File(PathBuf),
    // Terminal
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BufferId(pub usize);

impl Add<BufferId> for BufferId {
    type Output = BufferId;

    fn add(self, rhs: BufferId) -> Self::Output {
        BufferId(self.0 + rhs.0)
    }
}

impl Add<usize> for BufferId {
    type Output = BufferId;

    fn add(self, rhs: usize) -> Self::Output {
        BufferId(self.0 + rhs)
    }
}

#[derive(Debug)]
pub struct Buffer {
    pub id: BufferId,
    pub text: Vec<String>,
    pub source: BufSource,
    pub dirty: bool,
}

impl Buffer {
    pub fn new(id: usize, text: Vec<String>, source: BufSource) -> Self {
        Self {
            id: BufferId(id),
            text,
            source,
            dirty: false,
        }
    }
    pub fn new_empty(id: usize) -> Self {
        Self {
            id: BufferId(id),
            text: Vec::new(),
            source: BufSource::Scratch,
            dirty: false,
        }
    }
    pub fn new_from_file(id: BufferId, filepath: &PathBuf) -> Result<Self, std::io::Error> {
        let text = std::fs::read_to_string(filepath)?;
        Ok(Self {
            id,
            text: text.lines().map(|s| s.to_string()).collect(),
            source: BufSource::File(filepath.clone()),
            dirty: false,
        })
    }
    pub fn name(&self) -> Result<&str> {
        match &self.source {
            BufSource::Scratch => Ok("[No Name]"),
            BufSource::File(path) => Ok(path.to_str().ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid UTF-8 in file path",
            ))?),
        }
    }
}
