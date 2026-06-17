use uuid::Uuid;

pub trait IdGenerator: Send + Sync + 'static {
    fn generate_id(&self) -> String;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RealIdGenerator;

impl IdGenerator for RealIdGenerator {
    fn generate_id(&self) -> String {
        Uuid::now_v7().to_string()
    }
}

#[derive(Debug)]
pub struct TestIdGenerator {
    next_ids: std::sync::Mutex<Vec<String>>,
}

impl TestIdGenerator {
    pub fn new(ids: Vec<String>) -> Self {
        Self {
            next_ids: std::sync::Mutex::new(ids),
        }
    }
}

impl IdGenerator for TestIdGenerator {
    fn generate_id(&self) -> String {
        let mut guard = self.next_ids.lock().unwrap();
        if guard.is_empty() {
            Uuid::now_v7().to_string()
        } else {
            guard.remove(0)
        }
    }
}
