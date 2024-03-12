use crate::cli::Runnable;
use std::path::PathBuf;
use mlua::prelude::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::fs;
use serde::{
    Serialize,
    Deserialize,
};
use anyhow::Result;

pub struct Client {
    pub manifest: PathBuf,
    pub targets: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
    pub inventory: Option<PathBuf>,
}

impl Runnable for Client {
    fn run(&self) {
        println!("Running client!");
        println!("Manifest: {:?}", self.manifest);
        println!("Targets: {:?}", self.targets);
        println!("Groups: {:?}", self.groups);
        println!("Inventory: {:?}", self.inventory);
        println!("\n\n");

        if !&self.manifest.exists() {
            eprintln!("Provided manifest doesn't exist!");
            std::process::exit(1);
        }
        let modules = gather_modules();
    
        let lua_validate_namespace = Lua::new();


        for module in &modules {
            inject_lua_validate_module(&lua_validate_namespace, module.clone())
        }

        
        let manifest_entrypoint = fs::read_to_string(&self.manifest).unwrap();

        let manifest_validate_chunk: LuaChunk = lua_validate_namespace.load(&manifest_entrypoint)
        .set_name(self.manifest.to_str().unwrap());

        match manifest_validate_chunk.exec() {
            Ok(_) => {println!("=== validated ===")}
            Err(e) => {
                print!("{}", e);
                std::process::exit(1);
            }
        }

        let lua_apply_namespace = Lua::new();

        for module in modules {
            inject_lua_apply_module(&lua_apply_namespace, module)
        }


        let manifest_apply_chunk: LuaChunk = lua_apply_namespace.load(&manifest_entrypoint)
        .set_name(self.manifest.to_str().unwrap());
        // test creating a file resource from lua
        match manifest_apply_chunk.exec() {
            Ok(_) => {println!("=== applied ===")}
            Err(e) => {
                print!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum CaravelModuleResponseState {
    Success,
    Error,
}

#[derive(Serialize, Deserialize, Debug)]
struct CaravelModuleResponse {
    state: CaravelModuleResponseState,
    message: String,
}

fn call_dynamic(lib_path: &str, func_name: &str, input: &str) -> Result<CaravelModuleResponse> {
    unsafe {
        let lib = libloading::Library::new(lib_path).unwrap();
        let func: libloading::Symbol<unsafe extern fn(*const c_char) -> *mut c_char> = lib.get(func_name.as_bytes()).unwrap();
        let response: *mut c_char = func(CString::new(input).unwrap().into_raw());
        let c_str = CStr::from_ptr(response);
        let carevel_reponse: CaravelModuleResponse  = serde_json::from_str(c_str.to_str().unwrap()).unwrap();
        Ok(carevel_reponse)
    }
}

#[derive(Clone)]
struct ModuleInfo {
    name: String,
    path: String,
}

fn gather_modules() -> Vec<ModuleInfo>{
    let mut module_paths: Vec<String> = Vec::new();
    if !std::path::Path::new("./caravel_modules").exists() {
        return Vec::new()
    }
    match fs::read_dir("./caravel_modules") {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let mod_path = entry.path().to_str().unwrap().to_owned();
                        module_paths.push(mod_path);
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    let mut modules: Vec<ModuleInfo> = Vec::new();
    for module_path in module_paths.into_iter() {
        let module_parts: Vec<String> = module_path.split("/").map(|s| s.to_string()).collect();
        let module_name_part = module_parts.last().unwrap().to_string();

        let module_name = module_name_part.split(".")
        .map(|s| s.to_string()).collect::<Vec<String>>()
        .first().unwrap().to_string();


        modules.push(
            ModuleInfo {
                name: module_name,
                path: module_path,
            }
        )   
    }
    modules
}

fn inject_lua_validate_module(lua: &Lua, module: ModuleInfo) {
    let module_name = module.name.clone();
    let inject_func = lua.create_function(
        move |_, input: LuaTable| {
            match call_dynamic(
                module.path.as_str(), 
                {module.name.clone() + "Validate"}.as_str(),
                serde_json::to_string(&input).unwrap().as_str(),
            ) {
                Ok(response) => {
                    match response.state {
                        CaravelModuleResponseState::Success => {
                            Ok(response.message)
                        }
                        CaravelModuleResponseState::Error => {
                            Err(LuaError::SyntaxError{
                                message: response.message,
                                incomplete_input: false
                            })
                        }
                    }
                },
                Err(e) => Err(LuaError::RuntimeError(stringify!(e).into())),
            }
        }
    ).unwrap();
    lua.globals().set(module_name.as_str(), inject_func).unwrap();
}

fn inject_lua_apply_module(lua: &Lua, module: ModuleInfo) {
    let module_name = module.name.clone();
    let inject_func = lua.create_function(
        move |_, input: LuaTable| {
            match call_dynamic(
                module.path.as_str(), 
                {module.name.clone() + "Apply"}.as_str(),
                serde_json::to_string(&input).unwrap().as_str(),
            ) {
                Ok(response) => {
                    match response.state {
                        CaravelModuleResponseState::Success => {
                            Ok(response.message)
                        }
                        CaravelModuleResponseState::Error => {
                            Err(LuaError::RuntimeError(response.message))
                        }
                    }
                },
                Err(e) => Err(LuaError::RuntimeError(stringify!(e).into())),
            }
        }
    ).unwrap();
    lua.globals().set(module_name.as_str(), inject_func).unwrap();
}
