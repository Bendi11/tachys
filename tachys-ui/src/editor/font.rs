use std::fs::File;

use allsorts::{binary::read::{ReadBuf, ReadScope, ReadScopeOwned}, font_data::{DynamicFontTableProvider, FontData}, tables::{OffsetTableFontProvider, OpenTypeData, OpenTypeFont}, Font};
use memmap2::Mmap;

pub struct FontStorage {
    pub loaded: Vec<Mmap>,
}

pub struct FontCache<'s> {
    loaded: Option<Font<DynamicFontTableProvider<'s>>>,
}

impl<'s> FontCache<'s> {
    pub fn new() -> Self {
        Self {
            loaded: None,
        }
    }

    pub fn load<P: AsRef<std::path::Path>>(&mut self, store: &'s mut FontStorage, path: P) -> Result<(), FontError> {
        let scope = store.open(path)?;
        let font_file = scope.read::<FontData<'s>>()?;
        
        let count = match font_file {
            FontData::OpenType(ref otf) => match otf.data {
                OpenTypeData::Single(_) => 1,
                OpenTypeData::Collection(ref c) => c.offset_tables.len()
            },
            FontData::Woff(ref woff) => woff.woff_header.num_tables as usize,
            FontData::Woff2(ref woff2) => woff2.woff_header.num_tables as usize
        };

        for i in 0..count {
            let provider = font_file.table_provider(i)?;

            let font = Font::new(provider)?;
            log::info!("Loaded {} glyphs", font.num_glyphs());

            self.loaded = Some(font);
        }

        Ok(())
    }

    pub fn font(&mut self) -> &mut Font<DynamicFontTableProvider<'s>> {
        self.loaded.as_mut().unwrap()
    }
}

impl FontStorage {
    pub fn new() -> Self {
        Self {
            loaded: Vec::new(),
        }
    }

    pub fn open<'a, P: AsRef<std::path::Path>>(&'a mut self, path: P) -> Result<ReadScope<'a>, FontError> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file) }?;
        self.loaded.push(mmap);
        Ok(ReadScope::new(self.loaded.last_mut().unwrap()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Failed to parse font: {0}")]
    ParseFont(#[from] allsorts::error::ParseError),
    #[error("Read error when indexing font: {0}")]
    ReadFont(#[from] allsorts::error::ReadWriteError),
}
