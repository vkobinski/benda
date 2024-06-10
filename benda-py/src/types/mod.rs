use bend::imp;

pub mod u24;
pub mod tree;

pub trait BendType {
    fn to_bend(&self) -> imp::Expr;
}