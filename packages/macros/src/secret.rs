use std::process::id;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, GenericParam, ItemStruct, LitStr, TypeParam};
use syn::CapturedParam::Lifetime;
use syn::spanned::Spanned;

pub(crate) fn expand_struct(mut item: ItemStruct) -> proc_macro::TokenStream {
    let mut inline_fns: Vec<(String, TokenStream, TokenStream)> = vec![];
    let mut unlock_exprs: Vec<TokenStream> = vec![];
    let mut is_kvp = false;
    
    for attr in item.attrs.iter() {
        if !attr.path().is_ident("derive") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("KeyValueProvider") {
                is_kvp = true;
            }
            Ok(())
        }).unwrap();
    }

    for (i, field) in item.fields.iter_mut().enumerate() {
        let mut kvp_name = None::<String>;
        let mut is_secret = false;
        for (j, attr) in field.attrs.iter_mut().enumerate() {
            if !attr.path().is_ident("secret") {
                continue;
            }
            is_secret = true;
            let mut is_derived = false;
            let mut derived_key = String::default();
            attr.parse_nested_meta(|meta| {
                match meta.path.require_ident() {
                    Ok(ident) => {
                        match ident.to_string().as_str() { 
                            "derived" => {
                                let s: LitStr = meta.value()?.parse()?;
                                is_derived = true;
                                derived_key = s.value();
                                kvp_name = Some(s.value());
                                Ok(())
                            },
                            "encrypted" => {
                                let s: LitStr = meta.value()?.parse()?;
                                is_derived = false;
                                derived_key = s.value();
                                kvp_name = Some(s.value());
                                Ok(())
                            },
                            _ => Err(syn::Error::new(attr.span(), format!("Invalid secret type: {}", ident.to_string())))
                        }
                    },
                    Err(e) => Err(e)
                }
            })
                .unwrap();
            
            let fn_body= if is_derived {
                let derived_key_val = derived_key.clone();
                quote!(LockableSecret::new_derived_locked(#derived_key_val))
            } else { 
                quote!(LockableSecret::new_empty_locked())
            };

            // we check here if a function with the exact same return value already exists. if so,
            // this function gets used.
            let fn_name_lit = if let Some((fn_name_lit, _, _)) = inline_fns
                .iter()
                .find(|(_, def, _)| def.to_string() == fn_body.to_string())
            {
                fn_name_lit.clone()
            } else {
                let fn_name_lit = format!("__secret_{}_{}", item.ident, i);
                let fn_name_ident = Ident::new(&fn_name_lit, Span::call_site());

                let inline_fn = quote! {
                    #[doc(hidden)]
                    #[allow(non_snake_case)]
                    fn #fn_name_ident<'a> () -> LockableSecret<'a> {
                        #fn_body
                    }
                };
                inline_fns.push((fn_name_lit.clone(), fn_body, inline_fn));
                fn_name_lit
            };
            field.attrs.remove(j);
            field
                .attrs
                .insert(j, parse_quote!( #[serde(default = #fn_name_lit)] ));
            break;
        }
        if is_secret {
            if is_kvp {
                // Setup KeyValueProvider trait
                if let Some(key_name) = kvp_name {
                    if !field.attrs.iter().any(|a| a.path().is_ident("kvp")) {
                        field.attrs.push(parse_quote!(#[kvp(name = #key_name, fn="secret_to_secret_string")]));
                    }
                }
            }
            // Setup unlock fn
            let field_name = format_ident!("{}", field.ident.as_ref().unwrap());
            unlock_exprs.push(quote! {
                self.#field_name = self.#field_name.unlock(key, salt);
            })
        }
    }
    let name = &item.ident;

    let mut generics = item.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let real_inline_fns: Vec<TokenStream> =
        inline_fns.into_iter().map(|(_, _, func)| func).collect();
    
    let expanded = quote! {
        #( #real_inline_fns )*

        #item
        
        impl <'_secret> Unlockable <'_secret> for #name <'_secret> #where_clause {
            fn unlock(&mut self, key: &'_secret SecretVec<u8>, salt: Salt) -> Result<(), String> {
                #( #unlock_exprs )*
                Ok(())
            }
        }
    };
    expanded.into()
}