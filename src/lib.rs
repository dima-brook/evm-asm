use move_binary_format::file_format::*;
use move_core_types::identifier::*;
use std::collections::HashMap;

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

pub struct MoveCode {
    script: CompiledScriptMut,
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

    fn module_handle(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
        &self.script.module_handles[idx.0 as usize]
    }

    fn identifier_resolve(&self, idx: IdentifierIndex) -> &Identifier {
        &self.script.identifiers[idx.0 as usize]
    }

    fn fn_handle(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
        &self.script.function_handles[idx.0 as usize]
    }

    fn resolve_call(&self, idx: FunctionHandleIndex) -> Option<&CodeUnit> {
        let funh = self.fn_handle(idx);
        let module = &self.modules[self.module_handle(funh.module)];
        let name = self.identifier_resolve(funh.name).as_str();
        for function in &module.function_defs {
            let ffh = &module.function_handles[function.function.0 as usize];
            let fname = module.identifiers[ffh.name.0 as usize].as_str();
            if fname == name {
                return function.code.as_ref();
            }
        }
        panic!("Failed to find function! Incorrect module");
    }

    pub fn decompile(self) {
        let mut calls: Vec<FunctionHandleIndex> = Vec::new();
        for instruction in &self.script.code.code {
            match instruction {
                Bytecode::Call(idx) => {
                    calls.push(*idx);
                }
                c => println!("{:?}", c),
            }
        }

        println!("\nCall Data\n");
        for call in calls {
            let funh = self.fn_handle(call);
            let code = self.resolve_call(call);
            let name = self.identifier_resolve(funh.name).as_str();

            println!("{} - {}:", call, name);
            for instr in &code.unwrap().code {
                println!("  {:?}", instr);
            }
            println!("")
        }
    }
}
