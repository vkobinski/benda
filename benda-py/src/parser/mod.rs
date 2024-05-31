use core::panic;
use std::vec;

use bend::{ fun::{ Book, Name, Op }, hvm::ast::OP_XOR, imp::{ self, Definition, Expr, MatchArm } };
use rustpython_parser::{
    ast::{ located::{ self, Stmt }, ExprBinOp, StmtAssign, StmtMatch },
    text_size::TextRange,
};

use crate::benda_ffi::run;
use num_traits::cast::ToPrimitive;

pub struct Parser {
    statements: Vec<rustpython_parser::ast::Stmt<TextRange>>,
    book: Book,
    definitions: Vec<imp::Definition>,
}

impl Parser {
    pub fn new(statements: Vec<rustpython_parser::ast::Stmt<TextRange>>, index: usize) -> Self {
        Self {
            statements,
            book: Book::default(),
            definitions: vec![],
        }
    }

    fn parse_expr_type(
        &self,
        expr: Box<rustpython_parser::ast::Expr<TextRange>>
    ) -> Option<imp::Expr> {
        match *expr {
            rustpython_parser::ast::Expr::Attribute(att) => {
                if let imp::Expr::Var { nam: lib } = self.parse_expr_type(att.value).unwrap() {
                    let fun = att.attr.to_string();
                    if lib.to_string() == "benda" && fun == "switch" {
                        return Some(Expr::Call {
                            fun: Box::new(Expr::Var { nam: Name::new("switch") }),
                            args: vec!(),
                            kwargs: vec!(),
                        });
                    }
                }
                None
            }
            rustpython_parser::ast::Expr::Compare(comp) => {
                println!("EXPR : {:?}", comp);

                let left = self.parse_expr_type(comp.left);
                let right = self.parse_expr_type(
                    Box::new(comp.comparators.get(0).unwrap().clone())
                );

                let op = match comp.ops.get(0).unwrap() {
                    rustpython_parser::ast::CmpOp::Eq => Op::EQ,
                    rustpython_parser::ast::CmpOp::NotEq => Op::NEQ,
                    rustpython_parser::ast::CmpOp::Lt => Op::LT,
                    rustpython_parser::ast::CmpOp::LtE => todo!(),
                    rustpython_parser::ast::CmpOp::Gt => Op::GT,
                    rustpython_parser::ast::CmpOp::GtE => todo!(),
                    rustpython_parser::ast::CmpOp::Is => todo!(),
                    rustpython_parser::ast::CmpOp::IsNot => todo!(),
                    rustpython_parser::ast::CmpOp::In => todo!(),
                    rustpython_parser::ast::CmpOp::NotIn => todo!(),
                };

                Some(Expr::Bin {
                    op: op,
                    lhs: Box::new(left.unwrap()),
                    rhs: Box::new(right.unwrap()),
                })
            }
            rustpython_parser::ast::Expr::BinOp(bin_op) => { self.parse_bin_op(bin_op) }
            rustpython_parser::ast::Expr::Constant(c) => {
                match c.value {
                    located::Constant::None => todo!(),
                    located::Constant::Bool(_) => todo!(),
                    located::Constant::Str(str) => todo!(),
                    located::Constant::Bytes(_) => todo!(),
                    located::Constant::Int(val) => {
                        Some(imp::Expr::Num {
                            val: bend::fun::Num::U24(val.to_u32().unwrap()),
                        })
                    }
                    located::Constant::Tuple(_) => todo!(),
                    located::Constant::Float(_) => todo!(),
                    located::Constant::Complex { real, imag } => todo!(),
                    located::Constant::Ellipsis => todo!(),
                }
            }
            rustpython_parser::ast::Expr::Call(c) => {
                let fun = c.func;

                let expr = self.parse_expr_type(fun);
                expr
            }
            rustpython_parser::ast::Expr::Name(n) => {
                Some(imp::Expr::Var { nam: Name::new(n.id.to_string()) })
            }
            _ => todo!(),
        }
    }

