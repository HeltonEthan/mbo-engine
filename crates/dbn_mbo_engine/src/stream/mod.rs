use dbn::decode::DbnMetadataDecoder;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

mod parallel;
mod serial;

pub(super) enum Ext {
    Zst,
    Dbn,
}

pub(super) fn process_dir(path: &Path, start: u64, end: u64) -> anyhow::Result<Vec<(Ext, PathBuf)>> {
    let mut files: Vec<(Ext, PathBuf)> = Vec::new();
    for file in fs::read_dir(path)? {
        let file = file?.path();
        match file.extension() {
            Some(ext) if ext == OsStr::new("zst") => {
                let reader = zstd::Decoder::new(BufReader::new(File::open(&file)?))?;
                if check(reader, start, end)? {
                    files.push((Ext::Zst, file));
                }
            },
            Some(ext) if ext == OsStr::new("dbn") => {
                let reader = BufReader::new(File::open(&file)?);
                if check(reader, start, end)? {
                    files.push((Ext::Dbn, file));
                }
            },
            _ => {},
        }
    }
    Ok(files)
}

fn check<R: Read>(reader: R, start: u64, end: u64) -> anyhow::Result<bool> {
    let metadata = DbnMetadataDecoder::new(reader).decode()?;
    let m_start = metadata.start;
    let m_end = match metadata.end {
        Some(t) => u64::from(t),
        None => return Ok(false),
    };
    Ok(m_start <= end && start <= m_end)
}
