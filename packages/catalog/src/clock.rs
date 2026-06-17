use chrono::{DateTime, Utc};

pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Debug)]
pub struct TestClock {
    now: std::sync::Mutex<DateTime<Utc>>,
}

impl TestClock {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            now: std::sync::Mutex::new(now),
        }
    }

    pub fn set_now(&self, new_now: DateTime<Utc>) {
        let mut guard = self.now.lock().unwrap();
        *guard = new_now;
    }
}

impl Clock for TestClock {
    fn now(&self) -> DateTime<Utc> {
        let guard = self.now.lock().unwrap();
        *guard
    }
}
