use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

#[allow(dead_code)]
pub struct Stress {
    stop: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
    external: Option<std::process::Child>,
}

#[allow(dead_code)]
impl Stress {
    pub fn new() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(false)),
            workers: Vec::new(),
            external: None,
        }
    }

    pub fn running(&self) -> bool {
        !self.workers.is_empty() || self.external.is_some()
    }

    pub fn mode(&self) -> &'static str {
        if !self.workers.is_empty() {
            "builtin"
        } else if self.external.is_some() {
            "external"
        } else {
            "off"
        }
    }

    /// Spin `workers` threads on a sqrt burn loop (s-tui's hashlib/numpy strategy analog).
    pub fn start_builtin(&mut self, workers: usize) {
        if self.running() {
            return;
        }
        self.stop.store(false, Ordering::SeqCst);
        for _ in 0..workers.max(1) {
            let stop = self.stop.clone();
            self.workers.push(std::thread::spawn(move || {
                let mut x = 1.0f64;
                while !stop.load(Ordering::Relaxed) {
                    for i in 1..10_000u32 {
                        x = std::hint::black_box((x + f64::from(i)).sqrt());
                    }
                }
            }));
        }
    }

    /// Launch stress-ng or stress if installed; false when neither exists.
    pub fn start_external(&mut self, workers: usize) -> bool {
        if self.running() {
            return true;
        }
        for tool in ["stress-ng", "stress"] {
            match std::process::Command::new(tool)
                .args(["--cpu", &workers.max(1).to_string()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(child) => {
                    self.external = Some(child);
                    return true;
                }
                Err(_) => continue,
            }
        }
        false
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        for w in self.workers.drain(..) {
            let _ = w.join();
        }
        if let Some(mut child) = self.external.take() {
            // ponytail: kill the parent only — stress/stress-ng reap their own workers
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for Stress {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Default for Stress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Stress;

    #[test]
    fn builtin_starts_and_stops() {
        let mut s = Stress::new();
        assert!(!s.running());
        assert_eq!(s.mode(), "off");
        s.start_builtin(2);
        assert!(s.running());
        assert_eq!(s.mode(), "builtin");
        std::thread::sleep(std::time::Duration::from_millis(50));
        s.stop();
        assert!(!s.running());
        assert_eq!(s.mode(), "off");
    }

    #[test]
    fn stop_when_idle_is_noop() {
        let mut s = Stress::new();
        s.stop();
        assert!(!s.running());
    }
}
