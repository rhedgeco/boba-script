use std::marker::PhantomData;

use dashu::base::Sign;

use super::Value;

pub struct OpManager<Source> {
    _source: PhantomData<*const Source>,
}

impl<Source> Default for OpManager<Source> {
    fn default() -> Self {
        Self {
            _source: Default::default(),
        }
    }
}

impl<Source> OpManager<Source> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pos(&self, v: &Value<Source>) -> Option<Value<Source>> {
        match v {
            Value::Int(v) => Some(Value::Int(v.clone())),
            Value::Float(v) => Some(Value::Float(v.clone())),
            _ => None,
        }
    }

    pub fn neg(&self, v: &Value<Source>) -> Option<Value<Source>> {
        match v {
            Value::Int(v) => Some(Value::Int(-v)),
            Value::Float(v) => Some(Value::Float(-v)),
            _ => None,
        }
    }

    pub fn not(&self, v: &Value<Source>) -> Option<Value<Source>> {
        match v {
            Value::Bool(v) => Some(Value::Bool(!v)),
            _ => None,
        }
    }

    pub fn add(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Some(Value::Int(v1 + v2)),
            (Value::Int(v1), Value::Float(v2)) => Some(Value::Float(v1.to_f64().value() + v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Float(v1 + v2.to_f64().value())),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Float(v1 + v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Some(Value::String(format!("{v1}{v2}"))),
            (Value::String(v1), Value::Bool(v2)) => Some(Value::String(format!("{v1}{v2}"))),
            (Value::String(v1), Value::Int(v2)) => Some(Value::String(format!("{v1}{v2}"))),
            (Value::String(v1), Value::Float(v2)) => Some(Value::String(format!("{v1}{v2}"))),

            // FAIL
            _ => None,
        }
    }

    pub fn sub(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Some(Value::Int(v1 - v2)),
            (Value::Int(v1), Value::Float(v2)) => Some(Value::Float(v1.to_f64().value() - v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Float(v1 - v2.to_f64().value())),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Float(v1 - v2)),

            // FAIL
            _ => None,
        }
    }

    pub fn mul(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Some(Value::Int(v1 * v2)),
            (Value::Int(v1), Value::Float(v2)) => Some(Value::Float(v1.to_f64().value() * v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Float(v1 * v2.to_f64().value())),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Float(v1 * v2)),

            // STRING
            (Value::String(v1), Value::Bool(v2)) => match v2 {
                false => Some(Value::String("".into())),
                true => Some(Value::String(v1.clone())),
            },
            (Value::String(v1), Value::Int(v2)) => {
                let (sign, ubig) = v2.clone().into_parts();
                if let Sign::Negative = sign {
                    return Some(Value::String("".into()));
                }

                let count = match TryInto::<usize>::try_into(ubig) {
                    Ok(count) => count,
                    Err(_) => usize::MAX,
                };

                Some(Value::String(v1.repeat(count)))
            }

            // FAIL
            _ => None,
        }
    }

    pub fn div(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => {
                Some(Value::Float(v1.to_f64().value() / v2.to_f64().value()))
            }
            (Value::Int(v1), Value::Float(v2)) => Some(Value::Float(v1.to_f64().value() / v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Float(v1 / v2.to_f64().value())),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Float(v1 / v2)),

            // FAIL
            _ => None,
        }
    }

    pub fn modulo(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Some(Value::Int(v1 % v2)),
            (Value::Int(v1), Value::Float(v2)) => Some(Value::Float(v1.to_f64().value() % v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Float(v1 % v2.to_f64().value())),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Float(v1 % v2)),

            // FAIL
            _ => None,
        }
    }

    pub fn pow(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Some(Value::Float(
                v1.to_f64().value_ref().powf(v2.to_f64().value()),
            )),
            (Value::Int(v1), Value::Float(v2)) => {
                Some(Value::Float(v1.to_f64().value_ref().powf(*v2)))
            }

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Float(v1.powf(v2.to_f64().value()))),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Float(v1.powf(*v2))),

            // FAIL
            _ => None,
        }
    }

    pub fn eq(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Some(Value::Bool(v1 == v2)),
            (Value::Int(v1), Value::Float(v2)) => Some(Value::Bool(v1.to_f64().value_ref() == v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Bool(v1 == v2.to_f64().value_ref())),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Bool(v1 == v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Some(Value::Bool(v1 == v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Some(Value::Bool(v1 == v2)),

            // FAIL
            _ => None,
        }
    }

    pub fn lt(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Some(Value::Bool(v1 < v2)),
            (Value::Int(v1), Value::Float(v2)) => Some(Value::Bool(v1.to_f64().value_ref() < v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Bool(v1 < v2.to_f64().value_ref())),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Bool(v1 < v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Some(Value::Bool(v1 < v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Some(Value::Bool(v1 < v2)),

            // FAIL
            _ => None,
        }
    }

    pub fn gt(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Some(Value::Bool(v1 > v2)),
            (Value::Int(v1), Value::Float(v2)) => Some(Value::Bool(v1.to_f64().value_ref() > v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Bool(v1 > v2.to_f64().value_ref())),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Bool(v1 > v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Some(Value::Bool(v1 > v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Some(Value::Bool(v1 > v2)),

            // FAIL
            _ => None,
        }
    }

    pub fn neq(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Some(Value::Bool(v1 != v2)),
            (Value::Int(v1), Value::Float(v2)) => Some(Value::Bool(v1.to_f64().value_ref() != v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Bool(v1 != v2.to_f64().value_ref())),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Bool(v1 != v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Some(Value::Bool(v1 != v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Some(Value::Bool(v1 != v2)),

            // FAIL
            _ => None,
        }
    }

    pub fn lteq(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Some(Value::Bool(v1 <= v2)),
            (Value::Int(v1), Value::Float(v2)) => Some(Value::Bool(v1.to_f64().value_ref() <= v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Bool(v1 <= v2.to_f64().value_ref())),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Bool(v1 <= v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Some(Value::Bool(v1 <= v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Some(Value::Bool(v1 <= v2)),

            // FAIL
            _ => None,
        }
    }

    pub fn gteq(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // INT
            (Value::Int(v1), Value::Int(v2)) => Some(Value::Bool(v1 >= v2)),
            (Value::Int(v1), Value::Float(v2)) => Some(Value::Bool(v1.to_f64().value_ref() >= v2)),

            // FLOAT
            (Value::Float(v1), Value::Int(v2)) => Some(Value::Bool(v1 >= v2.to_f64().value_ref())),
            (Value::Float(v1), Value::Float(v2)) => Some(Value::Bool(v1 >= v2)),

            // STRING
            (Value::String(v1), Value::String(v2)) => Some(Value::Bool(v1 >= v2)),

            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Some(Value::Bool(v1 >= v2)),

            // FAIL
            _ => None,
        }
    }

    pub fn and(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Some(Value::Bool(*v1 && *v2)),

            // FAIL
            _ => None,
        }
    }

    pub fn or(&self, v1: &Value<Source>, v2: &Value<Source>) -> Option<Value<Source>> {
        match (v1, v2) {
            // BOOLEAN
            (Value::Bool(v1), Value::Bool(v2)) => Some(Value::Bool(*v1 || *v2)),

            // FAIL
            _ => None,
        }
    }
}
