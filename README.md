# erst

A small library for creating string templates, similar to [eRuby](https://ruby-doc.org/stdlib/libdoc/erb/rdoc/ERB.html)
and [JSP](https://en.wikipedia.org/wiki/JavaServer_Pages) (uses angle-bracket-percent tags: `<%= expr %>`).

Templates are precompiled for speed and safety, though partial dynamic rendering is provided when the `dynamic` flag is
enabled (some setup is required).

## Usage

```rust
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

```rust
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

```rust
erst::Raw("<p>Hello</p>")
```

## Dynamic

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

```rust
cargo install erst-prepare
```

erst-prepare is a small binary that copies the code part of your templates to `$XDG_CACHE_HOME`
so that the build script can re-run on changes only to the code part of your templates.

Then run your project like:

```rust
erst-prepare && cargo run
```

If you have a unique setup, you may need to use the `--pkg-name` and `--templates-dir` flags to `erst-prepare`:

```rust
erst-prepare --pkg-name my-project --templates-dir /path/to/your/templates/dir
```

If you are using `dynamic` and do not run `erst-prepare` before building, it is possible that
your templates will not render correctly.

License: MIT
