use pyo3::{pyclass, pymethods};

#[pyclass(module = "benda_py")]
#[allow(non_camel_case_types)]
pub struct u24(u32);

#[pymethods]
impl u24 {

    #[new]
    fn new(value: u32) -> Self {
        Self(value)
    }

    fn __add__(&self, other: &Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}
