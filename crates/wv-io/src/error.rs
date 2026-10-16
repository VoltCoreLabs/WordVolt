#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("not a valid WordVolt file: {0}")]
    Parse(String),
    #[error("unsupported file type: .{0}")]
    Unsupported(String),
}