    fn parse_bin_op(&self, bin: ExprBinOp<TextRange>) -> Option<imp::Expr> {
        // TODO(#5): Treat case where expr type returns None
        let left = self.parse_expr_type(bin.left).unwrap();
        let right = self.parse_expr_type(bin.right).unwrap();

        let op: Op = match bin.op {
            rustpython_parser::ast::Operator::Add => Op::ADD,
            rustpython_parser::ast::Operator::Sub => Op::SUB,
            rustpython_parser::ast::Operator::Mult => Op::MUL,
            rustpython_parser::ast::Operator::MatMult => todo!(),
            rustpython_parser::ast::Operator::Div => Op::DIV,
            rustpython_parser::ast::Operator::Mod => todo!(),
            rustpython_parser::ast::Operator::Pow => Op::POW,
            rustpython_parser::ast::Operator::LShift => Op::SHL,
            rustpython_parser::ast::Operator::RShift => Op::SHR,
            rustpython_parser::ast::Operator::BitOr => Op::OR,
            rustpython_parser::ast::Operator::BitXor => Op::XOR,
            rustpython_parser::ast::Operator::BitAnd => Op::AND,
            rustpython_parser::ast::Operator::FloorDiv => todo!(),
        };

        let operation = imp::Expr::Bin {
            op,
            lhs: Box::new(left),
            rhs: Box::new(right),
        };

        Some(operation)
    }

    fn parse_assign(&mut self, assign: &StmtAssign<TextRange>) -> Option<imp::Expr> {
        self.parse_expr_type(assign.value.clone())
    }

    fn parse_match(
        &mut self,
        m: &StmtMatch<TextRange>,
        stmts: &Vec<rustpython_parser::ast::Stmt<TextRange>>,
        index: &usize
    ) -> Option<imp::Stmt> {
        let mut arms: Vec<imp::MatchArm> = vec![];

        for case in &m.cases {
            let stmt_arm = self.parse_vec(&case.body.clone(), 0);

            let pat = match &case.pattern {
                rustpython_parser::ast::Pattern::MatchValue(val) => {
                    let expr = self.parse_expr_type(val.value.clone()).unwrap();
                    match expr {
                        imp::Expr::Var { nam } => { Some(nam) }
                        _ => None,
                    }
                }
                rustpython_parser::ast::Pattern::MatchSingleton(_) => todo!(),
                rustpython_parser::ast::Pattern::MatchSequence(_) => todo!(),
                rustpython_parser::ast::Pattern::MatchMapping(_) => todo!(),
                rustpython_parser::ast::Pattern::MatchClass(_) => todo!(),
                rustpython_parser::ast::Pattern::MatchStar(_) => todo!(),
                rustpython_parser::ast::Pattern::MatchAs(_) => todo!(),
                rustpython_parser::ast::Pattern::MatchOr(_) => todo!(),
            };

            match stmt_arm {
                Some(a) => {
                    let arm = MatchArm { lft: pat, rgt: a };
                    arms.push(arm);
                }
                None => {}
            }
        }

        Some(imp::Stmt::Match {
            arg: Box::new(self.parse_expr_type(m.subject.clone()).unwrap()),
            // TODO(#7): Add binding
            bind: None,
            arms: arms,
            nxt: Some(Box::new(self.parse_vec(stmts, index + 1)?)),
        })
    }

