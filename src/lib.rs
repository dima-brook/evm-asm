use move_binary_format::{file_format::*};
use move_core_types::identifier::*;
use std::collections::HashMap;

fn print_code_unit(code: CodeUnit) {
    for instruction in code.code {
        println!("{:?}", instruction);
    }
}

fn module_resoulution(handles: Vec<ModuleHandle>, idents: IdentifierPool, addrs: AddressIdentifierPool, search: Vec<CompiledModuleMut>) -> HashMap<ModuleHandle, CompiledModuleMut> {
    let mut res = HashMap::new();
    for handle in handles.into_iter() {
        let hname = idents[handle.name.0 as usize].clone();
        let haddr = addrs[handle.address.0 as usize];
        for module in &search {
            let idx = module.self_module_handle_idx.0 as usize;
            let mname = module.identifiers[module.module_handles[idx].name.0 as usize].clone();
            let maddr = module.address_identifiers[module.module_handles[idx].address.0 as usize].clone();
            if maddr == haddr && mname == hname {
                res.insert(handle.clone(), module.clone());
            }
        }
    }

    return res;
}

pub struct MoveCode {
    script: CompiledScriptMut,
    modules: HashMap<ModuleHandle, CompiledModuleMut>
}

impl MoveCode {
    pub fn new(script: CompiledScript, modules: Vec<CompiledModule>) -> Self {
        let script = script.into_inner();
        let modules = modules.iter().map(|m| m.clone().into_inner()).collect();
        let modules = module_resoulution(script.module_handles.clone(), script.identifiers.clone(), script.address_identifiers.clone(), modules); 

        Self {
            script,
            modules
        }
    }

    fn module_handle(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
        &self.script.module_handles[idx.0 as usize]
    }

    fn identifier_resolve(&self, idx: IdentifierIndex) -> &Identifier {
        &self.script.identifiers[idx.0 as usize]
    }

    fn resolve_call(&self, idx: FunctionHandleIndex) -> Option<&CodeUnit> {
        let funh = &self.script.function_handles[idx.0 as usize];
        let module = &self.modules[self.module_handle(funh.module)];
        module.function_defs[idx.0 as usize].code.as_ref()
    }

    pub fn decompile(self) {
        print_code_unit(self.resolve_call(FunctionHandleIndex::new(0)).unwrap().clone());
    }
}
