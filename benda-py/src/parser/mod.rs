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

#[derive(Clone)]
enum FromExpr {
    CtrField(Vec<CtrField>),
    Expr(imp::Expr),
    Statement(imp::Stmt),
}

pub struct Parser {
    statements: Vec<rStmt<TextRange>>,
    book: Book,
    definitions: Vec<imp::Definition>,
}

impl Parser {
    pub fn new(statements: Vec<rStmt<TextRange>>, _index: usize) -> Self {
        Self {
            statements,
            //book:  fun::Book::builtins(),
            book: Book::default(),
            definitions: vec![],
        }
    }

    fn parse_expr_type(&self, expr: Box<rExpr<TextRange>>) -> Option<FromExpr> {
        match *expr {
            rExpr::Attribute(att) => {
                if
                    let FromExpr::Expr(imp::Expr::Var { nam: lib }) = self
                        .parse_expr_type(att.value)
                        .unwrap()
                {
                    let fun = att.attr.to_string();
                    if lib.to_string() == "benda" && fun == "switch" {
                        return Some(
                            FromExpr::Expr(Expr::Call {
                                fun: Box::new(Expr::Var { nam: Name::new("switch") }),
                                args: vec!(),
                                kwargs: vec!(),
                            })
                        );
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

                if
                    let (FromExpr::Expr(left), FromExpr::Expr(right)) = (
                        left.unwrap(),
                        right.unwrap(),
                    )
                {
                    return Some(
                        FromExpr::Expr(Expr::Bin {
                            op,
                            lhs: Box::new(left),
                            rhs: Box::new(right),
                        })
                    );
                }
                None
            }
            rExpr::BinOp(bin_op) => { self.parse_bin_op(bin_op) }
            rExpr::Constant(c) => {
                match c.value {
                    located::Constant::None => todo!(),
                    located::Constant::Bool(_) => todo!(),
                    located::Constant::Str(str) => {
                        let nam = Name::new(str);
                        let adt = self.book.adts.get(&nam);

                        if let Some(_adt) = adt {
                            return Some(FromExpr::Expr(imp::Expr::Var { nam }));
                        }
                        None
                    }
                    located::Constant::Bytes(_) => todo!(),
                    located::Constant::Int(val) => {
                        Some(
                            FromExpr::Expr(imp::Expr::Num {
                                val: bend::fun::Num::U24(val.to_u32().unwrap()),
                            })
                        )
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

                if let Some(FromExpr::Expr(Expr::Var { ref nam })) = expr {
                    let mut args: Vec<Expr> = vec![];

                    for arg in c.args {
                        let arg = self.parse_expr_type(Box::new(arg));

                        if let Some(FromExpr::Expr(e)) = arg {
                            args.push(e);
                        }
                    }
                    if let Some(val) = self.book.adts.get(&nam.clone()) {
                        return Some(
                            FromExpr::Expr(imp::Expr::Constructor {
                                name: val.ctrs.first().unwrap().0.clone(),
                                args,
                                kwargs: vec![],
                            })
                        );
                    }
                    return Some(
                        FromExpr::Expr(imp::Expr::Call {
                            fun: Box::new(Expr::Var { nam: Name::new(nam.to_string()) }),
                            args,
                            kwargs: vec!(),
                        })
                    );
                }
                expr
            }
            rExpr::Name(n) => {
                Some(FromExpr::Expr(imp::Expr::Var { nam: Name::new(n.id.to_string()) }))
            }
            _ => todo!(),
        }
    }

    fn parse_bin_op(&self, bin: ExprBinOp<TextRange>) -> Option<FromExpr> {
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

        if
            let (
                FromExpr::Expr(Expr::Var { nam: nam_l }),
                FromExpr::Expr(Expr::Var { nam: nam_r }),
            ) = (left.clone(), right.clone())
        {
            let adt_l = self.book.adts.get(&nam_l);
            let adt_r = self.book.adts.get(&nam_r);

            let mut is_adt = false;

            match (adt_l, adt_r) {
                (Some(_), Some(_)) => {
                    is_adt = true;
                }
                (Some(_), _) => {
                    is_adt = true;
                }
                (_, Some(_)) => {
                    is_adt = true;
                }
                (_, _) => {}
            }

            let mut fields: Vec<CtrField> = vec![];

            if is_adt {
                fields.push(CtrField { nam: nam_l.clone(), rec: false });
            }

            return Some(FromExpr::CtrField(fields));
        }

        if let (FromExpr::Expr(left), FromExpr::Expr(right)) = (left, right) {
            let operation = imp::Expr::Bin {
                op,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };

            return Some(FromExpr::Expr(operation));
        }
        todo!()
    }

    fn parse_assign(&mut self, assign: &StmtAssign<TextRange>) -> Option<FromExpr> {
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
                        FromExpr::Expr(imp::Expr::Var { nam }) => { Some(nam) }
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

            if let Some(FromExpr::Statement(a)) = stmt_arm {
                let arm = MatchArm { lft: pat, rgt: a };
                arms.push(arm);
            }
        }

        if let Some(FromExpr::Expr(m)) = self.parse_expr_type(m.subject.clone()) {
            let nxt = self.parse_vec(stmts, index + 1);

            if let Some(FromExpr::Statement(e)) = nxt {
                return Some(imp::Stmt::Match {
                    arg: Box::new(m),
                    // TODO(#7): Add binding
                    bind: None,
                    arms,
                    nxt: Some(Box::new(e)),
                });
            }
        }
        None
    }

    fn parse_vec(&mut self, stmts: &Vec<rStmt<TextRange>>, index: usize) -> Option<FromExpr> {
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

                if let FromExpr::Expr(Expr::Call { fun, args: _, kwargs: _ }) = value.clone() {
                    if let Expr::Var { nam } = *fun {
                        if nam.to_string() == "switch" {
                            let mut arms: Vec<imp::Stmt> = vec![];
                            if let Some(rStmt::Match(m)) = stmts.get(index + 1) {
                                for case in &m.cases {
                                    let stmt_arm = self.parse_vec(&case.body.clone(), 0);

                                    if let Some(FromExpr::Statement(a)) = stmt_arm {
                                        arms.push(a);
                                    }
                                }

                                if
                                    let Some(FromExpr::Expr(expr)) = self.parse_expr_type(
                                        m.subject.clone()
                                    )
                                {
                                    return Some(
                                        FromExpr::Statement(imp::Stmt::Switch {
                                            arg: Box::new(expr),
                                            bind: Some(Name::new(name)),
                                            arms,
                                            nxt: nxt.map(|n| {
                                                if let FromExpr::Statement(n) = n {
                                                    return Box::new(n);
                                                }

                                                todo!()
                                            }),
                                        })
                                    );
                                }
                            }
                        }
                    }
                }

                if let FromExpr::Expr(val) = value {
                    return Some(
                        FromExpr::Statement(imp::Stmt::Assign {
                            pat: imp::AssignPattern::Var(Name::new(name)),
                            val: Box::new(val),
                            nxt: nxt.map(|n| {
                                if let FromExpr::Statement(n) = n {
                                    return Box::new(n);
                                }

                                todo!()
                            }),
                        })
                    );
                }

                Some(value)
            }
            rStmt::Return(r) => {
                match &r.value {
                    Some(val) => {
                        let term = self.parse_expr_type(val.clone()).unwrap();
                        if let FromExpr::Expr(term) = term {
                            return Some(
                                FromExpr::Statement(imp::Stmt::Return { term: Box::new(term) })
                            );
                        }

                        todo!()
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

                    if let Some(FromExpr::Statement(e)) = expr {
                        let def = imp::Definition {
                            name: Name::new(fun_def.name.to_string()),
                            params: names,
                            body: e,
                        };
                        self.definitions.push(def);
                    }
                }
                rStmt::Assign(assign) => {
                    let iden = assign.targets.first().unwrap();

                    let name: String;

                    if let rExpr::Name(iden) = iden {
                        name = iden.id.to_string();
                        let mut adt = Adt { ctrs: IndexMap::new(), builtin: false };

                        let body = self.parse_expr_type(assign.value);

                        if let Some(FromExpr::CtrField(ctr)) = body {
                            adt.ctrs.insert(
                                Name::new(format!("{}/{}", name, ctr.first().unwrap().nam)),
                                ctr
                            );
                        }

                        self.add_adt(Name::new(name), adt);
                    }
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
                    let mut adt = Adt { ctrs: IndexMap::new(), builtin: false };

                    if is_dataclass {
                        for stmt in class.body {
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
                        }
                        self.add_adt(Name::new(iden.clone()), adt);
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

        let main_def = imp::Definition {
            name: Name::new("main"),
            params: vec![],
            body: imp::Stmt::Return {
                term: Box::new(imp::Expr::Call {
                    fun: Box::new(imp::Expr::Var { nam: Name::new("sum_nums") }),
                    args: vec![
                        imp::Expr::Num { val: bend::fun::Num::U24(4) },
                        imp::Expr::Num { val: bend::fun::Num::U24(10) }
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
