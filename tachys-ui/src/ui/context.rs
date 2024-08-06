use std::marker::PhantomData;

use tiny_skia::{PixmapMut, Rect};

use super::Ui;



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
