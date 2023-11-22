use convert_case::*;
use proc_macro2::TokenStream;
use quote::*;

mod services {
    ecdar_protobuf_transpiler_macros::get_services!();
}

impl services::ProtobuffTypes {
    pub fn to_rust_type(&self) -> String {
        let is_google = self.name.split('.').next().unwrap() == "google";
        match self.name {
            _ if !is_google => format!(
                "ecdar_protobuf::services::{}",
                self.name.to_case(Case::Pascal)
            ),
            "google.protobuf.Empty" => "()".into(),
            _ => {
                panic!(
                    "{} is not maped, please map it in crate ecdar-protobuf-transpiler in file lib.rs",
                    self.name
                )
            }
        }
    }
}

pub struct CompileVariables {
    pub fn_name: TokenStream,
    pub endpoint_name: TokenStream,
    pub service_name: TokenStream,
    pub in_struct: TokenStream,
    pub in_struct_name: TokenStream,
    pub in_struct_has_body: bool,
    pub rtn_struct: TokenStream,
    pub client : TokenStream,
}

pub fn compile<T>(foreach: impl Fn(CompileVariables) -> T) -> Vec<T> {
    services::SERVICES
        .iter()
        .map(|service| {
            service
                .endpoints
                .iter()
                .map(|endpoint| {
                    let fn_name = format_ident!("{}", get_fn_name(service.name, endpoint.name));
                    let in_struct_name = format_ident!("In{}",
                        service.name.to_case(Case::Pascal) + 
                        endpoint.name.to_case(Case::Pascal).as_str()
                    );
                    let body_rust_type_str = endpoint.input_type.to_rust_type();
                    let in_struct_has_body = body_rust_type_str != "()";

                    let body = if in_struct_has_body {
                        let body_rust_type = format_ident!("{}", body_rust_type_str);
                        quote!{ pub body : #body_rust_type}
                    } else {
                        quote!{}
                    };

                    let rtn_struct = format_ident!("{}", endpoint.output_type.to_rust_type());

                    let client = format_ident!(
                        "services::{}_client::{}Client",
                        service.name.to_case(Case::Snake),
                        service.name.to_case(Case::Pascal)
                    );

                    let endpoint_name = format_ident!("{}", endpoint.name);
                    let service_name = format_ident!("{}", service.name);

                    foreach(CompileVariables {
                        endpoint_name : quote!(#endpoint_name),
                        service_name: quote!(#service_name),
                        fn_name : quote!(#fn_name),
                        in_struct : quote!{
                            #[derive(serde::Serialize, serde::Deserialize)]
                            pub struct #in_struct_name {
                                pub ip : String,
                                #body
                            } 
                        },
                        in_struct_name : quote!(#in_struct_name),
                        client : quote!(#client),
                        rtn_struct : quote!(#rtn_struct),
                        in_struct_has_body
                    })
                })
                .collect::<Vec<_>>()
        }).reduce(|mut acc, mut v| {
            acc.append(&mut v);
            acc
        }).unwrap()
}

fn get_fn_name(service_name: &str, enpoint_name: &str) -> String {
    format!(
        "{}_{}",
        service_name.to_case(Case::Snake),
        enpoint_name.to_case(Case::Snake)
    )
}

pub use services::*;
