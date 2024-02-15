use std::path::Path;

use gitql_ast::environment::Environment;
use gitql_ast::expression::Expression;
use gitql_ast::expression::SymbolExpression;
use gitql_ast::object::GitQLObject;
use gitql_ast::object::Group;
use gitql_ast::object::Row;
use gitql_ast::value::Value;
use gitql_engine::data_provider::DataProvider;
use gitql_engine::engine_evaluator::evaluate_expression;

pub struct FileDataProvider {
    pub paths: Vec<String>,
    pub excludes: Vec<String>,
}

impl FileDataProvider {
    pub fn new(paths: Vec<String>, excludes: Vec<String>) -> Self {
        Self { paths, excludes }
    }
}

impl DataProvider for FileDataProvider {
    fn provide(
        &self,
        env: &mut Environment,
        _table: &str,
        fields_names: &[String],
        titles: &[String],
        fields_values: &[Box<dyn Expression>],
    ) -> GitQLObject {
        let mut files: Vec<String> = vec![];
        for path in self.paths.iter() {
            let files_tree = traverse_file_tree(path, &self.excludes);
            for f in files_tree.iter() {
                if files.contains(f) {
                    continue;
                }
                files.push(f.to_string());
            }
        }

        let mut groups: Vec<Group> = vec![];
        let mut rows: Vec<Row> = vec![];

        let names_len = fields_names.len() as i64;
        let values_len = fields_values.len() as i64;
        let padding = names_len - values_len;

        for file in files {
            let mut values: Vec<Value> = vec![];

            for index in 0..names_len {
                let field_name = &fields_names[index as usize];

                if (index - padding) >= 0 {
                    let value = &fields_values[(index - padding) as usize];
                    if value.as_any().downcast_ref::<SymbolExpression>().is_none() {
                        let evaluated = evaluate_expression(env, value, titles, &values);
                        values.push(evaluated.unwrap_or(Value::Null));
                        continue;
                    }
                }

                if field_name == "path" {
                    let path = Path::new(&file);
                    let file_path_string = path.to_str().unwrap_or("");
                    values.push(Value::Text(file_path_string.to_string()));
                    continue;
                }

                if field_name == "parent" {
                    let path = Path::new(&file);
                    let parent_path = if let Some(parent) = path.parent() {
                        parent.to_str().unwrap_or("")
                    } else {
                        ""
                    };
                    values.push(Value::Text(parent_path.to_string()));
                    continue;
                }

                if field_name == "extension" {
                    let path = Path::new(&file);
                    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                    values.push(Value::Text(extension.to_string()));
                    continue;
                }

                if field_name == "size" {
                    let file_size = if let Ok(meta_data) = std::fs::metadata(&file) {
                        meta_data.len() as i64
                    } else {
                        0
                    };
                    values.push(Value::Integer(file_size));
                    continue;
                }
            }

            rows.push(Row { values });
        }

        groups.push(Group { rows });
        GitQLObject {
            titles: titles.to_vec(),
            groups,
        }
    }
}

fn traverse_file_tree(dir_path: &str, excludes: &[String]) -> Vec<String> {
    let mut file_paths = Vec::new();
    let mut stack: Vec<String> = vec![];

    if !excludes.contains(&dir_path.to_string()) {
        stack.push(dir_path.to_owned());
    }

    while let Some(path) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let file_type = entry.file_type().unwrap();
                let subpath = entry.path();

                if let Some(path) = entry.file_name().to_str() {
                    if excludes.contains(&path.to_string()) {
                        continue;
                    }

                    if file_type.is_dir() {
                        stack.push(subpath.to_str().unwrap_or("").to_string());
                    } else if let Some(file_path) = subpath.to_str() {
                        file_paths.push(file_path.to_string());
                    }
                }
            }
        }
    }

    file_paths
}
