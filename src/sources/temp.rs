use super::{Reading, Source};
use sysinfo::Components;

pub struct TempSource {
    /// ponytail: used by Task 3 (UI pane coloring for high temp)
    #[allow(dead_code)]
    pub threshold: f32,
    available: bool,
}

impl TempSource {
    pub fn new(threshold: f32) -> Self {
        let available = !Components::new_with_refreshed_list().is_empty();
        Self {
            threshold,
            available,
        }
    }
}

impl Source for TempSource {
    fn name(&self) -> &'static str {
        "Temp"
    }
    fn unit(&self) -> &'static str {
        "°C"
    }
    fn read(&mut self) -> Vec<Reading> {
        // ponytail: re-enumerate each tick (2s cadence) instead of chasing sysinfo's
        // per-component refresh API; also picks up hotplugged sensors for free
        let comps = Components::new_with_refreshed_list();
        let all: Vec<Reading> = comps
            .iter()
            .map(|c| Reading {
                label: c.label().to_string(),
                value: c.temperature() as f64,
            })
            .collect();
        let cpu: Vec<Reading> = all
            .iter()
            .filter(|r| {
                let l = r.label.to_lowercase();
                ["core", "cpu", "package", "tdie", "tctl", "soc"]
                    .iter()
                    .any(|k| l.contains(k))
            })
            .cloned()
            .collect();
        if cpu.is_empty() {
            all
        } else {
            cpu
        }
    }
    fn available(&self) -> bool {
        self.available
    }
}
