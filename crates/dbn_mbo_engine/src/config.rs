use std::path::PathBuf;

pub struct Config {
    pub dir: PathBuf,
    pub start: u64,
    pub end: u64,
}
