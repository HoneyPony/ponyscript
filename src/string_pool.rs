use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ptr;

#[derive(Debug)]
pub struct StringPool {
    str_to_int: RefCell<HashMap<Vec<u8>, u64>>,
    int_to_str: RefCell<HashMap<u64, Vec<u8>>>,
    next_key: Cell<u64>
}

#[derive(Copy, Clone)]
#[derive(Debug)]
#[derive(Hash, Eq)]
pub struct PoolS {
    value: u64,
    pool: *const StringPool
}

impl Display for PoolS {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.to_utf8()))
    }
}

impl PoolS {
    pub fn to_utf8(&self) -> String {
        unsafe {
            (*self.pool).unpool_to_utf8(*self)
        }
    }

    #[allow(unused)]
    pub fn to_vec(&self) -> Vec<u8> {
        // TODO: Figure out if this unwrap is correct?
        // It should work for any instance of PoolS that is actually returned by a pool.
        // The only way to get an invalid one is to construct one manually...?
        unsafe {
            (*self.pool).unpool_copy(*self).unwrap()
        }
    }

    pub fn eq_utf8(&self, string: &'static str) -> bool {
        unsafe {
            (*self.pool).pool_tmp_str(string).value == self.value
        }
    }
}

impl<'a> PartialEq for PoolS {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && ptr::eq(self.pool, other.pool)
    }
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

    #[allow(unused)]
    pub fn pool_str(&self, str: &'static str) -> PoolS {
        self.pool_ref(&str.as_bytes().to_vec())
    }

    /// Gets a pooled string, IF the given string already exists in the pool.
    /// Otherwise, returns a pooled string with a value of '0', which cannot exist legitimately
    /// in the pool.
    ///
    /// Can be used to compare pooled strings without filling up the pool with unused values.
    pub fn pool_tmp_str(&self, str: &'static str) -> PoolS {
        self.pool_tmp(&str.as_bytes().to_vec())
    }

    pub fn pool_tmp(&self, str: &Vec<u8>) -> PoolS {
        let map = self.str_to_int.borrow();
        let val = map.get(str).map(|v| *v);

        match val {
            Some(v) => { PoolS { value: v, pool: self } },
            None => { PoolS { value: 0, pool: self } }
        }
    }

    pub fn pool(&self, str: Vec<u8>) -> PoolS {
        let map = self.str_to_int.borrow();
        let val = map.get(&str).map(|v| *v);
        drop(map);

        let val = val.unwrap_or_else(|| {
            let new_key = self.consume_key();

            // This method saves one copy...
            self.str_to_int.borrow_mut().insert(str.clone(), new_key);
            self.int_to_str.borrow_mut().insert(new_key, str);

            new_key
        });

        PoolS { value: val, pool: self }
    }

    pub fn pool_ref(&self, str: &Vec<u8>) -> PoolS {
        let map = self.str_to_int.borrow();
        let val = map.get(str).map(|v| *v);
        drop(map);

        let val = val.unwrap_or_else(|| {
            let new_key = self.consume_key();

            self.str_to_int.borrow_mut().insert(str.clone(), new_key);
            self.int_to_str.borrow_mut().insert(new_key, str.clone());

            new_key
        });

        PoolS { value: val, pool: self }
    }

    #[allow(unused)]
    pub fn unpool_copy(&self, str: PoolS) -> Option<Vec<u8>> {
        self.int_to_str.borrow().get(&str.value).map(|val| val.clone())
    }

    #[allow(unused)]
    pub fn unpool_to_utf8(&self, str: PoolS) -> String {
        self.int_to_str
            .borrow()
            .get(&str.value)
            .map(|val| {
                String::from_utf8(val.clone())
                    .unwrap_or(String::from("<bad utf8>"))
            })
            .unwrap_or(String::from("<not in pool>"))
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

    #[test]
    fn test_to_utf8_not_in_pool() {
        let pool = StringPool::new();

        let bad = PoolS { value: 10, pool: &pool };

        assert_eq!(pool.unpool_to_utf8(bad), String::from("<not in pool>"));
    }
}