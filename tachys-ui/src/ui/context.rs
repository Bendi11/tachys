use tiny_skia::{PixmapMut, Rect};
use super::font::{FontStorage, Fonts};

/// Top-level structure containing all UI state.
pub struct UiContext<'s> {
    fonts: Fonts<'s>,
}

/// Context provided to UI elements during the layout phase
pub struct LayoutCtx {
    pub(super) available: Rect,
}

/// Context provided to UI elements during painting
pub struct PaintCtx<'a> {
    pub(super) pixmap: PixmapMut<'a>,
}

impl LayoutCtx {
    /// Get the total available space for layout
    pub const fn available(&self) -> Rect {
        self.available
    }
}

impl<'a> PaintCtx<'a> {
    /// Get a mutable reference to the pixel buffer for this update
    pub fn pixmap(&mut self) -> &mut PixmapMut<'a> {
        &mut self.pixmap
    }
}

impl<'s> UiContext<'s> {
    /// Create UI context from an immutable reference to the given font storage
    pub fn new(storage: &'s FontStorage) -> Self {
        Self {
            fonts: Fonts::new(storage),
        }
    }

    pub fn fonts(&self) -> &Fonts<'s> {
        &self.fonts
    }

    pub fn fonts_mut(&mut self) -> &mut Fonts<'s> {
        &mut self.fonts
    }
}
