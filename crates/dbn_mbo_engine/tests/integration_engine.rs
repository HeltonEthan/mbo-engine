use chrono::NaiveDate;
use dbn::record::MboMsg;
use dbn_mbo_engine::{api::latency::UnitNormalLatency, prelude::*};
use std::path::PathBuf;

// cargo test -p dbn_mbo_engine --test integration_engine engine_test --release
// samply record target\release\deps\integration_engine-d09ece3e58622256.exe
#[test]
fn engine_test() -> anyhow::Result<()> {
    let cfg = Config::new(
        PathBuf::from("C:/Users/helto/GLBX-20250915-NGKNUL4VBG"),
        NaiveDate::from_ymd_opt(2025, 05, 12).unwrap(),
        NaiveDate::from_ymd_opt(2025, 08, 11).unwrap(),
        0,
    );
    run(&cfg, || logic, || UnitNormalLatency::new(25_000_000, 1_000_000))?;
    Ok(())
}

fn logic(mbo: &MboMsg) -> Option<action::Request> {
    _ = mbo;
    None
}
