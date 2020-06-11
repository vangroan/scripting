use std::time::Duration;

pub struct DeltaTime(Duration);

impl DeltaTime {
    pub fn new(duration: Duration) -> Self {
        DeltaTime(duration)
    }

    pub fn duration(&self) -> Duration {
        self.0
    }

    pub fn as_secs(&self) -> f32 {
        self.0.as_millis() as f32 / 1000.0
    }
}
