/*!
A small library for creating string templates, similar to [eRuby](https://ruby-doc.org/stdlib/libdoc/erb/rdoc/ERB.html)
and [JSP](https://en.wikipedia.org/wiki/JavaServer_Pages) (uses angle-bracket-percent tags: `<%= expr %>`).

Templates are precompiled for speed and safety, though partial dynamic rendering is provided when the `dynamic` flag is
enabled (some setup is required).

# Usage

```rust,no_run
use erst::Template;

#[derive(Template)]
#[template(path = "simple.erst", type = "html")]
pub struct Container<'a> {
    pub collection: Vec<&'a str>,
}

fn main() {
    println!("{}", Container { collection: vec!["Hello", "<>", "World"] });
}
```

Where `simple.erst` looks like:

```
<div>
    <p>Hello!</p>
    <%
        let desc = format!("Here is your list of {} items:", self.collection.len());
    -%>
    <p><%= desc %></p>
    <ul>
        <% for x in &self.collection { -%>
            <li><%= x %></li>
        <%- } %>
    </ul>
</div>
```

By default, the template's `path` will resolve to a file inside a `templates` directory in the current project context
(i.e., `CARGO_MANIFEST_DIR`). If you need to change this, you can set the `ERST_TEMPLATES_DIR` env variable to the
appropriate path. Note that this is only a concern when building; since the templates are compiled into your binary,
you don't need this structure/environment variables when running a compiled binary.

Note that, unlike `Askama` and many other template systems, you need to reference any members of your `Template` item
with `self` inside the template file. The template file is basically the body of a function that takes `&self`
(where `self` is the linked `Container` object).

Note that, like Askama and other precompiled template systems, you can reference any item (structs, functions, etc.)
available in your crate.

Currently, only the `html` type (or none) is supported, with very basic HTML escaping. To unescape HTML content in your
template file, wrap the content in [Raw](struct.Raw.html), e.g.:

```rust,no_run
erst::Raw("<p>Hello</p>")
```

# Dynamic

This library also provides a way to avoiding (re-)compiling the static/non-Rust parts of your template.

To enable this feature, add the following to your `Cargo.toml`:

```toml
[dependencies]
erst = { version = "0.2", features = ["dynamic"] }

[build-dependencies]
erst = { version = "0.2", features = ["dynamic"] }
```

And add a `build.rs` file with the following line:

```rust
fn main() {
    // This function is a no-op when the `dynamic` feature is not enabled.
    // It is safe to leave this in `build.rs` even when not using `dynamic`
    erst::rerun_if_templates_changed().unwrap();
}
```

You must also install a helper binary, [erst-prepare](https://crates.io/crates/erst-prepare):

```
cargo install erst-prepare
```

erst-prepare is a small binary that copies the code part of your templates to `$XDG_CACHE_HOME`
so that the build script can re-run on changes only to the code part of your templates.

Then run your project like:

```
erst-prepare && cargo run
```

If you have a unique setup, you may need to use the `--pkg-name` and `--templates-dir` flags to `erst-prepare`:

```
erst-prepare --pkg-name my-project --templates-dir /path/to/your/templates/dir
```

If you are using `dynamic` and do not run `erst-prepare` before building, it is possible that
your templates will not render correctly.
*/

pub use erst_derive::Template;

use std::fmt::{Display, Write};

/// The rendering trait derived by the proc macro
pub trait Template {
    fn render_into(&self, writer: &mut std::fmt::Write) -> std::fmt::Result;

    fn size_hint() -> usize;

    fn render(&self) -> Result<String, std::fmt::Error> {
        let mut buffer = String::with_capacity(Self::size_hint());
        self.render_into(&mut buffer)?;
        Ok(buffer)
    }
}

#[doc(hidden)]
pub struct Html<T>(pub T);

/// Wrap any `Display` content in this tuple struct to unescape any included HTML
pub struct Raw<T>(pub T);

struct HtmlWriter<'a, 'b: 'a>(&'a mut std::fmt::Formatter<'b>);

