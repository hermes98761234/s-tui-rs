#[cfg(target_os = "linux")]
pub mod fan;
pub mod freq;
#[cfg(target_os = "linux")]
pub mod power;
pub mod temp;
pub mod util;

#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub label: String,
    pub value: f64,
}

pub trait Source {
    /// Short pane name: "Temp", "Freq", "Util", "Power", "Fan".
    fn name(&self) -> &'static str;
    /// Display unit: "°C", "MHz", "%", "W", "RPM".
    fn unit(&self) -> &'static str;
    /// Current values; empty vec means nothing readable this tick (keep stale UI data).
    fn read(&mut self) -> Vec<Reading>;
    /// False when the sensor produced nothing at init; such sources are dropped.
    fn available(&self) -> bool;
}

/// All sources available on this machine, unavailable ones dropped.
pub fn all_sources(temp_threshold: f32) -> Vec<Box<dyn Source>> {
    let mut v: Vec<Box<dyn Source>> = vec![
        Box::new(util::UtilSource::new()),
        Box::new(freq::FreqSource::new()),
        Box::new(temp::TempSource::new(temp_threshold)),
    ];
    #[cfg(target_os = "linux")]
    {
        v.push(Box::new(power::PowerSource::new()));
        v.push(Box::new(fan::FanSource::new()));
    }
    v.retain(|s| s.available());
    v
}
