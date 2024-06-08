use std::{mem::replace, ops::Deref};

use crate::parser::ast::{init::InitStyle, Expr, Node, Statement};

use super::{error::RunError, load_builtins, scope::Scope, value::ValueType, OpManager, Value};

enum GlobalValue<Data> {
    Static(Value<Data>),
    Const(Value<Data>),
}

impl<Data> GlobalValue<Data> {
    pub fn value(&self) -> &Value<Data> {
        match self {
            Self::Const(value) | Self::Static(value) => value,
        }
    }
}

pub enum SetError {
    DoesNotExist,
    Const,
}

pub struct Engine<Data: Clone> {
    ops: OpManager<Data>,
    globals: Scope<GlobalValue<Data>>,
    locals: Scope<Value<Data>>,
}

impl<Data: Clone> Default for Engine<Data> {
    fn default() -> Self {
        let mut engine = Self::empty();
        load_builtins(&mut engine);
        engine
    }
}

impl<Data: Clone> Engine<Data> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty() -> Self {
        Self {
            ops: Default::default(),
            globals: Default::default(),
            locals: Default::default(),
        }
    }

    pub fn push_scope(&mut self) {
        self.locals.push_scope();
        self.globals.push_scope();
    }

    pub fn pop_scope(&mut self) {
        self.locals.pop_scope();
        self.globals.pop_scope();
    }

    pub fn stash_scope(&mut self) {
        self.locals.stash();
        self.globals.push_scope();
    }

    pub fn unstash_scope(&mut self) {
        self.locals.unstash();
        self.globals.pop_scope();
    }

    pub fn get_value(&self, ident: impl AsRef<str>) -> Option<&Value<Data>> {
        match self.locals.get(ident.as_ref()) {
            None => Some(self.globals.get(ident)?.value()),
            Some(value) => Some(value),
        }
    }

    pub fn set_value(
        &mut self,
        ident: impl AsRef<str>,
        value: Value<Data>,
    ) -> Result<Value<Data>, SetError> {
        match self.locals.get_mut(ident.as_ref()) {
            Some(old_value) => Ok(replace(old_value, value)),
            None => match self.globals.get_mut(ident) {
                Some(GlobalValue::Static(old_value)) => Ok(replace(old_value, value)),
                Some(GlobalValue::Const(_)) => Err(SetError::Const),
                None => Err(SetError::DoesNotExist),
            },
        }
    }

    pub fn init_value(&mut self, ident: impl Into<String>, value: Value<Data>) {
        self.locals.init(ident, value)
    }

    pub fn init_static(&mut self, ident: impl Into<String>, value: Value<Data>) {
        self.init_global(ident, GlobalValue::Static(value))
    }

    pub fn init_const(&mut self, ident: impl Into<String>, value: Value<Data>) {
        self.init_global(ident, GlobalValue::Const(value))
    }

    fn init_global(&mut self, ident: impl Into<String>, value: GlobalValue<Data>) {
        self.globals.init(ident, value)
    }

    pub fn eval_statement(
        &mut self,
        statement: &Node<Data, Statement<Data>>,
    ) -> Result<Value<Data>, RunError<Data>> {
        match statement.deref() {
            Statement::Expr(expr) => self.eval(expr),
            Statement::Init(init) => {
                let value = self.eval(&init.expr)?;
                let ident = init.ident.deref().clone();
                match init.style.deref() {
                    InitStyle::Let => self.init_value(ident, value),
                    InitStyle::Static => self.init_static(ident, value),
                    InitStyle::Const => self.init_const(ident, value),
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
            Expr::Call(_ident, _params) => todo!("implement function calls"),
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
                    expected: ValueType::Bool,
                    found: value.get_type(),
                    data: cond.data().clone(),
                }),
            },
            Expr::Assign(ident, rhs) => {
                let new_value = self.eval(rhs)?;
                match self.set_value(ident.deref(), new_value) {
                    Ok(_old_value) => Ok(Value::None), // return nothing
                    Err(SetError::Const) => Err(RunError::ConstAssignment {
                        data: expr.data().clone(),
                    }),
                    Err(SetError::DoesNotExist) => Err(RunError::UnknownVariable {
                        ident: ident.deref().clone(),
                        data: ident.data().clone(),
                    }),
                }
            }
            Expr::Walrus(ident, rhs) => {
                let new_value = self.eval(rhs)?;
                match self.set_value(ident.deref(), new_value.clone()) {
                    Ok(_old_value) => Ok(new_value), // return newly created value
                    Err(SetError::Const) => Err(RunError::ConstAssignment {
                        data: expr.data().clone(),
                    }),
                    Err(SetError::DoesNotExist) => Err(RunError::UnknownVariable {
                        ident: ident.deref().clone(),
                        data: ident.data().clone(),
                    }),
                }
            }
        }
    }
}
