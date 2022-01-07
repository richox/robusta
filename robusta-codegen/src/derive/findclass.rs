use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput, ExprPath};
use syn::spanned::Spanned;

use crate::transformation::JavaPath;

use super::utils::generic_params_to_args;

pub(crate) fn findclass_macro_derive(input: DeriveInput) -> TokenStream {
    let input_span = input.span();
    match findclass_macro_derive_impl(input) {
        Ok(t) => t,
        Err(_) => quote_spanned! { input_span => }
    }
}

fn findclass_macro_derive_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let input_span = input.span();

    match input.data {
        Data::Struct(DataStruct { .. }) => {
            let package_attr = input.attrs.iter().find(|a| a.path.get_ident().map(ToString::to_string).as_deref() == Some("package"));
            let package_str = match package_attr {
                None => None,
                Some(attr) => {
                    let package = attr.parse_args::<JavaPath>()?;
                    let package_str = {
                        let mut s = package.to_classpath_path();
                        if !s.is_empty() {
                            s.push('/')
                        }
                        s
                    };
                    Some(package_str)
                }
            };

            let findclass_attr = input.attrs.iter().find(|a| a.path.get_ident().map(ToString::to_string).as_deref() == Some("findclass"));
            match findclass_attr {
                None => abort!(input_span, "missing `#[findclass()]` attribute"),
                Some(attr) => {
                    let struct_name = input.ident;
                    let findclass_fn = attr.parse_args::<ExprPath>()?;
                    let findclass_name = match package_str {
                        Some(package_str) => format!("{}{}", package_str, struct_name.to_string()),
                        None => struct_name.to_string(),
                    };

                    let generics = input.generics.clone();
                    let generic_args = generic_params_to_args(input.generics);

                    Ok(quote! {
                        #[automatically_derived]
                        impl#generics ::robusta_jni::convert::FindClass<'env, 'borrow> for #struct_name#generic_args {
                            fn find_class(
                                env: &'borrow ::robusta_jni::jni::JNIEnv<'env>,
                            ) -> ::robusta_jni::jni::errors::Result<::robusta_jni::jni::objects::JClass<'env>> {
                                #findclass_fn(env, #findclass_name)
                            }
                        }
                    })
                }
            }
        },
        _ => abort!(input_span, "`FindClass` auto-derive implemented for structs only"),
    }
}
