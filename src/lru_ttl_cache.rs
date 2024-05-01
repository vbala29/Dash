use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;


type CacheEntryLinkOpt<K, V> = Option<Rc<RefCell<CacheEntry<K, V>>>>;
type CacheEntryLink<K, V> = Rc<RefCell<CacheEntry<K, V>>>;


struct CacheEntry<K, V> {
   key : K,
   value : V,
   prev : CacheEntryLinkOpt<K, V>,
   next : CacheEntryLinkOpt<K, V>
}

pub struct Cache<K, V> {
    cache_map : HashMap<K, V>,
    capacity : usize,
    list_head : CacheEntryLinkOpt<K, V>,
    list_tail : CacheEntryLinkOpt<K, V>
}


impl<K, V> Cache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Cache { cache_map : HashMap::new(), capacity, list_head : None, list_tail : None }
    }

    pub fn get(&self, k: K) -> Option<V> {
        self.get(k)
    }

    fn evict(link: CacheEntryLink<K, V>) {
        let entry = link.borrow();
        let prev = &entry.prev;
        let next = &entry.next;

        match (prev, next) {
            (Some(p), Some(n)) => panic!("Called evict on a non tail cache entry"),
            (None, Some(n)) => panic!("Called evict on a non tail cache entry"),
            (Some(p), None) => () ,
            (None, None) => (),
        }
    }
}

