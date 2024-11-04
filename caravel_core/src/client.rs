use crate::errors::ClientError;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::path::PathBuf;

pub struct Client {
    pub manifest: PathBuf,
    pub targets: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
    pub inventory: Option<PathBuf>,
}

impl Client {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running client!");
        println!("Manifest: {:?}", self.manifest);
        println!("Targets: {:?}", self.targets);
        println!("Groups: {:?}", self.groups);
        println!("Inventory: {:?}", self.inventory);
        println!("\n\n");

        if !&self.manifest.exists() {
            return Err(ClientError::ManifestNotFound(self.manifest.display().to_string()).into());
        }

        if self.targets.is_none() && self.groups.is_none() {
            return Err(ClientError::TargetsOrGroupsRequired.into());
        }

        // if let Some(targets) = &self.targets {
        //     for target in targets {
        //         let client = reqwest::Client::new();
        //         let res = client
        //             .post(format!("{}:1336", target))
        //             .body(Json(self.manifest))
        //             .send()
        //             .await?;
        //     }
        // }

        let modules = gather_modules();

        let lua_validate_namespace = Lua::new();

        // inject module resource validate functions at resource name
        for module in &modules {
            print_lua_doc(&lua_validate_namespace, module.clone());
            inject_lua_validate_module(&lua_validate_namespace, module.clone());
        }

        let manifest_entrypoint = fs::read_to_string(&self.manifest).unwrap();

        let manifest_validate_chunk: LuaChunk = lua_validate_namespace
            .load(&manifest_entrypoint)
            .set_name(self.manifest.to_str().unwrap());

        // run the manifest, allowing lua and the module
        // to bubble up syntax errors
        match manifest_validate_chunk.exec() {
            Ok(_) => {
                println!("=== validated ===")
            }
            Err(e) => {
                print!("{}", e);
                std::process::exit(1);
            }
        }

        let lua_apply_namespace = Lua::new();

        // inject module resource apply functions at resource name
        for module in modules {
            inject_lua_apply_module(&lua_apply_namespace, module)
        }

        // run the manifest again, allowing lua and the module
        // to bubble up errors from the process
        let manifest_apply_chunk: LuaChunk = lua_apply_namespace
            .load(&manifest_entrypoint)
            .set_name(self.manifest.to_str().unwrap());

