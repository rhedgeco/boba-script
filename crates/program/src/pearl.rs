use fxhash::FxBuildHasher;
use indexmap::IndexMap;

use crate::{ModuleBuilder, ProgramBuilder};

pub(crate) struct Pearl {
    pub(crate) fields: Box<[usize]>,
}

pub(crate) struct PearlData {
    pub(crate) module: usize,
    pub(crate) fields: IndexMap<String, Vec<Vec<String>>, FxBuildHasher>,
}

pub struct PearlBuilder<'a> {
    pub(crate) program: &'a mut ProgramBuilder,
    pub(crate) index: usize,
}

impl<'a> PearlBuilder<'a> {
    pub fn id(&self) -> usize {
        self.index
    }

    pub fn parent_module(&mut self) -> ModuleBuilder {
        ModuleBuilder {
            index: self.program.pearls[self.index].module,
            program: self.program,
        }
    }

    pub fn get_field(&mut self, name: impl AsRef<str>) -> Option<&mut Vec<Vec<String>>> {
        self.program.pearls[self.index]
            .fields
            .get_mut(name.as_ref())
    }

    pub fn create_field(&mut self, name: impl Into<String>) -> Option<&mut Vec<Vec<String>>> {
        use indexmap::map::Entry as E;
        match self.program.pearls[self.index].fields.entry(name.into()) {
            E::Vacant(entry) => Some(entry.insert(Default::default())),
            E::Occupied(_) => None,
        }
    }
}
