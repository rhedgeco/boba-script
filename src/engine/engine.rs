use std::{mem::replace, ops::Deref};

use dashu::{
    base::{RemEuclid, Sign},
    float::DBig,
};
use derive_more::Display;

use crate::parser::ast::{Expr, Node, Statement};

use super::{error::RunError, value::FuncValue, Value};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnaryOpType {
    #[display(fmt = "-")]
    Neg,
    #[display(fmt = "!")]
    Not,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinaryOpType {
    #[display(fmt = "+")]
    Add,
    #[display(fmt = "-")]
    Sub,
    #[display(fmt = "*")]
    Mul,
    #[display(fmt = "/")]
    Div,
    #[display(fmt = "%")]
    Mod,
    #[display(fmt = "**")]
    Pow,

    #[display(fmt = "==")]
    Eq,
    #[display(fmt = "<")]
    Lt,
    #[display(fmt = ">")]
    Gt,
    #[display(fmt = "!=")]
    NEq,
    #[display(fmt = "<=")]
    LtEq,
    #[display(fmt = ">=")]
    GtEq,
    #[display(fmt = "and")]
    And,
    #[display(fmt = "or")]
    Or,
}

enum GlobalValue<Data> {
    Static(Value<Data>),
    Const(Value<Data>),
}

pub enum SetError {
    Const,
    None,
}

pub struct Engine<Data> {
    globals: Vec<Vec<(String, GlobalValue<Data>)>>,
    locals: Vec<Vec<(String, Value<Data>)>>,
    stash: Vec<Vec<Vec<(String, Value<Data>)>>>,
}

impl<Data> Default for Engine<Data> {
    fn default() -> Self {
        Self {
            globals: Default::default(),
            locals: Default::default(),
            stash: Default::default(),
        }
    }
}

impl<Data> Engine<Data> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_scope(&mut self) {
        self.globals.push(Vec::new());
        self.locals.push(Vec::new());
    }

    pub fn pop_scope(&mut self) {
        self.globals.pop();
        self.locals.pop();
    }

    pub fn stash_scope(&mut self) {
        self.globals.push(Vec::new());
        let old_local = replace(&mut self.locals, Vec::new());
        self.stash.push(old_local);
    }

    pub fn unstash_scope(&mut self) {
        self.globals.pop();
        match self.stash.pop() {
            Some(stash) => self.locals = stash,
            None => self.locals = Vec::new(),
        }
    }

    pub fn init(&mut self, ident: impl Into<String>, value: Value<Data>) {
        let Some(locals) = self.locals.last_mut() else {
            self.locals.push(vec![(ident.into(), value)]);
            return;
        };

        locals.push((ident.into(), value));
    }

    pub fn init_const(&mut self, ident: impl Into<String>, value: Value<Data>) {
        self.init_global(ident.into(), GlobalValue::Const(value))
    }

    pub fn init_static(&mut self, ident: impl Into<String>, value: Value<Data>) {
        self.init_global(ident.into(), GlobalValue::Static(value))
    }

    fn init_global(&mut self, ident: String, value: GlobalValue<Data>) {
        let Some(globals) = self.globals.last_mut() else {
            self.globals.push(vec![(ident.into(), value)]);
            return;
        };

        globals.push((ident.into(), value));
    }

    pub fn set(
        &mut self,
        ident: impl AsRef<str>,
        value: Value<Data>,
    ) -> Result<Value<Data>, SetError> {
        let ident = ident.as_ref();
        for locals in self.locals.iter_mut().rev() {
            for (name, old_value) in locals.iter_mut().rev() {
                if name == ident {
                    return Ok(replace(old_value, value));
                }
            }
        }

        for globals in self.globals.iter_mut().rev() {
            for (name, old_value) in globals.iter_mut().rev() {
                if name == ident {
                    match old_value {
                        GlobalValue::Static(old_value) => return Ok(replace(old_value, value)),
                        GlobalValue::Const(_) => return Err(SetError::Const),
                    }
                }
            }
        }

        Err(SetError::None)
    }

    pub fn get(&self, ident: impl AsRef<str>) -> Option<&Value<Data>> {
        let ident = ident.as_ref();
        for locals in self.locals.iter().rev() {
            for (name, value) in locals.iter().rev() {
                if name == ident {
                    return Some(value);
                }
            }
        }

        for globals in self.globals.iter().rev() {
            for (name, value) in globals.iter().rev() {
                if name == ident {
                    match value {
                        GlobalValue::Static(value) | GlobalValue::Const(value) => {
                            return Some(value);
                        }
                    }
                }
            }
        }

        None
    }
}

