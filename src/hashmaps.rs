use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug)]
pub struct HashMap<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K: Eq + Hash, V: Clone> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(index) = self.keys.iter().position(|k| k == &key) {
            let old_value = self.values[index].clone();
            self.values[index] = value;
            Some(old_value)
        } else {
            self.keys.push(key);
            self.values.push(value);
            None
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(index) = self.keys.iter().position(|k| k == key) {
            self.values.get(index)
        } else {
            None
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(index) = self.keys.iter().position(|k| k == key) {
            let value = self.values.remove(index);
            self.keys.remove(index);
            Some(value)
        } else {
            None
        }
    }
}

trait Hash {
    fn hash(&self) -> usize;
}

impl Hash for String {
    fn hash(&self) -> usize {
        self.len()
    }
}

impl<T> Hash for Box<T>
where
    T: Hash,
{
    fn hash(&self) -> usize {
        (**self).hash()
    }
}
