use crate::config::Config;
use crate::orderbook::market::Market;
use crate::parser::file;
use dbn::{
    Action,
    decode::{
        DecodeStream,
        dbn::{Decoder, MetadataDecoder},
    },
    record::MboMsg,
};
use fallible_streaming_iterator::FallibleStreamingIterator;
use std::{fs::File, io::BufReader, path::PathBuf};


/// Run is the entry point of the engine
/// 
/// It iterates through each file and creates a dbn stream for each,
/// it passes a clone of mbo_msg to the limit orderbook for reconstruction. 
/// Then passes a reference of mbo to the callback function 'logic'.
pub fn run<F: FnMut(&MboMsg) -> Option<Action>>(mut logic: F, cfg: &Config) -> anyhow::Result<()> {
    let start_unix = cfg.start_unix()?;
    let end_unix = cfg.end_unix()?;
    let mut market = Market::new();
    for path in file::get_files(&cfg)?.iter() { // Find and iterate through valid files
        let mut dbn_stream = Decoder::from_zstd_file(path)?.decode_stream::<MboMsg>();
        while let Some(mbo_msg) = dbn_stream.next()? {
            if mbo_msg.ts_recv < start_unix {
                continue
            }
            if mbo_msg.ts_recv > end_unix {
                break
            }
            market.apply(mbo_msg.clone());
            logic(mbo_msg);
        }
    }
    Ok(())
}

// Returns the metadata of a path.
pub fn decode_metadata(path: &PathBuf) -> anyhow::Result<dbn::Metadata> {
    let reader = zstd::stream::Decoder::new(BufReader::new(File::open(path)?)).unwrap();
    Ok(MetadataDecoder::new(reader).decode()?)
}
