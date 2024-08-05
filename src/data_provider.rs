use std::path::Path;

use gitql_core::object::Row;
use gitql_core::value::Value;
use gitql_engine::data_provider::DataProvider;

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
    fn provide(&self, table: &str, selected_columns: &[String]) -> Result<Vec<Row>, String> {
        // If table is empty, thats mean it's a set of expressions query
        if table.is_empty() {
            return Ok(vec![Row { values: vec![] }]);
        }

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

        let names_len = selected_columns.len() as i64;
        let mut rows: Vec<Row> = Vec::with_capacity(files.len());

        for file in files {
            let mut values: Vec<Value> = Vec::with_capacity(names_len as usize);
            let path = Path::new(&file);

            for index in 0..names_len {
                let field_name = &selected_columns[index as usize];
                if field_name == "path" {
                    let file_path_string = path.to_str().unwrap_or("");
                    values.push(Value::Text(file_path_string.to_string()));
                    continue;
                }

                if field_name == "parent" {
                    let parent_path = if let Some(parent) = path.parent() {
                        parent.to_str().unwrap_or("")
                    } else {
                        ""
                    };
                    values.push(Value::Text(parent_path.to_string()));
                    continue;
                }

                if field_name == "extension" {
                    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                    values.push(Value::Text(extension.to_string()));
                    continue;
                }

                if field_name == "is_dir" {
                    values.push(Value::Boolean(path.is_dir()));
                    continue;
                }

                if field_name == "is_file" {
                    values.push(Value::Boolean(path.is_file()));
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

                values.push(Value::Null);
            }

            rows.push(Row { values });
        }

        Ok(rows)
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
                        let path = subpath.to_str().unwrap_or("").to_string();
                        stack.push(path.to_string());
                        file_paths.push(path.to_string());
                    } else if let Some(file_path) = subpath.to_str() {
                        file_paths.push(file_path.to_string());
                    }
                }
            }
        }
    }

    file_paths
}
