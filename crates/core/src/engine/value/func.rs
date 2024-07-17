use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    ops::Deref,
    rc::Rc,
};

use derive_more::Display;

use crate::{ast::func::Func, engine::EvalError, Engine};

use super::Value;

enum FuncDef<Source> {
    Native(NativeFunc<Source>),
    Custom(Func<Source>),
}

impl<Source: Debug> Debug for FuncDef<Source> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(arg0) => f.debug_tuple("Native").field(arg0).finish(),
            Self::Custom(arg0) => f.debug_tuple("Custom").field(arg0).finish(),
        }
    }
}

impl<Source: Clone> Clone for FuncDef<Source> {
    fn clone(&self) -> Self {
        match self {
            Self::Native(arg0) => Self::Native(arg0.clone()),
            Self::Custom(arg0) => Self::Custom(arg0.clone()),
        }
    }
}

impl<Source: PartialEq> PartialEq for FuncDef<Source> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Native(l0), Self::Native(r0)) => l0 == r0,
            (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
            _ => false,
        }
    }
}

pub struct FuncPtr<Source> {
    def: Rc<FuncDef<Source>>,
}

impl<Source: Debug> Debug for FuncPtr<Source> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FuncPtr").field("def", &self.def).finish()
    }
}

impl<Source> Clone for FuncPtr<Source> {
    fn clone(&self) -> Self {
        Self {
            def: self.def.clone(),
        }
    }
}

impl<Source: PartialEq> PartialEq for FuncPtr<Source> {
    fn eq(&self, other: &Self) -> bool {
        self.def == other.def
    }
}

impl<Source> fmt::Display for FuncPtr<Source> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "fn()")
    }
}

impl<Source> FuncPtr<Source> {
    pub fn params(&self) -> usize {
        match self.def.deref() {
            FuncDef::Native(native) => native.params(),
            FuncDef::Custom(custom) => custom.params.len(),
        }
    }
}

impl<Source> FuncPtr<Source> {
    pub fn native(
        params: usize,
        native: fn(Vec<Value<Source>>) -> Result<Value<Source>, String>,
    ) -> Self {
        let native = NativeFunc {
            params,
            native,
            _source: PhantomData,
        };

        FuncPtr {
            def: Rc::new(FuncDef::Native(native)),
        }
    }

    pub fn custom(func: Func<Source>) -> Self {
        Self {
            def: Rc::new(FuncDef::Custom(func)),
        }
    }

    pub fn kind(&self) -> FuncKind {
        FuncKind {
            params: self.params(),
        }
    }
}

impl<Source: Clone> FuncPtr<Source> {
    pub fn call(
        &self,
        call_source: &Source,
        values: Vec<Value<Source>>,
        engine: &mut Engine<Source>,
    ) -> Result<Value<Source>, EvalError<Source>> {
        match self.def.deref() {
            FuncDef::Native(native) => native.call(call_source, values),
            FuncDef::Custom(custom) => {
                if custom.params.len() != values.len() {
                    return Err(EvalError::InvalidParameters {
                        found: values.len(),
                        expect: custom.params.len(),
                        source: call_source.clone(),
                    });
                }

                engine.vars_mut().stash();
                for (name, value) in custom.params.iter().zip(values.into_iter()) {
                    engine.vars_mut().init_local(name, value);
                }

                let mut output = Value::None;
                for statement in custom.body.iter() {
                    output = match engine.eval(statement) {
                        Ok(value) => value,
                        Err(error) => {
                            engine.vars_mut().unstash();
                            return Err(error);
                        }
                    };
                }

                engine.vars_mut().unstash();
                Ok(output)
            }
        }
    }
}

#[derive(Debug, Display, Clone, PartialEq)]
#[display(fmt = "fn({})", params)]
pub struct FuncKind {
    params: usize,
}

impl FuncKind {
    pub fn new(params: usize) -> Self {
        Self { params }
    }
}

struct NativeFunc<Source> {
    params: usize,
    native: fn(Vec<Value<Source>>) -> Result<Value<Source>, String>,
    _source: PhantomData<*const Source>,
}

impl<Source> Debug for NativeFunc<Source> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeFunc")
            .field("params", &self.params)
            .field("native", &self.native)
            .field("_source", &self._source)
            .finish()
    }
}

impl<Source> Clone for NativeFunc<Source> {
    fn clone(&self) -> Self {
        Self {
            params: self.params.clone(),
            native: self.native.clone(),
            _source: self._source.clone(),
        }
    }
}

impl<Source> PartialEq for NativeFunc<Source> {
    fn eq(&self, other: &Self) -> bool {
        self.params == other.params && self.native == other.native && self._source == other._source
    }
}

impl<Source> NativeFunc<Source> {
    pub fn params(&self) -> usize {
        self.params
    }
}

impl<Source: Clone> NativeFunc<Source> {
    pub fn call(
        &self,
        call_source: &Source,
        values: Vec<Value<Source>>,
    ) -> Result<Value<Source>, EvalError<Source>> {
        if values.len() != self.params {
            return Err(EvalError::InvalidParameters {
                found: values.len(),
                expect: self.params,
                source: call_source.clone(),
            });
        }

        match (self.native)(values) {
            Ok(value) => Ok(value),
            Err(message) => Err(EvalError::NativeCall {
                message,
                source: call_source.clone(),
            }),
        }
    }
}
