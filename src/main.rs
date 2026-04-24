use std::time::Duration;

use anyhow::Result;
use net_debug_rs::cli::args::parse_args;
use net_debug_rs::crossterm;

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args();
    let tick_rate = Duration::from_millis(100);
    crossterm::run(tick_rate, true, args).await?;
    Ok(())
}
