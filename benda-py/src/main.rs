use pyo3::prelude::*;
use pyo3::types::PyTuple;

fn main() -> PyResult<()> {

    pyo3::prepare_freethreaded_python();

    let code = std::fs::read_to_string("main.py").unwrap();

    Python::with_gil(|py| {
        let fun: Py<PyAny> = PyModule::from_code_bound(
            py,
            &code,
            "main.py",
            "example"
        ).unwrap()
        .getattr("print_ast")
        .unwrap()
        .into();

        let ast: String = fun.call0(py).unwrap().extract(py).unwrap();

        println!("AST gerada pelo Python: \n{}", ast);

    });
    Ok(())
}