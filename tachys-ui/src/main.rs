use log::Level;
use text::font::FontCache;

mod app;
mod text;

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
                _ => "*"
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
    
    let start = std::time::SystemTime::now();
    let mut cache = FontCache::new();
    if let Err(e) = cache.index_dir(&dirs_2::font_dir().unwrap()).and_then(|_| cache.index_dir("/usr/share/fonts")) {
        log::error!("Failed to index font directory: {}", e);
    }

    let end = std::time::SystemTime::now();
    let dur = end.duration_since(start).unwrap();
    log::info!("Loaded fonts in {}ms", dur.as_millis());

    if let Err(e) = app::App::run() {
        log::error!("Failed to run event loop: {e}");
    }
}
