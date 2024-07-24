use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;


fn main() {
    colog::init();
    
    let events = EventLoop::new().unwrap();
    events.set_control_flow(ControlFlow::Wait);
    
    let mut app = App::default();
    events.run_app(&mut app).unwrap();
}
