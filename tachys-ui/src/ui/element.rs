use tiny_skia::Rect;
use winit::event::WindowEvent;

use super::{context::{LayoutCtx, PaintCtx}, UiContext, UiError};


/// Trait implemented by all UI elements with methods to perform layout and painting
pub trait Element {
    fn layout(&mut self, ui: &UiContext<'_>, ctx: LayoutCtx) -> Result<Rect, UiError>;

    fn paint(&mut self, ui: &mut UiContext<'_>, ctx: PaintCtx<'_>) -> Result<(), UiError>;

    fn event(&mut self, ui: &mut UiContext<'_>, event: WindowEvent) -> Result<(), UiError>;
}