    fn parse_vec(
        &mut self,
        stmts: &Vec<rustpython_parser::ast::Stmt<TextRange>>,
        index: usize
    ) -> Option<imp::Stmt> {
        let stmt = match stmts.get(index) {
            Some(s) => s,
            None => {
                return None;
            }
        };

        match stmt {
            rustpython_parser::ast::Stmt::Assign(assign) => {
                let value = self.parse_assign(assign).unwrap();
                let name = assign.targets
                    .get(0)
                    .unwrap()
                    .clone()
                    .name_expr()
                    .unwrap()
                    .id.to_string();

                let nxt = self.parse_vec(stmts, index + 1);

                match value {
                    Expr::Call { fun, args, kwargs } => {
                        if let Expr::Var { nam } = *fun {
                            if nam.to_string() != "switch" {
                                return None;
                            }
                        }

                        let mut arms: Vec<imp::Stmt> = vec![];
                        let m: &StmtMatch;

                        match stmts.get(index + 1).unwrap() {
                            rustpython_parser::ast::Stmt::Match(ma) => {
                                m = ma;
                            }
                            _ => {
                                panic!();
                            }
                        }

                        for case in &m.cases {
                            let stmt_arm = self.parse_vec(&case.body.clone(), 0);

                            if let Some(a) = stmt_arm {
                                arms.push(a);
                            }
                        }

                        return Some(imp::Stmt::Switch {
                            arg: Box::new(self.parse_expr_type(m.subject.clone()).unwrap()),
                            bind: Some(Name::new(name)),
                            arms: arms,
                            nxt: match nxt {
                                Some(n) => { Some(Box::new(n)) }
                                None => None,
                            },
                        });
                    }
                    _ => {}
                }

                Some(imp::Stmt::Assign {
                    pat: imp::AssignPattern::Var(Name::new(name)),
                    val: Box::new(value),
                    nxt: match nxt {
                        Some(n) => { Some(Box::new(n)) }
                        None => None,
                    },
                })
            }
            rustpython_parser::ast::Stmt::Return(r) => {
                match &r.value {
                    Some(val) => {
                        let term = self.parse_expr_type(val.clone()).unwrap();
                        Some(imp::Stmt::Return { term: Box::new(term) })
                    }
                    None => None,
                }
            }
            rustpython_parser::ast::Stmt::Match(m) => {
                //let mut arms: Vec<imp::Stmt> = vec![];

                //for case in &m.cases {
                //    let stmt_arm = self.parse_vec(&case.body.clone(), 0);

                //    if let Some(a) = stmt_arm {
                //        arms.push(a);
                //    }
                //}

                //Some(imp::Stmt::Switch {
                //    arg: Box::new(self.parse_expr_type(m.subject.clone()).unwrap()),
                //    bind: None,
                //    arms: arms,
                //    nxt: Some(Box::new(self.parse_vec(stmts, index + 1)?)),
                //})
                None
            }
            _ => None,
        }
    }

    pub fn parse(&mut self, fun: &String) -> String {

        for stmt in self.statements.clone() {
            match stmt {
                rustpython_parser::ast::Stmt::FunctionDef(fun_def) => {
                    let args = *fun_def.args;
                    let mut names: Vec<Name> = vec![];

                    for arg in args.args {
                        names.push(Name::new(arg.def.arg.to_string()));
                    }

                    let expr = self.parse_vec(&fun_def.body, 0);

                    let def = imp::Definition {
                        name: Name::new(fun_def.name.to_string()),
                        params: names,
                        body: expr.unwrap(),
                    };

                    self.definitions.push(def);
                }
                _ => {
                    //self.parse_part(self.statements.get(self.index).unwrap().clone());
                }
            }
        }

        for def in &self.definitions {
            let fun_def = def.clone().to_fun(false).unwrap();
            self.book.defs.insert(fun_def.name.clone(), fun_def.clone());
        }

        let main_def = imp::Definition {
            name: Name::new("main"),
            params: vec![],
            body: imp::Stmt::Return {
                term: Box::new(imp::Expr::Call {
                    fun: Box::new(imp::Expr::Var { nam: Name::new("sum_nums") }),
                    args: vec![
                        imp::Expr::Num { val: bend::fun::Num::U24(2) },
                        imp::Expr::Num { val: bend::fun::Num::U24(2) },
                        imp::Expr::Num { val: bend::fun::Num::U24(12) }
                    ],
                    kwargs: vec![],
                }),
            },
        };

        self.book.defs.insert(Name::new("main"), main_def.to_fun(true).unwrap());

        self.book.entrypoint = None;

        println!("BEND:\n {}", self.book.display_pretty());

        let return_val = run(&self.book);

        match return_val {
            Some(val) => { val.0.to_string() }
            None => panic!("Could not run Bend code."),
        }
    }
}
