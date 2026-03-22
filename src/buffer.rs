use std::path::PathBuf;

pub enum BufSource {
    Scratch,
    File(PathBuf),
    // Terminal
}

pub struct Buffer {
    pub id: usize,
    pub text: Vec<String>,
    pub source: BufSource,
    pub dirty: bool,
}

impl Buffer {
    pub fn new(id: usize, text: Vec<String>, source: BufSource) -> Self {
        Self {
            id,
            text,
            source,
            dirty: false,
        }
    }
    pub fn new_empty(id: usize) -> Self {
        Self {
            id,
            text: Vec::new(),
            source: BufSource::Scratch,
            dirty: false,
        }
    }
    pub fn new_from_file(id: usize, filepath: &PathBuf) -> Result<Self, std::io::Error> {
        let text = std::fs::read_to_string(filepath)?;
        Ok(Self {
            id,
            text: text.lines().map(|s| s.to_string()).collect(),
            source: BufSource::File(filepath.clone()),
            dirty: false,
        })
    }
}
