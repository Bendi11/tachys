use font::{EditorFonts, FontId};
use tiny_skia::{
    Color, Mask, Pixmap, PixmapPaint, Point, Rect, Transform
};

use crate::ui::PixmapExtensions;

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

    pub fn paint(&mut self, buf: &mut Pixmap, _mask: &mut Mask) {
        let time = std::time::SystemTime::now();
        let Some(font) = self.selected_font.map(|f| self.font_cache.get_mut(f)) else {
            return;
        };

        let family = font.family().to_owned();
        let mut advance = 0;
        for c in family.chars() {
            let render = font
                .glyph(font::Glyph {
                    character: c,
                    size_px: 220,
                })
                .unwrap();
            let glyph_pos = Point::from_xy(advance as f32, 0f32) + render.pos;
            if let Some(ref pixmap) = render.pixmap {
                buf.outline_rect(
                    Rect::from_xywh(glyph_pos.x, glyph_pos.y, pixmap.width() as f32, pixmap.height() as f32).unwrap(),
                    Color::from_rgba8(255, 0, 0, 255)
                );

                buf.draw_pixmap(
                    glyph_pos.x.round() as i32,
                    glyph_pos.y.round() as i32,
                    pixmap.as_ref(),
                    &PixmapPaint::default(),
                    Transform::default(),
                    None,
                );
            }

            advance += render.advance as i32;
        }

        let end = std::time::SystemTime::now();
        let duration = end.duration_since(time).unwrap();
        log::info!("Render {}ms", duration.as_millis())
    }
}
