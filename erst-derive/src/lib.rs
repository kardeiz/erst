#![recursion_limit = "128"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;

use std::path::PathBuf;

#[proc_macro_derive(Template, attributes(template))]
pub fn template_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    template_derive_inner(input).unwrap()
}

fn template_derive_inner(input: syn::DeriveInput) -> Result<TokenStream, Box<std::error::Error>> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut path = None;
    let mut type_ = None;

    for pair in input
        .attrs
        .iter()
        .flat_map(|x| x.parse_meta())
        .filter(|x| x.name() == "template")
        .filter_map(|x| match x {
            syn::Meta::List(ml) => Some(ml),
            _ => None,
        })
        .flat_map(|x| x.nested)
        .filter_map(|x| match x {
            syn::NestedMeta::Meta(m) => Some(m),
            _ => None,
        })
        .filter_map(|x| match x {
            syn::Meta::NameValue(nv) => Some(nv),
            _ => None,
        })
    {
        if pair.ident == "path" {
            if let syn::Lit::Str(ref s) = pair.lit {
                path = Some(s.value());
            }
        }

        if pair.ident == "type" {
            if let syn::Lit::Str(ref s) = pair.lit {
                type_ = Some(s.value());
            }
        }
    }

    let type_ = type_.as_ref().map(|x| x.as_str()).unwrap_or_else(|| "");

    let path = std::env::var("ERST_TEMPLATES_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("CARGO_MANIFEST_DIR").map(|x| PathBuf::from(x).join("templates"))
        })?
        .join(path.ok_or_else(|| "No path given")?);

    let body = std::fs::read_to_string(&path)?;

    let body = format!("{{ {} }}", parse(&body, type_)?);

    let block = syn::parse_str::<syn::Block>(&body)?;

    let stmts = &block.stmts;

    let body_marker =
        syn::Ident::new(&format!("__ERST_BODY_MARKER_{}", &name), proc_macro2::Span::call_site());
    let path_display = path.display().to_string();

    let out = quote! {

        pub const #body_marker: () = { include_str!(#path_display); };

        impl #impl_generics erst::Template for #name #ty_generics #where_clause {
            fn render_into(&self, writer: &mut std::fmt::Write) -> std::fmt::Result {
                use std::fmt::Write;
                let __erst_buffer = writer;
                #(#stmts)*
                Ok(())
            }
        }

        impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                erst::Template::render_into(self, f).map_err(|_| std::fmt::Error {})
            }
        }
    };

    Ok(out.into())
}

fn parse(template: &str, type_: &str) -> Result<String, Box<std::error::Error>> {
    use erst_shared::{
        exp::Parser as _,
        parser::{ErstParser, Rule},
    };

    let pairs = ErstParser::parse(erst_shared::parser::Rule::template, template)?;

    let mut buffer = String::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::code => {
                buffer.push_str(pair.into_inner().as_str());
            }
            Rule::expr => match type_ {
                "html" => {
                    buffer.push_str(&format!(
                        "write!(__erst_buffer, \"{{}}\", erst::Html({}))?;",
                        pair.into_inner().as_str()
                    ));
                }
                _ => {
                    buffer.push_str(&format!(
                        "write!(__erst_buffer, \"{{}}\", {})?;",
                        pair.into_inner().as_str()
                    ));
                }
            },
            Rule::text => {
                buffer.push_str(&format!("write!(__erst_buffer, \"{}\")?;", pair.as_str()));
            }
            _ => {}
        }
    }

    Ok(buffer)
}
