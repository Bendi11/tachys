use std::{cell::OnceCell, marker::PhantomData, num::NonZero, sync::Arc};

use tiny_skia::{Pixmap, PixmapMut, Rect};
use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::ActiveEventLoop, window::{Window, WindowAttributes}};

use crate::ui::font::{EditorFonts, FontStorage};

/// Data required for window handling and software rendering
struct RenderData {
    window: Arc<Window>,
    ctx: softbuffer::Context<Arc<Window>>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    pixmap: Pixmap,
}


/// Top-level structure containing all UI state
pub struct Ui<'s> {
    render: OnceCell<RenderData>,
    fonts: EditorFonts<'s>,
}

/// Context provided to UI elements during the layout phase
pub struct LayoutCtx<'a> {
    _unused: PhantomData<&'a ()>,
    available: Rect,
}

/// Context provided to UI elements during painting
pub struct PaintCtx<'a> {
    pixmap: PixmapMut<'a>,
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
        let window = event_loop.create_window(WindowAttributes::default()
            .with_title("Tachys".to_owned())
        )?;

        let window = Arc::new(window);

        let ctx = softbuffer::Context::new(window.clone())?;
        let surface = softbuffer::Surface::new(&ctx, window.clone())?;
        
        let window_size = window.inner_size();
        let pixmap = Pixmap::new(window_size.width, window_size.height).ok_or(UiError::Pixmap)?;

        Ok(Self {
            window,
            ctx,
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

impl<'s> Ui<'s> {
    pub fn new(storage: &'s FontStorage) -> Self {
        Self {
            render: OnceCell::default(),
            fonts: EditorFonts::new(storage),
        }
    }
}

impl<'s> ApplicationHandler for Ui<'s> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.render.get().is_none() {
            let render_data = match RenderData::create(event_loop) {
                Ok(d) => d,
                Err(e) => {
                    log::error!("Failed to create render context: {e}");
                    return
                },
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
        let Some(render) = self.render.get_mut() else { return };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit()
            },
            WindowEvent::Resized(sz) => {
                if let Err(e) = render.resize(sz) {
                    log::error!("Failed to resize buffers: {e}");
                }
            },
            WindowEvent::RedrawRequested => {
                if let Err(e) = render.flush() {
                    log::error!("Failed to flush pixmap to screen: {e}");
                }
            }
            _ => (),
        }
    }
}


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
