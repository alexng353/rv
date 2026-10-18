use std::{
    ops::{Add, Index, IndexMut},
    path::PathBuf,
};

use anyhow::Result;

use crate::window::BufferCursor;

#[derive(Debug)]
pub enum BufSource {
    Scratch,
    File(PathBuf),
    // Terminal
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BufferId(pub usize);

impl Index<BufferId> for Vec<Buffer> {
    type Output = Buffer;

    fn index(&self, id: BufferId) -> &Self::Output {
        &self[id.0]
    }
}

impl IndexMut<BufferId> for Vec<Buffer> {
    fn index_mut(&mut self, id: BufferId) -> &mut Self::Output {
        &mut self[id.0]
    }
}

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

impl From<usize> for BufferId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Buffer {
    pub fn new(id: impl Into<BufferId>, text: Vec<String>, source: BufSource) -> Self {
        Self {
            id: id.into(),
            text,
            source,
            dirty: false,
        }
    }
    pub fn new_empty(id: impl Into<BufferId>) -> Self {
        Self {
            id: id.into(),
            text: Vec::new(),
            source: BufSource::Scratch,
            dirty: false,
        }
    }
    pub fn new_from_file(
        id: impl Into<BufferId>,
        filepath: &PathBuf,
    ) -> Result<Self, std::io::Error> {
        let text = std::fs::read_to_string(filepath)?;
        Ok(Self {
            id: id.into(),
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
    pub fn new_from_str(id: impl Into<BufferId>, text: &str) -> Self {
        Self {
            id: id.into(),
            text: text.lines().map(|s| s.to_string()).collect(),
            source: BufSource::Scratch,
            dirty: false,
        }
    }

    pub fn write(&mut self) -> Result<()> {
        match &self.source {
            BufSource::Scratch => {
                anyhow::bail!("Cannot write to scratch buffer");
            }
            BufSource::File(path) => {
                std::fs::write(path, self.text.join("\n"))?;
            }
        }

        self.dirty = false;

        Ok(())
    }

    pub fn delete_lines(&mut self, from: usize, to: usize) {
        self.dirty = true;
        self.text.drain(from..=to);
    }

    pub fn delete_range(&mut self, from: BufferCursor, to: BufferCursor, inclusive: bool) {
        self.dirty = true;

        if from.line == to.line {
            let line = &mut self.text[from.line];
            if inclusive {
                line.drain(from.col..=to.col);
            } else {
                line.drain(from.col..to.col);
            }
        } else {
            let line = &mut self.text[from.line];
            line.drain(from.col..);

            self.text.drain(from.line + 1..to.line.saturating_sub(1));

            let last = &mut self.text[to.line];

            if inclusive {
                last.drain(..=to.col);
            } else {
                last.drain(..to.col);
            }
        }
    }
}
