use std::marker::PhantomData;

use tiny_skia::{PixmapMut, Rect};

mod context;
mod ext;
mod font;

pub use ext::PixmapExtensions;

pub use context::{LayoutCtx, PaintCtx, Ui};
use winit::event_loop::EventLoop;

use font::FontStorage;

pub fn run() {
    let mut storage = FontStorage::new();
    let ev = match EventLoop::new() {
        Ok(ev) => ev,
        Err(e) => {
            log::error!("Failed to create event loop: {e}");
            return;
        }
    };

    ev.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut ui = Ui::new(&mut storage);

    if let Err(e) = ev.run_app(&mut ui) {
        log::error!("Event loop error: {e}");
    }
}

pub enum UiEvent {}

pub trait Element {
    fn layout(&mut self, ctx: LayoutCtx<'_>) -> Rect;

    fn paint(&mut self, ctx: PaintCtx<'_>) -> Result<(), PaintError>;

    fn event(&mut self, event: UiEvent);
}

#[derive(Debug, thiserror::Error)]
pub enum PaintError {}
