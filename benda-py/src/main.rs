mod parser;

use parser::Parser;
use pyo3::{ prelude::*, types::{ PyCode, PyFunction } };

use rustpython_parser::{ast::ModModule, parse, Mode, Parse};


mod benda_ffi;

fn main() -> PyResult<()> {

    let filename = String::from("main.py");

    let code = std::fs::read_to_string(filename.to_string()).unwrap();
    let module = parse(code.as_str(), Mode::Module, "main.py").unwrap();

    match module {
        rustpython_parser::ast::Mod::Module(mods) => {
            let mut parser = Parser::new(mods.body, 0);
            parser.parse(&String::from("sum_nums"));
        },
        _ => todo!(),
    };

    Ok(())
}
