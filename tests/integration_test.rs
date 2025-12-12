use mbo_engine::Config;
use color_eyre::eyre::Result;

#[test]
fn integration_test() -> Result<()> {
    let config = Config::new(
        mbo_engine::str_to_pathbuf("C:/Users/helto/GLBX-20250915-NGKNUL4VBG".to_string())?,
        mbo_engine::str_to_naivedate("2025-05-12".to_string()).unwrap(),
        mbo_engine::str_to_naivedate("2025-05-17".to_string()).unwrap(),
    );

    println!("config: {:#?}", config);

    mbo_engine::run(
        |mbo_msg| {
            _ = mbo_msg;

            return None;
        },
        &config,
    )?;
    
    Ok(())
}
