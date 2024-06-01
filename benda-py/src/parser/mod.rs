use core::panic;
use std::vec;

use bend::{ fun::{ Adt, Book, CtrField, Name, Op }, imp::{ self, Expr, MatchArm } };
use indexmap::IndexMap;
use rustpython_parser::{ ast::{ located, ExprBinOp, StmtAssign, StmtMatch }, text_size::TextRange };

use rustpython_parser::ast::Expr as rExpr;
use rustpython_parser::ast::Stmt as rStmt;
use rustpython_parser::ast::CmpOp as rCmpOp;
use rustpython_parser::ast::Operator as rOperator;
use rustpython_parser::ast::Pattern as rPattern;

use crate::benda_ffi::run;
use num_traits::cast::ToPrimitive;

pub struct Parser {
    statements: Vec<rStmt<TextRange>>,
    book: Book,
    definitions: Vec<imp::Definition>,
}

impl Parser {
    pub fn new(statements: Vec<rStmt<TextRange>>, _index: usize) -> Self {
        Self {
            statements,
            book: Book::default(),
            definitions: vec![],
        }
    }

    fn parse_expr_type(&self, expr: Box<rExpr<TextRange>>) -> Option<imp::Expr> {
        match *expr {
            rExpr::Attribute(att) => {
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
            rExpr::Compare(comp) => {
                let left = self.parse_expr_type(comp.left);
                let right = self.parse_expr_type(
                    Box::new(comp.comparators.first().unwrap().clone())
                );

                let op = match comp.ops.first().unwrap() {
                    rCmpOp::Eq => Op::EQ,
                    rCmpOp::NotEq => Op::NEQ,
                    rCmpOp::Lt => Op::LT,
                    rCmpOp::LtE => todo!(),
                    rCmpOp::Gt => Op::GT,
                    rCmpOp::GtE => todo!(),
                    rCmpOp::Is => todo!(),
                    rCmpOp::IsNot => todo!(),
                    rCmpOp::In => todo!(),
                    rCmpOp::NotIn => todo!(),
                };

                Some(Expr::Bin {
                    op,
                    lhs: Box::new(left.unwrap()),
                    rhs: Box::new(right.unwrap()),
                })
            }
            rExpr::BinOp(bin_op) => { self.parse_bin_op(bin_op) }
            rExpr::Constant(c) => {
                match c.value {
                    located::Constant::None => todo!(),
                    located::Constant::Bool(_) => todo!(),
                    located::Constant::Str(_) => todo!(),
                    located::Constant::Bytes(_) => todo!(),
                    located::Constant::Int(val) => {
                        Some(imp::Expr::Num {
                            val: bend::fun::Num::U24(val.to_u32().unwrap()),
                        })
                    }
                    located::Constant::Tuple(_) => todo!(),
                    located::Constant::Float(_) => todo!(),
                    located::Constant::Complex { real: _, imag: _ } => todo!(),
                    located::Constant::Ellipsis => todo!(),
                }
            }
            rExpr::Call(c) => {
                let fun = c.func;

                let expr = self.parse_expr_type(fun);

                if let Some(Expr::Var { nam }) = expr {
                    match self.book.adts.get(&nam.clone()) {
                        Some(val) => {
                            let arg = self.parse_expr_type(
                                Box::new(c.args.first().unwrap().clone())
                            );

                            return Some(imp::Expr::Constructor {
                                name: val.ctrs.first().unwrap().0.clone(),
                                args: vec![arg.unwrap()],
                                kwargs: vec![],
                            });
                        }
                        None => panic!("Type not defined."),
                    }
                }

                expr
            }
            rExpr::Name(n) => { Some(imp::Expr::Var { nam: Name::new(n.id.to_string()) }) }
            _ => todo!(),
        }
    }

    fn parse_bin_op(&self, bin: ExprBinOp<TextRange>) -> Option<imp::Expr> {
        // TODO(#5): Treat case where expr type returns None
        let left = self.parse_expr_type(bin.left).unwrap();
        let right = self.parse_expr_type(bin.right).unwrap();

        let op: Op = match bin.op {
            rOperator::Add => Op::ADD,
            rOperator::Sub => Op::SUB,
            rOperator::Mult => Op::MUL,
            rOperator::MatMult => todo!(),
            rOperator::Div => Op::DIV,
            rOperator::Mod => todo!(),
            rOperator::Pow => Op::POW,
            rOperator::LShift => Op::SHL,
            rOperator::RShift => Op::SHR,
            rOperator::BitOr => Op::OR,
            rOperator::BitXor => Op::XOR,
            rOperator::BitAnd => Op::AND,
            rOperator::FloorDiv => todo!(),
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
        stmts: &Vec<rStmt<TextRange>>,
        index: &usize
    ) -> Option<imp::Stmt> {
        let mut arms: Vec<imp::MatchArm> = vec![];

        for case in &m.cases {
            let stmt_arm = self.parse_vec(&case.body.clone(), 0);

            let pat = match &case.pattern {
                rPattern::MatchValue(val) => {
                    let expr = self.parse_expr_type(val.value.clone()).unwrap();
                    match expr {
                        imp::Expr::Var { nam } => { Some(nam) }
                        _ => None,
                    }
                }
                rPattern::MatchSingleton(_) => todo!(),
                rPattern::MatchSequence(_) => todo!(),
                rPattern::MatchMapping(_) => todo!(),
                rPattern::MatchClass(_) => todo!(),
                rPattern::MatchStar(_) => todo!(),
                rPattern::MatchAs(_) => todo!(),
                rPattern::MatchOr(_) => todo!(),
            };

            if let Some(a) = stmt_arm {
                let arm = MatchArm { lft: pat, rgt: a };
                arms.push(arm);
            }
        }

        Some(imp::Stmt::Match {
            arg: Box::new(self.parse_expr_type(m.subject.clone()).unwrap()),
            // TODO(#7): Add binding
            bind: None,
            arms,
            nxt: Some(Box::new(self.parse_vec(stmts, index + 1)?)),
        })
    }

    fn parse_vec(&mut self, stmts: &Vec<rStmt<TextRange>>, index: usize) -> Option<imp::Stmt> {
        let stmt = match stmts.get(index) {
            Some(s) => { s }
            None => {
                return None;
            }
        };

        match stmt {
            rStmt::Assign(assign) => {
                let value = self.parse_assign(assign).unwrap();
                let name = assign.targets
                    .first()
                    .unwrap()
                    .clone()
                    .name_expr()
                    .unwrap()
                    .id.to_string();

                let nxt = self.parse_vec(stmts, index + 1);

                if let Expr::Call { fun, args: _, kwargs: _ } = value.clone() {
                    if let Expr::Var { nam } = *fun {
                        if nam.to_string() == "switch" {
                            let mut arms: Vec<imp::Stmt> = vec![];
                            if let Some(rStmt::Match(m)) = stmts.get(index + 1) {
                                for case in &m.cases {
                                    let stmt_arm = self.parse_vec(&case.body.clone(), 0);

                                    if let Some(a) = stmt_arm {
                                        arms.push(a);
                                    }
                                }

                                return Some(imp::Stmt::Switch {
                                    arg: Box::new(self.parse_expr_type(m.subject.clone()).unwrap()),
                                    bind: Some(Name::new(name)),
                                    arms,
                                    nxt: nxt.map(Box::new),
                                });
                            }
                        }
                    }
                }

                Some(imp::Stmt::Assign {
                    pat: imp::AssignPattern::Var(Name::new(name)),
                    val: Box::new(value),
                    nxt: nxt.map(Box::new),
                })
            }
            rStmt::Return(r) => {
                match &r.value {
                    Some(val) => {
                        let term = self.parse_expr_type(val.clone()).unwrap();
                        Some(imp::Stmt::Return { term: Box::new(term) })
                    }
                    None => None,
                }
            }
            rStmt::Match(_m) => {
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

    fn add_adt(&mut self, nam: Name, adt: Adt) {
        if let Some(adt) = self.book.adts.get(&nam) {
            if adt.builtin {
                panic!("{} is a built-in datatype and should not be overridden.", nam);
            } else {
                panic!("Repeated datatype '{}'", nam);
            }
        } else {
            for ctr in adt.ctrs.keys() {
                match self.book.ctrs.entry(ctr.clone()) {
                    indexmap::map::Entry::Vacant(e) => {
                        _ = e.insert(nam.clone());
                    }
                    indexmap::map::Entry::Occupied(e) => {
                        if self.book.adts.get(e.get()).is_some_and(|adt| adt.builtin) {
                            panic!(
                                "{} is a built-in constructor and should not be overridden.",
                                e.key()
                            );
                        } else {
                            panic!("Repeated constructor '{}'", e.key());
                        }
                    }
                }
            }
        }
        self.book.adts.insert(nam.clone(), adt);
    }

    pub fn parse(&mut self, _fun: &str) -> String {
        for stmt in self.statements.clone() {
            match stmt {
                rStmt::FunctionDef(fun_def) => {
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
                rStmt::ClassDef(class) => {
                    let is_dataclass = class.decorator_list.iter().any(|exp| {
                        if let rExpr::Name(nam) = exp {
                            if nam.id.to_string() == "dataclass" {
                                return true;
                            }
                        }
                        false
                    });

                    let iden = class.name.to_string();

                    if is_dataclass {
                        let stmt = class.body.first().unwrap();
                        let mut adt = Adt { ctrs: IndexMap::new(), builtin: false };

                        match stmt {
                            rStmt::AnnAssign(assign) => {
                                let mut e_type = String::default();
                                let mut target = String::default();

                                if let rExpr::Name(nam) = *assign.annotation.clone() {
                                    e_type = nam.id.to_string();
                                }

                                if let rExpr::Name(nam) = *assign.target.clone() {
                                    target = nam.id.to_string();
                                }

                                let ctr_field = CtrField { nam: Name::new(target), rec: true };

                                adt.ctrs.insert(
                                    Name::new(format!("{}/{}", iden, e_type)),
                                    vec![ctr_field]
                                );
                            }
                            _ => todo!(),
                        }

                        self.add_adt(Name::new(iden), adt);
                    }
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

        //let main_def = imp::Definition {
        //    name: Name::new("main"),
        //    params: vec![],
        //    body: imp::Stmt::Return {
        //        term: Box::new(imp::Expr::Call {
        //            fun: Box::new(imp::Expr::Var { nam: Name::new("sum_nums") }),
        //            args: vec![imp::Expr::Num { val: bend::fun::Num::U24(2) }],
        //            kwargs: vec![],
        //        }),
        //    },
        //};

        let main_def = imp::Definition {
            name: Name::new("main"),
            params: vec![],
            body: imp::Stmt::Return {
                term: Box::new(imp::Expr::Call {
                    fun: Box::new(imp::Expr::Var { nam: Name::new("sum_nums") }),
                    args: vec![imp::Expr::Num { val: bend::fun::Num::U24(2) }],
                    kwargs: vec![],
                }),
            },
        };

        self.book.defs.insert(Name::new("main"), main_def.to_fun(true).unwrap());

        self.book.entrypoint = None;

        println!("BEND:\n {}", self.book.display_pretty());

        //println!("BEND: \n{:?}", self.book.defs);

        let return_val = run(&self.book);

        match return_val {
            Some(val) => { val.0.to_string() }
            None => panic!("Could not run Bend code."),
        }
    }
}
