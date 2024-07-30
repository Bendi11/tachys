
use std::{borrow::Cow, fs::File, io::Read, path::PathBuf, rc::Rc};

use memmap2::Mmap;
use once_map::OnceMap;

use super::FontError;


enum FontBuffer {
    Map(Mmap),
    Memory(Cow<'static, [u8]>),
}

/// Object maintaining in-memory buffers and memory-mapped font files from which a
/// [FontCache](super::cache::FontCache) can maintain references to font data.
#[derive(Default)]
pub struct FontStorage {
    loaded: OnceMap<PathBuf, Rc<FontBuffer>>,
}

impl FontStorage {
    /// Get a new font storage with no files loaded
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a file from the given path and return a reference to its contents
    pub fn load<'a, P: AsRef<std::path::Path>>(
        &'a self,
        path: P,
    ) -> Result<&'a [u8], FontError> {
        let path = path.as_ref().to_owned();
        let mut file = File::open(&path)?;

        let storage = match unsafe { Mmap::map(&file) } {
            Ok(map) => FontBuffer::Map(map),
            Err(e) => match e.kind() {
                std::io::ErrorKind::Unsupported => {
                    log::info!("Memory mapping unsupported, falling back to in-memory buffer");

                    let mut buf = Vec::with_capacity(
                        file.metadata().map(|m| m.len() as usize).unwrap_or(65536),
                    );
                    file.read_to_end(&mut buf)?;
                    FontBuffer::Memory(Cow::from(buf))
                }
                _ => return Err(e.into()),
            },
        };

        let buf = self.loaded.insert(path, move |_| Rc::from(storage));

        Ok(buf.as_ref())
    }
}

impl AsRef<[u8]> for FontBuffer {
    fn as_ref<'a>(&'a self) -> &'a [u8] {
        match self {
            Self::Map(map) => map.as_ref(),
            Self::Memory(cow) => cow.as_ref(),
        }
    }
}
