extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse_macro_input;
use syn::{Data, DataStruct, DeriveInput, Fields, ItemStruct, Type};

/// Bring Struct.validate and Struct.apply to top level scope
/// as C ABI compatible functions.
///
/// Creates fn StructNameValidate() and fn StructNameApply(),
/// wrapping them in Serialization/Deserialization
#[proc_macro_attribute]
pub fn caravel_resource(_: TokenStream, input: TokenStream) -> TokenStream {
    let input_copy = input.clone();
    let item = parse_macro_input!(input as ItemStruct);
    let input = parse_macro_input!(input_copy as DeriveInput);

    let resource_ident = item.ident.clone();

    let apply_ident = item.ident.to_string() + "Apply";
    let apply_shim_ident = Ident::new(apply_ident.as_str(), Span::call_site());

    let validate_ident = item.ident.to_string() + "Validate";
    let validate_shim_ident = Ident::new(validate_ident.as_str(), Span::call_site());

    let dump_lua_ident = item.ident.to_string() + "DumpLua";
    let dump_lua_shim_ident = Ident::new(dump_lua_ident.as_str(), Span::call_site());

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    let field_name = fields.iter().map(|field| &field.ident);
    let field_type = fields.iter().map(|field| &field.ty);

    // let field_string = "---@field";
    // let endline = "\n";

    quote! {
    #item

    use std::ffi::CStr;
    use std::ffi::CString;
    use std::ffi::c_char;
    use caravel_core::client::{CaravelModuleResponse, CaravelModuleResponseState};

    impl #resource_ident {
      fn from_json_string(in_str: &str) -> Result<#resource_ident, Box<dyn std::error::Error>> {
        let resource: #resource_ident = serde_json::from_str(in_str)?;
        Ok(resource)
      }
    }

    #[no_mangle]
    pub extern "C" fn #validate_shim_ident(input: *const c_char) -> *const c_char {
      let c_str = unsafe { CStr::from_ptr(input) };
      let new_str = c_str.to_str().unwrap();
      match #resource_ident::from_json_string(new_str) {
        Ok(mut resource) => {
          match resource.validate() {
            Ok(_) => {
              let response = CaravelModuleResponse {
                  state: CaravelModuleResponseState::Success,
                message: "Success".into(),
                raw_module: serde_json::to_string(&resource).unwrap(),
                module: None,
              };
              CString::new(serde_json::to_string(&response).unwrap().as_str()).unwrap().into_raw()
            },
            Err(e)=> {
              let response = CaravelModuleResponse {
                  state: CaravelModuleResponseState::Error,
                message: e.to_string(),
                raw_module: "".into(),
                module: None,
              };
              CString::new(serde_json::to_string(&response).unwrap().as_str()).unwrap().into_raw()
            }
          }
        },
        Err(e) => {
          let response = CaravelModuleResponse {
              state: CaravelModuleResponseState::Error,
            message: e.to_string(),
            raw_module: "".into(),
            module: None,
          };
          CString::new(serde_json::to_string(&response).unwrap().as_str()).unwrap().into_raw()
        }
      }
    }

    #[no_mangle]
    pub extern "C" fn #apply_shim_ident(input: *const c_char) -> *const c_char {
        let c_str = unsafe { CStr::from_ptr(input) };
        let new_str = c_str.to_str().unwrap();

        match #resource_ident::from_json_string(new_str) {
          Ok(mut resource) => {
            match resource.apply() {
              Ok(_) => {
                let response = CaravelModuleResponse {
                    state: CaravelModuleResponseState::Success,
                  message: "Success".into(),
                  raw_module: serde_json::to_string(&resource).unwrap(),
                  module: None,
                };
                CString::new(serde_json::to_string(&response).unwrap().as_str()).unwrap().into_raw()
              },
              Err(e)=> {
                let response = CaravelModuleResponse {
                    state: CaravelModuleResponseState::Error,
                  message: e.to_string(),
                  raw_module: "".into(),
                  module: None,
                };
                CString::new(serde_json::to_string(&response).unwrap().as_str()).unwrap().into_raw()
              }
            }
          },
          Err(e) => {
            let response = CaravelModuleResponse {
                state: CaravelModuleResponseState::Error,
              message: e.to_string(),
              raw_module: "".into(),
              module: None,
            };
            CString::new(serde_json::to_string(&response).unwrap().as_str()).unwrap().into_raw()
          }
        }
    }

    #[no_mangle]
    pub extern "C" fn #dump_lua_shim_ident() -> *const c_char {
        // concat!(
        // #(
        //     stringify!("---@field" #field_name #field_type),
        // )*
        // )
      CString::new(
          concat!(
              "---@class ",
              stringify!(#resource_ident),
              "\n",
          #(
              "---@field ",
              stringify!(#field_name #field_type),
              "\n",
          )*
            "\n",
            "---@alias ModuleResponse {module: ",
            stringify!(#resource_ident),
            "}\n\n",
            "---@return ModuleResponse\n",
            "function ",
            stringify!(#resource_ident),
            "(params)\n",
            "end\n",
          )
          // r#"

          // ---@class #resource_ident
          // #(
          // ---@field #field_name #field_type
          // )*

          // ---@alias ModuleResponse {module: #resource_ident}


          // ---@param params #resource_ident
          // ---@return ModuleResponse
          // function #resource_ident(params)
          // end

          // "#
      ).unwrap().into_raw()
    }
  }.into()
}

// fn resolve_type(ty: Type) -> String {
//     match ty {}
// }
