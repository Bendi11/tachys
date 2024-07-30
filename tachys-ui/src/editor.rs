use font::{FontCache, FontId, FontStorage};
use tiny_skia::{Color, FillRule, Mask, Paint, Path, PathBuilder, Pixmap, PixmapMut, Shader, Stroke, Transform};

mod rope;
pub mod font;

pub struct Editor<'s> {
    pub font_cache: FontCache<'s>,
    pub selected_font: Option<FontId>,
}

#[derive(Default)]
struct TinySkiaOutlineVisitor(PathBuilder);

impl<'s> Editor<'s> {
    pub fn new() -> Self {
        Self {
            font_cache: FontCache::default(),
            selected_font: None,
        }
    }

    pub fn paint(&mut self, buf: &mut Pixmap, mask: &mut Mask) {
        let Some(selected_font) = self.selected_font else { return };
        let cached = self.font_cache.get(selected_font);
        

        
    }
}
