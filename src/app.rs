use crate::config::Config;
use crate::export::Snapshot;
use crate::sources::{self, Reading, Source};
use crate::stress::Stress;
use std::collections::VecDeque;

pub const HISTORY: usize = 1000;

pub struct Pane {
    pub name: &'static str,
    pub unit: &'static str,
    pub visible: bool,
    pub latest: Vec<Reading>,
    pub history: VecDeque<f64>,
}

pub struct App {
    sources: Vec<Box<dyn Source>>,
    pub panes: Vec<Pane>,
    pub stress: Stress,
    pub cfg: Config,
    pub show_help: bool,
    pub quit: bool,
}

impl App {
    pub fn new(cfg: Config) -> Self {
        let sources = sources::all_sources(cfg.temp_threshold);
        let panes = sources
            .iter()
            .map(|s| Pane {
                name: s.name(),
                unit: s.unit(),
                visible: !cfg.hidden_sources.contains(&s.name().to_string()),
                latest: Vec::new(),
                history: VecDeque::with_capacity(HISTORY),
            })
            .collect();
        Self {
            sources,
            panes,
            stress: Stress::new(),
            cfg,
            show_help: false,
            quit: false,
        }
    }

    #[cfg(test)]
    pub fn with_sources(sources: Vec<Box<dyn Source>>, cfg: Config) -> Self {
        let panes = sources
            .iter()
            .map(|s| Pane {
                name: s.name(),
                unit: s.unit(),
                visible: true,
                latest: Vec::new(),
                history: VecDeque::with_capacity(HISTORY),
            })
            .collect();
        Self {
            sources,
            panes,
            stress: Stress::new(),
            cfg,
            show_help: false,
            quit: false,
        }
    }

    /// Read all sources; graph series is "Avg" (first reading) for Util, max reading otherwise.
    /// A source returning no readings keeps its stale pane data.
    pub fn tick(&mut self) -> Vec<Snapshot> {
        let mut snaps = Vec::new();
        for (src, pane) in self.sources.iter_mut().zip(self.panes.iter_mut()) {
            let readings = src.read();
            if !readings.is_empty() {
                let graphed = if pane.name == "Util" {
                    readings[0].value // "Avg" is inserted first by UtilSource
                } else {
                    readings.iter().map(|r| r.value).fold(f64::MIN, f64::max)
                };
                if pane.history.len() >= HISTORY {
                    pane.history.pop_front();
                }
                pane.history.push_back(graphed);
                pane.latest = readings;
            }
            snaps.push(Snapshot {
                name: pane.name,
                unit: pane.unit,
                readings: pane.latest.clone(),
            });
        }
        snaps
    }

    pub fn on_key(&mut self, key: char) {
        match key {
            'q' => self.quit = true,
            'h' => self.show_help = !self.show_help,
            's' => self.toggle_stress(false),
            'e' => self.toggle_stress(true),
            c @ '1'..='9' => {
                let i = c as usize - '1' as usize;
                if let Some(p) = self.panes.get_mut(i) {
                    p.visible = !p.visible;
                    let name = p.name.to_string();
                    if p.visible {
                        self.cfg.hidden_sources.retain(|n| n != &name);
                    } else if !self.cfg.hidden_sources.contains(&name) {
                        self.cfg.hidden_sources.push(name);
                    }
                }
            }
            _ => {}
        }
    }

    fn toggle_stress(&mut self, external: bool) {
        if self.stress.running() {
            self.stress.stop();
        } else {
            let n = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4);
            if external {
                let _ = self.stress.start_external(n); // silently no-op if tool missing
            } else {
                self.stress.start_builtin(n);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::tests::FakeSource;

    fn test_app() -> App {
        App::with_sources(
            vec![Box::new(FakeSource {
                values: vec![1.0, 2.0],
            })],
            Config::default(),
        )
    }

    #[test]
    fn tick_records_history_and_latest() {
        let mut app = test_app();
        app.tick();
        assert_eq!(app.panes[0].history.len(), 1);
        assert_eq!(app.panes[0].history[0], 2.0); // max of readings
        assert_eq!(app.panes[0].latest.len(), 2);
    }

    #[test]
    fn history_is_capped() {
        let mut app = test_app();
        for _ in 0..(HISTORY + 10) {
            app.tick();
        }
        assert_eq!(app.panes[0].history.len(), HISTORY);
    }

    #[test]
    fn keys_toggle_state() {
        let mut app = test_app();
        app.on_key('1');
        assert!(!app.panes[0].visible);
        assert_eq!(app.cfg.hidden_sources, vec!["Fake".to_string()]);
        app.on_key('1');
        assert!(app.panes[0].visible);
        assert!(app.cfg.hidden_sources.is_empty());
        app.on_key('h');
        assert!(app.show_help);
        app.on_key('q');
        assert!(app.quit);
    }
}
