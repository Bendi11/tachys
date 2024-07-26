mod cache;
mod store;

pub use store::FontStorage;
pub use cache::{FontCache, FontId};



#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Failed to parse font: {0}")]
    ParseFont(#[from] allsorts::error::ParseError),
    #[error("Read error when indexing font: {0}")]
    ReadFont(#[from] allsorts::error::ReadWriteError),
}
