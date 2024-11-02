extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::parse_macro_input;
use syn::ItemStruct;
use quote::quote;

/// Bring Struct.validate and Struct.apply to top level scope
/// as C ABI compatible functions.
/// 
/// Creates fn StructNameValidate() and fn StructNameApply(),
/// wrapping them in Serialization/Deserialization
#[proc_macro_attribute]
pub fn caravel_resource(_: TokenStream, input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as ItemStruct);

  let resource_ident = item.ident.clone();

  let apply_ident = item.ident.to_string()+"Apply";
  let apply_shim_ident = Ident::new(apply_ident.as_str(), Span::call_site());

  let validate_ident = item.ident.to_string()+"Validate";
  let validate_shim_ident = Ident::new(validate_ident.as_str(), Span::call_site());

  quote! {
    #item

    use std::ffi::CStr;
    use std::ffi::CString;
    use std::ffi::c_char;

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

    impl #resource_ident {
      fn from_json_string(in_str: &str) -> Result<#resource_ident> {
        let resource: #resource_ident = serde_json::from_str(in_str)?;
        Ok(resource)
      }
    }

    #[no_mangle]
    pub extern "C" fn #validate_shim_ident(input: *const c_char) -> *const c_char {
      let c_str = unsafe { CStr::from_ptr(input) };
      let new_str = c_str.to_str().unwrap();
      match #resource_ident::from_json_string(new_str) {
        Ok(resource) => {
          match resource.validate() {
            Ok(_) => {
              let response = CaravelModuleResponse {
                state: CaravelModuleResponseState::Success,
                message: "Success".into()
              };
              CString::new(serde_json::to_string(&response).unwrap().as_str()).unwrap().into_raw()
            },
            Err(e)=> {
              let response = CaravelModuleResponse {
                state: CaravelModuleResponseState::Error,
                message: e.to_string()
              };
              CString::new(serde_json::to_string(&response).unwrap().as_str()).unwrap().into_raw()
            }
          }
        },
        Err(e) => {
          let response = CaravelModuleResponse {
            state: CaravelModuleResponseState::Error,
            message: e.to_string()
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
          Ok(resource) => {
            match resource.apply() {
              Ok(_) => {
                let response = CaravelModuleResponse {
                  state: CaravelModuleResponseState::Success,
                  message: "Success".into()
                };
                CString::new(serde_json::to_string(&response).unwrap().as_str()).unwrap().into_raw()
              },
              Err(e)=> {
                let response = CaravelModuleResponse {
                  state: CaravelModuleResponseState::Error,
                  message: e.to_string()
                };
                CString::new(serde_json::to_string(&response).unwrap().as_str()).unwrap().into_raw()
              }
            }
          },
          Err(e) => {
            let response = CaravelModuleResponse {
              state: CaravelModuleResponseState::Error,
              message: e.to_string()
            };
            CString::new(serde_json::to_string(&response).unwrap().as_str()).unwrap().into_raw()
          }
        }
    }
  }.into()
}