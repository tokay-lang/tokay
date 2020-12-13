use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::hash::{Hash, Hasher};
use std::iter::Iterator;
use std::cmp::Eq;
use std::borrow::Borrow;
use std::fmt::Debug;


// Provide a MapKey trait to define required traits
pub trait MapKey: Clone + Hash + Eq + Debug {}
impl<K> MapKey for K where K: Clone + Hash + Eq + Debug {}

// MapItem is used to distinguish between list entries and hashmap entries
#[derive(Clone, Debug)]
enum MapItem<K: MapKey, V> {
    Key(K),
    Value(V)
}

// Map defines a dynamic, combined HashMap with insertation order using a Vec.
#[derive(Clone, Debug)]
pub struct Map<K: MapKey, V>
{
    map: HashMap<K, V>,
    vec: Vec<MapItem<K, V>>
}

impl<K: MapKey, V> Map<K, V> {
    pub fn new() -> Self {
        Self{
            map: HashMap::new(),
            vec: Vec::new()
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self{
            map: HashMap::new(),
            vec: Vec::with_capacity(capacity)
        }
    }

    pub fn push_key_value(&mut self, k: K, v: V) -> Option<V> {
        let ret = self.map.insert(k.clone(), v);
        self.vec.push(MapItem::Key(k));
        ret
    }

    pub fn push_value(&mut self, v: V) {
        self.vec.push(MapItem::Value(v));
    }

    pub fn insert_key_value(&mut self, index: usize, k: K, v: V) -> Option<V> {
        let ret = self.map.insert(k.clone(), v);
        self.vec.insert(index, MapItem::Key(k));
        ret
    }

    pub fn insert_value(&mut self, index: usize, v: V) {
        self.vec.insert(index, MapItem::Value(v))
    }

    pub fn pop(&mut self) -> Option<(Option<K>, V)> {
        if let Some(item) = self.vec.pop() {
            match item {
                MapItem::Key(key) => {
                    let value = self.map.remove(&key).unwrap();

                    Some((Some(key), value))
                },
                MapItem::Value(value) => {
                    Some((None, value))
                }
            }
        } else {
            None
        }
    }

    /*
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> std::vec::Drain<(Option<&K>, &V) {

    }
    */

    pub fn get(&self, index: usize) -> Option<(Option<&K>, &V)> {
        if let Some(item) = self.vec.get(index) {
            match item {
                MapItem::Key(key) => {
                    Some((Some(key), self.map.get(key).unwrap()))
                },
                MapItem::Value(value) => {
                    Some((None, &value))
                }
            }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<(Option<&K>, &mut V)> {
        if let Some(item) = self.vec.get_mut(index) {
            match item {
                MapItem::Key(key) => {
                    Some((Some(key), self.map.get_mut(key).unwrap()))
                },
                MapItem::Value(value) => {
                    Some((None, value))
                }
            }
        } else {
            None
        }
    }

    pub fn get_by_key<Q: ?Sized>(&self, k: &Q) -> Option<&V>
        where K: Borrow<Q>, Q: Hash + Eq
    {
        self.map.get(k)
    }

    pub fn get_by_key_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
        where K: Borrow<Q>, Q: Hash + Eq
    {
        self.map.get_mut(k)
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        self.map.keys()
    }

    pub fn iter(&self) -> KeyValueIter<K, V> {
        KeyValueIter{
            map: self,
            index: 0
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.vec.clear();
    }
}

impl<K: MapKey, V> std::ops::Index<usize> for Map<K, V> {
    type Output = V;

    fn index(&self, index: usize) -> &Self::Output {
        if let Some(res) = self.get(index) {
            res.1
        } else {
            panic!("index out of bounds: the len is {} but the index is {}", self.vec.len(), index)
        }
    }
}

impl<K: MapKey, V> std::ops::IndexMut<usize> for Map<K, V> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.vec.len() {
            panic!("index out of bounds: the len is {} but the index is {}", self.vec.len(), index)
        }

        self.get_mut(index).unwrap().1
    }
}

impl<K: MapKey, V> PartialEq for Map<K, V> {
    fn eq(&self, _other: &Self) -> bool {
        panic!("partial not implemented for map");
    }
}

impl<K: MapKey, V> Eq for  Map<K, V> {}

impl<K: MapKey, V> Hash for  Map<K, V> {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        //fixme: use map pointer here?
        panic!("hash for map not supported");
    }
}

#[macro_export]
macro_rules! map_key_value {
    [$($key:expr => $value:expr),*] => {
        {
            let mut map = Map::new();
            $( map.push_key_value($key, $value); )*
            map
        }
    }
}

#[macro_export]
macro_rules! map_value {
    [$($value:expr),*] => {
        {
            let mut map = Map::new();
            $( map.push_value($value); )*
            map
        }
    }
}


pub struct KeyValueIter<'a, K: MapKey, V> {
    map: &'a Map<K, V>,
    index: usize
}

impl<'a, K: MapKey, V> Iterator for KeyValueIter<'a, K, V> {
    type Item = (Option<&'a K>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.map.vec.len() {
            //let ret = self.map.map.get_key_value(&self.map.vec[self.index]).unwrap();
            self.index += 1;

            match &self.map.vec[self.index - 1] {
                MapItem::Key(key) => Some((Some(key), self.map.map.get(key).unwrap())),
                MapItem::Value(value) => Some((None, value)),
            }
        } else {
            None
        }
    }
}

#[test]
fn test() {
    let mut map: Map<String, String> = Map::new();
    map.push_key_value("a".to_string(), "Hello".to_string());
    map.push_value("yeah".to_string());
    map.push_key_value("b".to_string(), "Rust".to_string());

    assert_eq!(map.get(2),
        Some((
            Some(&"b".to_string()),
            &"Rust".to_string()
        ))
    );
    assert_eq!(map.get(3), None);

    if let Some((_, s)) = map.get_mut(2) {
        s.push_str("- and C-");
    }

    map.push_key_value("c".to_string(), "lings!".to_string());
    map.push_value("this looks fine!".to_string());

    for (k, v) in map.iter() {
        println!("{:?} = {:?}", k, v);
    }

    assert_eq!(map.pop(), Some((None, "this looks fine!".to_string())));
    assert_eq!(map.pop(), Some((Some("c".to_string()), "lings!".to_string())));
    assert_eq!(map.pop(), Some((Some("b".to_string()), "Rust- and C-".to_string())));
    assert_eq!(map.pop(), Some((None, "yeah".to_string())));
    assert_eq!(map.pop(), Some((Some("a".to_string()), "Hello".to_string())));
    assert_eq!(map.pop(), None);
}
