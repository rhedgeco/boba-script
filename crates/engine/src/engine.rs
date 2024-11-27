use boba_script_program::{
    func::FuncCaller,
    indexers::FuncIndex,
    int::IBig,
    program::ValueKind,
    value::{ClassBox, ClassStore, StackValue, Store},
    Program,
};

pub struct Engine {
    program: Program,
    stack: Vec<StackValue>,
    store: ClassStore,
}

impl Engine {
    pub fn load(program: Program) -> Self {
        Self {
            program,
            stack: Vec::new(),
            store: ClassStore::empty(),
        }
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn store(&self) -> &dyn Store {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut dyn Store {
        &mut self.store
    }

    pub fn function(&mut self, index: FuncIndex) -> FuncCallBuilder {
        FuncCallBuilder {
            index,
            engine: self,
            inputs: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum FuncError {
    InvalidIndex,
    InvalidParams,
}

pub enum CallValue {
    None,
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
    Class(ClassBox),
}

pub struct FuncCallBuilder<'a> {
    index: FuncIndex,
    engine: &'a mut Engine,
    inputs: Vec<CallValue>,
}

impl<'a> FuncCallBuilder<'a> {
    pub fn with_param(mut self, param: CallValue) -> Self {
        self.inputs.push(param);
        self
    }

    pub fn call(self) -> Result<Option<StackValue>, FuncError> {
        // validate function index
        let Some(func) = self.engine.program.get_func(self.index) else {
            return Err(FuncError::InvalidIndex);
        };

        // validate function params
        for (index, input) in func.inputs().enumerate() {
            match self.inputs.get(index) {
                None => return Err(FuncError::InvalidParams),
                Some(value) => {
                    let value_kind = match value {
                        CallValue::None => ValueKind::None,
                        CallValue::Bool(_) => ValueKind::Bool,
                        CallValue::Int(_) => ValueKind::Int,
                        CallValue::Float(_) => ValueKind::Float,
                        CallValue::String(_) => ValueKind::String,
                        CallValue::Class(_) => {
                            todo!("class inputs are unimplemented")
                        }
                    };

                    if !input.contains(&value_kind) {
                        return Err(FuncError::InvalidParams);
                    }
                }
            }
        }

        // push all call values onto the stack
        let input_count = self.inputs.len();
        let stack_size = self.engine.stack.len();
        for value in self.inputs {
            match value {
                CallValue::None => self.engine.stack.push(StackValue::None),
                CallValue::Bool(v) => self.engine.stack.push(StackValue::Bool(v)),
                CallValue::Int(v) => self.engine.stack.push(StackValue::Int(v)),
                CallValue::Float(v) => self.engine.stack.push(StackValue::Float(v)),
                CallValue::String(v) => self.engine.stack.push(StackValue::String(v)),
                CallValue::Class(class) => {
                    let link = self.engine.store.store(class);
                    self.engine.stack.push(StackValue::Class(link));
                }
            }
        }

        // TODO: call function
        let output = match func.caller() {
            FuncCaller::Native(f) => (f)(&self.engine.stack[stack_size..], &mut self.engine.store),
        };

        // pop all the values off the stack
        for _ in 0..input_count {
            self.engine.stack.pop();
        }

        // return the function output
        Ok(output)
    }
}
