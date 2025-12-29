use chrono::{NaiveDate, NaiveTime};
use dbn_mbo_engine::{
    config::Config, enums::Ack, lob::book::{Book, LobMbo}, stream::hotloop::{self, Mbo, RxMsg}
};
use std::path::PathBuf;

// cargo test -p dbn_mbo_engine --test integration engine_test --release
// samply record
#[test]
fn engine_test() -> anyhow::Result<()> {
    let start = NaiveDate::from_ymd_opt(2025, 05, 12)
        .unwrap()
        .and_time(NaiveTime::from_hms_nano_opt(0, 0, 0, 0).unwrap())
        .and_utc()
        .timestamp() as u64;
    let end = NaiveDate::from_ymd_opt(2025, 05, 17)
        .unwrap()
        .and_time(NaiveTime::from_hms_nano_opt(0, 0, 0, 0).unwrap())
        .and_utc()
        .timestamp() as u64;
    let cfg = Config::new(
        PathBuf::from("C:/Users/helto/GLBX-20250915-NGKNUL4VBG"),
        start * 1_000_000_000,
        end * 1_000_000_000,
        std::thread::available_parallelism()?.get(),
        65_536,
    );
    let rx_msg = RxMsg {
        make_rm: || move |_mbo: Mbo| {},
        make_ra: || move |_ack: Ack| {},
    };
    hotloop::run(&cfg, rx_msg)?;
    Ok(())
}
