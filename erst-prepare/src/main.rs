fn main() -> Result<(), Box<std::error::Error>> {
    let mut pkg_name = None;
    let mut templates_dir = None;

    for arg in std::env::args() {
        match (arg.as_str(), &pkg_name, &templates_dir) {
            ("--pkg-name", &None, _) => {
                pkg_name = Some(None);
            }
            ("--templates-dir", _, &None) => {
                templates_dir = Some(None);
            }
            (x, &Some(None), _) => {
                pkg_name = Some(Some(String::from(x)));
            }
            (x, _, &Some(None)) => {
                templates_dir = Some(Some(String::from(x)));
            }
            (_, &None, &None) => {}
            _ => {
                return Err("Invalid arguments to erst-prepare".into());
            }
        }
    }

    if let Some(Some(pkg_name)) = pkg_name {
        std::env::set_var("CARGO_PKG_NAME", pkg_name);
    } else {
        if let Ok(cargo_toml_str) = std::fs::read_to_string("Cargo.toml") {
            if let Ok(cargo_toml) = cargo_toml_str.parse::<toml::Value>() {
                if let Some(pkg_name) = cargo_toml["package"]["name"].as_str() {
                    std::env::set_var("CARGO_PKG_NAME", pkg_name);
                }
            }
        }

        if std::env::var("CARGO_PKG_NAME").is_err() {            
            return Err("Missing --pkg-name argument to erst-prepare".into());
        }
    }

    if let Some(Some(templates_dir)) = templates_dir {
        std::env::set_var("ERST_TEMPLATES_DIR", templates_dir);
    } else if erst_shared::utils::templates_dir().is_err() {
        return Err("Missing --templates-dir argument to erst-prepare".into());
    }

    erst_shared::dynamic::generate_code_cache()?;

    Ok(())
}
