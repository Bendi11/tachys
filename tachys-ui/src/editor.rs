use allsorts::{binary::read::ReadScope, cff::CFF, context::Glyph, gpos::Placement, gsub::{FeatureMask, Features, GlyphOrigin, RawGlyph}, outline::{OutlineBuilder, OutlineSink}, pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F}, tables::{glyf::GlyfTable, loca::LocaTable, FontTableProvider, HmtxTable}, tag};
use font::{FontCache, FontStorage};
use tiny_skia::{Color, FillRule, Mask, Paint, Path, PathBuilder, Pixmap, PixmapMut, Shader, Stroke, Transform};

mod rope;
pub mod font;

pub struct Editor<'s> {
    pub font_cache: FontCache<'s>,
}

#[derive(Default)]
struct TinySkiaOutlineVisitor(PathBuilder);

impl<'s> Editor<'s> {
    pub fn new() -> Self {
        Self {
            font_cache: FontCache::new()
        }
    }

    pub fn paint(&mut self, buf: &mut Pixmap, mask: &mut Mask) {
        let font = self.font_cache.font();

        let glyphs = font.map_glyphs("ζ Testing the Font", allsorts::tag::LATN, allsorts::font::MatchingPresentation::NotRequired);

        let shapes = font.shape(glyphs, allsorts::tag::LATN, Some(allsorts::tag::DFLT), &Features::Mask(FeatureMask::all()), None, true).unwrap();
        let hmtx_data = font.font_table_provider.read_table_data(tag::HMTX).unwrap();
        let hmtx = ReadScope::new(&hmtx_data).read_dep::<HmtxTable<'_>>((
            font.maxp_table.num_glyphs as usize,
            font.hhea_table.num_h_metrics as usize
        )).unwrap();

        let cff = font.font_table_provider.read_table_data(tag::LOCA).unwrap();
        let loca = ReadScope::new(&cff).read_dep::<LocaTable<'_>>((font.maxp_table.num_glyphs as usize, font.head_table().unwrap().unwrap().index_to_loc_format)).unwrap();

        let glyf_data = font.font_table_provider.read_table_data(tag::GLYF).unwrap();
        let mut glyf = ReadScope::new(&glyf_data).read_dep::<GlyfTable<'_>>(&loca).unwrap();
        
        let mut placement = Vector2F::zero();
        for shape in shapes {
            let mut builder = PathBuilder::new();
            
            placement.set_x(placement.x() + hmtx.horizontal_advance(shape.get_glyph_index()).unwrap() as f32); 
            log::info!("Position is {:?}", shape.placement);

            if shape.kerning != 0 {
                log::info!("Got kerning {}!", shape.kerning);
            }

            let mut outline = TinySkiaOutlineVisitor::new(builder);

            outline.render_glyphs(&mut glyf, &[shape.glyph]).unwrap();
            if let Some(path) = outline.finish() {
                //log::info!("Stroke {:?}", path);
                buf.fill_path(
                    &path,
                    &Paint { shader: Shader::SolidColor(Color::BLACK), anti_alias: true, force_hq_pipeline: true, ..Default::default() },
                    FillRule::EvenOdd,
                    Transform::from_scale(0.5, -0.5).post_translate(0., (buf.height() - 500) as f32).pre_translate(placement.x(), 0.),
                    None
                );
            }
        }
    }
}

impl OutlineSink for TinySkiaOutlineVisitor {
    fn move_to(&mut self, to: Vector2F) {
        self.0.move_to(to.x(), to.y())
    }

    fn line_to(&mut self, to: Vector2F) {
        self.0.line_to(to.x(), to.y())
    }

    fn quadratic_curve_to(&mut self, ctrl: Vector2F, to: Vector2F) {
        self.0.quad_to(ctrl.x(), ctrl.y(), to.x(), to.y())
    }

    fn cubic_curve_to(&mut self, ctrl: LineSegment2F, to: Vector2F) {
        self.0.cubic_to(ctrl.from_x(), ctrl.from_y(), ctrl.to_x(), ctrl.to_y(), to.x(), to.y())
    }

    fn close(&mut self) {
        self.0.close()
    }
}

impl TinySkiaOutlineVisitor {
    pub fn new(builder: PathBuilder) -> Self {
        Self(builder)
    }

    pub fn finish(self) -> Option<Path> { self.0.finish() }

    pub fn render_glyphs<'a, T: OutlineBuilder>(&mut self, builder: &mut T, glyphs: impl IntoIterator<Item = &'a RawGlyph<()>>) -> Result<(), T::Error>  {
        for glyph in glyphs.into_iter() {
            builder.visit(glyph.glyph_index, self)?;
        }

        Ok(())
    }
}
