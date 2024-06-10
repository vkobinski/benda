use bend::imp;

use pyo3::{types::PyAnyMethods, Bound, FromPyObject, PyAny, PyTypeCheck};

pub mod tree;
pub mod u24;

pub trait BendType {
    fn to_bend(&self) -> imp::Expr;
}

pub fn extract_inner<'py, T: PyTypeCheck + FromPyObject<'py>>(arg: Bound<'py, PyAny>) -> Option<T> {
    let inner = arg.downcast::<T>();
    if let Ok(inner) = inner {
        let inner = <T as FromPyObject>::extract_bound(inner.as_any());
        return Some(inner.unwrap());
    }
    None
}
