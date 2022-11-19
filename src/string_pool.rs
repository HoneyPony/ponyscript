use std::borrow::BorrowMut;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

pub struct StringPool {
    str_to_int: RefCell<HashMap<Vec<u8>, u64>>,
    int_to_str: RefCell<HashMap<u64, Vec<u8>>>,
    next_key: Cell<u64>
}

#[derive(Copy, Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct PoolS {
    value: u64
}

impl StringPool {
    fn consume_key(&self) -> u64 {
        let result = self.next_key.get();
        self.next_key.set(result + 1);
        result
    }

    pub fn new() -> StringPool {
        StringPool {
            str_to_int: Default::default(),
            int_to_str: Default::default(),
            next_key: Cell::new(1)
        }
    }

    pub fn pool_str(&self, str: &'static str) -> PoolS {
        self.pool(&str.as_bytes().to_vec())
    }

    pub fn pool(&self, str: &Vec<u8>) -> PoolS {
        let map = self.str_to_int.borrow();
        let val = map.get(str).map(|v| *v);
        drop(map);

        let val = val.unwrap_or_else(|| {
            let new_key = self.consume_key();

            self.str_to_int.borrow_mut().insert(str.clone(), new_key);
            self.int_to_str.borrow_mut().insert(new_key, str.clone());

            new_key
        });

        PoolS { value: val }
    }

    pub fn unpool_copy(&self, str: PoolS) -> Option<Vec<u8>> {
        self.int_to_str.borrow().get(&str.value).map(|val| val.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_eq() {
        let pool = StringPool::new();

        let first = pool.pool_str("abc");
        let second = pool.pool_str("abc");

        assert_eq!(first, second);
    }

    #[test]
    fn test_pool_ne() {
        let pool = StringPool::new();

        let first = pool.pool_str("abc");
        let second = pool.pool_str("abd");

        assert_ne!(first, second);
    }

    #[test]
    fn test_pool_unpool() {
        let pool = StringPool::new();

        let ps = pool.pool_str("abc");
        let str = pool.unpool_copy(ps).unwrap();

        assert_eq!(str, vec![b'a', b'b', b'c']);
    }
}