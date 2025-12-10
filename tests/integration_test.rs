use mbo_engine;
use color_eyre::eyre::Result;

#[test]
fn test_run() -> Result<()> {
    let config0 = mbo_engine::Config::new(
        "C:/Users/helto/GLBX-20250915-NGKNUL4VBG".to_string(),
        "2025-05-14".to_string(),
        "2025-05-23".to_string(),
    )?;

    mbo_engine::run(config0)?;
    
    Ok(())
}
