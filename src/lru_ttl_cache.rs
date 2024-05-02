use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, SystemTime};


type CacheEntryLinkOpt<K, V> = Option<Rc<RefCell<CacheEntry<K, V>>>>;
type CacheEntryLink<K, V> = Rc<RefCell<CacheEntry<K, V>>>;


struct CacheEntry<K, V> {
   key : K,
   value : V,
   ttl :  SystemTime,
   prev : CacheEntryLinkOpt<K, V>,
   next : CacheEntryLinkOpt<K, V>
}

pub struct Cache<K : Hash + Eq + Sized, V : Clone> {
    average_ttl : Duration,
    cache_map : HashMap<K, CacheEntryLink<K, V>>,
    capacity : usize,
    list_head : CacheEntryLinkOpt<K, V>,
    list_tail : CacheEntryLinkOpt<K, V>
}


impl<K, V> Cache<K, V>
where K : Hash + Eq + Sized + Clone, V : Clone
{
    pub fn new(capacity: usize) -> Self {
        // Cloudflare DNS only Entrerprise TTL standard is 30 seconds
        // https://developers.cloudflare.com/dns/manage-dns-records/reference/ttl/
        Cache { cache_map : HashMap::new(), capacity, list_head : None, list_tail : None, average_ttl : Duration::from_secs(30)}
    }

    pub fn get(&self, k: K) -> Option<V> {
        self.cache_map.get(&k).map(|entry_link| entry_link.borrow().value.clone())
    }

    fn evict(&mut self, link: CacheEntryLink<K, V>) {
        let entry = link.borrow();
        let prev = &entry.prev;
        let next = &entry.next;

        match (prev, next) {
            (Some(_), Some(_)) => panic!("Called evict on a non tail cache entry"),
            (None, Some(_)) => panic!("Called evict on a non tail cache entry"),
            (Some(p), None) => {
                p.borrow_mut().next = None;
                self.list_tail = prev.clone();

                self.cache_map.remove(&link.borrow().key);
            }
            (None, None) => {
                self.list_head = None;
                self.list_tail = None;
                self.cache_map.remove(&link.borrow().key);
            },
        }
    }

    pub fn add(&mut self, k : K, v : V, ttl : SystemTime) -> bool {
        if self.cache_map.contains_key(&k) {
            return false
        }

        let new_entry = Rc::new(RefCell::new(CacheEntry { key : k.clone(), value : v, prev : None, next : None, ttl}));
        self.cache_map.insert(k, new_entry.clone());

        match &self.list_head {
            Some(h) => {
                h.borrow_mut().prev = Some(new_entry.clone());
                self.list_tail = Some(new_entry.clone());
            }
            None => ()
        }

       if self.list_tail.is_none() {
            self.list_tail = Some(new_entry);
       }

        true
    }
}

