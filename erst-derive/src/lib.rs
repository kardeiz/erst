#![recursion_limit = "128"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use std::convert::TryFrom;

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
    let mut size_hint = None;

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

        if pair.ident == "size_hint" {
            if let syn::Lit::Int(i) = pair.lit {
                size_hint = Some(i.value());
            }
        }
    }

    let type_ = type_.as_ref().map(|x| x.as_str()).unwrap_or_else(|| "");

    let size_hint: usize = size_hint.and_then(|x| usize::try_from(x).ok()).unwrap_or(1024);

    let path = path.ok_or_else(|| "No path given")?;

    let full_path = erst_shared::utils::templates_dir()?.join(&path);

    let body = std::fs::read_to_string(&full_path)?;

    let body = parse(&full_path.display().to_string(), &body, type_)?;

    let body = format!("{{ {} }}", body);

    let block = syn::parse_str::<syn::Block>(&body)?;

    let stmts = &block.stmts;

    let template_marker = if cfg!(feature = "dynamic") {
        quote!()
    } else {
        let path_display = full_path.display().to_string();
        let template_marker = syn::Ident::new(
            &format!("__ERST_TEMPLATE_MARKER_{}", &name),
            proc_macro2::Span::call_site(),
        );
        quote!(pub const #template_marker: () = { include_str!(#path_display); };)
    };

    let out = quote! {

        #template_marker

        impl #impl_generics erst::Template for #name #ty_generics #where_clause {
            fn render_into(&self, writer: &mut std::fmt::Write) -> std::fmt::Result {
                let __erst_buffer = writer;
                #(#stmts)*
                Ok(())
            }

            fn size_hint() -> usize { #size_hint }
        }

        impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                erst::Template::render_into(self, f)
            }
        }
    };

    Ok(out.into())
}

#[cfg(not(feature = "dynamic"))]
fn parse(_: &str, template: &str, type_: &str) -> Result<String, Box<std::error::Error>> {
    use erst_shared::{
        exp::Parser as _,
        parser::{ErstParser, Rule},
    };

    let pairs = ErstParser::parse(erst_shared::parser::Rule::template, template)?;

    let mut buffer = String::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::code => {
                let inner = pair.into_inner();
                buffer.push_str(inner.as_str());
            }
            Rule::expr => {
                match type_ {
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
                }
            }
            Rule::text => {
                buffer
                    .push_str(&format!("__erst_buffer.write_str(r####\"{}\"####)?;", pair.as_str()));
            }
            _ => {}
        }
    }

    Ok(buffer)
}

#[cfg(feature = "dynamic")]
fn parse(
    path: &str,
    template: &str,
    type_: &str,
) -> Result<String, Box<std::error::Error>> {
    use erst_shared::{
        exp::Parser as _,
        parser::{ErstParser, Rule},
    };

    let pairs = ErstParser::parse(erst_shared::parser::Rule::template, template)?;

    let mut buffer = String::new();

    for (idx, pair) in pairs.enumerate() {
        match pair.as_rule() {
            Rule::code => {
                let inner = pair.into_inner();
                buffer.push_str(inner.as_str());
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
                buffer.push_str(&format!(
                    "write!(__erst_buffer, \"{{}}\", 
                    erst::dynamic::get(\"{}\", {}).unwrap_or_default())?;",
                    path, idx
                ));
            }
            _ => {}
        }
    }

    Ok(buffer)
}
