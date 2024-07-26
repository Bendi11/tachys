use std::{borrow::Cow, fs::File, io::Read};

use memmap2::Mmap;

use super::FontError;

/// Structure held for each loaded font file maintaining either a memory-mapped file handle or an
/// in-memory buffer
struct FontBuffer {
    storage: FontBufferStorage,
}

enum FontBufferStorage {
    Map(Mmap),
    Memory(Cow<'static, [u8]>),
}

/// Object maintaining in-memory buffers and memory-mapped font files from which a
/// [FontCache](super::cache::FontCache) can maintain references to font data.
#[derive(Default)]
pub struct FontStorage {
    loaded: Vec<FontBuffer>,
}

impl FontStorage {
    /// Get a new font storage with no files loaded
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a file from the given path and return a reference to its contents
    pub fn load<'a, P: AsRef<std::path::Path>>(
        &'a mut self,
        path: P,
    ) -> Result<&'a [u8], FontError> {
        let mut file = File::open(path)?;

        let storage = match unsafe { Mmap::map(&file) } {
            Ok(map) => FontBufferStorage::Map(map),
            Err(e) => match e.kind() {
                std::io::ErrorKind::Unsupported => {
                    log::info!("Memory mapping unsupported, falling back to in-memory buffer");

                    let mut buf = Vec::with_capacity(
                        file.metadata().map(|m| m.len() as usize).unwrap_or(65536),
                    );
                    file.read_to_end(&mut buf)?;
                    FontBufferStorage::Memory(Cow::from(buf))
                }
                _ => return Err(e.into()),
            },
        };

        let buffer = FontBuffer { storage };

        self.loaded.push(buffer);

        Ok(self.loaded.last().unwrap().storage.as_ref())
    }
}

impl AsRef<[u8]> for FontBufferStorage {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Map(map) => map.as_ref(),
            Self::Memory(cow) => cow.as_ref(),
        }
    }
}
