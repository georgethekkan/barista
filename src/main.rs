use ::tracing::{debug, info};
use anyhow::Result;
use barista::{args, clean, tracing};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();
    let cli = args::parse();

    tracing::init(cli.verbose);

    debug!("Starting");

    use args::Commands::*;
    match cli.command {
        Build => barista::build(vec![]).await?,
        Run { bin_args } => barista::run(&bin_args).await?,
        Clean => clean()?,
    }

    info!("Finished in {} ms", start.elapsed().as_millis());

    Ok(())
}
