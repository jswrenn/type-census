use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(Tabulate), forward_attrs(allow, doc, cfg))]
struct Opts {
    #[darling(rename = "Counter")]
    counter: Option<syn::TypePath>,
}

#[proc_macro_derive(Tabulate, attributes(Tabulate))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let counter_ty = match opts.counter {
        Some(counter_ty) => quote! { #counter_ty },
        None => quote! { type_census::counter::RelaxedCounter },
    };

    let output = quote! {
        #[automatically_derived]
        impl #impl_generics type_census::Tabulate for #ident #ty_generics #where_clause {
            type Counter = #counter_ty;
            fn counter() -> &'static #counter_ty {
                static COUNTER: #counter_ty = <#counter_ty as type_census::counter::Counter>::ZERO;
                &COUNTER
            }
        }
    };
    output.into()
}
