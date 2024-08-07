use std::{cell::OnceCell, num::NonZero, sync::Arc};
use element::Element;
use tiny_skia::{Color, Pixmap, PixmapMut, Rect};
use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowAttributes}};

use font::{FontStorage, Fonts};

mod context;
mod ext;
pub mod font;

pub mod element;

pub use context::{UiContext, LayoutCtx, PaintCtx};
pub use ext::PixmapExtensions;

use crate::editor::Editor;



/// Data required for window handling and software rendering.
/// Contains softbuffer surface that is presented to the user, and Pixmap in 
/// RGBA format that tiny_skia routines can render to.
struct RenderData {
    window: Arc<Window>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    pixmap: Pixmap,
}

struct UiHandler<'s> {
    render: OnceCell<RenderData>,
    ui: UiContext<'s>,
    root: Editor,
}

pub fn run() {
    let storage = FontStorage::new();
    let ev = match EventLoop::new() {
        Ok(ev) => ev,
        Err(e) => {
            log::error!("Failed to create event loop: {e}");
            return;
        }
    };

    ev.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut ui = UiHandler {
        render: OnceCell::default(),
        ui: UiContext::new(&storage),
        root: Default::default()
    };

    ui.ui.fonts_mut().open("/usr/share/fonts/TTF/DejaVuSans.ttf").unwrap();

    if let Err(e) = ev.run_app(&mut ui) {
        log::error!("Event loop error: {e}");
    }
}

impl RenderData {
    /// Get a mutable reference to the tiny_skia pixel buffer
    pub fn pixmap_mut(&mut self) -> PixmapMut<'_> {
        self.pixmap.as_mut()
    }

    /// Flush the tiny_skia pixel buffer to the physical window, displaying any draw changes that
    /// have occurred onscreen.
    pub fn flush(&mut self) -> Result<(), UiError> {
        let mut buffer = self.surface.buffer_mut()?;

        for (px, out) in self.pixmap.pixels().iter().zip(buffer.iter_mut()) {
            let r = (px.red() as u32 * px.alpha() as u32) >> 8;
            let g = (px.green() as u32 * px.alpha() as u32) >> 8;
            let b = (px.blue() as u32 * px.alpha() as u32) >> 8;
            *out = r << 16 | g << 8 | b;
        }

        buffer.present()?;

        Ok(())
    }

    /// Initialize a new winit window using the given event loop, and attempt to create both
    /// softbuffer display buffers and tiny_skia pixmap for the given window, returning an error if
    /// any resources could not be created
    pub fn create(event_loop: &ActiveEventLoop) -> Result<Self, UiError> {
        let window = event_loop
            .create_window(WindowAttributes::default().with_title("Tachys".to_owned()))?;

        let window = Arc::new(window);

        let ctx = softbuffer::Context::new(window.clone())?;
        let surface = softbuffer::Surface::new(&ctx, window.clone())?;

        let window_size = window.inner_size();
        let pixmap = Pixmap::new(window_size.width, window_size.height).ok_or(UiError::Pixmap)?;

        Ok(Self {
            window,
            surface,
            pixmap,
        })
    }

    /// Resize all pixel buffers to fit the given window size.
    /// Note that any UI elements must also be re-laid out in order to scale to the new window
    /// size.
    pub fn resize(&mut self, sz: PhysicalSize<u32>) -> Result<(), UiError> {
        let (width, height) = NonZero::<u32>::new(sz.width)
            .zip(NonZero::<u32>::new(sz.height))
            .ok_or_else(|| UiError::InvalidSize(sz))?;

        self.surface.resize(width, height)?;
        self.pixmap = Pixmap::new(width.into(), height.into()).ok_or(UiError::Pixmap)?;

        Ok(())
    }
}

impl<'s> ApplicationHandler for UiHandler<'s> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.render.get().is_none() {
            let render_data = match RenderData::create(event_loop) {
                Ok(d) => d,
                Err(e) => {
                    log::error!("Failed to create render context: {e}");
                    return;
                }
            };

            let _ = self.render.set(render_data);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Self { ref mut render, ref mut ui, ref mut root } = self;
        let Some(render) = render.get_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(sz) => {
                root.layout(ui, LayoutCtx { available: Rect::from_xywh(0., 0., 1., 1.).unwrap() }).unwrap();

                if let Err(e) = render.resize(sz) {
                    log::error!("Failed to resize buffers: {e}");
                }
            }
            WindowEvent::RedrawRequested => {
                let mut pixmap = render.pixmap.as_mut();
                
                pixmap.fill(Color::WHITE);

                let paint = PaintCtx {
                    pixmap
                };

                root.paint(ui, paint).unwrap();

                if let Err(e) = render.flush() {
                    log::error!("Failed to flush pixmap to screen: {e}");
                }
            },
            e => {
                root.event(ui, e).unwrap();
            },
        }
    }
}

/// Any error that may occur when initializing the UI, painting, and handling input
#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error("winit OS error: {0}")]
    OsError(#[from] winit::error::OsError),
    #[error("softbuffer error: {0}")]
    Softbuffer(#[from] softbuffer::SoftBufferError),
    #[error("Failed to create tiny_skia pixmap")]
    Pixmap,
    #[error("Failed to resize graphics buffers: invalid size {0:?}")]
    InvalidSize(PhysicalSize<u32>),
}
