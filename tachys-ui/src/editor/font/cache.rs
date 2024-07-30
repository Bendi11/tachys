use skrifa::{attribute::Attributes, raw::{tables::name::Name, TableProvider}, string::StringId, FontRef, MetadataProvider};
use slotmap::SlotMap;

use super::FontError;

/// Cached font data referencing a memory mapped file or in-memory buffer in a
/// [FontStorage](crate::editor::font::FontStorage)
pub struct CachedFont<'s> {
    tables: FontRef<'s>,
    attr: Attributes,
    family: String,
}

/// Container of loaded fonts for the editor, enabling higher-level font loading and searching by
/// attributes.
#[derive(Default,)]
pub struct FontCache<'s> {
    loaded: SlotMap<FontId, CachedFont<'s>>,
}

impl<'s> FontCache<'s> {
    /// Attempt to load a font from the given buffer, and return the ID of the stored font if
    /// successful
    pub fn load(&mut self, buf: &'s [u8]) -> Result<FontId, FontError> {
        let font = CachedFont::new(buf)?;
        let stored = self.loaded.iter().find_map(|(k, v)| {
            (v.attr == font.attr && v.family.eq_ignore_ascii_case(&font.family)).then_some(k)
        });
        if let Some(stored) = stored {
            return Ok(stored);
        }

        let id = self.loaded.insert(font);
        Ok(id)
    }
    
    /// Get an immutable reference to the cached font with the given ID
    pub fn get<'a>(&'a self, id: FontId) -> &'a CachedFont {
        &self.loaded[id]
    }
    
    /// Attempt to locate a font of the given family, with optional attributes.
    /// If not given, the first font located of the family will be returned
    pub fn search<'a, 'f>(&'a self, family: &'f str, attr: Option<Attributes>) -> impl Iterator<Item = FontId> + 'f
    where 'a: 'f {
        let family = family.trim();

        self
            .loaded
            .iter()
            .filter_map(move |(k, v)| {
                (attr.map(|attr| attr == v.attr).unwrap_or(true) && v.family.eq_ignore_ascii_case(family)).then_some(k)
            })
    }
}

impl<'s> CachedFont<'s> {
    pub fn new(data: &'s [u8]) -> Result<Self, FontError> {
        let tables = FontRef::new(data)?;
        
        let family = tables.localized_strings(StringId::FAMILY_NAME).english_or_first().ok_or(FontError::NoFamilyName)?;
        let family = family.chars().collect::<String>();
        let attr = tables.attributes();

        Ok(Self {
            tables,
            family,
            attr,
        })
    }
}

slotmap::new_key_type! {
    /// Identifier used to access a [CachedFont] from a [FontCache]
    pub struct FontId;
}

impl std::fmt::Display for FontId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0.as_ffi())
    }
}
