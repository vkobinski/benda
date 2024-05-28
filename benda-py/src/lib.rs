use pyo3::{ prelude::*, types::PyFunction };
use types::u24::u24;
mod types;
mod parser;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn bjit(fun: Bound<PyFunction>) -> PyResult<PyObject> {
    let name = match fun.downcast::<PyFunction>() {
        Ok(inner) => { 
            println!("{:?}", inner.dir());
            println!("{:?}", inner.getattr("file"));
        }
        Err(_) => todo!(),
    };

    todo!();
}

/// A Python module implemented in Rust.
#[pymodule]
fn benda(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(bjit, m)?)?;
    m.add_class::<u24>()?;
    Ok(())
}
