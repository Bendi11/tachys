mod gap;

pub enum DocumentNode {
    Header {
        level: u8,
        text: String,
    }
}