impl HtmlWriter<'_, '_> {
    fn write_slice(&mut self, bytes: &[u8]) -> std::fmt::Result {
        let bstr = erst_shared::exp::B(bytes);
        if let Ok(s) = bstr.to_str() {
            self.0.write_str(s)?;
        } else {
            for chr in bstr.chars() {
                self.0.write_char(chr)?;
            }
        }
        Ok(())
    }
}

impl std::io::Write for HtmlWriter<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        use std::io::{Error, ErrorKind};

        let mut from = 0;

        for (idx, byte) in bytes.into_iter().enumerate() {
            if byte < &b'"' || byte > &b'>' {
                continue;
            }

            let rep_opt = match byte {
                b'<' => Some("&lt;"),
                b'>' => Some("&gt;"),
                b'&' => Some("&amp;"),
                b'"' => Some("&quot;"),
                b'\'' => Some("&#x27;"),
                b'/' => Some("&#x2f;"),
                _ => None,
            };

            if let Some(rep) = rep_opt {
                self.write_slice(&bytes[from..idx]).map_err(|e| Error::new(ErrorKind::Other, e))?;
                self.0.write_str(rep).map_err(|e| Error::new(ErrorKind::Other, e))?;
                from = idx + 1;
            }
        }

        let bytes_len = bytes.len();

        self.write_slice(&bytes[from..bytes_len]).map_err(|e| Error::new(ErrorKind::Other, e))?;

        Ok(bytes_len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<T> Display for Html<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::io::Write;
        write!(HtmlWriter(f), "{}", &self.0).map_err(|_| std::fmt::Error)?;
        Ok(())
    }
}

impl<T> Display for Html<Raw<T>>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Html(Raw(ref inner)) = *self;
        inner.fmt(f)
    }
}

/// Creates a code cache for templates in `$XDG_CACHE_HOME` when using `dynamic`, to
/// indicate when code needs to be re-compiled.
///
/// Is a no-op when `dynamic` is not enabled, and is safe to leave in `build.rs`.
#[cfg(feature = "dynamic")]
pub fn rerun_if_templates_changed() -> erst_shared::err::Result<()> {
    erst_shared::dynamic::rerun_if_templates_changed()
}

/// Creates a code cache for templates in `$XDG_CACHE_HOME` when using `dynamic`, to
/// indicate when code needs to be re-compiled.
///
/// Is a no-op when `dynamic` is not enabled, and is safe to leave in `build.rs`.
#[cfg(not(feature = "dynamic"))]
pub fn rerun_if_templates_changed() -> erst_shared::err::Result<()> {
    Ok(())
}

#[doc(hidden)]
#[cfg(feature = "dynamic")]
pub mod dynamic {

    use std::collections::HashMap;

    #[doc(hidden)]
    pub fn get(path: &str, idx: usize) -> Option<String> {
        use std::sync::Mutex;

        lazy_static::lazy_static! {
            static ref MAP: Mutex<HashMap<String, HashMap<usize, String>>> = Mutex::new(HashMap::new());
        }

        if let Ok(lock) = MAP.lock() {
            if let Some(inner_map) = lock.get(path) {
                return inner_map.get(&idx).cloned();
            }
        }

        if let Ok(mut lock) = MAP.lock() {
            if let Ok(inner_map) = parse(path) {
                lock.insert(path.into(), inner_map.clone());
                if let Some(out) = inner_map.get(&idx) {
                    return Some(out.clone());
                }
            }
        }

        None
    }

    fn parse(path: &str) -> erst_shared::err::Result<HashMap<usize, String>> {
        use erst_shared::{
            exp::Parser as _,
            parser::{ErstParser, Rule},
        };

        let template = std::fs::read_to_string(&path)?;

        let pairs = ErstParser::parse(Rule::template, &template)
            .map_err(|e| erst_shared::err::Error::Parse(e.to_string()))?;

        let mut map = HashMap::new();

        for (idx, pair) in pairs.enumerate() {
            match pair.as_rule() {
                Rule::text => {
                    map.insert(idx, pair.as_str().into());
                }
                _ => {}
            }
        }
        Ok(map)
    }

}
