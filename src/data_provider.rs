use std::path::Path;

use gitql_core::object::Row;
use gitql_core::values::boolean::BoolValue;
use gitql_core::values::integer::IntValue;
use gitql_core::values::null::NullValue;
use gitql_core::values::text::TextValue;
use gitql_core::values::Value;
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
        match table {
            "files" => select_files(&self.paths, &self.excludes, selected_columns),
            _ => Ok(vec![Row { values: vec![] }]),
        }
    }
}

fn select_files(
    paths: &[String],
    excludes: &[String],
    selected_columns: &[String],
) -> Result<Vec<Row>, String> {
    let files = collect_paths_nested_files(paths, excludes);
    let mut rows: Vec<Row> = Vec::with_capacity(paths.len());
    for file in files.iter() {
        let mut values: Vec<Box<dyn Value>> = Vec::with_capacity(selected_columns.len());
        let path = Path::new(&file);

        for column_name in selected_columns {
            if column_name == "path" {
                let file_path_string = path.to_str().unwrap_or("");
                let value = file_path_string.to_string();
                values.push(Box::new(TextValue::new(value)));
                continue;
            }

            if column_name == "parent" {
                let parent_path = if let Some(parent) = path.parent() {
                    parent.to_str().unwrap_or("")
                } else {
                    ""
                };
                let value = parent_path.to_string();
                values.push(Box::new(TextValue::new(value)));
                continue;
            }

            if column_name == "extension" {
                let value = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_string();
                values.push(Box::new(TextValue::new(value)));
                continue;
            }

            if column_name == "is_dir" {
                let value = path.is_dir();
                values.push(Box::new(BoolValue::new(value)));
                continue;
            }

            if column_name == "is_file" {
                let value = path.is_file();
                values.push(Box::new(BoolValue::new(value)));
                continue;
            }

            if column_name == "size" {
                let file_size = if let Ok(meta_data) = std::fs::metadata(file) {
                    meta_data.len() as i64
                } else {
                    0
                };
                values.push(Box::new(IntValue { value: file_size }));
                continue;
            }

            values.push(Box::new(NullValue));
        }

        rows.push(Row { values });
    }
    Ok(rows)
}

fn collect_paths_nested_files(paths: &[String], excludes: &[String]) -> Vec<String> {
    let mut files: Vec<String> = vec![];
    for path in paths {
        let files_tree = collect_path_nested_files(path, excludes);
        for file in files_tree.iter() {
            if files.contains(file) {
                continue;
            }
            files.push(file.to_string());
        }
    }
    files
}

fn collect_path_nested_files(dir_path: &str, excludes: &[String]) -> Vec<String> {
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
