use gitql_core::types::DataType;
use std::collections::HashMap;
use std::sync::OnceLock;

pub fn tables_fields_types() -> &'static HashMap<&'static str, DataType> {
    static HASHMAP: OnceLock<HashMap<&'static str, DataType>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert("path", DataType::Text);
        map.insert("parent", DataType::Text);
        map.insert("extension", DataType::Text);
        map.insert("is_dir", DataType::Boolean);
        map.insert("is_file", DataType::Boolean);
        map.insert("size", DataType::Integer);
        map
    })
}

pub fn tables_fields_names() -> &'static HashMap<&'static str, Vec<&'static str>> {
    static HASHMAP: OnceLock<HashMap<&'static str, Vec<&'static str>>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert(
            "files",
            vec!["path", "parent", "extension", "is_dir", "is_file", "size"],
        );
        map
    })
}
