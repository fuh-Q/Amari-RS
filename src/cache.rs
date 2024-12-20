use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::defs::FetchType;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    data: Arc<dyn std::any::Any + Send + Sync>,
    timestamp: u64,
    size: usize,
}

impl CacheEntry {
    pub fn new(data: Arc<dyn std::any::Any + Send + Sync>, size: usize) -> Self {
        Self {
            data,
            timestamp: Instant::now().elapsed().as_secs(),
            size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cache {
    ttl: u32,
    max_bytes: usize,
    cache: HashMap<FetchType, CacheEntry>, // TODO: Fix this type.
    total_size: usize,
}

impl Cache {
    #[must_use]
    pub fn new(ttl: u32, max_bytes: usize) -> Self {
        Self {
            ttl,
            max_bytes,
            cache: HashMap::new(),
            total_size: 0,
        }
    }

    pub fn get(&mut self, key: &FetchType) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        let entry = self.cache.get(key);
        match entry {
            Some(ent) => {
                if Instant::now().elapsed().as_secs() - ent.timestamp < u64::from(self.ttl) {
                    Some(ent.data.clone())
                } else {
                    self.remove_entry(key);
                    None
                }
            }
            None => None,
        }
    }

    pub fn set(&mut self, key: &FetchType, data: Arc<dyn std::any::Any + Send + Sync>) {
        // Dirty downcasting to unsigned char* type and get length.
        let data_size = data.downcast_ref::<Vec<u8>>().map_or(0, std::vec::Vec::len);

        if self.total_size + data_size > self.max_bytes {
            self.enforce_size_limit();
        }

        let entry = CacheEntry::new(data, data_size);
        self.cache.insert(key.clone(), entry);

        self.total_size += data_size;
    }

    fn remove_entry(&mut self, key: &FetchType) {
        let ent = self.cache.remove(key);
        if let Some(ent) = ent {
            self.total_size -= ent.size;
        }
    }

    fn enforce_size_limit(&mut self) {
        while self.total_size > self.max_bytes {
            let cache = self.cache.clone();
            if let Some((key, ent)) = cache.iter().next() {
                self.total_size -= ent.size;
                self.cache.remove(key);
            }
        }
    }
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}
