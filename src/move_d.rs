use move_binary_format::file_format::*;
use move_core_types::{identifier::*, value::*};
use std::collections::HashMap;
use crate::errors;

fn module_resoulution(
    handles: &[ModuleHandle],
    idents: &IdentifierPool,
    addrs: &AddressIdentifierPool,
    search: Vec<CompiledModuleMut>,
) -> HashMap<ModuleHandle, CompiledModuleMut> {
    let mut res = HashMap::new();
    for handle in handles.iter() {
        let hname = &idents[handle.name.0 as usize];
        let haddr = addrs[handle.address.0 as usize];
        for module in &search {
            let idx = module.self_module_handle_idx.0 as usize;
            let mname = &module.identifiers[module.module_handles[idx].name.0 as usize];
            let maddr = module.address_identifiers[module.module_handles[idx].address.0 as usize];
            if maddr == haddr && mname.as_str() == hname.as_str() {
                res.insert(handle.clone(), module.clone());
            }
        }
    }

    return res;
}

#[derive(Clone)]
pub struct MoveCode {
    pub script: CompiledScriptMut,
    modules: HashMap<ModuleHandle, CompiledModuleMut>,
}

impl MoveCode {
    pub fn new(script: CompiledScript, modules: Vec<CompiledModule>) -> Self {
        let script = script.into_inner();
        let modules = modules.into_iter().map(|m| m.into_inner()).collect();
        let modules = module_resoulution(
            &script.module_handles,
            &script.identifiers,
            &script.address_identifiers,
            modules,
        );

        Self { script, modules }
    }

    pub fn new_no_mods(script: CompiledScript) -> Self {
        Self {
            script: script.into_inner(),
            modules: HashMap::new(),
        }
    }

    pub fn module_handle(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
        &self.script.module_handles[idx.0 as usize]
    }

    pub fn identifier_resolve(&self, idx: IdentifierIndex) -> &Identifier {
        &self.script.identifiers[idx.0 as usize]
    }

    pub fn fn_handle(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
        &self.script.function_handles[idx.0 as usize]
    }

    pub fn resolve_call(&self, idx: FunctionHandleIndex) -> Result<&CodeUnit, errors::MoveError> {
        let funh = self.fn_handle(idx);
        let module = &self.modules[self.module_handle(funh.module)];
        let name = self.identifier_resolve(funh.name).as_str();
        for function in &module.function_defs {
            let ffh = &module.function_handles[function.function.0 as usize];
            let fname = module.identifiers[ffh.name.0 as usize].as_str();
            if fname == name {
                let fc = function.code.as_ref();
                return fc.ok_or(errors::MoveError::InvalidModule);
            }
        }
        Err(errors::MoveError::ModuleMissing)
    }

    pub fn resolve_const(&self, idx: ConstantPoolIndex) -> &Constant {
        &self.script.constant_pool[idx.0 as usize]
    }

    pub fn const_to_vec8(&self, c: Constant) -> Option<Vec<u8>> {
        let mv = c.deserialize_constant()?;
        match mv {
            MoveValue::Vector(v) => match v.get(0) {
                Some(MoveValue::U8(_)) => Some(
                    v.into_iter()
                        .map(|u| match u {
                            MoveValue::U8(r) => r,
                            _ => unreachable!(),
                        })
                        .collect(),
                ),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn disassemble_script(self) -> CodeUnit {
        self.script.code
    }

    pub fn disassemble_with_mods(&self) -> Result<(), errors::MoveError> {
        let mut calls: Vec<FunctionHandleIndex> = Vec::new();
        let mut structs: HashMap<&str, HashMap<StructDefinitionIndex, &StructDefinition>> =
            HashMap::new();
        for instruction in &self.script.code.code {
            match instruction {
                Bytecode::Call(idx) => {
                    calls.push(*idx);
                    println!("Call({})", idx);
                }
                c => println!("{:?}", c),
            }
        }

        println!("\nCall Data\n");
        for call in calls {
            let funh = self.fn_handle(call);
            let moduleh = self.module_handle(funh.module);
            let modnm = self.identifier_resolve(moduleh.name).as_str();
            let module = &self.modules[moduleh];
            let code = self.resolve_call(call)?;
            let name = self.identifier_resolve(funh.name).as_str();

            println!("{} - {}:", call, name);
            for instr in &code.code {
                match instr {
                    Bytecode::Pack(i)
                    | Bytecode::Unpack(i)
                    | Bytecode::MutBorrowGlobal(i)
                    | Bytecode::ImmBorrowGlobal(i)
                    | Bytecode::Exists(i)
                    | Bytecode::MoveFrom(i)
                    | Bytecode::MoveTo(i) => {
                        let v = if let Some(v) = structs.get_mut(modnm) {
                            v
                        } else {
                            structs.insert(modnm, HashMap::new());
                            structs.get_mut(modnm).unwrap()
                        };
                        v.insert(*i, &module.struct_defs[i.0 as usize]);
                    }
                    _ => (),
                }
                println!("  {:?}", instr);
            }
            println!("")
        }

        println!("\nModule Data\n");
        for (module, structs) in structs.into_iter() {
            println!("{}:", module);
            for (i, s) in structs.into_iter() {
                println!("  {}:", i);
                println!("      {:?}", s.field_information);
            }
        }

        Ok(())
    }
}
