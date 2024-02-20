use gitql_ast::types::DataType;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref TABLES_FIELDS_TYPES: HashMap<&'static str, DataType> = {
        let mut map = HashMap::new();
        map.insert("path", DataType::Text);
        map.insert("parent", DataType::Text);
        map.insert("extension", DataType::Text);
        map.insert("is_dir", DataType::Boolean);
        map.insert("is_file", DataType::Boolean);
        map.insert("size", DataType::Integer);
        map
    };
}

lazy_static! {
    pub static ref TABLES_FIELDS_NAMES: HashMap<&'static str, Vec<&'static str>> = {
        let mut map = HashMap::new();
        map.insert(
            "files",
            vec!["path", "parent", "extension", "is_dir", "is_file", "size"],
        );
        map
    };
}
