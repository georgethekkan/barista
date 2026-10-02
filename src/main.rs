use ::tracing::{debug, info};
use anyhow::{Ok, Result};
use barista::{
    args::{self, Cli},
    clean, tracing,
};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();
    let cli = args::parse();

    tracing::init(cli.verbose, cli.quiet);

    debug!("Starting");

    run(cli).await?;

    info!("Finished in {} ms", start.elapsed().as_millis());

    Ok(())
}

pub async fn run(cli: Cli) -> Result<()> {
    use args::Commands::*;
    match cli.command {
        Build => barista::build().await?,
        Run { bin_args } => barista::run(&bin_args).await?,
        Clean => clean()?,
    }

    Ok(())
}
