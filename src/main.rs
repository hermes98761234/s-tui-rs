mod app;
mod config;
mod export;
mod sources;
mod stress;
mod ui;

use clap::Parser;
use config::Config;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

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

struct TermGuard;

impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
        let _ = crossterm::terminal::disable_raw_mode();
    }
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

    run_tui(&cli, cfg)
}

fn run_tui(cli: &Cli, cfg: Config) -> anyhow::Result<()> {
    let mut app = app::App::new(cfg);
    let mut csv = if cli.csv {
        export::CsvLogger::new(&cli.csv_file).ok()
    } else {
        None
    };

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    let _guard = TermGuard; // restores the terminal even on panic/early return
    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;

    let tick = std::time::Duration::from_secs_f64(app.cfg.refresh_rate.max(0.1));
    let mut last = std::time::Instant::now() - tick; // fire the first tick immediately
    while !app.quit {
        if last.elapsed() >= tick {
            let snaps = app.tick();
            if let Some(c) = csv.as_mut() {
                c.log(&snaps);
            }
            last = std::time::Instant::now();
        }
        terminal.draw(|f| ui::draw(f, &app))?;
        let wait = tick
            .saturating_sub(last.elapsed())
            .min(std::time::Duration::from_millis(250));
        if crossterm::event::poll(wait)? {
            if let Event::Key(k) = crossterm::event::read()? {
                if k.kind == KeyEventKind::Press {
                    match k.code {
                        KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.quit = true;
                        }
                        KeyCode::Char(c) => app.on_key(c),
                        KeyCode::Esc => app.quit = true,
                        KeyCode::F(1) => app.show_help = !app.show_help,
                        _ => {}
                    }
                }
            }
        }
    }
    app.stress.stop();
    config::save(&app.cfg);
    Ok(())
}