        match manifest_apply_chunk.exec() {
            Ok(_) => {
                println!("=== applied ===")
            }
            Err(e) => {
                print!("{}", e);
                std::process::exit(1);
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CaravelModuleResponseState {
    Success,
    Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CaravelModuleResponse {
    pub state: CaravelModuleResponseState,
    pub message: String,
    pub raw_module: String,
    pub module: Option<serde_json::Value>,
}

/// Dynamically open library at path, find given function name, and pass input to it.
///
/// The linked function expects to take a C string and return a C string.
/// It's used to pass json serialized strings both ways.
///
/// Input: JSON representation of the modules' resource.
///
/// Output: JSON representation of CaravelModuleResponse.
fn call_dynamic(
    lib_path: &str,
    func_name: &str,
    input: Option<&str>,
) -> Result<CaravelModuleResponse, ClientError> {
    unsafe {
        let lib = libloading::Library::new(lib_path).unwrap();

        let response: *mut c_char;
        match input {
            Some(input_str) => {
                let func: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_char> =
                    lib.get(func_name.as_bytes()).unwrap();
                response = func(CString::new(input_str).unwrap().into_raw());
            }
            None => {
                let func: libloading::Symbol<unsafe extern "C" fn() -> *mut c_char> =
                    lib.get(func_name.as_bytes()).unwrap();
                response = func();
            }
        }
        // func(CString::new(input).unwrap().into_raw());
        let c_str = CStr::from_ptr(response);
        let carevel_reponse: CaravelModuleResponse =
            serde_json::from_str(c_str.to_str().unwrap()).unwrap();
        Ok(carevel_reponse)
    }
}

fn call_dynamic_string_return(lib_path: &str, func_name: &str) -> Result<String, ClientError> {
    unsafe {
        let lib = libloading::Library::new(lib_path).unwrap();

        let func: libloading::Symbol<unsafe extern "C" fn() -> *mut c_char> =
            lib.get(func_name.as_bytes()).unwrap();
        let response: *mut c_char = func();
        let c_str = CStr::from_ptr(response);
        let carevel_reponse = c_str.to_str().unwrap().to_string();
        Ok(carevel_reponse)
    }
}

/// Tracks Caravel Module file path, and remote function identifier prefix.
#[derive(Clone)]
struct ModuleInfo {
    name: String,
    path: String,
}

/// Enumerate ./caravel_modules for Caravel module binaries.
///
/// Does no verification of local or remote platform/architechture support, yet.
fn gather_modules() -> Vec<ModuleInfo> {
    let mut module_paths: Vec<String> = Vec::new();
    if !std::path::Path::new("./caravel_modules").exists() {
        return Vec::new();
    }
    match fs::read_dir("./caravel_modules") {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let mod_path = entry.path().to_str().unwrap().to_owned();
                        if mod_path.ends_with(".dylib") {
                            module_paths.push(mod_path);
                        }
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

        let module_name = module_name_part
            .split(".")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .first()
            .unwrap()
            .to_string();

        modules.push(ModuleInfo {
            name: module_name,
            path: module_path,
        })
    }
    modules
}

fn print_lua_doc(_: &Lua, module: ModuleInfo) {
    match call_dynamic_string_return(
        module.path.as_str(),
        { module.name.clone() + "DumpLua" }.as_str(),
    ) {
        Ok(res_string) => println!("{}", res_string),
        Err(_) => {}
    }
}

/// Injects a function into the given Lua namespace
/// at module.name. The injected function wraps
/// the module.name+"Validate" function from the Caravel Module.
///
/// The Lua function takes a single table representing the desired resource,
/// which is serialize into JSON and passed to the wrapped function.
/// The wrapped function will return a JSON serialized CaravelModuleResponse.
///
/// If either side can't Serialize/Deserialize the provided JSON, it will bubble
/// up a syntax error.
fn inject_lua_validate_module(lua: &Lua, module: ModuleInfo) {
    let module_name = module.name.clone();
    let inject_func = lua
        .create_function(move |lua, input: LuaTable| {
            match call_dynamic(
                module.path.as_str(),
                { module.name.clone() + "Validate" }.as_str(),
                Some(serde_json::to_string(&input).unwrap().as_str()),
            ) {
                Ok(mut response) => match response.state {
                    CaravelModuleResponseState::Success => {
                        response.module = Some(serde_json::from_str(&response.raw_module).unwrap());
                        Ok(lua.to_value(&response))
                    }
                    CaravelModuleResponseState::Error => Err(LuaError::SyntaxError {
                        message: response.message,
                        incomplete_input: false,
                    }),
                },
                Err(_) => Err(LuaError::RuntimeError(stringify!(e).into())),
            }
        })
        .unwrap();
    lua.globals()
        .set(module_name.as_str(), inject_func)
        .unwrap();
}

/// Injects a function into the given Lua namespace
/// at module.name. The injected function wraps
/// the module.name+"Apply" function from the Caravel Module.
///
/// The Lua function takes a single table representing the desired resource,
/// which is serialize into JSON and passed to the wrapped function.
/// The wrapped function will return a JSON serialized CaravelModuleResponse.
///
/// If either side can't Serialize/Deserialize the provided JSON, it will bubble
/// up a runtime error.
fn inject_lua_apply_module(lua: &Lua, module: ModuleInfo) {
    let module_name = module.name.clone();
    let inject_func = lua
        .create_function(move |lua, input: LuaTable| {
            match call_dynamic(
                module.path.as_str(),
                { module.name.clone() + "Apply" }.as_str(),
                Some(serde_json::to_string(&input).unwrap().as_str()),
            ) {
                Ok(mut response) => match response.state {
                    CaravelModuleResponseState::Success => {
                        response.module = Some(serde_json::from_str(&response.raw_module).unwrap());
                        Ok(lua.to_value(&response))
                    }
                    CaravelModuleResponseState::Error => {
                        Err(LuaError::RuntimeError(response.message))
                    }
                },
                Err(_) => Err(LuaError::RuntimeError(stringify!(e).into())),
            }
        })
        .unwrap();
    lua.globals()
        .set(module_name.as_str(), inject_func)
        .unwrap();
}
