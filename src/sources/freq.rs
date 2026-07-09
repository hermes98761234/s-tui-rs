use super::{Reading, Source};
use sysinfo::System;

pub struct FreqSource {
    sys: System,
}

impl FreqSource {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu();
        Self { sys }
    }
}

#[cfg(target_os = "linux")]
fn throttle_count() -> Option<f64> {
    let mut total: u64 = 0;
    let mut found = false;
    let entries = std::fs::read_dir("/sys/devices/system/cpu").ok()?;
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !name.starts_with("cpu") || !name[3..].chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let p = e.path().join("thermal_throttle/core_throttle_count");
        if let Some(n) = std::fs::read_to_string(p)
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
        {
            total += n;
            found = true;
        }
    }
    found.then_some(total as f64)
}

impl Source for FreqSource {
    fn name(&self) -> &'static str {
        "Freq"
    }
    fn unit(&self) -> &'static str {
        "MHz"
    }
    fn read(&mut self) -> Vec<Reading> {
        self.sys.refresh_cpu();
        let mut v: Vec<Reading> = self
            .sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, c)| Reading {
                label: format!("Core {i}"),
                value: c.frequency() as f64,
            })
            .collect();
        #[cfg(target_os = "linux")]
        if let Some(t) = throttle_count() {
            v.push(Reading {
                label: "Throttle #".into(),
                value: t,
            });
        }
        v
    }
    fn available(&self) -> bool {
        !self.sys.cpus().is_empty()
    }
}
