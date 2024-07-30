mod store;
mod cache;

pub use store::FontStorage;
pub use cache::{FontCache, FontId};



#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Failed to read font: {0}")]
    Parse(#[from] skrifa::outline::error::ReadError),
    #[error("No font family name located")]
    NoFamilyName,
}
