use dbn::{
    decode::{DecodeStream, dbn::Decoder},
    record::MboMsg,
};
use fallible_streaming_iterator::FallibleStreamingIterator;

use crate::{
    config::Config,
    stream::{Ext, process_dir},
};

pub fn run(cfg: &Config) -> anyhow::Result<()> {
    for (ext, path) in process_dir(cfg.dir.as_path(), cfg.start, cfg.end)? {
        match ext {
            Ext::Zst => {
                let mut dbn_stream = Decoder::from_zstd_file(path)?.decode_stream::<MboMsg>();
                while let Some(mbo) = dbn_stream.next()? {
                    if mbo.ts_recv < cfg.start {
                        continue;
                    } else if mbo.ts_recv >= cfg.end {
                        break;
                    }
                }
            },
            Ext::Dbn => {
                let mut dbn_stream = Decoder::from_file(path)?.decode_stream::<MboMsg>();
                while let Some(mbo) = dbn_stream.next()? {
                    if mbo.ts_recv < cfg.start {
                        continue;
                    } else if mbo.ts_recv >= cfg.end {
                        break;
                    }
                }
            },
        }
    }
    Ok(())
}
