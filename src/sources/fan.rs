use super::{Reading, Source};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FanSource {
    fans: Vec<(String, PathBuf)>,
}

/// Scan a hwmon-style tree for fan*_input files. Split out for testability.
pub fn scan(base: &Path) -> Vec<(String, PathBuf)> {
    let mut fans = Vec::new();
    let Ok(chips) = fs::read_dir(base) else {
        return fans;
    };
    for chip in chips.flatten() {
        let chip_name = fs::read_to_string(chip.path().join("name"))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| chip.file_name().to_string_lossy().to_string());
        let Ok(files) = fs::read_dir(chip.path()) else {
            continue;
        };
        for f in files.flatten() {
            let fname = f.file_name().to_string_lossy().to_string();
            if fname.starts_with("fan") && fname.ends_with("_input") {
                fans.push((
                    format!("{chip_name} {}", fname.trim_end_matches("_input")),
                    f.path(),
                ));
            }
        }
    }
    fans.sort();
    fans
}

impl FanSource {
    pub fn new() -> Self {
        Self {
            fans: scan(Path::new("/sys/class/hwmon")),
        }
    }
}

impl Source for FanSource {
    fn name(&self) -> &'static str {
        "Fan"
    }
    fn unit(&self) -> &'static str {
        "RPM"
    }
    fn read(&mut self) -> Vec<Reading> {
        self.fans
            .iter()
            .filter_map(|(label, path)| {
                let rpm = fs::read_to_string(path).ok()?.trim().parse::<f64>().ok()?;
                Some(Reading {
                    label: label.clone(),
                    value: rpm,
                })
            })
            .collect()
    }
    fn available(&self) -> bool {
        !self.fans.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::scan;

    #[test]
    fn scan_finds_fan_inputs() {
        let base = std::env::temp_dir().join(format!("stui_fan_test_{}", std::process::id()));
        let chip = base.join("hwmon0");
        std::fs::create_dir_all(&chip).unwrap();
        std::fs::write(chip.join("name"), "testchip\n").unwrap();
        std::fs::write(chip.join("fan1_input"), "1200\n").unwrap();
        std::fs::write(chip.join("temp1_input"), "42000\n").unwrap();
        let fans = scan(&base);
        assert_eq!(fans.len(), 1);
        assert_eq!(fans[0].0, "testchip fan1");
        std::fs::remove_dir_all(&base).unwrap();
    }
}
