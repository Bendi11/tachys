use memmap2::Mmap;
use swash::FontDataRef;
use thiserror::Error;
use walkdir::WalkDir;
use std::{collections::BTreeMap, fs::File, path::{Path, PathBuf}, rc::Rc};


#[derive(Debug, Clone,)]
struct FontCacheEntry {
    pub family: String,
    pub attributes: swash::Attributes,
    pub path: PathBuf,
    pub offset: u32,
    pub data: Option<Rc<[u8]>>,
}


/// Cache created by scanning the available .otf and .ttf files on a machine used to lookup fonts
/// by name and features
#[derive(Debug,)]
pub struct FontCache {
    map: BTreeMap<swash::CacheKey, FontCacheEntry>,
}

impl FontCache {
    /// Create a new empty font cache with no stored fonts
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
    
    /// Index all located font files in the given directory.
    /// Returns [FontError] when reading the directory fails, and logs any errors that occur when
    /// loading individual font files.
    /// Returns the count of indexed fonts when loading succeeds
    pub fn index_dir<P: AsRef<Path> + ?Sized>(&mut self, dir: &P) -> Result<usize, FontError> {
        let entries = WalkDir::new(dir);

        let mut font_count = 0;

        for entry in entries {
            let entry = entry?; 
            let path = entry.path();
            if path.extension().map(|e| e.eq_ignore_ascii_case("ttf") || e.eq_ignore_ascii_case("otf")) == Some(true) {
                if self.map.iter().find(|(_, p)| p.path == path).is_none() {
                    font_count += match self.index(path) {
                        Ok(count) => count,
                        Err(e) => {
                            log::warn!("Failed to index font file {}: {}", path.display(), e);
                            0
                        },
                    };
                }
            }
        }

        log::info!("Loaded {} fonts from directory {}, size: {}", font_count, dir.as_ref().display(), self.map.len() * std::mem::size_of::<FontCacheEntry>());

        Ok(font_count)
    }
    
    /// Index the font file located at the given path
    /// Returns the number of fonts that have been added to the cache
    pub fn index<T: ToOwned<Owned = PathBuf> + ?Sized>(&mut self, font_path: &T) -> Result<usize, FontError> {
        let font_path = font_path.to_owned();

        let file = File::open(&font_path)?;

        let buf = unsafe { Mmap::map(&file) }?;

        let fonts = FontDataRef::new(&buf).ok_or(FontError::ParseFont)?;

        let mut loaded = 0;

        for font in fonts.fonts() {
            if font.offset != 0 {
                log::info!("Got font {}", font.offset);
            }
            let Some(family) = font.localized_strings().find_by_id(swash::StringId::Family, None) else {
                log::warn!("Failed to retrieve font family for font loaded from {}", font_path.display());
                continue
            };

            if !family.is_decodable() {
                log::warn!("Font loaded from {} at offset {}: Font family string  could not be decoded", font_path.display(), font.offset);
                continue
            }

            let family = family.chars().collect::<String>();
            log::debug!("Indexed font {}:{} from {} at offset {}", family, font.attributes(), font_path.display(), font.offset);

            let entry = FontCacheEntry {
                family,
                attributes: font.attributes(),
                path: font_path.clone(),
                offset: font.offset,
                data: None,
            };

            self.map.insert(font.key, entry);
            loaded += 1;
        }

        Ok(loaded)
    }
}

#[derive(Debug, Error)]
pub enum FontError {
    #[error("IO Error: {:?}", 0)]
    IO(#[from] std::io::Error),
    #[error("Directory traversal error: {}", 0)]
    WalkDir(#[from] walkdir::Error),
    #[error("Font could not be read from file")]
    ParseFont,
}
