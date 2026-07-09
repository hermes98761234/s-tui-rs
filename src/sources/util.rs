use super::{Reading, Source};
use sysinfo::System;

pub struct UtilSource {
    sys: System,
}

impl UtilSource {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu(); // first sample; usage becomes meaningful from the second refresh
        Self { sys }
    }
}

impl Source for UtilSource {
    fn name(&self) -> &'static str {
        "Util"
    }
    fn unit(&self) -> &'static str {
        "%"
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
                value: c.cpu_usage() as f64,
            })
            .collect();
        if !v.is_empty() {
            let avg = v.iter().map(|r| r.value).sum::<f64>() / v.len() as f64;
            v.insert(
                0,
                Reading {
                    label: "Avg".into(),
                    value: avg,
                },
            );
        }
        v
    }
    fn available(&self) -> bool {
        !self.sys.cpus().is_empty()
    }
}
