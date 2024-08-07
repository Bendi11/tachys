use log::Level;

mod ui;
mod editor;

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        println!(
            "[{}] {}",
            match record.level() {
                Level::Error => "E",
                Level::Warn => "W",
                _ => "*",
            },
            record.args()
        );
    }

    fn flush(&self) {}
}

static LOG: Logger = Logger;


fn main() {
    log::set_max_level(log::LevelFilter::Info);
    if let Err(e) = log::set_logger(&LOG) {
        eprintln!("Failed to set logger: {}", e);
    }

    ui::run()
}
