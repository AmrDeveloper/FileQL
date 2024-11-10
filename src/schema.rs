use std::collections::HashMap;
use std::sync::OnceLock;

use gitql_ast::types::base::DataType;
use gitql_ast::types::boolean::BoolType;
use gitql_ast::types::integer::IntType;
use gitql_ast::types::text::TextType;

pub fn tables_fields_types() -> HashMap<&'static str, Box<dyn DataType>> {
    let mut map: HashMap<&'static str, Box<dyn DataType>> = HashMap::new();
    map.insert("path", Box::new(TextType));
    map.insert("parent", Box::new(TextType));
    map.insert("extension", Box::new(TextType));
    map.insert("is_dir", Box::new(BoolType));
    map.insert("is_file", Box::new(BoolType));
    map.insert("size", Box::new(IntType));
    map
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
