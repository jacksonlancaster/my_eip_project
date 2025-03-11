use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct AssemblyObject {
    pub connection_id: u32,
    pub data: Vec<u8>,
    pub rpi_ms: u32, // Requested Packet Interval in milliseconds
    pub next_send_time: Instant,
    pub last_activity: Instant, // Track last message timestamp
}

#[derive(Clone)]
pub struct AssemblyManager {
    assemblies: Arc<Mutex<HashMap<u32, AssemblyObject>>>,
}

impl AssemblyManager {
    pub fn new() -> Self {
        Self {
            assemblies: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register(&self, connection_id: u32, rpi_ms: u32) {
        let mut assemblies = self.assemblies.lock().await;
        assemblies.insert(
            connection_id,
            AssemblyObject {
                connection_id,
                data: vec![0; 4], // Example: 4 bytes of zeroed data
                rpi_ms,
                next_send_time: Instant::now() + Duration::from_millis(rpi_ms as u64),
                last_activity: Instant::now(),
            },
        );
    }

    pub async fn update_activity(&self, connection_id: u32) {
        let mut assemblies = self.assemblies.lock().await;
        if let Some(assembly) = assemblies.get_mut(&connection_id) {
            assembly.last_activity = Instant::now();
        }
    }

    pub async fn get_ready_transmissions(&self) -> Vec<AssemblyObject> {
        let now = Instant::now();
        let mut ready_objects = Vec::new();
        let mut assemblies = self.assemblies.lock().await;

        for assembly in assemblies.values_mut() {
            if now >= assembly.next_send_time {
                ready_objects.push(assembly.clone());
                assembly.next_send_time = now + Duration::from_millis(assembly.rpi_ms as u64);
            }
        }

        ready_objects
    }

    pub async fn cleanup_timed_out_connections(&self, timeout_ms: u64) -> Vec<u32> {
        let now = Instant::now();
        let mut removed_connections = Vec::new();
        let mut assemblies = self.assemblies.lock().await;

        assemblies.retain(|&conn_id, assembly| {
            let alive = now.duration_since(assembly.last_activity).as_millis() < timeout_ms as u128;
            if !alive {
                removed_connections.push(conn_id);
            }
            alive
        });

        removed_connections
    }
}
