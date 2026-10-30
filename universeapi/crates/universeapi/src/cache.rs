//! Two-level TTL cache: in-memory map in front of optional JSON files on disk.
//! Disk entries survive restarts so home rails render instantly on launch.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Standard TTLs.
pub mod ttl {
    use std::time::Duration;
    pub const SEARCH: Duration = Duration::from_secs(10 * 60);
    pub const RAIL: Duration = Duration::from_secs(30 * 60);
    pub const DETAIL: Duration = Duration::from_secs(6 * 60 * 60);
    pub const MIRROR: Duration = Duration::from_secs(24 * 60 * 60);
    pub const SOURCES: Duration = Duration::from_secs(20 * 60);
}

#[derive(Serialize, Deserialize)]
struct Entry {
    expires_at: i64,
    value: serde_json::Value,
}

#[derive(Debug)]
pub struct Cache {
    mem: Mutex<HashMap<String, (i64, serde_json::Value)>>,
    dir: Option<PathBuf>,
    max_entries: usize,
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

impl Cache {
    pub fn memory() -> Self {
        Self { mem: Mutex::new(HashMap::new()), dir: None, max_entries: 2_000 }
    }

    pub fn with_dir(dir: impl Into<PathBuf>) -> Self {
        Self { dir: Some(dir.into()), ..Self::memory() }
    }

    fn file_for(&self, key: &str) -> Option<PathBuf> {
        let mut h = DefaultHasher::new();
        key.hash(&mut h);
        self.dir.as_ref().map(|d| d.join(format!("{:016x}.json", h.finish())))
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let t = now();
        // Scope the guard so it is never held across an await point.
        let hit = { self.mem.lock().ok().and_then(|m| m.get(key).cloned()) };
        if let Some((exp, v)) = hit {
            if exp > t {
                return serde_json::from_value(v).ok();
            }
        }
        let path = self.file_for(key)?;
        let bytes = tokio::fs::read(&path).await.ok()?;
        let entry: Entry = serde_json::from_slice(&bytes).ok()?;
        if entry.expires_at <= t {
            let _ = tokio::fs::remove_file(&path).await;
            return None;
        }
        let out = serde_json::from_value(entry.value.clone()).ok();
        if let Ok(mut mem) = self.mem.lock() {
            mem.insert(key.to_string(), (entry.expires_at, entry.value));
        }
        out
    }

    pub async fn put<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) {
        let Ok(v) = serde_json::to_value(value) else { return };
        let expires_at = now() + ttl.as_secs() as i64;
        if let Ok(mut mem) = self.mem.lock() {
            if mem.len() >= self.max_entries {
                let t = now();
                mem.retain(|_, (exp, _)| *exp > t);
                if mem.len() >= self.max_entries {
                    let drop: Vec<String> = mem.keys().take(self.max_entries / 2).cloned().collect();
                    for k in drop {
                        mem.remove(&k);
                    }
                }
            }
            mem.insert(key.to_string(), (expires_at, v.clone()));
        }
        if let Some(path) = self.file_for(key) {
            if let Some(parent) = path.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            if let Ok(bytes) = serde_json::to_vec(&Entry { expires_at, value: v }) {
                // write-then-rename so a crash never leaves a torn file
                let tmp = path.with_extension("tmp");
                if tokio::fs::write(&tmp, bytes).await.is_ok() {
                    let _ = tokio::fs::rename(&tmp, &path).await;
                }
            }
        }
    }

    pub async fn invalidate(&self, key: &str) {
        if let Ok(mut mem) = self.mem.lock() {
            mem.remove(key);
        }
        if let Some(path) = self.file_for(key) {
            let _ = tokio::fs::remove_file(path).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_roundtrip_and_expiry() {
        let c = Cache::memory();
        c.put("k", &vec![1, 2, 3], Duration::from_secs(60)).await;
        assert_eq!(c.get::<Vec<i32>>("k").await, Some(vec![1, 2, 3]));
        c.put("old", &1, Duration::from_secs(0)).await;
        assert_eq!(c.get::<i32>("old").await, None);
    }

    #[tokio::test]
    async fn disk_survives_new_instance() {
        let dir = std::env::temp_dir().join(format!("universeapi-cache-test-{}", fastrand::u64(..)));
        Cache::with_dir(&dir).put("rail", &"hello", Duration::from_secs(60)).await;
        assert_eq!(Cache::with_dir(&dir).get::<String>("rail").await.as_deref(), Some("hello"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
