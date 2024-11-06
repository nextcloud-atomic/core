use std::collections::HashMap;

pub trait KeyValueProvider {
    fn to_map(&self) -> HashMap<String, String>;
}