impl<Data: Clone> Engine<Data> {
    pub fn call_fn(
        &mut self,
        ident: &Node<Data, String>,
        params: &Vec<Node<Data, Expr<Data>>>,
    ) -> Result<Value<Data>, RunError<Data>> {
        // get the function
        let func = match self.get(ident.deref()) {
            Some(Value::Func(func)) => func,
            Some(value) => {
                return Err(RunError::InvalidCall {
                    ident: ident.deref().clone(),
                    found: format!("'{}'", value.type_name()),
                    data: ident.data().clone(),
                })
            }
            None => {
                return Err(RunError::UnknownFunction {
                    ident: ident.deref().clone(),
                    data: ident.data().clone(),
                })
            }
        };

        // ensure parameter count matches
        if func.param_count() != params.len() {
            return Err(RunError::ParameterCount {
                expected: func.param_count(),
                found: params.len(),
                data: ident.data().clone(),
            });
        }

        // clone func
        let func = func.clone();

        // calculate all the values
        let mut values = Vec::with_capacity(params.len());
        for expr in params.iter() {
            values.push(self.eval(expr)?);
        }

        // calculate function
        let value = match func {
            FuncValue::Native(func) => match (func.native)(values) {
                Ok(value) => value,
                Err(message) => {
                    return Err(RunError::NativeCallError {
                        message,
                        data: ident.data().clone(),
                    });
                }
            },
            FuncValue::Custom(func) => {
                // stash scope
                self.stash_scope();

                // load all values into new scope
                for (ident, value) in func.params.iter().zip(values) {
                    self.init(ident.deref(), value);
                }

                // calculate all function statements
                let mut value = Value::None;
                for statement in func.body.iter() {
                    match self.eval_statement(statement) {
                        Ok(new_value) => value = new_value,
                        Err(e) => {
                            // ensure scope is unstashed
                            self.unstash_scope();
                            return Err(e);
                        }
                    }
                }

                // unstash scope
                self.unstash_scope();

                // return final value
                return Ok(value);
            }
        };

        // return final value
        Ok(value)
    }

    pub fn eval_statement(
        &mut self,
        statement: &Node<Data, Statement<Data>>,
    ) -> Result<Value<Data>, RunError<Data>> {
        match statement.deref() {
            Statement::Expr(expr) => self.eval(expr),
            Statement::LetAssign(ident, expr) => {
                let value = self.eval(expr)?;
                self.init(ident.deref(), value);
                Ok(Value::None)
            }
            Statement::Assign(ident, expr) => {
                let value = self.eval(expr)?;
                match self.set(ident.deref(), value.clone()) {
                    Ok(_) => Ok(Value::None),
                    Err(SetError::None) => Err(RunError::UnknownVariable {
                        ident: ident.deref().clone(),
                        data: ident.data().clone(),
                    }),
                    Err(SetError::Const) => Err(RunError::ConstAssign {
                        data: ident.data().clone(),
                    }),
                }
            }
            Statement::Func(func) => {
                self.init_const(
                    func.ident.deref(),
                    Value::Func(FuncValue::Custom(func.deref().clone())),
                );
                Ok(Value::None)
            }
            Statement::While(w) => {
                while match self.eval(&w.cond)? {
                    Value::Bool(true) => true,
                    Value::Bool(false) => false,
                    value => {
                        return Err(RunError::TypeMismatch {
                            expected: format!("boolean condition"),
                            found: format!("'{}'", value.type_name()),
                            data: w.cond.data().clone(),
                        })
                    }
                } {
                    for statement in w.body.iter() {
                        self.eval_statement(statement)?;
                    }
                }
                Ok(Value::None)
            }
        }
    }

