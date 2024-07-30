use font::{EditorFonts, FontId, FontStorage};
use skrifa::{instance::Location, outline::DrawSettings, prelude::{LocationRef, Size}, raw::TableProvider, MetadataProvider};
use tiny_skia::{BlendMode, Color, FillRule, Mask, Paint, Path, PathBuilder, Pixmap, PixmapMut, Shader, Stroke, Transform};

mod rope;
pub mod font;

pub struct Editor<'s> {
    pub font_cache: EditorFonts<'s>,
    pub selected_font: Option<FontId>,
}

#[derive(Default)]
struct TinySkiaOutlineVisitor(PathBuilder);

impl<'s> Editor<'s> {
    pub fn new() -> Self {
        Self {
            font_cache: EditorFonts::default(),
            selected_font: None,
        }
    }

    pub fn paint(&mut self, buf: &mut Pixmap, mask: &mut Mask) {
        let Some(selected_font) = self.selected_font else { return };
        let cached = self.font_cache.get(selected_font);
        let ppem = cached.head().unwrap().units_per_em();
        log::info!("PPEM is {}", ppem);
        
        let lambda = cached.charmap().map('λ').unwrap();

        let outline = cached.outline_glyphs().get(lambda).unwrap();

        let mut visitor = TinySkiaOutlineVisitor(PathBuilder::new());
        outline.draw(DrawSettings::unhinted(Size::new(ppem as f32), LocationRef::default()), &mut visitor).unwrap();

        let path = visitor.0.finish().unwrap();

        buf.fill_path(
            &path,
            &Paint { shader: Shader::SolidColor(Color::BLACK), anti_alias: true, blend_mode: BlendMode::default(), ..Default::default() },
            FillRule::EvenOdd,
            Transform::from_scale(1. / ppem as f32, -1. / ppem as f32)
                .post_translate(0., 1.)
                .post_scale(64., 64.),
            None
        );
    }
}

impl skrifa::outline::OutlinePen for TinySkiaOutlineVisitor {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y)
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y)
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.0.quad_to(cx0, cy0, x, y)
    }
    
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.0.cubic_to(cx0, cy0, cx1, cy1, x, y)
    }

    fn close(&mut self) {
        self.0.close()
    }
}
