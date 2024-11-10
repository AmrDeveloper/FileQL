use gitql_ast::types::integer::IntType;
use gitql_ast::types::text::TextType;
use gitql_core::signature::Function;
use gitql_core::signature::Signature;
use gitql_core::values::base::Value;
use gitql_core::values::integer::IntValue;
use gitql_std::function::standard_function_signatures;
use gitql_std::function::standard_functions;
use std::collections::HashMap;
use std::sync::OnceLock;

pub fn fileql_std_functions() -> &'static HashMap<&'static str, Function> {
    static HASHMAP: OnceLock<HashMap<&'static str, Function>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut map: HashMap<&'static str, Function> = standard_functions().to_owned();
        map.insert("files_count", files_count);
        map
    })
}

pub fn fileql_std_signatures() -> HashMap<&'static str, Signature> {
    let mut map: HashMap<&'static str, Signature> = standard_function_signatures().to_owned();
    map.insert(
        "files_count",
        Signature {
            parameters: vec![Box::new(TextType)],
            return_type: Box::new(IntType),
        },
    );
    map
}

fn files_count(values: &[Box<dyn Value>]) -> Box<dyn Value> {
    let path = values[0].as_text().unwrap();
    if let Ok(entries) = std::fs::read_dir(path) {
        let count = entries.flatten().count();
        return Box::new(IntValue {
            value: count as i64,
        });
    }
    Box::new(IntValue { value: 0 })
}
