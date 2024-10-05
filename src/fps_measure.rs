use std::time::{Duration, Instant};

/// Update FPS measurements every given time in seconds
pub const FPS_MEASURE_UPDATE_SECONDS: f32 = 2.0;

/// Makes FPS measurements by accepting a timestamp once every frame.
pub struct FPSMeasureData {
    avg: f32,
    worst: f32,
    render_timestamps: Vec<Instant>,
    changed: bool,
}

impl std::fmt::Display for FPSMeasureData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Avg: {:.2}, Worst: {:.2}", self.avg, self.worst)
    }
}

impl FPSMeasureData {
    pub fn new() -> Self {
        Self {
            avg: -1.0,
            worst: -1.0,
            render_timestamps: Vec::new(),
            changed: false,
        }
    }
    /// Call this once per frame
    pub fn record_timestamp(&mut self) {
        self.render_timestamps.push(Instant::now());
        let sum = self
            .render_timestamps
            .last()
            .unwrap()
            .duration_since(self.render_timestamps.first().cloned().unwrap())
            .as_secs_f32();
        self.changed = false;
        if sum > FPS_MEASURE_UPDATE_SECONDS {
            self.changed = true;
            let mut durations: Vec<Duration> =
                Vec::with_capacity(self.render_timestamps.len() - 1);
            for t in self.render_timestamps.windows(2) {
                durations.push(t[1] - t[0]);
            }

            let avg = sum / durations.len() as f32;
            let worst =
                durations.iter().max().unwrap_or(&Duration::ZERO).as_secs_f32();
            self.avg = 1.0 / avg;
            self.worst = 1.0 / worst;
            self.render_timestamps.clear();
        }
    }

    pub fn is_changed(&self) -> bool {
        self.changed
    }
}
