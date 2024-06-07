use std::marker::PhantomData;

use dashu::{base::Sign, float::DBig};

use super::{error::RunError, Value};

pub struct OpManager<Data: Clone> {
    _data: PhantomData<*const Data>,
}

impl<Data: Clone> Default for OpManager<Data> {
    fn default() -> Self {
        Self {
            _data: Default::default(),
        }
    }
}

impl<Data: Clone> OpManager<Data> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn not(&self, v: Value<Data>, data: &Data) -> Result<Value<Data>, RunError<Data>> {
        match v {
            Value::Bool(v) => Ok(Value::Bool(!v)),
            _ => Err(RunError::InvalidUnary {
                op: format!("!"),
                vtype: v.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn neg(&self, v: Value<Data>, data: &Data) -> Result<Value<Data>, RunError<Data>> {
        match v {
            Value::Int(v) => Ok(Value::Int(-v)),
            Value::Float(v) => Ok(Value::Float(-v)),
            _ => Err(RunError::InvalidUnary {
                op: format!("-"),
                vtype: v.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn add(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1 + v2)),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Float(v1 + v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Float(v1 + v2)),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1 + v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Ok(Value::String(format!("{v1}{v2}"))),
            (Value::String(v1), Value::Bool(v2)) => Ok(Value::String(format!("{v1}{v2}"))),
            (Value::String(v1), Value::Int(v2)) => Ok(Value::String(format!("{v1}{v2}"))),
            (Value::String(v1), Value::Float(v2)) => Ok(Value::String(format!("{v1}{v2}"))),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("+"),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn sub(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1 - v2)),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Float(v1 - v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Float(v1 - v2)),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1 - v2)),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("-"),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn mul(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1 * v2)),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Float(v1 * v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Float(v1 * v2)),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1 * v2)),

            // STRING
            (Value::String(v1), Value::Bool(v2)) => match v2 {
                false => Ok(Value::String("".into())),
                true => Ok(Value::String(v1)),
            },
            (Value::String(v1), Value::Int(v2)) => {
                let (sign, ubig) = v2.into_parts();
                if let Sign::Negative = sign {
                    return Ok(Value::String("".into()));
                }
                match TryInto::<isize>::try_into(ubig).map(|i| i as usize) {
                    Ok(count) => Ok(Value::String(v1.repeat(count))),
                    Err(_) => Err(RunError::StringAllocError { data: data.clone() }),
                }
            }

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("*"),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn div(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Float(DBig::from(v1) / v2)),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Float(v1 / v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Float(v1 / v2)),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1 / v2)),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("/"),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn modulo(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1 % v2)),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Float(DBig::from(v1) % v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Float(v1 % DBig::from(v2))),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1 % v2)),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("%"),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn pow(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Float(DBig::from(v1).powi(v2))),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Float(DBig::from(v1).powf(&v2))),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Float(v1.powi(v2))),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1.powf(&v2))),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("**"),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn eq(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Bool(v1 == v2)),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Bool(DBig::from(v1) == v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Bool(v1 == DBig::from(v2))),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Bool(v1 == v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Ok(Value::Bool(v1 == v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Bool(v1 == v2)),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("=="),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn lt(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Bool(v1 < v2)),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Bool(DBig::from(v1) < v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Bool(v1 < DBig::from(v2))),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Bool(v1 < v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Ok(Value::Bool(v1 < v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Bool(v1 < v2)),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("<"),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn gt(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Bool(v1 > v2)),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Bool(DBig::from(v1) > v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Bool(v1 > DBig::from(v2))),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Bool(v1 > v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Ok(Value::Bool(v1 > v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Bool(v1 > v2)),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!(">"),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn neq(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Bool(v1 != v2)),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Bool(DBig::from(v1) != v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Bool(v1 != DBig::from(v2))),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Bool(v1 != v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Ok(Value::Bool(v1 != v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Bool(v1 != v2)),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("!="),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn lteq(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Bool(v1 <= v2)),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Bool(DBig::from(v1) <= v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Bool(v1 <= DBig::from(v2))),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Bool(v1 <= v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Ok(Value::Bool(v1 <= v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Bool(v1 <= v2)),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("<="),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn gteq(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Bool(v1 >= v2)),
            (Value::Int(v1), Value::Float(v2)) => Ok(Value::Bool(DBig::from(v1) >= v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Ok(Value::Bool(v1 >= DBig::from(v2))),
            (Value::Float(v1), Value::Float(v2)) => Ok(Value::Bool(v1 >= v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Ok(Value::Bool(v1 >= v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Bool(v1 >= v2)),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!(">="),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn and(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Bool(v1 && v2)),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("and"),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }

    pub fn or(
        &self,
        v1: Value<Data>,
        v2: Value<Data>,
        data: &Data,
    ) -> Result<Value<Data>, RunError<Data>> {
        match (v1, v2) {
            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Bool(v1 || v2)),

            // FAIL
            (v1, v2) => Err(RunError::InvalidBinary {
                op: format!("or"),
                vtype1: v1.type_name(),
                vtype2: v2.type_name(),
                data: data.clone(),
            }),
        }
    }
}
