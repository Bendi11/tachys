use tiny_skia::{Color, PixmapPaint, Point, Rect, Transform};
use winit::event::{ElementState, WindowEvent};

use crate::ui::{element::Element, font::Glyph, LayoutCtx, PaintCtx, PixmapExtensions, Ui, UiError};


#[derive(Default,)]
pub struct Editor {
    edit: String,
    cursor: usize,
}

impl Element for Editor {
    fn layout(&mut self, _ui: &Ui<'_>, ctx: LayoutCtx) -> Result<Rect, UiError> {
        Ok(ctx.available())
    }

    fn paint(&mut self, ui: &mut Ui<'_>, mut ctx: PaintCtx<'_>) -> Result<(), UiError> {
        let pixmap = ctx.pixmap();
        
        let font_id = ui.fonts_mut().search("DejaVu Sans", None).next().unwrap();
        let font = ui.fonts_mut().get_mut(font_id);

        let mut pos = Point::default();

        for (i, ch) in self.edit.char_indices() {
            let glyph = Glyph {
                character: ch,
                size_px: 20
            };

            let render = font.glyph(glyph).unwrap();
        
            if let Some(ref glyph_map) = render.pixmap {
                pixmap.draw_pixmap(
                    pos.x.round() as i32,
                    pos.y.round() as i32,
                    glyph_map.as_ref(),
                    &PixmapPaint::default(),
                    Transform::from_translate(render.pos.x, render.pos.y),
                    None
                );
            }

            if i == self.cursor - 1 {
                pixmap.outline_rect(Rect::from_xywh(pos.x, pos.y, render.advance as f32, 20f32).unwrap(), Color::from_rgba8(255, 0, 0, 255));
            }

            pos.x += render.advance as f32;
        }

        Ok(())
    }

    fn event(&mut self, ui: &mut Ui<'_>, event: winit::event::WindowEvent) -> Result<(), UiError> {
        match event {
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                if event.state == ElementState::Pressed {
                    if let Some(text) = event.text {
                        self.edit.insert_str(self.cursor, text.as_str());
                        self.cursor += 1;
                        ui.request_redraw();
                    }
                }

                Ok(())
            },
            _ => Ok(())
        }
    }
}
