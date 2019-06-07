#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate quote;

pub mod parser {
    #[derive(Parser)]
    #[grammar = "erst.pest"]
    pub struct ErstParser;
}

pub mod exp {
    pub use pest::Parser;
}

pub mod utils {

    use std::path::{Path, PathBuf};

    pub fn templates_dir() -> Result<PathBuf, std::env::VarError> {
        let out = std::env::var("ERST_TEMPLATES_DIR").map(PathBuf::from).or_else(|_| {
            std::env::var("CARGO_MANIFEST_DIR").map(|x| PathBuf::from(x).join("templates"))
        })?;
        Ok(out)
    }

    #[cfg(feature = "dynamic")]
    pub fn generate_code_cache() -> Result<(), Box<std::error::Error>> {
        let pkg_name = std::env::var("CARGO_PKG_NAME")?;
        let xdg_dirs = xdg::BaseDirectories::with_prefix(format!("erst/{}", &pkg_name))?;

        for path in template_paths() {
            let path_name = path.file_name().ok_or_else(|| "No file name")?;
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

    #[cfg(feature = "dynamic")]
    pub fn rerun_if_templates_changed() -> Result<(), Box<std::error::Error>> {
        let pkg_name = std::env::var("CARGO_PKG_NAME")?;
        let cache_dir =
            xdg::BaseDirectories::with_prefix(format!("erst/{}", &pkg_name))?.get_cache_home();

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
        templates_dir().as_ref().map(collect_paths).unwrap_or_else(|_| Vec::new())
    }

    fn get_template_code(path: impl AsRef<Path>) -> Result<String, Box<std::error::Error>> {
        use crate::{
            exp::Parser as _,
            parser::{ErstParser, Rule},
        };

        let template = std::fs::read_to_string(&path)?;

        let mut buffer = String::from("{");

        let pairs = ErstParser::parse(Rule::template, &template)?;

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

        let block = syn::parse_str::<syn::Block>(&buffer)?;

        let stmts = &block.stmts;

        Ok(quote!(#(#stmts)*).to_string())
    }

}
