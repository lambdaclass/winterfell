// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Expr, ImplItem, Item, ItemFn, ItemImpl, ItemTrait, TraitItem, TraitItemFn,
};

/// Parses a function (regular or trait) and conditionally adds the `async` keyword depending on
/// the `async` feature flag being enabled.
///
/// For example:
/// ```ignore
/// trait ExampleTrait {
///     #[maybe_async]
///     fn say_hello(&self);
///
///     #[maybe_async]
///     fn get_hello(&self) -> String;
/// }
///
///
/// #[maybe_async]
/// fn hello_world() {
///     // ...
/// }
/// ```
///
/// When the `async` feature is enabled, will be transformed into:
/// ```ignore
/// trait ExampleTrait {
///     async fn say_hello(&self);
///
///     async fn get_hello(&self) -> String;
/// }
///
///
/// async fn hello_world() {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn maybe_async(_attr: TokenStream, input: TokenStream) -> TokenStream {
    // Check if the input is a function
    if let Ok(func) = syn::parse::<ItemFn>(input.clone()) {
        if cfg!(feature = "async") {
            let ItemFn { attrs, vis, mut sig, block } = func;
            sig.asyncness = Some(syn::token::Async::default());
            quote! {
                #(#attrs)* #vis #sig { #block }
            }
            .into()
        } else {
            quote!(#func).into()
        }
    }
    // Check if the input is a trait function
    else if let Ok(func) = syn::parse::<TraitItemFn>(input.clone()) {
        if cfg!(feature = "async") {
            let TraitItemFn { attrs, mut sig, default, semi_token } = func;
            sig.asyncness = Some(syn::token::Async::default());
            quote! {
                #(#attrs)* #sig #default #semi_token
            }
            .into()
        } else {
            quote!(#func).into()
        }
    }
    // Check if the input is a trait definition
    else if let Ok(mut trait_item) = syn::parse::<ItemTrait>(input.clone()) {
        let vis = &trait_item.vis;
        let trait_ident = &trait_item.ident;
        let trait_generics = &trait_item.generics;

        if cfg!(feature = "async") {
            // Modify each function in the trait to add async keyword
            trait_item.items.iter_mut().for_each(|item| {
                if let TraitItem::Fn(method) = item {
                    method.sig.asyncness = Some(syn::token::Async::default());
                }
            });
            let items = &trait_item.items;
            quote! {
                #[async_trait::async_trait(?Send)]
                #vis trait #trait_ident #trait_generics {
                    #( #items )*
                }
            }
            .into()
        } else {
            let items = &trait_item.items;
            quote! {
                #vis trait #trait_ident #trait_generics {
                    #( #items )*
                }
            }
            .into()
        }
    }
    // Check if the input is an impl block
    else if let Ok(mut impl_item) = syn::parse::<ItemImpl>(input.clone()) {
        let impl_generics = &impl_item.generics;
        let self_ty = &impl_item.self_ty;

        if cfg!(feature = "async") {
            // Modify each function in the impl to add async keyword
            impl_item.items.iter_mut().for_each(|item| {
                if let ImplItem::Fn(method) = item {
                    method.sig.asyncness = Some(syn::token::Async::default());
                }
            });

            let items = &impl_item.items;

            if let Some((bang, trait_path, for_token)) = &impl_item.trait_ {
                // Trait implementation
                quote! {
                    #[async_trait::async_trait(?Send)]
                    impl #impl_generics #bang #trait_path #for_token #self_ty {
                        #( #items )*
                    }
                }
                .into()
            } else {
                // Inherent impl block
                quote! {
                    #[async_trait::async_trait(?Send)]
                    impl #impl_generics #self_ty {
                        #( #items )*
                    }
                }
                .into()
            }
        } else {
            // No need to modify functions
            quote!(#impl_item).into()
        }
    }
    // If none of the above matches, return the input unchanged
    else {
        input
    }
}

/// Parses an expression and conditionally adds the `.await` keyword at the end of it depending on
/// the `async` feature flag being enabled.
///
/// ```ignore
/// #[maybe_async]
/// fn hello_world() {
///     // Adding `maybe_await` to an expression
///     let w = maybe_await!(world());
///
///     println!("hello {}", w);
/// }
///
/// #[maybe_async]
/// fn world() -> String {
///     "world".to_string()
/// }
/// ```
///
/// When the `async` feature is enabled, will be transformed into:
/// ```ignore
/// async fn hello_world() {
///     let w = world().await;
///
///     println!("hello {}", w);
/// }
///
/// async fn world() -> String {
///     "world".to_string()
/// }
/// ```
#[proc_macro]
pub fn maybe_await(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Expr);

    let quote = if cfg!(feature = "async") {
        quote!(#item.await)
    } else {
        quote!(#item)
    };

    quote.into()
}
