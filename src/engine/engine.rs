use std::ops::Deref;

use crate::parser::ast::{Expr, Node, Statement};

use super::{error::RunError, load_builtins, scope::Scope, FuncValue, OpManager, Value};

pub struct Engine<Data: Clone> {
    ops: OpManager<Data>,
    funcs: Scope<FuncValue<Data>>,
    locals: Scope<Value<Data>>,
}

impl<Data: Clone> Default for Engine<Data> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Data: Clone> Engine<Data> {
    pub fn new() -> Self {
        let mut engine = Self::empty();
        load_builtins(&mut engine);
        engine
    }

    pub fn empty() -> Self {
        Self {
            ops: Default::default(),
            funcs: Default::default(),
            locals: Default::default(),
        }
    }

    pub fn push_scope(&mut self) {
        self.locals.push_scope();
        self.funcs.push_scope();
    }

    pub fn pop_scope(&mut self) {
        self.locals.pop_scope();
        self.funcs.pop_scope();
    }

    pub fn stash_scope(&mut self) {
        self.locals.stash();
        self.funcs.push_scope();
    }

    pub fn unstash_scope(&mut self) {
        self.locals.unstash();
        self.funcs.pop_scope();
    }

    pub fn get_value(&self, ident: impl AsRef<str>) -> Option<&Value<Data>> {
        self.locals.get(ident)
    }

    pub fn set_value(&mut self, ident: impl AsRef<str>, value: Value<Data>) -> Option<Value<Data>> {
        self.locals.set(ident, value)
    }

    pub fn init_value(&mut self, ident: impl Into<String>, value: Value<Data>) {
        self.locals.init(ident, value)
    }

    pub fn init_func(&mut self, func: FuncValue<Data>) {
        self.funcs.init(func.ident().to_string(), func)
    }

    pub fn call_fn(
        &mut self,
        ident: &Node<Data, String>,
        params: &Vec<Node<Data, Expr<Data>>>,
    ) -> Result<Value<Data>, RunError<Data>> {
        // get the function
        let func = match self.funcs.get(ident.deref()) {
            Some(func) => func,
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
                    self.init_value(ident.deref(), value);
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
                self.init_value(ident.deref(), value);
                Ok(Value::None)
            }
            Statement::Assign(ident, expr) => {
                let value = self.eval(expr)?;
                match self.set_value(ident.deref(), value.clone()) {
                    Some(_old_value) => Ok(Value::None),
                    None => Err(RunError::UnknownVariable {
                        ident: ident.deref().clone(),
                        data: ident.data().clone(),
                    }),
                }
            }
            Statement::Func(func) => {
                self.init_func(FuncValue::custom(func.deref().clone()));
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
                self.ops.neg(inner, expr.data())
            }
            Expr::Not(inner) => {
                let inner = self.eval(inner)?;
                self.ops.not(inner, expr.data())
            }
            Expr::Add(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.add(lhs, rhs, expr.data())
            }
            Expr::Sub(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.sub(lhs, rhs, expr.data())
            }
            Expr::Mul(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.mul(lhs, rhs, expr.data())
            }
            Expr::Div(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.div(lhs, rhs, expr.data())
            }
            Expr::Pow(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.pow(lhs, rhs, expr.data())
            }
            Expr::Mod(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.modulo(lhs, rhs, expr.data())
            }
            Expr::Eq(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.eq(lhs, rhs, expr.data())
            }
            Expr::Lt(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.lt(lhs, rhs, expr.data())
            }
            Expr::Gt(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.gt(lhs, rhs, expr.data())
            }
            Expr::NEq(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.neq(lhs, rhs, expr.data())
            }
            Expr::LtEq(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.lteq(lhs, rhs, expr.data())
            }
            Expr::GtEq(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.gteq(lhs, rhs, expr.data())
            }
            Expr::And(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.and(lhs, rhs, expr.data())
            }
            Expr::Or(lhs, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                self.ops.or(lhs, rhs, expr.data())
            }
            Expr::Var(ident) => match self.get_value(ident.deref()) {
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
                let new_value = self.eval(expr)?;
                match self.set_value(ident.deref(), new_value.clone()) {
                    Some(_old_value) => Ok(new_value), // return the new value
                    None => Err(RunError::UnknownVariable {
                        ident: ident.deref().clone(),
                        data: ident.data().clone(),
                    }),
                }
            }
        }
    }
}
