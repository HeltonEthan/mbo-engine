use std::{fs::File, path::PathBuf, io::BufReader};
use color_eyre::eyre::{Result};
use dbn::{
    decode::{DecodeStream, dbn::{Decoder, MetadataDecoder}
    },
    record::MboMsg,
};
use fallible_streaming_iterator::FallibleStreamingIterator;

pub fn dbn_stream(path: &PathBuf, start_unix: Option<u64>, end_unix: Option<u64>) -> Result<()> {
    let mut dbn_stream = Decoder::from_zstd_file(path)?.decode_stream::<MboMsg>();
    
    while let Ok(Some(mbo_msg)) = dbn_stream.next() {
        if let Some(start_unix) = start_unix { if mbo_msg.ts_recv < start_unix { continue; } }
        if let Some(end_unix) = end_unix { if mbo_msg.ts_recv > end_unix { break; } }

        _ = mbo_msg;
    }

    Ok(())
}

pub fn decode_metadata(path: &PathBuf) -> Result<dbn::Metadata> {
    let reader = zstd::stream::Decoder::new(BufReader::new(File::open(path)?)).unwrap();
    Ok(MetadataDecoder::new(reader).decode()?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn dbn_stream() -> Result<()> {
        let path = PathBuf::from(
            r"C:/Users/helto/GLBX-20250915-NGKNUL4VBG/glbx-mdp3-20250512-20250517.mbo.dbn.zst"
        );
        let end_unix = None;

        println!("-----dbn_stream-----");
        println!("file: {:#?}", path);

        let mut dbn_stream = Decoder::from_zstd_file(path)?.decode_stream::<MboMsg>();

        let mut count: i64 = 0;

        while let Ok(Some(mbo_msg)) = dbn_stream.next() {
            count += 1;

            if let Some(end_unix) = end_unix {
                if mbo_msg.ts_recv > end_unix {
                    println!("mbo_msg: {:#?}", mbo_msg);
                    println!("end_unix: {}", end_unix);

                    break;
                }
            }

            _ = mbo_msg;
        }

        println!("count: {}", count);
        println!();

        Ok(())
    }

    #[test]
    pub fn decode_metadata() -> Result<()> {
        let path = PathBuf::from(r"C:/Users/helto/GLBX-20250915-NGKNUL4VBG/glbx-mdp3-20250512-20250517.mbo.dbn.zst");
        let reader = zstd::stream::Decoder::new(BufReader::new(File::open(path)?)).unwrap();

        let _decode = MetadataDecoder::new(reader).decode()?;
        
        Ok(())
    }
}
