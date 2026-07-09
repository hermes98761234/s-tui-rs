mod config;

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
    // ponytail: modes are wired in later tasks; scaffold just proves CLI + config compile
    println!("s-tui-rs scaffold OK (refresh {}s)", cfg.refresh_rate);
    Ok(())
}
