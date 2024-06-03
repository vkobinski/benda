mod parser;

use parser::Parser;
use pyo3::prelude::*;

use rustpython_parser::{ parse, Mode };

mod benda_ffi;

fn main() -> PyResult<()> {
    let filename = String::from("tree.py");

    let code = std::fs::read_to_string(&filename).unwrap();
    let module = parse(code.as_str(), Mode::Module, "tree.py").unwrap();

    match module {
        rustpython_parser::ast::Mod::Module(mods) => {
            let mut parser = Parser::new(mods.body, 0);
            parser.parse(&String::from("gen_tree"));
        }
        _ => todo!(),
    }

    Ok(())
}
