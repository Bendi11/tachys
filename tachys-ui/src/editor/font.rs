mod data;
mod store;

use std::collections::{hash_map, HashMap};

use data::FontDrawData;
use skrifa::attribute::Attributes;
use slotmap::SlotMap;
pub use store::FontStorage;
use tiny_skia::{Pixmap, Point};

pub struct EditorFont<'s> {
    data: FontDrawData<'s>,
    glyph_cache: HashMap<Glyph, RenderedGlyph>,
}

/// Container of loaded fonts for the editor, enabling higher-level font loading and searching by
/// attributes.
#[derive(Default)]
pub struct EditorFonts<'s> {
    loaded: SlotMap<FontId, EditorFont<'s>>,
}

/// A glyph that has been rendered to a temporary buffer and cached to reduce rasterization work
#[derive(Clone)]
pub struct RenderedGlyph {
    pub pixmap: Option<Pixmap>,
    pub pos: Point,
    pub advance: i16,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Glyph {
    pub character: char,
    pub size_px: u16,
}

impl<'s> EditorFonts<'s> {
    pub fn open<'a, P: AsRef<std::path::Path>>(
        &'a mut self,
        storage: &'s FontStorage,
        path: P,
    ) -> Result<FontId, FontError> {
        let buf = storage.load(path)?;
        self.load(buf)
    }

    /// Attempt to load a font from the given buffer, and return the ID of the stored font if
    /// successful
    pub fn load(&mut self, buf: &'s [u8]) -> Result<FontId, FontError> {
        let font = EditorFont::new(buf)?;
        let stored = self.loaded.iter().find_map(|(k, v)| {
            (v.data.attr == font.data.attr
                && v.data
                    .family
                    .eq_ignore_ascii_case(&font.data.family))
            .then_some(k)
        });
        if let Some(stored) = stored {
            return Ok(stored);
        }

        log::info!("Loaded font {}", font.data.family());
        let id = self.loaded.insert(font);

        Ok(id)
    }

    /// Get an immutable reference to the cached font with the given ID
    pub fn get(&self, id: FontId) -> &EditorFont {
        &self.loaded[id]
    }

    /// Get a mutable reference to the cached font with the given ID
    pub fn get_mut<'a>(&'a mut self, id: FontId) -> &'a mut EditorFont<'s> {
        &mut self.loaded[id]
    }

    /// Attempt to locate a font of the given family, with optional attributes.
    /// If not given, the first font located of the family will be returned
    pub fn search<'a, 'f>(
        &'a self,
        family: &'f str,
        attr: Option<Attributes>,
    ) -> impl Iterator<Item = FontId> + 'f
    where
        'a: 'f,
    {
        let family = family.trim();

        self.loaded.iter().filter_map(move |(k, v)| {
            (attr.map(|attr| attr == v.data.attr).unwrap_or(true)
                && v.data.family.eq_ignore_ascii_case(family))
            .then_some(k)
        })
    }
}

impl<'s> EditorFont<'s> {
    /// Read a font from the given buffer and initialize all caches empty
    pub fn new(buf: &'s [u8]) -> Result<Self, FontError> {
        Ok(Self {
            data: FontDrawData::new(buf)?,
            glyph_cache: HashMap::new(),
        })
    }

    /// Return the cached glyph for the given specifier or rasterize and return it
    pub fn glyph(&mut self, spec: Glyph) -> Result<&RenderedGlyph, FontError> {
        match self.glyph_cache.entry(spec) {
            hash_map::Entry::Occupied(occ) => Ok(occ.into_mut()),
            hash_map::Entry::Vacant(vacant) => {
                Ok(vacant.insert(self.data.render_glyph(spec)?))
            }
        }
    }
    
    /// Get the font family name of this font
    pub fn family(&self) -> &str {
        self.data.family()
    }
}

slotmap::new_key_type! {
    /// Identifier used to access an [EditorFont] from a [FontCache]
    pub struct FontId;
}

impl std::fmt::Display for FontId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0.as_ffi())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Failed to read font: {0}")]
    Parse(#[from] skrifa::outline::error::ReadError),
    #[error("Failed to draw glyph: {0}")]
    Draw(skrifa::outline::error::DrawError),
    #[error("No font family name located")]
    NoFamilyName,
    #[error("Font does not contain a mapping for the character: '{0}'")]
    NoMap(char),
    #[error("Font provides invalid bounding box for glyph")]
    InvalidBounds,
}
