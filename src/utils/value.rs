use std::borrow::Cow;
use std::collections::{HashMap, hash_map::Entry};
use std::mem;

#[derive(Debug)]
pub enum Value<'buf> {
    Single(Cow<'buf, str>),
    Multiple(Vec<Cow<'buf, str>>),
}

#[derive(Debug)]
pub struct ValueMap<'buf> {
    data: HashMap<Cow<'buf, str>, Value<'buf>>,
}

impl<'buf> Value<'buf> {
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        let elements = match self {
            Value::Single(single) => std::slice::from_ref(single),
            Value::Multiple(multiple) => multiple
        };
        elements.iter().map(Cow::as_ref)
    }
}

impl<'buf> ValueMap<'buf>{
    pub fn new() -> Self {
        Self { 
            data: HashMap::new()
        }
    }

    pub fn put(&mut self, key: Cow<'buf, str>, val: Cow<'buf, str>) {
        match self.data.entry(key) {
            Entry::Occupied(mut entry) => {
                let existing = entry.get_mut();
                match existing {
                    Value::Single(prev) => {
                        *existing = Value::Multiple(vec![mem::take(prev), val]);
                    }
                    Value::Multiple(vec) => vec.push(val),
                };
            },
            Entry::Vacant(entry) => {
                entry.insert(Value::Single(val));
            }
        };
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.data.keys().map(Cow::as_ref)
    }

    pub fn values(&self, key: &str) -> Option<impl Iterator<Item = &str>> {
        self.data.get(key).map(Value::iter)
    }
}
