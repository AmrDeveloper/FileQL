use gitql_core::signature::Function;
use gitql_core::signature::Signature;
use gitql_core::types::DataType;
use gitql_core::value::Value;
use gitql_std::function::standard_function_signatures;
use gitql_std::function::standard_functions;
use std::collections::HashMap;
use std::sync::OnceLock;

pub fn fileql_std_functions() -> &'static HashMap<&'static str, Function> {
    static HASHMAP: OnceLock<HashMap<&'static str, Function>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut map: HashMap<&'static str, Function> =
            HashMap::from(standard_functions().to_owned());
        map.insert("files_count", files_count);
        map
    })
}

pub fn fileql_std_signatures() -> &'static HashMap<&'static str, Signature> {
    static HASHMAP: OnceLock<HashMap<&'static str, Signature>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut map: HashMap<&'static str, Signature> =
            HashMap::from(standard_function_signatures().to_owned());
        map.insert(
            "files_count",
            Signature {
                parameters: vec![DataType::Text],
                return_type: DataType::Integer,
            },
        );
        map
    })
}

fn files_count(values: &[Value]) -> Value {
    let path = values[0].as_text();
    if let Ok(entries) = std::fs::read_dir(path) {
        let count = entries.flatten().count();
        return Value::Integer(count as i64);
    }
    Value::Integer(0)
}
