
use std::borrow::Cow;

use allsorts::{binary::read::ReadScope, font_data::{DynamicFontTableProvider, FontData}, tables::{self, FontTableProvider, HmtxTable, NameTable, OpenTypeData, OpenTypeFont}, tag, woff2, Font};
use slotmap::SlotMap;

use super::FontError;


pub struct CachedFont<'s> {
    pub family: String,
    pub font: Font<DynamicFontTableProvider<'s>>,
}

slotmap::new_key_type! {
    /// Identifier used to access a [CachedFont] from a [FontCache]
    pub struct FontId;
}

/// Cache containing fonts that have been loaded from in-memory buffers
#[derive(Default)]
pub struct FontCache<'s> {
    loaded: SlotMap<FontId, CachedFont<'s>>,
}

impl<'s> FontCache<'s> {
    /// Attempt to load an opentype of WOFF font from the given buffer, and return the number of
    /// fonts loaded from the buffer (for collections this may be > 1)
    pub fn load(&mut self, buf: &'s [u8]) -> Result<usize, FontError> {
        let font = ReadScope::new(buf).read::<FontData<'_>>()?;
        
        let fonts_count = match font {
            FontData::OpenType(ref f) => match f.data {
                OpenTypeData::Single(_) => 1,
                OpenTypeData::Collection(ref c) => c.offset_tables.len(),
            },
            FontData::Woff(_) => 1,
            FontData::Woff2(ref font) => match font.collection_directory {
                Some(ref col) => col.fonts().count(),
                None => 1,
            }
        };

        for i in 0..fonts_count {
            let tbl_provider = font.table_provider(i)?;
            let entry = CachedFont::create(tbl_provider)?;

            self.loaded.insert(entry);
        }

        Ok(fonts_count)
    }
    
    /// Search the currently loaded fonts for one with a matching font family
    pub fn search(&self, family: &str) -> Option<FontId> {
        self
            .loaded
            .iter()
            .find_map(|(k, v)| v.family.trim().eq_ignore_ascii_case(family.trim()).then_some(k))
    }
    
    /// Get the loaded font data corresponding to the given font ID
    pub fn get<'a>(&'a self, id: FontId) -> &'a CachedFont<'s> {
        &self.loaded[id]
    }
    
    /// Get a mutable reference to the font data for the given font ID
    pub fn get_mut<'a>(&'a mut self, id: FontId) -> &'a mut CachedFont<'s> {
        &mut self.loaded[id]
    }
}

impl<'s> CachedFont<'s> {
    /// Load all required metadata from the given font table
    fn create(font: DynamicFontTableProvider<'s>) -> Result<Self, FontError> {
        let font = Font::new(font)?;
        
        let name_buf = font.font_table_provider.read_table_data(tag::NAME)?;
        let name_tbl = ReadScope::new(&name_buf).read::<NameTable::<'_>>()?;

        let family = name_tbl.string_for_id(NameTable::FONT_FAMILY_NAME)
            .unwrap_or_else(|| "UNKNOWN FONT".to_owned());


        log::info!("Loaded font {}", family);

        Ok(Self {
            family,
            font,
        })
    }
}
