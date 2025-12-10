use color_eyre::eyre::Result;
use std::{
    fs,
    num::NonZero,
    path::PathBuf
};
use std::ffi::OsStr;

use crate::parser::dbn;
use crate::Config;

pub fn run(config: Config) -> Result<()> {
    for path in get_files(&config)?{
        dbn::dbn_stream(&path, Some(config.start_unix()?), Some(config.end_unix()?))?;
    }

    Ok(())
}

//gets the proper files to be run from the directory
pub fn get_files(config: &Config) -> Result<Vec<PathBuf>> {
    let mut files_in_dir = Vec::new();

    for file in fs::read_dir(config.dir())? {
        let file = file?;
        let path = file.path();

        if !path.is_file() || path.extension() != Some(OsStr::new("zst")) {
            continue;
        }

        let file_metadata = dbn::decode_metadata(&path)?;

        if config.start_unix()? <= file_metadata.start && file_metadata.start <= config.end_unix()? ||
        Some(NonZero::new(config.start_unix()?).unwrap()) <= file_metadata.end && file_metadata.end <= Some(NonZero::new(config.end_unix()?).unwrap()) {
            files_in_dir.push(path);
        }
    }

    assert!(files_in_dir.len() != 0);

    Ok(files_in_dir)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn get_files() -> Result<()> {
        println!("-----get_files-----");

        let config = Config::new(
            "C:/Users/helto/GLBX-20250915-NGKNUL4VBG".to_string(),
            "2025-05-14".to_string(),
            "2025-05-23".to_string(),
        )?;

        println!("config: {:#?}", config);
        println!("start_unix: {}", config.start_unix()?);
        println!("end_unix: {}", config.end_unix()?);

        let mut files_in_dir = Vec::new();

        for file in fs::read_dir(config.dir())? {
            let file = file?;
            let path = file.path();

            println!("path extension: {:#?}", path.extension());

            if !path.is_file() || path.extension() != Some(OsStr::new("zst")) {
                continue;
            }

            let file_metadata = dbn::decode_metadata(&path)?;

            println!("file_metadata.start: {:#?}", file_metadata.start);
            println!("file_metadata.end: {:#?}", file_metadata.end);

            if config.start_unix()? <= file_metadata.start && file_metadata.start <= config.end_unix()? ||
            Some(NonZero::new(config.start_unix()?).unwrap()) <= file_metadata.end && file_metadata.end <= Some(NonZero::new(config.end_unix()?).unwrap()) {
                files_in_dir.push(path);
            }
        }

        println!("files in directory: {:#?}", files_in_dir);
        println!();

        Ok(())
    }
}
