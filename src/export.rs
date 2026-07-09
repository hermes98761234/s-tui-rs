use crate::sources::{Reading, Source};
use std::io::Write;

pub struct Snapshot {
    pub name: &'static str,
    pub unit: &'static str,
    pub readings: Vec<Reading>,
}

pub fn take(sources: &mut [Box<dyn Source>]) -> Vec<Snapshot> {
    sources
        .iter_mut()
        .map(|s| Snapshot {
            name: s.name(),
            unit: s.unit(),
            readings: s.read(),
        })
        .collect()
}

/// {"Temp":{"Core 0":55.0},"Util":{"Avg":12.5,...},...} on one line.
pub fn to_json(snaps: &[Snapshot]) -> String {
    let mut root = serde_json::Map::new();
    for s in snaps {
        let mut m = serde_json::Map::new();
        for r in &s.readings {
            m.insert(r.label.clone(), serde_json::json!(r.value));
        }
        root.insert(s.name.to_string(), serde_json::Value::Object(m));
    }
    serde_json::Value::Object(root).to_string()
}

pub fn to_terminal(snaps: &[Snapshot]) -> String {
    snaps
        .iter()
        .map(|s| {
            let vals = s
                .readings
                .iter()
                .map(|r| format!("{} {:.1}{}", r.label, r.value, s.unit))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}: [{}]", s.name, vals)
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

pub fn csv_header(snaps: &[Snapshot]) -> String {
    let mut cols = vec!["Time".to_string()];
    for s in snaps {
        for r in &s.readings {
            cols.push(format!("{}:{}", s.name, r.label));
        }
    }
    cols.join(",")
}

pub fn csv_row(snaps: &[Snapshot]) -> String {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut cols = vec![t.to_string()];
    for s in snaps {
        for r in &s.readings {
            cols.push(format!("{:.1}", r.value));
        }
    }
    cols.join(",")
}

pub struct CsvLogger {
    file: std::fs::File,
    wrote_header: bool,
}

impl CsvLogger {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self {
            file,
            wrote_header: false,
        })
    }

    /// ponytail: header is written once per run; column drift across runs is the user's problem
    pub fn log(&mut self, snaps: &[Snapshot]) {
        if !self.wrote_header {
            let _ = writeln!(self.file, "{}", csv_header(snaps));
            self.wrote_header = true;
        }
        let _ = writeln!(self.file, "{}", csv_row(snaps));
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::sources::{Reading, Source};

    pub struct FakeSource {
        pub values: Vec<f64>,
    }

    impl Source for FakeSource {
        fn name(&self) -> &'static str {
            "Fake"
        }
        fn unit(&self) -> &'static str {
            "X"
        }
        fn read(&mut self) -> Vec<Reading> {
            self.values
                .iter()
                .enumerate()
                .map(|(i, v)| Reading {
                    label: format!("F{i}"),
                    value: *v,
                })
                .collect()
        }
        fn available(&self) -> bool {
            true
        }
    }

    fn snaps() -> Vec<Snapshot> {
        let mut src: Vec<Box<dyn Source>> = vec![Box::new(FakeSource {
            values: vec![1.0, 2.5],
        })];
        take(&mut src)
    }

    #[test]
    fn json_format() {
        assert_eq!(to_json(&snaps()), r#"{"Fake":{"F0":1.0,"F1":2.5}}"#);
    }

    #[test]
    fn terminal_format() {
        assert_eq!(to_terminal(&snaps()), "Fake: [F0 1.0X, F1 2.5X]");
    }

    #[test]
    fn csv_format() {
        assert_eq!(csv_header(&snaps()), "Time,Fake:F0,Fake:F1");
        let row = csv_row(&snaps());
        assert!(row.ends_with(",1.0,2.5"), "row was: {row}");
    }
}
