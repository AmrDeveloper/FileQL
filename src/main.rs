use std::io;
use std::io::IsTerminal;
use std::path::Path;

use crate::engine::EvaluationResult::SelectedGroups;
use arguments::Arguments;
use arguments::Command;
use data_provider::FileDataProvider;
use gitql_cli::arguments::OutputFormat;
use gitql_cli::diagnostic_reporter;
use gitql_cli::diagnostic_reporter::DiagnosticReporter;
use gitql_cli::printer::base::OutputPrinter;
use gitql_cli::printer::csv_printer::CSVPrinter;
use gitql_cli::printer::json_printer::JSONPrinter;
use gitql_cli::printer::table_printer::TablePrinter;
use gitql_core::environment::Environment;
use gitql_engine::data_provider::DataProvider;
use gitql_engine::engine;
use gitql_parser::diagnostic::Diagnostic;
use gitql_parser::parser;
use gitql_parser::tokenizer::Tokenizer;
use schema::create_fileql_environment;

mod arguments;
mod data_provider;
mod functions;
mod schema;

fn main() {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_BACKTRACE", "1");
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    let args: Vec<String> = std::env::args().collect();
    let command = arguments::parse_arguments(&args);
    match command {
        Command::ReplMode(arguments) => launch_fileql_repl(arguments),
        Command::QueryMode(query, arguments) => {
            let mut reporter = DiagnosticReporter::default();
            let files = &arguments.files;
            if let Err(error) = validate_files_paths(files) {
                reporter.report_diagnostic("", Diagnostic::error(error.as_str()));
                return;
            }

            let mut env = create_fileql_environment();
            execute_fileql_query(query, &arguments, files, &mut env, &mut reporter);
        }
        Command::Help => {
            arguments::print_help_list();
        }
        Command::Version => {
            println!("FileQL version {}", env!("CARGO_PKG_VERSION"));
        }
        Command::Error(error_message) => {
            println!("{}", error_message);
        }
    }
}

fn launch_fileql_repl(arguments: Arguments) {
    let mut reporter = diagnostic_reporter::DiagnosticReporter::default();
    let files = &arguments.files;
    if let Err(error) = validate_files_paths(files) {
        reporter.report_diagnostic("", Diagnostic::error(error.as_str()));
        return;
    }

    let mut global_env = create_fileql_environment();

    let mut input = String::new();

    loop {
        let stdio = io::stdin();

        // Render Prompt only if input is received from terminal
        if stdio.is_terminal() {
            print!("fileql > ");
        }

        std::io::Write::flush(&mut std::io::stdout()).expect("flush failed!");
        match stdio.read_line(&mut input) {
            Ok(buffer_length) => {
                if buffer_length == 0 {
                    break;
                }
            }
            Err(error) => {
                reporter.report_diagnostic(&input, Diagnostic::error(&format!("{}", error)));
            }
        }

        let stdin_input = input.trim();
        if stdin_input.is_empty() || stdin_input == "\n" {
            continue;
        }

        if stdin_input == "exit" {
            println!("Goodbye!");
            break;
        }

        execute_fileql_query(
            stdin_input.to_owned(),
            &arguments,
            files,
            &mut global_env,
            &mut reporter,
        );

        input.clear();
        global_env.clear_session();
    }
}

fn execute_fileql_query(
    query: String,
    arguments: &Arguments,
    files: &[String],
    env: &mut Environment,
    reporter: &mut DiagnosticReporter,
) {
    let front_start = std::time::Instant::now();
    let tokenizer_result = Tokenizer::tokenize(query.clone());
    if tokenizer_result.is_err() {
        let diagnostic = tokenizer_result.err().unwrap();
        reporter.report_diagnostic(&query, *diagnostic);
        return;
    }

    let tokens = tokenizer_result.ok().unwrap();
    if tokens.is_empty() {
        return;
    }

    let parser_result = parser::parse_gql(tokens, env);
    if parser_result.is_err() {
        let diagnostic = parser_result.err().unwrap();
        reporter.report_diagnostic(&query, *diagnostic);
        return;
    }

    let query_node = parser_result.ok().unwrap();
    let front_duration = front_start.elapsed();

    let engine_start = std::time::Instant::now();
    let files = files.to_vec();
    let excludes = arguments.excludes.to_vec();
    let file_provider = FileDataProvider::new(files, excludes);
    let provider: Box<dyn DataProvider> = Box::new(file_provider);
    let evaluation_result = engine::evaluate(env, &provider, query_node);

    // Report Runtime exceptions if they exists
    if evaluation_result.is_err() {
        reporter.report_diagnostic(
            &query,
            Diagnostic::exception(&evaluation_result.err().unwrap()),
        );
        return;
    }

    // Render the result only if they are selected groups not any other statement
    // Render the result only if they are selected groups not any other statement
    let engine_duration = engine_start.elapsed();

    let printer: Box<dyn OutputPrinter> = match arguments.output_format {
        OutputFormat::Render => {
            Box::new(TablePrinter::new(arguments.pagination, arguments.page_size))
        }
        OutputFormat::JSON => Box::new(JSONPrinter {}),
        OutputFormat::CSV => Box::new(CSVPrinter {}),
    };

    // Render the result only if they are selected groups not any other statement
    let evaluations_results = evaluation_result.ok().unwrap();
    for evaluation_result in evaluations_results {
        let mut rows_count = 0;
        if let SelectedGroups(mut groups) = evaluation_result {
            if !groups.is_empty() {
                rows_count += groups.groups[0].len();
                printer.print(&mut groups);
            }
        }

        if arguments.analysis {
            let total_time = front_duration + engine_duration;
            println!(
                "{} row in set (total: {:?}, front: {:?}, engine: {:?})",
                rows_count, total_time, front_duration, engine_duration
            );
        }
    }
}

fn validate_files_paths(files: &[String]) -> Result<(), String> {
    for file in files {
        if !Path::new(file).exists() {
            return Err(format!("File ${} is not exists", file));
        }
    }
    Ok(())
}
