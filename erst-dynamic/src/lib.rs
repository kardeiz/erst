use erst_shared::err;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::process::Command;

pub fn compile(path: impl AsRef<Path>) -> err::Result<()> {

	let deps_dir = std::env::var("CARGO_MANIFEST_DIR")
		.map(PathBuf::from)
		.map_err(err::Error::boxed)?
		.join("target/debug/deps");

	let mut rlibs = std::fs::read_dir(&deps_dir).into_iter()
		.flatten()
		.flatten()
		.flat_map(|x| match x.metadata() { Ok(md) => Some((x, md)), _ => None })
		.map(|(x, md)| (x.path(), md) )
		.filter(|(x, md)| x.extension() == Some("rlib".as_ref()))
		.flat_map(|(x, md)| match x.file_stem() { 
			Some(stem) => {
				let name = stem.to_string_lossy();
				let name = name.split('-').next().unwrap();
				let name = if name.starts_with("lib") {
					name.replacen("lib", "", 1)
				} else {
					name.to_string()
				};
				Some((name, x, md.modified().ok()))
			}
			_ => None 
		})
		.collect::<Vec<_>>();

	rlibs.sort_by(|(an, _, amd), (bn, _, bmd)| (&an, amd).cmp(&(&bn, bmd)) );

	let rlibs = rlibs.into_iter().map(|(n, x, _)| (n, x)).collect::<HashMap<_, _>>();

	println!("{:?}", std::env::var("CARGO_PKG_NAME"));

	let mut compilation = Command::new("rustc");

	compilation
	    .arg("--crate-type")
	    .arg("dylib")
	    .arg(path.as_ref())
	    .arg("-L")
	    .arg(&deps_dir)
	    .arg("-o")
	    .arg("/tmp/out.so");

	for (k, v) in rlibs {
		compilation
			.arg("--extern")
			.arg(format!("{}={}", k, v.display()));
	}

	let compilation = compilation
	    .output()
	    .unwrap();

	println!("{:?}", &compilation);

	Ok(())

}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        crate::compile("../../test2.rs").unwrap();
    }
}