    pub fn eval(&mut self, expr: &Node<Data, Expr<Data>>) -> Result<Value<Data>, RunError<Data>> {
        match expr.deref() {
            Expr::None => Ok(Value::None),
            Expr::Bool(v) => Ok(Value::Bool(v.clone())),
            Expr::Int(v) => Ok(Value::Int(v.clone())),
            Expr::Float(v) => Ok(Value::Float(v.clone())),
            Expr::String(v) => Ok(Value::String(v.clone())),
            Expr::Call(ident, params) => self.call_fn(ident, params),
            Expr::Neg(inner) => {
                let inner = self.eval(inner)?;
                self.eval_unary(UnaryOpType::Neg, inner, expr.data())
            }
            Expr::Not(inner) => {
                let inner = self.eval(inner)?;
                self.eval_unary(UnaryOpType::Not, inner, expr.data())
            }
            Expr::Add(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::Add, rhs, expr.data())
            }
            Expr::Sub(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::Sub, rhs, expr.data())
            }
            Expr::Mul(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::Mul, rhs, expr.data())
            }
            Expr::Div(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::Div, rhs, expr.data())
            }
            Expr::Pow(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::Pow, rhs, expr.data())
            }
            Expr::Mod(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::Mod, rhs, expr.data())
            }
            Expr::Eq(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::Eq, rhs, expr.data())
            }
            Expr::Lt(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::Lt, rhs, expr.data())
            }
            Expr::Gt(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::Gt, rhs, expr.data())
            }
            Expr::NEq(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::NEq, rhs, expr.data())
            }
            Expr::LtEq(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::LtEq, rhs, expr.data())
            }
            Expr::GtEq(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::GtEq, rhs, expr.data())
            }
            Expr::And(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::And, rhs, expr.data())
            }
            Expr::Or(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.eval_binary(lhs, BinaryOpType::Or, rhs, expr.data())
            }
            Expr::Var(ident) => match self.get(ident.deref()) {
                Some(value) => Ok(value.clone()),
                None => Err(RunError::UnknownVariable {
                    ident: ident.clone(),
                    data: expr.data().clone(),
                }),
            },
            Expr::Ternary(cond, lhs, rhs) => match self.eval(cond)? {
                Value::Bool(true) => self.eval(lhs),
                Value::Bool(false) => self.eval(rhs),
                value => Err(RunError::TypeMismatch {
                    expected: format!("boolean condition"),
                    found: format!("'{}'", value.type_name()),
                    data: cond.data().clone(),
                }),
            },
            Expr::Walrus(ident, expr) => {
                let value = self.eval(expr)?;
                match self.set(ident.deref(), value.clone()) {
                    Ok(_) => Ok(value), // return the new value
                    Err(SetError::None) => Err(RunError::UnknownVariable {
                        ident: ident.deref().clone(),
                        data: ident.data().clone(),
                    }),
                    Err(SetError::Const) => Err(RunError::ConstAssign {
                        data: ident.data().clone(),
                    }),
                }
            }
        }
    }

    fn eval_unary(
        &self,
        op: UnaryOpType,
        val: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        let vtype = val.type_name();
        match (val, op) {
            (Value::Bool(v), UnaryOpType::Not) => Ok(Value::Bool(!v)),
            (Value::Int(v), UnaryOpType::Neg) => Ok(Value::Int(-v)),
            (Value::Float(v), UnaryOpType::Neg) => Ok(Value::Float(-v)),
            _ => Err(RunError::InvalidUnary {
                op,
                vtype,
                data: data.clone(),
            }),
        }
    }
    fn eval_binary(
        &self,
        val1: Value<Data>,
        op: BinaryOpType,
        val2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        let vtype1 = val1.type_name();
        let vtype2 = val2.type_name();
        match (val1, op, val2) {
            // ---------------
            // --- INT OPS ---
            // int add
            (Value::Int(v1), BinaryOpType::Add, Value::Bool(v2)) => Ok(Value::Int(v1 + v2 as i64)),
            (Value::Int(v1), BinaryOpType::Add, Value::Int(v2)) => Ok(Value::Int(v1 + v2)),
            (Value::Int(v1), BinaryOpType::Add, Value::Float(v2)) => Ok(Value::Float(v1 + v2)),
            // int sub
            (Value::Int(v1), BinaryOpType::Sub, Value::Bool(v2)) => Ok(Value::Int(v1 - v2 as i64)),
            (Value::Int(v1), BinaryOpType::Sub, Value::Int(v2)) => Ok(Value::Int(v1 - v2)),
            (Value::Int(v1), BinaryOpType::Sub, Value::Float(v2)) => Ok(Value::Float(v1 - v2)),
            // int mul
            (Value::Int(v1), BinaryOpType::Mul, Value::Bool(v2)) => Ok(Value::Int(v1 * v2 as i64)),
            (Value::Int(v1), BinaryOpType::Mul, Value::Int(v2)) => Ok(Value::Int(v1 * v2)),
            (Value::Int(v1), BinaryOpType::Mul, Value::Float(v2)) => Ok(Value::Float(v1 * v2)),
            // int div
            (Value::Int(v1), BinaryOpType::Div, Value::Int(v2)) => {
                Ok(Value::Float(DBig::from(v1) / v2))
            }
            (Value::Int(v1), BinaryOpType::Div, Value::Float(v2)) => Ok(Value::Float(v1 / v2)),
            // int mod
            (Value::Int(v1), BinaryOpType::Mod, Value::Int(v2)) => {
                Ok(Value::Int(v1.rem_euclid(v2).into()))
            }
            (Value::Int(v1), BinaryOpType::Mod, Value::Float(v2)) => {
                Ok(Value::Float((DBig::from(v1)).rem_euclid(v2)))
            }
            // int pow
            (Value::Int(v1), BinaryOpType::Pow, Value::Int(v2)) => {
                Ok(Value::Float((DBig::from(v1)).powf(&DBig::from(v2))))
            }
            (Value::Int(v1), BinaryOpType::Pow, Value::Float(v2)) => {
                Ok(Value::Float((DBig::from(v1)).powf(&v2)))
            }
            // int equality
            (Value::Int(v1), BinaryOpType::Eq, Value::Int(v2)) => Ok(Value::Bool(v1 == v2)),
            (Value::Int(v1), BinaryOpType::Eq, Value::Float(v2)) => {
                Ok(Value::Bool(DBig::from(v1) == v2))
            }
            // int less than
            (Value::Int(v1), BinaryOpType::Lt, Value::Int(v2)) => Ok(Value::Bool(v1 < v2)),
            (Value::Int(v1), BinaryOpType::Lt, Value::Float(v2)) => {
                Ok(Value::Bool(DBig::from(v1) < v2))
            }
            // int greater than
            (Value::Int(v1), BinaryOpType::Gt, Value::Int(v2)) => Ok(Value::Bool(v1 > v2)),
            (Value::Int(v1), BinaryOpType::Gt, Value::Float(v2)) => {
                Ok(Value::Bool(DBig::from(v1) > v2))
            }
            // int not equal
            (Value::Int(v1), BinaryOpType::NEq, Value::Int(v2)) => Ok(Value::Bool(v1 != v2)),
            (Value::Int(v1), BinaryOpType::NEq, Value::Float(v2)) => {
                Ok(Value::Bool(DBig::from(v1) != v2))
            }
            // int less than equal
            (Value::Int(v1), BinaryOpType::LtEq, Value::Int(v2)) => Ok(Value::Bool(v1 <= v2)),
            (Value::Int(v1), BinaryOpType::LtEq, Value::Float(v2)) => {
                Ok(Value::Bool(DBig::from(v1) <= v2))
            }
            // int greater than equal
            (Value::Int(v1), BinaryOpType::GtEq, Value::Int(v2)) => Ok(Value::Bool(v1 >= v2)),
            (Value::Int(v1), BinaryOpType::GtEq, Value::Float(v2)) => {
                Ok(Value::Bool(DBig::from(v1) >= v2))
            }
            // -----------------
            // --- FLOAT OPS ---
            // float add
            (Value::Float(v1), BinaryOpType::Add, Value::Bool(v2)) => {
                Ok(Value::Float(v1 + v2 as u8))
            }
            (Value::Float(v1), BinaryOpType::Add, Value::Int(v2)) => Ok(Value::Float(v1 + v2)),
            (Value::Float(v1), BinaryOpType::Add, Value::Float(v2)) => Ok(Value::Float(v1 + v2)),
            // float sub
            (Value::Float(v1), BinaryOpType::Sub, Value::Bool(v2)) => {
                Ok(Value::Float(v1 - v2 as u8))
            }
            (Value::Float(v1), BinaryOpType::Sub, Value::Int(v2)) => Ok(Value::Float(v1 - v2)),
            (Value::Float(v1), BinaryOpType::Sub, Value::Float(v2)) => Ok(Value::Float(v1 - v2)),
            // float mul
            (Value::Float(v1), BinaryOpType::Mul, Value::Bool(v2)) => {
                Ok(Value::Float(v1 * v2 as u8))
            }
            (Value::Float(v1), BinaryOpType::Mul, Value::Int(v2)) => Ok(Value::Float(v1 * v2)),
            (Value::Float(v1), BinaryOpType::Mul, Value::Float(v2)) => Ok(Value::Float(v1 * v2)),
            // float div
            (Value::Float(v1), BinaryOpType::Div, Value::Int(v2)) => Ok(Value::Float(v1 / v2)),
            (Value::Float(v1), BinaryOpType::Div, Value::Float(v2)) => Ok(Value::Float(v1 / v2)),
            // float mod
            (Value::Float(v1), BinaryOpType::Mod, Value::Int(v2)) => {
                Ok(Value::Float(v1.rem_euclid(DBig::from(v2))))
            }
            (Value::Float(v1), BinaryOpType::Mod, Value::Float(v2)) => {
                Ok(Value::Float(v1.rem_euclid(v2)))
            }
            // float pow
            (Value::Float(v1), BinaryOpType::Pow, Value::Int(v2)) => {
                Ok(Value::Float(v1.powf(&DBig::from(v2))))
            }
            (Value::Float(v1), BinaryOpType::Pow, Value::Float(v2)) => {
                Ok(Value::Float(v1.powf(&v2)))
            }
            // float equality
            (Value::Float(v1), BinaryOpType::Eq, Value::Int(v2)) => {
                Ok(Value::Bool(v1 == DBig::from(v2)))
            }
            (Value::Float(v1), BinaryOpType::Eq, Value::Float(v2)) => Ok(Value::Bool(v1 == v2)),
            // float less than
            (Value::Float(v1), BinaryOpType::Lt, Value::Int(v2)) => {
                Ok(Value::Bool(v1 < DBig::from(v2)))
            }
            (Value::Float(v1), BinaryOpType::Lt, Value::Float(v2)) => Ok(Value::Bool(v1 < v2)),
            // float greater than
            (Value::Float(v1), BinaryOpType::Gt, Value::Int(v2)) => {
                Ok(Value::Bool(v1 > DBig::from(v2)))
            }
            (Value::Float(v1), BinaryOpType::Gt, Value::Float(v2)) => Ok(Value::Bool(v1 > v2)),
            // float not equal
            (Value::Float(v1), BinaryOpType::NEq, Value::Int(v2)) => {
                Ok(Value::Bool(v1 != DBig::from(v2)))
            }
            (Value::Float(v1), BinaryOpType::NEq, Value::Float(v2)) => Ok(Value::Bool(v1 != v2)),
            // float less than equal
            (Value::Float(v1), BinaryOpType::LtEq, Value::Int(v2)) => {
                Ok(Value::Bool(v1 <= DBig::from(v2)))
            }
            (Value::Float(v1), BinaryOpType::LtEq, Value::Float(v2)) => Ok(Value::Bool(v1 <= v2)),
            // float greater than equal
            (Value::Float(v1), BinaryOpType::GtEq, Value::Int(v2)) => {
                Ok(Value::Bool(v1 >= DBig::from(v2)))
            }
            (Value::Float(v1), BinaryOpType::GtEq, Value::Float(v2)) => Ok(Value::Bool(v1 >= v2)),
            // ------------------
            // --- STRING OPS ---
            // string add
            (Value::String(v1), BinaryOpType::Add, Value::Bool(v2)) => {
                Ok(Value::String(format!("{v1}{v2}")))
            }
            (Value::String(v1), BinaryOpType::Add, Value::Int(v2)) => {
                Ok(Value::String(format!("{v1}{v2}")))
            }
            (Value::String(v1), BinaryOpType::Add, Value::Float(v2)) => {
                Ok(Value::String(format!("{v1}{v2}")))
            }
            (Value::String(v1), BinaryOpType::Add, Value::String(v2)) => {
                Ok(Value::String(format!("{v1}{v2}")))
            }
            // string mul
            (Value::String(v1), BinaryOpType::Mul, Value::Bool(v2)) => match v2 {
                false => Ok(Value::String("".into())),
                true => Ok(Value::String(v1)),
            },
            (Value::String(v1), BinaryOpType::Mul, Value::Int(v2)) => {
                let (sign, ubig) = v2.into_parts();
                if let Sign::Negative = sign {
                    return Ok(Value::String("".into()));
                }
                match TryInto::<isize>::try_into(ubig).map(|i| i as usize) {
                    Ok(count) => Ok(Value::String(v1.repeat(count))),
                    Err(_) => Err(RunError::StringAllocError { data: data.clone() }),
                }
            }
            // string equality
            (Value::String(v1), BinaryOpType::Eq, Value::String(v2)) => Ok(Value::Bool(v1 == v2)),
            // string less than
            (Value::String(v1), BinaryOpType::Lt, Value::String(v2)) => Ok(Value::Bool(v1 < v2)),
            // string greater than
            (Value::String(v1), BinaryOpType::Gt, Value::String(v2)) => Ok(Value::Bool(v1 > v2)),
            // string not equal
            (Value::String(v1), BinaryOpType::NEq, Value::String(v2)) => Ok(Value::Bool(v1 != v2)),
            // string less than equal
            (Value::String(v1), BinaryOpType::LtEq, Value::String(v2)) => Ok(Value::Bool(v1 <= v2)),
            // string greater than equal
            (Value::String(v1), BinaryOpType::GtEq, Value::String(v2)) => Ok(Value::Bool(v1 >= v2)),
            // -------------------
            // --- BOOLEAN OPS ---
            // boolean equality
            (Value::Bool(v1), BinaryOpType::Eq, Value::Bool(v2)) => Ok(Value::Bool(v1 == v2)),
            // boolean less than
            (Value::Bool(v1), BinaryOpType::Lt, Value::Bool(v2)) => Ok(Value::Bool(v1 < v2)),
            // boolean greater than
            (Value::Bool(v1), BinaryOpType::Gt, Value::Bool(v2)) => Ok(Value::Bool(v1 > v2)),
            // boolean not equal
            (Value::Bool(v1), BinaryOpType::NEq, Value::Bool(v2)) => Ok(Value::Bool(v1 != v2)),
            // boolean less than equal
            (Value::Bool(v1), BinaryOpType::LtEq, Value::Bool(v2)) => Ok(Value::Bool(v1 <= v2)),
            // boolean greater than equal
            (Value::Bool(v1), BinaryOpType::GtEq, Value::Bool(v2)) => Ok(Value::Bool(v1 >= v2)),
            // boolean and
            (Value::Bool(v1), BinaryOpType::And, Value::Bool(v2)) => Ok(Value::Bool(v1 && v2)),
            // boolean or
            (Value::Bool(v1), BinaryOpType::Or, Value::Bool(v2)) => Ok(Value::Bool(v1 || v2)),
            // --------------------
            // --- FAILURE CASE ---
            _ => Err(RunError::InvalidBinary {
                op,
                vtype1,
                vtype2,
                data: data.clone(),
            }),
        }
    }
}
