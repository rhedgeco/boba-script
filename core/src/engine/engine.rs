use super::{ops::OpManager, ShadowScope};

#[derive(Debug, Default)]
pub struct Engine {
    vars: ShadowScope,
    ops: OpManager,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ops(&self) -> &OpManager {
        &self.ops
    }

    pub fn vars(&self) -> &ShadowScope {
        &self.vars
    }

    pub fn vars_mut(&mut self) -> &mut ShadowScope {
        &mut self.vars
    }
}
