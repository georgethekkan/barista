use ::tracing::info;
use anyhow::Result;
use barista::{args, clean, tracing};

#[tokio::main]
async fn main() -> Result<()> {
    tracing::init();

    info!("Starting");

    let cli = args::parse();

    use args::Commands::*;
    match cli.command {
        Build => barista::build(vec![]).await?,
        Run { bin_args } => barista::run(bin_args).await?,
        Clean => clean()?,
    }

    Ok(())
}
