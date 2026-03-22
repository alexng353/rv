use thiserror::Error;

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("buffer {0} not found")]
    BufferNotFound(usize),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}
