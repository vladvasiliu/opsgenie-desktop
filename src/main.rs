use crate::config::Config;
use crate::opsgenie::OpsGenieInterface;
use color_eyre::Result;

mod config;
mod opsgenie;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let config = Config::from_clap();
    setup_logger()?;

    let mut interface = OpsGenieInterface::new_with_config(config);
    interface.run().await?;

    Ok(())
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{:5}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
