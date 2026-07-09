mod config;
#[allow(dead_code)]
mod export;
mod sources;
mod stress;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "s-tui-rs",
    version,
    about = "Terminal CPU monitoring and stress testing UI"
)]
pub struct Cli {
    /// Print one reading as JSON and exit
    #[arg(short, long)]
    pub json: bool,
    /// Print one reading as plain text and exit
    #[arg(short, long)]
    pub terminal: bool,
    /// Append readings to a CSV file while running
    #[arg(short, long)]
    pub csv: bool,
    /// CSV file path
    #[arg(long, default_value = "s-tui.csv")]
    pub csv_file: String,
    /// Refresh rate in seconds (default 2.0, or saved config value)
    #[arg(short, long)]
    pub refresh_rate: Option<f64>,
    /// High temperature alert threshold in °C (default 80, or saved config value)
    #[arg(long)]
    pub t_thresh: Option<f32>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut cfg = config::load();
    if let Some(r) = cli.refresh_rate {
        cfg.refresh_rate = r;
    }
    if let Some(t) = cli.t_thresh {
        cfg.temp_threshold = t;
    }

    if cli.json || cli.terminal {
        let mut srcs = sources::all_sources(cfg.temp_threshold);
        // first read primes cpu_usage and RAPL deltas; second read is the real sample
        let _ = export::take(&mut srcs);
        std::thread::sleep(std::time::Duration::from_millis(200));
        let snaps = export::take(&mut srcs);
        if cli.json {
            println!("{}", export::to_json(&snaps));
        } else {
            println!("{}", export::to_terminal(&snaps));
        }
        return Ok(());
    }

    // ponytail: TUI mode lands in the next task
    println!("TUI mode not implemented yet; use --json or --terminal");
    Ok(())
}
