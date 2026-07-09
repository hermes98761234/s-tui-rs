use super::{Reading, Source};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

struct Domain {
    name: String,
    energy_path: PathBuf,
    max_uj: u64,
    last_uj: u64,
    last_t: Instant,
}

pub struct PowerSource {
    domains: Vec<Domain>,
}

impl PowerSource {
    pub fn new() -> Self {
        let mut domains = Vec::new();
        if let Ok(entries) = fs::read_dir("/sys/class/powercap") {
            for e in entries.flatten() {
                if !e.file_name().to_string_lossy().starts_with("intel-rapl:") {
                    continue;
                }
                let p = e.path();
                let energy_path = p.join("energy_uj");
                let Ok(name) = fs::read_to_string(p.join("name")) else {
                    continue;
                };
                let Some(last_uj) = fs::read_to_string(&energy_path)
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok())
                else {
                    continue;
                };
                let max_uj = fs::read_to_string(p.join("max_energy_range_uj"))
                    .ok()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(u64::MAX);
                domains.push(Domain {
                    name: name.trim().to_string(),
                    energy_path,
                    max_uj,
                    last_uj,
                    last_t: Instant::now(),
                });
            }
        }
        domains.sort_by(|a, b| a.name.cmp(&b.name));
        Self { domains }
    }
}

/// Watts from two RAPL energy_uj samples, handling counter wraparound.
pub fn watts(prev_uj: u64, now_uj: u64, max_uj: u64, dt_secs: f64) -> f64 {
    if dt_secs <= 0.0 {
        return 0.0;
    }
    let delta = if now_uj >= prev_uj {
        now_uj - prev_uj
    } else {
        max_uj - prev_uj + now_uj
    };
    delta as f64 / 1e6 / dt_secs
}

impl Source for PowerSource {
    fn name(&self) -> &'static str {
        "Power"
    }
    fn unit(&self) -> &'static str {
        "W"
    }
    fn read(&mut self) -> Vec<Reading> {
        let mut out = Vec::new();
        for d in &mut self.domains {
            let Some(now_uj) = fs::read_to_string(&d.energy_path)
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
            else {
                continue;
            };
            let now = Instant::now();
            let w = watts(
                d.last_uj,
                now_uj,
                d.max_uj,
                now.duration_since(d.last_t).as_secs_f64(),
            );
            d.last_uj = now_uj;
            d.last_t = now;
            out.push(Reading {
                label: d.name.clone(),
                value: w,
            });
        }
        out
    }
    fn available(&self) -> bool {
        !self.domains.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::watts;

    #[test]
    fn watts_simple() {
        assert_eq!(watts(0, 2_000_000, u64::MAX, 2.0), 1.0);
    }

    #[test]
    fn watts_wraparound() {
        assert_eq!(watts(9_000_000, 1_000_000, 10_000_000, 2.0), 1.0);
    }

    #[test]
    fn watts_zero_dt_is_zero() {
        assert_eq!(watts(0, 1_000_000, u64::MAX, 0.0), 0.0);
    }
}
