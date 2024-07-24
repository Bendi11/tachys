use std::{num::NonZero, rc::Rc};

use softbuffer::Surface;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

#[derive(Default)]
pub struct App {
    window: Option<Rc<Window>>,
    ctx: Option<softbuffer::Context<Rc<Window>>>,
    root: Option<Surface<Rc<Window>, Rc<Window>>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = match event_loop.create_window(WindowAttributes::default()) {
            Ok(win) => {
                Some(Rc::new(win))
            },
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

        self.root = self.ctx.as_ref().zip(self.window.clone()).and_then(|(ctx, window)| {
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
            WindowEvent::Resized(sz) => if let Some(ref mut root) = self.root {
                let Some((width, height)) = NonZero::new(sz.width).zip(NonZero::new(sz.height)) else {
                    return log::error!("Window resize event received with invalid size {} x {}", sz.width, sz.height);
                };

                if let Err(e) = root.resize(width, height) {
                    return log::error!("Failed to resize root surface in response to window size event: {e}");
                }
            },
            WindowEvent::RedrawRequested => if let Some(ref mut root) = self.root {
                let mut buffer = match root.buffer_mut() {
                    Ok(b) => b,
                    Err(e) => return log::error!("Failed to get pixel buffer for redraw request: {e}"),
                };

                for px in buffer.iter_mut() {
                    *px = 0x00FF00;
                }

                if let Err(e) = buffer.present() {
                    return log::error!("Failed to present pixel buffer to window: {e}");
                }
            }
            _ => (),
        }
    }
}
