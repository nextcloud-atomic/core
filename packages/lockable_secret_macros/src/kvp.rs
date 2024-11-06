use proc_macro::{TokenStream};
use quote::{quote};
use syn::{LitStr, DeriveInput, Field};
use syn::__private::quote;

pub(crate) fn get_kv_from_field(field: &Field) -> Result<Option<proc_macro2::TokenStream>, String> {
    let field_name_ident = field.ident.as_ref()
        .ok_or("Anonymous fields are not supported")?;
    match field.attrs.iter()
        .find(|a| a.path().is_ident("kvp")) {
        None => {
            let key_name = field_name_ident.to_string();
            Some(Ok(quote! {
                map.insert(#key_name.to_string(), self.#field_name_ident.to_string());
            }))
        },
        
        Some(attr) => {
            let mut key_name_opt = None::<String>;
            let mut skip = false;
            let mut serialize_fn_opt = None::<String>;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let s: LitStr = meta.value()?.parse()?;
                    key_name_opt = Some(s.value());
                }
                if meta.path.is_ident("skip") {
                    skip = true;
                }
                if meta.path.is_ident("fn") {
                    let s: LitStr = meta.value()?.parse()?;
                    serialize_fn_opt = Some(s.value());
                }
                Ok(())
            })
                .unwrap();
            
            if skip {
                return Ok(None);
            }

            let key_name = match key_name_opt {
                None => field_name_ident.to_string(),
                Some(name) => name
            };
            // let method_call = quote!(&self.#field_name_ident.to_string());
            
            let serialize_fn = match serialize_fn_opt {
                None => quote!(self.#field_name_ident.to_string()),
                Some(f) => {
                    let f_token_stream: proc_macro2::TokenStream = f.parse().unwrap();
                    quote!(#f_token_stream ( &self.#field_name_ident ))
                }
            };
            
            Some(Ok(quote! {
                map.insert(#key_name.to_string(), #serialize_fn);
            }))
        }
    }.transpose()
}

pub(crate) fn expand_struct(ast: DeriveInput) -> TokenStream {

    let syn::Data::Struct(data) = ast.data else {
        unimplemented!()
    };
    let kvs = data.fields.iter().enumerate().filter_map(|(i, field)| {
        match get_kv_from_field(field) {
            Err(e) => panic!("Error while parsing field {} of {}: {e:?}", ast.ident, i),
            Ok(opt) => opt
        }
    }).collect::<Vec<_>>();

    let name = &ast.ident;
    
    let generics = ast.generics.clone();

    let expanded = quote! {
        impl #generics key_value_provider::KeyValueProvider for #name #generics {
            fn to_map(&self) -> std::collections::HashMap<String, String> {
                let mut map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                #(#kvs)*
                map
            }
        }
    };

    expanded.into()
}
