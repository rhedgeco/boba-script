use crate::{
    module::ModuleData,
    pearl::{Pearl, PearlData},
    BuildError, ModuleBuilder,
};

#[derive(Default)]
pub struct ProgramBuilder {
    pub(crate) modules: Vec<ModuleData>,
    pub(crate) pearls: Vec<PearlData>,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn root_module(&mut self) -> ModuleBuilder {
        if self.modules.is_empty() {
            self.modules.push(ModuleData {
                children: Default::default(),
                pearls: Default::default(),
                parent: None,
            })
        }

        ModuleBuilder {
            program: self,
            index: 0,
        }
    }

    pub fn build(&self) -> Result<Program, Vec<BuildError>> {
        let mut errors = Vec::new();

        // resolve all pearl fields
        let mut pearls = Vec::new();
        for pearl in self.pearls.iter() {
            let mut fields = Vec::new();
            for union in pearl.fields.values() {
                for path in union.iter() {
                    // get the pearl name at the end of the path
                    let mut parts = path.iter().map(|s| s.as_str()).peekable();
                    let pearl_name = match parts.next_back() {
                        Some(last) => last,
                        None => {
                            errors.push(BuildError::EmptyPath);
                            continue;
                        }
                    };

                    // check the first segment of the path for 'root' keyword
                    let mut module = match parts.peek().map(|s| *s) {
                        Some("root") => &self.modules[0],
                        _ => &self.modules[pearl.module],
                    };

                    // check all remaining path parts and update the search module
                    for part in parts {
                        module = match part {
                            "root" => {
                                errors.push(BuildError::InvalidRootKeyword);
                                continue;
                            }
                            "super" => match module.parent {
                                Some(parent) => &self.modules[parent],
                                None => {
                                    errors.push(BuildError::SuperFromRoot);
                                    continue;
                                }
                            },
                            part => match module.children.get(part) {
                                Some(index) => &self.modules[*index],
                                None => {
                                    errors.push(BuildError::ModuleDoesNotExist(part.to_string()));
                                    continue;
                                }
                            },
                        };
                    }

                    // try to get the target pearl from the search module
                    match module.pearls.get(pearl_name) {
                        Some(index) => fields.push(*index),
                        None => {
                            errors.push(BuildError::PearlDoesNotExist(pearl_name.to_string()));
                            continue;
                        }
                    };
                }
            }

            pearls.push(Pearl {
                fields: fields.into_boxed_slice(),
            })
        }

        // only return built program if there was no errors
        match errors.is_empty() {
            true => Ok(Program { pearls }),
            false => Err(errors),
        }
    }
}

pub struct Program {
    pearls: Vec<Pearl>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn type_resolution() {
        let mut builder = ProgramBuilder::new();

        // build root module and pearl
        let mut root_module = builder.root_module();
        let mut root_pearl = root_module.create_pearl("RootPearl").unwrap();
        let root_pearl_id = root_pearl.id();
        let field1 = root_pearl.create_field("field1").unwrap();
        field1.push(vec!["sub_module".to_string(), "NestedPearl".to_string()]);
        let field2 = root_pearl.create_field("field2").unwrap();
        field2.push(vec![
            "sub_module".to_string(),
            "sub_module2".to_string(),
            "NestedPearl2".to_string(),
        ]);

        // build nested module and pearl
        let mut sub_module = root_module.create_module("sub_module").unwrap();
        let mut nested_pearl = sub_module.create_pearl("NestedPearl").unwrap();
        let nested_pearl_id = nested_pearl.id();
        let field = nested_pearl.create_field("field1").unwrap();
        field.push(vec!["super".to_string(), "RootPearl".to_string()]);

        // build double nested module and pearl
        let mut sub_module2 = sub_module.create_module("sub_module2").unwrap();
        let mut nested_pearl2 = sub_module2.create_pearl("NestedPearl2").unwrap();
        let nested_pearl2_id = nested_pearl2.id();
        let field1 = nested_pearl2.create_field("field1").unwrap();
        field1.push(vec![
            "super".to_string(),
            "super".to_string(),
            "RootPearl".to_string(),
        ]);
        let field2 = nested_pearl2.create_field("field2").unwrap();
        field2.push(vec!["root".to_string(), "RootPearl".to_string()]);

        let program = builder.build().unwrap();
        assert_eq!(program.pearls[0].fields[0], nested_pearl_id);
        assert_eq!(program.pearls[0].fields[1], nested_pearl2_id);
        assert_eq!(program.pearls[1].fields[0], root_pearl_id);
        assert_eq!(program.pearls[2].fields[0], root_pearl_id);
        assert_eq!(program.pearls[2].fields[1], root_pearl_id);
    }
}
