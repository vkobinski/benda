mod parser;

use std::path::Path;

use bend::{diagnostics::DiagnosticsConfig, CompileOpts};
use parser::Parser;
use pyo3::{ prelude::*, types::{ PyCode, PyFunction } };

use python_ast::parse;

mod benda_ffi;

fn main() -> PyResult<()> {

    let mut book = bend::load_file_to_book(Path::new("main.bend")).unwrap();

    println!("{}", book.display_pretty());

    let opts = CompileOpts::default();
    let diagnostics_cfg = DiagnosticsConfig::default();
    let args = None;

    let new_book = bend::compile_book(&mut book, opts, diagnostics_cfg, args);

    println!("NEW BOOK \n{}", book.display_pretty());

    let name = String::from("sum_nums");
    let filename = String::from("main.py");

    let code = std::fs::read_to_string(filename.to_string()).unwrap();
    let ast = parse(&code, "").unwrap();

    for stmt in &ast.raw.body {
        match &stmt.statement {
            python_ast::StatementType::FunctionDef(fun_def) => {
                if fun_def.name == name.to_string() {
                    let mut parser = Parser::new(fun_def.body.clone());
                    parser.parse();
                }
            }
            _ => {}
        }
    }

    Ok(())
}
