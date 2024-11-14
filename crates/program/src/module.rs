use fxhash::FxBuildHasher;
use indexmap::IndexMap;

use crate::{pearl::PearlData, PearlBuilder, ProgramBuilder};

pub(crate) struct ModuleData {
    pub(crate) parent: Option<usize>,
    pub(crate) children: IndexMap<String, usize, FxBuildHasher>,
    pub(crate) pearls: IndexMap<String, usize, FxBuildHasher>,
}

pub struct ModuleBuilder<'a> {
    pub(crate) program: &'a mut ProgramBuilder,
    pub(crate) index: usize,
}

impl<'a> ModuleBuilder<'a> {
    pub fn id(&self) -> usize {
        self.index
    }

    pub fn parent(&mut self) -> Option<ModuleBuilder> {
        let data = &self.program.modules[self.index];
        Some(ModuleBuilder {
            index: data.parent?,
            program: self.program,
        })
    }

    pub fn get_module(&mut self, name: impl AsRef<str>) -> Option<ModuleBuilder> {
        let data = &self.program.modules[self.index];
        Some(ModuleBuilder {
            index: *data.children.get(name.as_ref())?,
            program: self.program,
        })
    }

    pub fn get_pearl(&mut self, name: impl AsRef<str>) -> Option<PearlBuilder> {
        let data = &self.program.modules[self.index];
        Some(PearlBuilder {
            index: *data.pearls.get(name.as_ref())?,
            program: self.program,
        })
    }

    pub fn create_module(&mut self, name: impl Into<String>) -> Option<ModuleBuilder> {
        use indexmap::map::Entry as E;
        let next_index = self.program.modules.len();
        let data = &mut self.program.modules[self.index];
        match data.children.entry(name.into()) {
            E::Occupied(_) => None,
            E::Vacant(entry) => {
                entry.insert(next_index);
                self.program.modules.push(ModuleData {
                    parent: Some(self.index),
                    children: Default::default(),
                    pearls: Default::default(),
                });

                Some(ModuleBuilder {
                    program: self.program,
                    index: next_index,
                })
            }
        }
    }

    pub fn create_pearl(&mut self, name: impl Into<String>) -> Option<PearlBuilder> {
        use indexmap::map::Entry as E;
        let next_index = self.program.pearls.len();
        let data = &mut self.program.modules[self.index];
        match data.pearls.entry(name.into()) {
            E::Occupied(_) => None,
            E::Vacant(entry) => {
                entry.insert(next_index);
                self.program.pearls.push(PearlData {
                    module: self.index,
                    fields: Default::default(),
                });

                Some(PearlBuilder {
                    program: self.program,
                    index: next_index,
                })
            }
        }
    }
}
