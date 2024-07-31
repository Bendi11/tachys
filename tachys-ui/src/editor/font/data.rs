use skrifa::{
    attribute::Attributes,
    outline::{DrawSettings, HintingInstance, HintingMode},
    prelude::LocationRef,
    raw::TableProvider,
    string::StringId,
    FontRef, MetadataProvider, OutlineGlyphCollection,
};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Point, Rect, Shader, Transform};

use super::{FontError, Glyph, RenderedGlyph};

/// Cached font data referencing a memory mapped file or in-memory buffer in a
/// [FontStorage](crate::editor::font::FontStorage)
pub struct FontDrawData<'s> {
    pub(super) tables: FontRef<'s>,
    pub(super) attr: Attributes,
    pub(super) family: String,
    glyph_outlines: OutlineGlyphCollection<'s>,
}

impl<'s> FontDrawData<'s> {
    pub fn new(data: &'s [u8]) -> Result<Self, FontError> {
        let tables = FontRef::new(data)?;

        let family = tables
            .localized_strings(StringId::FAMILY_NAME)
            .english_or_first()
            .ok_or(FontError::NoFamilyName)?;
        let family = family.chars().collect::<String>();
        let attr = tables.attributes();

        let glyph_outlines = tables.outline_glyphs();

        Ok(Self {
            tables,
            family,
            attr,
            glyph_outlines,
        })
    }

    pub fn render_glyph(&self, glyph: Glyph) -> Result<RenderedGlyph, FontError> {
        const AA_PADDING: u32 = 4;

        let size = glyph.size_px as f32;
        let skrifa_sz = skrifa::instance::Size::new(size);

        let id = self
            .tables
            .charmap()
            .map(glyph.character)
            .ok_or(FontError::NoMap(glyph.character))?;
        let outline = self
            .glyph_outlines
            .get(id)
            .ok_or(FontError::NoMap(glyph.character))?;

        let metric = self.tables.glyph_metrics(skrifa_sz, LocationRef::default());

        let metric_bound = metric.bounds(id).ok_or(FontError::NoMap(glyph.character))?;
        let advance = metric
            .advance_width(id)
            .ok_or(FontError::NoMap(glyph.character))? as i16;

        let hinted = true;
        let hints = HintingInstance::new(
            &self.glyph_outlines,
            skrifa_sz,
            LocationRef::default(),
            HintingMode::Smooth {
                lcd_subpixel: None,
                preserve_linear_metrics: false,
            },
        )
        .map_err(FontError::Draw)?;

        let settings = if hinted {
            DrawSettings::hinted(&hints, false)
        } else {
            DrawSettings::unhinted(skrifa_sz, LocationRef::default())
        };

        let mut builder = TinySkiaOutlineVisitor::default();

        outline
            .draw(settings, &mut builder)
            .map_err(FontError::Draw)?;

        let (pixmap, pos) = match builder.0.finish() {
            Some(path) => {
                let bounds = Rect::from_ltrb(
                    metric_bound.x_min,
                    metric_bound.y_min,
                    metric_bound.x_max,
                    metric_bound.y_max,
                )
                .ok_or(FontError::InvalidBounds)?;

                let pos = Point::from_xy(metric_bound.x_min, size - metric_bound.y_max);

                let mut map = Pixmap::new(
                    bounds.width() as u32 + AA_PADDING,
                    bounds.height() as u32 + AA_PADDING,
                )
                .ok_or(FontError::InvalidBounds)?;

                map.fill_path(
                    &path,
                    &Paint {
                        shader: Shader::SolidColor(Color::BLACK),
                        anti_alias: true,
                        ..Default::default()
                    },
                    FillRule::EvenOdd,
                    Transform::from_translate(
                        -metric_bound.x_min + (AA_PADDING / 2) as f32,
                        -metric_bound.y_min - (AA_PADDING / 2) as f32,
                    )
                    .post_scale(1., -1.)
                    .post_translate(0., bounds.height()),
                    None,
                );

                (Some(map), pos)
            }
            None => (None, Point::default()),
        };

        Ok(RenderedGlyph {
            pixmap,
            pos,
            advance,
        })
    }

    /// Get the font family name of the loaded font
    pub fn family(&self) -> &str {
        &self.family
    }
}

impl<'s> skrifa::raw::TableProvider<'s> for FontDrawData<'s> {
    fn data_for_tag(&self, tag: skrifa::Tag) -> Option<skrifa::raw::FontData<'s>> {
        self.tables.data_for_tag(tag)
    }
}

#[derive(Default)]
struct TinySkiaOutlineVisitor(PathBuilder);

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
