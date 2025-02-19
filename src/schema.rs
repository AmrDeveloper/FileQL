use std::collections::HashMap;
use std::sync::OnceLock;

use gitql_ast::types::base::DataType;
use gitql_ast::types::boolean::BoolType;
use gitql_ast::types::integer::IntType;
use gitql_ast::types::text::TextType;
use gitql_core::environment::Environment;
use gitql_core::schema::Schema;
use gitql_std::aggregation::aggregation_function_signatures;
use gitql_std::aggregation::aggregation_functions;
use gitql_std::window::window_function_signatures;
use gitql_std::window::window_functions;

use crate::functions;

fn tables_fields_types() -> HashMap<&'static str, Box<dyn DataType>> {
    let mut map: HashMap<&'static str, Box<dyn DataType>> = HashMap::new();
    map.insert("path", Box::new(TextType));
    map.insert("parent", Box::new(TextType));
    map.insert("extension", Box::new(TextType));
    map.insert("is_dir", Box::new(BoolType));
    map.insert("is_file", Box::new(BoolType));
    map.insert("size", Box::new(IntType));
    map
}

fn tables_fields_names() -> &'static HashMap<&'static str, Vec<&'static str>> {
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

pub fn create_fileql_environment() -> Environment {
    let schema = Schema {
        tables_fields_names: tables_fields_names().to_owned(),
        tables_fields_types: tables_fields_types().to_owned(),
    };

    let std_signatures = functions::fileql_std_signatures();
    let std_functions = functions::fileql_std_functions();

    let aggregation_signatures = aggregation_function_signatures();
    let aggregation_functions = aggregation_functions();

    let window_signatures = window_function_signatures();
    let window_functions = window_functions();

    let mut env = Environment::new(schema);
    env.with_standard_functions(&std_signatures, std_functions);
    env.with_aggregation_functions(&aggregation_signatures, aggregation_functions);
    env.with_window_functions(&window_signatures, window_functions);

    env
}
