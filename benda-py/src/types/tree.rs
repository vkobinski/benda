use core::panic;
use std::vec;

use bend::{fun, imp};
use pyo3::{ pyclass, pymethods, types::{ PyAnyMethods, PyTuple, PyTypeMethods }, Bound };

use super::{u24::u24, BendType};

#[derive(Clone, Debug)]
#[pyclass(module = "benda", name = "Leaf")]
pub struct Leaf {
    pub value: u24,
}

impl BendType for Leaf {
    fn to_bend(&self) -> imp::Expr {
        imp::Expr::Num { val: fun::Num::U24(self.value.get()) }
    }
}

#[pymethods]
impl Leaf {
    #[new]
    fn __new__(val: u32) -> Self {
        Self {
            value: u24::new(val),
        }
    }
}

#[derive(Clone, Debug)]
#[pyclass(module = "benda", name = "Node")]
pub struct Node {
    pub left: Option<Box<Tree>>,
    pub right: Option<Box<Tree>>,
}

#[pymethods]
impl Node {

    #[new]
    #[pyo3(signature = (*py_args))]
    fn new(py_args: &Bound<'_, PyTuple>) -> Self {

        let mut trees: Option<Tree> = None;

        for arg in py_args {
            let t_type = arg.get_type();
            let name = t_type.name().unwrap();

            let tree_type = TreeType::from(name.to_string());

            match tree_type {
                TreeType::Leaf => {

                let tree = arg.downcast::<Leaf>();
                    if let Ok(tree) = tree {
                        let new_tree = tree.extract::<Leaf>().unwrap();

                        let add_tree = Tree {leaf: Some(new_tree), node: None};

                        if let Some(tree) = trees {
                            return Self {
                                left: Some(Box::new(tree)),
                                right: Some(Box::new(add_tree))
                            }

                        } else {
                            trees = Some(add_tree);
                        }
                    }

                },
                TreeType::Node => {
                    let tree = arg.downcast::<Node>();
                    if let Ok(tree) = tree {
                        let new_tree = tree.extract::<Node>().unwrap();

                        let new_add = Tree {node: Some(new_tree), leaf: None};

                        if let Some(tree) = trees {
                            return Self {
                                left: Some(Box::new(tree)),
                                right: Some(Box::new(new_add))
                            }

                        } else {
                            trees = Some(new_add);
                        }
                    }
                }
                TreeType::Tree => {
                    let tree = arg.downcast::<Tree>();
                    if let Ok(tree) = tree {
                        let new_tree = tree.extract::<Tree>().unwrap();

                        if let Some(tree) = trees {
                            return Self {
                                left: Some(Box::new(tree)),
                                right: Some(Box::new(new_tree))
                            }

                        } else {
                            trees = Some(new_tree);
                        }
                    }

                }
            }
        };

        panic!("Node must receive two trees in its constructor")
    }

}

impl BendType for Node {
    fn to_bend(&self) -> imp::Expr {
        let mut trees: Vec<imp::Expr> = vec![];

        if let Some(left) = &self.left {
            trees.push(left.to_bend());
        }

        if let Some(right) = &self.right {
            trees.push(right.to_bend());
        }

        imp::Expr::Ctr { name: fun::Name::new("Tree/Node"), args: trees, kwargs: vec![] }
 
    }

}

#[derive(Clone, Debug)]
#[pyclass(module = "benda", name = "Tree")]
pub struct Tree {
    pub leaf: Option<Leaf>,
    pub node: Option<Node>,
}


impl BendType for Tree {
    fn to_bend(&self) -> bend::imp::Expr {
        if let Some(leaf) = &self.leaf {
            return leaf.to_bend();
        }

        if let Some(node) = &self.node {
            return node.to_bend();
        }

        todo!()
    }
}

#[derive(Debug)]
pub enum TreeType {
    Leaf,
    Node,
    Tree,
}


impl From<String> for TreeType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "benda.Leaf" => {TreeType::Leaf},
            "benda.Node" => {TreeType::Node},
            "benda.Tree" => {TreeType::Tree},
            _ => panic!("Tree __new__ must receive either Leaf or Node")
        }
    }
}

#[pymethods]
impl Tree {
    #[new]
    #[pyo3(signature = (*py_args))]
    fn new(py_args: &Bound<'_, PyTuple>) -> Self {

        for arg in py_args {
            let t_type = arg.get_type();
            let name = t_type.name().unwrap();

            let tree_type = TreeType::from(name.to_string());

            match tree_type {
                TreeType::Leaf => {
                    let leaf = arg.downcast::<Leaf>();
                    if let Ok(leaf) = leaf {
                        return Self {
                            leaf: Some(leaf.extract().unwrap()),
                            node: None
                        };
                    }
                },
                TreeType::Node => {
                    panic!("Tree must receive a Leaf in constructor")
                },
                TreeType::Tree => {
                    panic!("Tree must receive a Leaf in constructor")
                }
            }

        }

        todo!()
    }
}
