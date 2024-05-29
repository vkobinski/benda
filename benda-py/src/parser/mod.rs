use core::panic;

use bend::{
    diagnostics::DiagnosticsConfig,
    fun::{ Book, Definition, Name, Pattern, Rule, Term },
    CompileOpts,
};
use python_ast::{ Assign, BinOp, BinOps, ExprType, Statement };

use crate::benda_ffi::run;

fn parse_expr_type(expr: Box<ExprType>) -> Option<Term> {
    match *expr {
        ExprType::BoolOp(_) => todo!(),
        ExprType::NamedExpr(_) => todo!(),
        ExprType::BinOp(_) => todo!(),
        ExprType::UnaryOp(_) => todo!(),
        ExprType::Await(_) => todo!(),
        ExprType::Compare(_) => todo!(),
        ExprType::Call(_) => todo!(),
        ExprType::Constant(c) => Some(Term::Nat { val: c.0.unwrap().to_string().parse().unwrap() }),
        ExprType::Attribute(_) => todo!(),
        ExprType::Name(n) => { Some(Term::var_or_era(Some(Name::new(n.id)))) }
        ExprType::List(_) => todo!(),
        ExprType::NoneType(_) => todo!(),
        ExprType::Unimplemented(_) => todo!(),
        ExprType::Unknown => todo!(),
    }
}

fn parse_add(target: &String, bin: BinOp) -> Option<Rule> {
    // TODO: Treat case where expr type returns None
    let left = parse_expr_type(bin.left).unwrap();
    let right = parse_expr_type(bin.right).unwrap();

    Some(Rule {
        pats: vec![Pattern::Var(Some(Name::new(target)))],
        body: Term::Oper { opr: bend::fun::Op::ADD, fst: Box::new(left), snd: Box::new(right) },
    })
}

pub struct Parser {
    statements: Vec<Statement>,
    book: Book,
    rules: Vec<Rule>,
    vars: Vec<String>,
}

impl Parser {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self {
            statements,
            book: Book::default(),
            rules: vec![],
            vars: vec![],
        }
    }

    fn parse_assign(&mut self, assign: &Assign) -> Option<Rule> {
        // TODO: Implement tuple assignment
        let target = &assign.targets.get(0).unwrap().id;

        let mut is_let = false;

        if !self.vars.contains(target) {
            is_let = true;
        }

        let value: Rule = match &assign.value {
            python_ast::ExprType::Constant(c) => {
                match is_let {
                    true => {
                        let pattern = Pattern::Var(Some(Name::new(target)));
                        Rule {
                            pats: vec![],
                            body: Term::Let {
                                pat: Box::new(pattern),
                                val: Box::new(Term::Nat { val: c.to_string().parse().unwrap() }),
                                nxt: Box::new(Term::Var { nam: Name::new(target) }),
                            },
                        }
                    }
                    false => {
                        Rule {
                            pats: vec![Pattern::Var(Some(Name::new(target)))],
                            body: Term::Nat { val: c.to_string().parse().unwrap() },
                        }
                    }
                }
            }
            python_ast::ExprType::BinOp(bin) => {
                match bin.op {
                    BinOps::Add => { parse_add(target, bin.clone())? }
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

    fn parse_part(&mut self, stmt: Statement) -> Option<Rule> {
        match stmt.statement {
            python_ast::StatementType::Assign(assign) => {
                let value = self.parse_assign(&assign).unwrap();
                Some(value)
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
                        let term = parse_expr_type(Box::new(val.value)).unwrap();

                        Some(Rule { body: term, pats: vec![] })
                    }
                    None => None,
                }
            }
            _ => { None }
        }
    }

    pub fn parse(&mut self) {

        for stmt in self.statements.clone() {
            // TODO: Statements can have another statments inside of them
            // Make parsing recursive

            let rule = self.parse_part(stmt);

            match rule {
                Some(r) => self.rules.push(r),
                None => {}
            }
        }

        self.book.defs.insert(Name::new("sum_nums"), Definition {
            name: Name::new("sum_nums"),
            rules: self.rules.clone(),
            builtin: false,
        });

        self.book.defs.insert(Name::new("main"), Definition {
            name: Name::new("main"),
            rules: vec![Rule {
                body: Term::call(
                    Term::Var { nam: Name::new("sum_nums") },
                    vec![]
                ),
                pats: vec![],
            }],
            builtin: false,
        });

        println!("BEND:\n {}", self.book.display_pretty());

        run(&self.book);
    }
}
