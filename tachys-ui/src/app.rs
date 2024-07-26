use std::{num::NonZero, rc::Rc, cell::OnceCell};

use softbuffer::Surface;
use tiny_skia::{Color, Mask, Pixmap, PixmapMut};
use winit::{
    application::ApplicationHandler, error::EventLoopError, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowAttributes, WindowId}
};

use crate::editor::{font::FontStorage, Editor};

pub struct App<'s> {
    window: Option<Rc<Window>>,
    ctx: Option<softbuffer::Context<Rc<Window>>>,
    root: Option<Surface<Rc<Window>, Rc<Window>>>,
    pixmap: Pixmap,
    mask: Mask,
    editor: Editor<'s>,
}

/// Create a winit event loop and run the GUI application to completion
pub fn run() -> Result<(), EventLoopError> {
    let mut store = FontStorage::new();
    
    let mut app = App {
        window: None,
        ctx: None,
        root: None,
        editor: Editor::new(),
        pixmap: Pixmap::new(1, 1).unwrap(),
        mask: Mask::new(1, 1).unwrap(),
    };
    
    let ev = EventLoop::new()?;
    ev.set_control_flow(winit::event_loop::ControlFlow::Wait);

    
    if let Err(e) = app.editor.font_cache.load(&mut store, "/usr/share/fonts/TTF/FiraCodeNerdFontPropo-Medium.ttf") {
        log::error!("Failed to load font: {e}");
    }
    
    ev.run_app(&mut app)
}


impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = match event_loop.create_window(WindowAttributes::default()) {
            Ok(win) => Some(Rc::new(win)),
            Err(e) => {
                log::error!("Failed to create a winit window: {e}");
                None
            }
        };

        self.ctx = self.window.clone().and_then(|w| {
            softbuffer::Context::new(w).map_or_else(
                |e| {
                    log::error!("Failed to create softbuffer context from window handle: {e}");
                    None
                },
                Some,
            )
        });

        self.root = self
            .ctx
            .as_ref()
            .zip(self.window.clone())
            .and_then(|(ctx, window)| {
                softbuffer::Surface::new(&ctx, window).map_or_else(
                    |e| {
                        log::error!("Failed to create softbuffer surface: {e}");
                        None
                    },
                    Some,
                )
            });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(sz) => {
                if let Some(ref mut root) = self.root {
                    let Some((width, height)) = NonZero::new(sz.width).zip(NonZero::new(sz.height))
                    else {
                        return log::error!(
                            "Window resize event received with invalid size {} x {}",
                            sz.width,
                            sz.height
                        );
                    };

                    if let Err(e) = root.resize(width, height) {
                        return log::error!(
                            "Failed to resize root surface in response to window size event: {e}"
                        );
                    }

                    self.pixmap = Pixmap::new(width.into(), height.into()).unwrap();
                    self.mask = Mask::new(width.into(), height.into()).unwrap();
                    self.mask.invert();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(ref mut root) = self.root {
                    let mut buffer = match root.buffer_mut() {
                        Ok(b) => b,
                        Err(e) => {
                            return log::error!(
                                "Failed to get pixel buffer for redraw request: {e}"
                            )
                        }
                    };
                    
                    self.pixmap.fill(Color::WHITE);
                    self.editor.paint(&mut self.pixmap, &mut self.mask);

                    for (px, out) in self.pixmap.pixels().into_iter().zip(buffer.iter_mut())  {
                        let r = (px.red() as u32 * px.alpha() as u32) >> 8;
                        let g = (px.green() as u32 * px.alpha() as u32) >> 8;
                        let b = (px.blue() as u32 * px.alpha() as u32) >> 8;
                        *out = r << 16 | g << 8 | b;
                    }

                    if let Err(e) = buffer.present() {
                        return log::error!("Failed to present pixel buffer to window: {e}");
                    }
                }
            }
            _ => (),
        }
    }
}
