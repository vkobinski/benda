use bend::{fun::Num, imp};

use pyo3::{types::PyAnyMethods, Bound, FromPyObject, PyAny, PyTypeCheck};

pub mod tree;
pub mod u24;

pub trait BendType {
    fn to_bend(&self) -> imp::Expr;
}

pub fn extract_inner<'py, T: BendType + PyTypeCheck + FromPyObject<'py>>(
    arg: Bound<'py, PyAny>,
) -> Option<T> {
    let inner = arg.downcast::<T>();
    if let Ok(inner) = inner {
        let inner = <T as FromPyObject>::extract_bound(inner.as_any());
        return Some(inner.unwrap());
    }
    None
}

pub fn extract_type<'py, T: BendType>(arg: Bound<'py, PyAny>) -> Option<T> {}

#[derive(Debug)]
pub enum BuiltinType {
    U32,
    F32,
    I32,
}

impl From<String> for BuiltinType {
    fn from(value: String) -> Self {
        println!("Type value: {:?}", value);
        match value.as_str() {
            "builtin.int" => BuiltinType::U32,
            //"benda.Node" => BuiltinType::Node,
            //"benda.Tree" => BuiltinType::Tree,
            _ => panic!("Could not parse type"),
        }
    }
}

impl BendType for u32 {
    fn to_bend(&self) -> imp::Expr {
        imp::Expr::Num {
            val: Num::U24(*self),
        }
    }
}

impl BendType for f32 {
    fn to_bend(&self) -> imp::Expr {
        imp::Expr::Num {
            val: Num::F24(*self),
        }
    }
}

impl BendType for i32 {
    fn to_bend(&self) -> imp::Expr {
        imp::Expr::Num {
            val: Num::I24(*self),
        }
    }
}
