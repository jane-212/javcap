use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, DeriveInput,
    Ident,
};

#[proc_macro_derive(Engine, attributes(engine))]
pub fn engine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let res = match ast.data {
        syn::Data::Struct(ref s) => struct_engine(&ast, &s.fields),
        syn::Data::Enum(_) => panic!("enum not support"),
        syn::Data::Union(_) => panic!("union not support"),
    };

    res.into()
}

fn struct_engine(ast: &DeriveInput, fields: &syn::Fields) -> TokenStream {
    match *fields {
        syn::Fields::Named(_) => struct_engine_named(ast),
        syn::Fields::Unnamed(_) => panic!("unnamed not support"),
        syn::Fields::Unit => panic!("unit not support"),
    }
}

fn struct_engine_named(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut image_loader = quote! {};
    let args = parse_args::<Ident>(&ast.attrs);
    if let Some(args) = args.as_ref() {
        if args.iter().any(|attr| attr == "image_loader") {
            image_loader = quote! {
                async fn load_img(&self, url: &str) -> anyhow::Result<Vec<u8>> {
                    Ok(self.client.get(url).send().await?.bytes().await?.to_vec())
                }
            };
        }
    }

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            fn key(&self) -> &'static str {
                stringify!(#name)
            }

            #image_loader
        }
    }
}

fn parse_args<T: Parse>(attrs: &[Attribute]) -> Option<Punctuated<T, Comma>> {
    if let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident("engine")) {
        match &attr.meta {
            syn::Meta::List(list) => Some(
                list.parse_args_with(Punctuated::<T, Comma>::parse_terminated)
                    .unwrap(),
            ),
            _ => panic!("only list support, like #[engined(image_loader)]"),
        }
    } else {
        None
    }
}
