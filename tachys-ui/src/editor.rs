use font::{EditorFonts, FontId};
use skrifa::{outline::DrawSettings, prelude::{LocationRef, Size}, raw::TableProvider, MetadataProvider};
use tiny_skia::{BlendMode, Color, FillRule, Mask, Paint, Path, PathBuilder, Pixmap, PixmapPaint, Point, Rect, Shader, Stroke, Transform};

pub mod font;

pub struct Editor<'s> {
    pub font_cache: EditorFonts<'s>,
    pub selected_font: Option<FontId>,
}

impl<'s> Editor<'s> {
    pub fn new() -> Self {
        Self {
            font_cache: EditorFonts::default(),
            selected_font: None,
        }
    }

    pub fn paint(&mut self, buf: &mut Pixmap, mask: &mut Mask) {
        let Some(font) = self.selected_font.map(|f| self.font_cache.get_mut(f)) else {
            return
        };
        
        let mut pos = 0i32;
        for c in "Hello, World! λ ƒ".chars() {
            let render = font.glyph(font::Glyph { character: c, size_px: 24 }).unwrap();
            let glyph_pos = Point::from_xy(pos as f32, 0f32) + render.pos;
            if let Some(ref pixmap) = render.pixmap {
                let mut path = PathBuilder::new();
                path.move_to(glyph_pos.x, glyph_pos.y);
                path.line_to(glyph_pos.x + pixmap.width() as f32, glyph_pos.y);
                path.line_to(glyph_pos.x + pixmap.width() as f32, pixmap.height() as f32 + glyph_pos.y);
                path.line_to(glyph_pos.x, pixmap.height() as f32 + glyph_pos.y);
                path.line_to(glyph_pos.x, glyph_pos.y);
                let path = path.finish().unwrap();
                //buf.stroke_path(&path, &tiny_skia::Paint { shader: Shader::SolidColor(Color::from_rgba8(255, 0, 0, 255)), ..Default::default() }, &Stroke { width: 1f32, ..Default::default() }, Transform::identity(), None);
                buf.draw_pixmap(glyph_pos.x as i32, glyph_pos.y as i32, pixmap.as_ref(), &PixmapPaint::default(), Transform::default(), None);
            }

            pos += render.advance as i32;
        }
    }
}
