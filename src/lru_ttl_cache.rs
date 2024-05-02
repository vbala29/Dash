use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

type CacheEntryLinkOpt<K, V> = Option<Arc<Mutex<CacheEntry<K, V>>>>;
type CacheEntryLink<K, V> = Arc<Mutex<CacheEntry<K, V>>>;

pub struct CacheEntry<K, V> {
    key: K,
    value: V,
    pub ttl: SystemTime,
    prev: CacheEntryLinkOpt<K, V>,
    next: CacheEntryLinkOpt<K, V>,
}

pub struct Cache<K: Hash + Eq + Sized, V: Clone> {
    pub cache_map: Arc<Mutex<HashMap<K, CacheEntryLink<K, V>>>>,
    capacity: usize,
    list_head: CacheEntryLinkOpt<K, V>,
    list_tail: CacheEntryLinkOpt<K, V>,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Sized + Clone + Send + 'static,
    V: Clone + Send + 'static,
{
    pub fn new(capacity: usize) -> Self {
        let cache_map = Arc::new(Mutex::new(HashMap::<K, CacheEntryLink<K, V>>::new()));

        Cache {
            cache_map,
            capacity,
            list_head: None,
            list_tail: None,
        }
    }

    pub fn get(&self, k: &K) -> Option<V> {
        self.cache_map
            .lock()
            .unwrap()
            .get(k)
            .map(|entry_link| entry_link.lock().unwrap().value.clone())
    }

    fn evict(&mut self, link: CacheEntryLink<K, V>) {
        let entry = link.lock().unwrap();
        let prev = &entry.prev;
        let next = &entry.next;

        match (prev, next) {
            (Some(_), Some(_)) => panic!("Called evict on a non tail cache entry"),
            (None, Some(_)) => panic!("Called evict on a non tail cache entry"),
            (Some(p), None) => {
                p.lock().unwrap().next = None;
                self.list_tail = prev.clone();

                self.cache_map
                    .lock()
                    .unwrap()
                    .remove(&link.lock().unwrap().key);
            }
            (None, None) => {
                self.list_head = None;
                self.list_tail = None;
                self.cache_map
                    .lock()
                    .unwrap()
                    .remove(&link.lock().unwrap().key);
            }
        }
    }

    /// Returns true if an item was removed with key k, else returns false
    pub fn add(&mut self, k: K, v: V, ttl: SystemTime) -> bool {
        println!("RUNNNING");
        if self.cache_map.lock().unwrap().contains_key(&k) {
            return false;
        }

        while self.cache_map.lock().unwrap().len() > self.capacity {
            self.evict(self.list_tail.clone().unwrap().clone());
        }

        let new_entry = Arc::new(Mutex::new(CacheEntry {
            key: k.clone(),
            value: v,
            prev: None,
            next: None,
            ttl,
        }));
        self.cache_map.lock().unwrap().insert(k, new_entry.clone());

        match &self.list_head {
            Some(h) => {
                h.lock().unwrap().prev = Some(new_entry.clone());
                self.list_tail = Some(new_entry.clone());
            }
            None => (),
        }

        if self.list_tail.is_none() {
            self.list_tail = Some(new_entry);
        }

        true
    }

    pub fn start_ttl_daemon(cache: Arc<Mutex<Cache<K, V>>>, cache_capacity: usize) {
        // Cloudflare DNS only Entrerprise TTL standard is 30 seconds
        // https://developers.cloudflare.com/dns/manage-dns-records/reference/ttl/
        let duration = Arc::new(Mutex::new(Duration::from_secs(30)));
        std::thread::spawn(move || {
            let mut index = 0;
            loop {
                let (start, end) = (
                    (cache_capacity / 10) * index,
                    (cache_capacity / 10) * (index + 1),
                );
                let mut keys_to_remove = Vec::new();
                let cache_unlocked = cache.lock().unwrap();
                let map = cache_unlocked.cache_map.lock().unwrap();
                for (k, v) in map.iter().skip(start).take(end - start) {
                    let ttl: SystemTime = v.lock().unwrap().ttl;
                    if ttl > SystemTime::now() {
                        keys_to_remove.push(k);
                    }
                }

                let mut map = cache_unlocked.cache_map.lock().unwrap();
                for k in keys_to_remove {
                    map.remove(k);
                }

                index = (index + 1) % 10;

                std::thread::sleep(*duration.lock().unwrap());
            }
        });
    }
}
