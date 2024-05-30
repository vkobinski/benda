use core::panic;

use bend::{ fun::{ Book, Definition, Name, Pattern, Rule, Term }, imp::{ self, Expr, Stmt } };
use python_ast::{ Assign, BinOp, BinOps, ExprType, Statement };

use crate::benda_ffi::run;

pub struct Parser {
    statements: Vec<Statement>,
    book: Book,
    index: usize,
}

impl Parser {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self {
            statements,
            book: Book::default(),
            index: 0,
        }
    }

    fn parse_expr_type(&self, expr: Box<ExprType>) -> Option<Expr> {
        match *expr {
            ExprType::BoolOp(_) => todo!(),
            ExprType::NamedExpr(_) => todo!(),
            ExprType::BinOp(bin_op) => {
                let left = self.parse_expr_type(bin_op.left);
                let right = self.parse_expr_type(bin_op.right);

                match bin_op.op {
                    BinOps::Add => {
                        Some(Expr::Bin {
                            op: bend::fun::Op::ADD,
                            lhs: Box::new(left.unwrap()),
                            rhs: Box::new(right.unwrap()),
                        })
                    }
                    _ => panic!(),
                }
            }
            ExprType::UnaryOp(_) => todo!(),
            ExprType::Await(_) => todo!(),
            ExprType::Compare(_) => todo!(),
            ExprType::Call(_) => todo!(),
            ExprType::Constant(c) =>
                Some(Expr::Num {
                    val: bend::fun::Num::U24(c.0.unwrap().to_string().parse().unwrap()),
                }),
            ExprType::Attribute(_) => todo!(),
            ExprType::Name(n) => { Some(Expr::Var { nam: Name::new(n.id) }) }
            ExprType::List(_) => todo!(),
            ExprType::NoneType(_) => todo!(),
            ExprType::Unimplemented(_) => todo!(),
            ExprType::Unknown => todo!(),
        }
    }

    fn parse_add(&self, bin: BinOp) -> Option<Expr> {
        // TODO: Treat case where expr type returns None
        let left = self.parse_expr_type(bin.left).unwrap();
        let right = self.parse_expr_type(bin.right).unwrap();

        let operation = Expr::Bin {
            op: bend::fun::Op::ADD,
            lhs: Box::new(left),
            rhs: Box::new(right),
        };

        Some(operation)
    }

    fn parse_assign(&mut self, assign: &Assign) -> Option<Expr> {
        // TODO: Implement tuple assignment
        let target = &assign.targets.get(0).unwrap().id;

        let value = match &assign.value {
            python_ast::ExprType::Constant(c) => {
                Expr::Num {
                    val: bend::fun::Num::U24(c.clone().0.unwrap().to_string().parse().unwrap()),
                }
            }
            python_ast::ExprType::BinOp(bin) => {
                match bin.op {
                    BinOps::Add => { self.parse_add(bin.clone())? }
                    BinOps::Sub => todo!(),
                    BinOps::Mult => todo!(),
                    BinOps::Div => todo!(),
                    BinOps::FloorDiv => todo!(),
                    BinOps::Mod => todo!(),
                    BinOps::Pow => todo!(),
                    BinOps::LShift => todo!(),
                    BinOps::RShift => todo!(),
                    BinOps::BitOr => todo!(),
                    BinOps::BitXor => todo!(),
                    BinOps::BitAnd => todo!(),
                    BinOps::MatMult => todo!(),
                    BinOps::Unknown => todo!(),
                }
            }
            _ => { panic!("Could not get assignment value.") }
        };

        Some(value)
    }

    fn parse_part(&mut self, stmt: Statement) -> Option<Stmt> {
        self.index += 1;
        match stmt.statement {
            python_ast::StatementType::Assign(assign) => {
                let value = self.parse_assign(&assign).unwrap();
                Some(Stmt::Assign {
                    pat: imp::AssignPattern::Var(
                        Name::new(assign.targets.get(0).unwrap().id.clone())
                    ),
                    val: Box::new(value),
                    nxt: Some(
                        Box::new(self.parse_part(self.statements.get(self.index).unwrap().clone())?)
                    ),
                })
            }
            python_ast::StatementType::Call(call) => {
                let expr_type = *call.func;

                match expr_type {
                    python_ast::ExprType::BoolOp(_) => todo!(),
                    python_ast::ExprType::NamedExpr(_) => todo!(),
                    python_ast::ExprType::BinOp(_) => todo!(),
                    python_ast::ExprType::UnaryOp(_) => todo!(),
                    python_ast::ExprType::Await(_) => todo!(),
                    python_ast::ExprType::Compare(_) => todo!(),
                    python_ast::ExprType::Call(_) => todo!(),
                    python_ast::ExprType::Constant(_) => todo!(),
                    python_ast::ExprType::Attribute(_) => todo!(),
                    python_ast::ExprType::Name(_) => todo!(),
                    python_ast::ExprType::List(_) => todo!(),
                    python_ast::ExprType::NoneType(_) => todo!(),
                    python_ast::ExprType::Unimplemented(_) => todo!(),
                    python_ast::ExprType::Unknown => todo!(),
                }
            }
            python_ast::StatementType::Return(r) => {
                match r {
                    Some(val) => {
                        let term = self.parse_expr_type(Box::new(val.value)).unwrap();

                        Some(Stmt::Return { term: Box::new(term) })
                    }
                    None => None,
                }
            }
            _ => { None }
        }
    }

    pub fn parse(&mut self) -> String {
        // TODO: Statements can have another statments inside of them
        // Make parsing recursive

        let expr = self.parse_part(self.statements.get(self.index).unwrap().clone());

        let sum_nums = imp::Definition {
            name: Name::new("sum_nums"),
            params: vec![Name::new("a"), Name::new("b")],
            body: expr.unwrap(),
        };
        let fun_def = sum_nums.to_fun(false).unwrap();
        self.book.defs.insert(Name::new("sum_nums"), fun_def);

        let main_def = imp::Definition {
            name: Name::new("main"),
            params: vec![],
            body: Stmt::Return {
                term: Box::new(Expr::Call {
                    fun: Box::new(Expr::Var { nam: Name::new("sum_nums") }),
                    args: vec![
                        Expr::Num { val: bend::fun::Num::U24(2) },
                        Expr::Num { val: bend::fun::Num::U24(3) }
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
