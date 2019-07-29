#[macro_use]
extern crate pest_derive;

pub mod err {

    #[derive(Debug, derive_more::From, derive_more::Display)]
    pub enum Error {
        EnvVar(std::env::VarError),
        Parse(String),
        Msg(Msg),
        Io(std::io::Error),
    }

    #[derive(Debug, derive_more::Display)]
    pub struct Msg(pub String);

    impl Error {
        pub fn msg<T: std::fmt::Display>(error: T) -> Self {
            Msg(error.to_string()).into()
        }
    }

    impl std::error::Error for Error {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match *self {
                Error::EnvVar(ref inner) => Some(inner),
                Error::Io(ref inner) => Some(inner),
                _ => None,
            }
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;
}

pub mod parser {
    #[derive(Parser)]
    #[grammar = "erst.pest"]
    pub struct ErstParser;
}

pub mod exp {
    pub use bstr::{BString, B};
    pub use pest::Parser;
}

pub mod utils {

    use std::path::PathBuf;

    pub fn templates_dir() -> crate::err::Result<PathBuf> {
        let out = std::env::var("ERST_TEMPLATES_DIR")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("CARGO_MANIFEST_DIR").map(|x| PathBuf::from(x).join("templates"))
            })
            .unwrap_or_else(|_| PathBuf::from("templates"));
        Ok(out)
    }
}

#[cfg(feature = "dynamic")]
pub mod dynamic {

    use std::path::{Path, PathBuf};

    pub fn generate_code_cache() -> crate::err::Result<()> {
        let pkg_name = std::env::var("CARGO_PKG_NAME")?;
        let xdg_dirs = xdg::BaseDirectories::with_prefix(format!("erst/{}", &pkg_name))
            .map_err(crate::err::Error::msg)?;

        for path in template_paths() {
            let path_name =
                path.file_name().ok_or_else(|| crate::err::Error::msg("No file name"))?;
            let cache_file_path = xdg_dirs.place_cache_file(path_name)?;
            let template_code = get_template_code(&path)?;

            if let Ok(cache_file_contents) = std::fs::read_to_string(&cache_file_path) {
                if cache_file_contents == template_code {
                    continue;
                }
            }

            std::fs::write(&cache_file_path, &template_code)?;
        }

        Ok(())
    }

    pub fn rerun_if_templates_changed() -> crate::err::Result<()> {
        let pkg_name = std::env::var("CARGO_PKG_NAME")?;
        let cache_dir = xdg::BaseDirectories::with_prefix(format!("erst/{}", &pkg_name))
            .map_err(crate::err::Error::msg)?
            .get_cache_home();

        for path in collect_paths(&cache_dir) {
            println!("cargo:rerun-if-changed={}", path.display());
        }

        Ok(())
    }

    fn collect_paths(path: impl AsRef<Path>) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let path_as_ref = path.as_ref();
        for entry in std::fs::read_dir(path_as_ref)
            .into_iter()
            .flat_map(|x| x)
            .flat_map(|x| x)
            .map(|x| x.path())
        {
            if entry.is_dir() {
                out.extend(collect_paths(&entry))
            } else {
                out.push(entry);
            }
        }
        out
    }

    fn template_paths() -> Vec<PathBuf> {
        super::utils::templates_dir().as_ref().map(collect_paths).unwrap_or_else(|_| Vec::new())
    }

    fn get_template_code(path: impl AsRef<Path>) -> crate::err::Result<String> {
        use crate::{
            exp::Parser as _,
            parser::{ErstParser, Rule},
        };

        let template = std::fs::read_to_string(&path)?;

        let mut buffer = String::from("{");

        let pairs = ErstParser::parse(Rule::template, &template)
            .map_err(|e| crate::err::Error::Parse(e.to_string()))?;

        for pair in pairs {
            match pair.as_rule() {
                Rule::code => {
                    buffer.push_str(pair.into_inner().as_str());
                }
                Rule::expr => {
                    buffer.push_str(pair.into_inner().as_str());
                    buffer.push_str(";");
                }
                _ => {}
            }
        }

        buffer.push_str("}");

        let block = syn::parse_str::<syn::Block>(&buffer).map_err(crate::err::Error::msg)?;

        let stmts = &block.stmts;

        Ok(quote::quote!(#(#stmts)*).to_string())
    }
}
