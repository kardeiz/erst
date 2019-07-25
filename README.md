# Erst: Embedded Rust Templates

A tiny library for creating string templates, similar to ERB and JSP (angle-bracket-percent tags).

Precompiled along the lines of [Askama](https://github.com/djc/askama).

## Usage:

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

```erb
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

And then call it like:

    ERST_TEMPLATES_DIR=/path/to/folder-containing-simple-erst cargo run

By default, the template's `path` will resolve to a file inside a `templates` directory in the current project context (i.e., `CARGO_MANIFEST_DIR`).

Note that, unlike `Askama` and other template systems, you need to reference any struct members with `self`. The template file is basically a function that takes `&self` (where `self` is the linked container object).

Currently, only the `html` type (or none) is supported, with very basic HTML escaping. To unescape HTML content in your template file, wrap the content in `erst::Raw("<p>Hello</p>")`.

## Dynamic

This library also provides a way to avoiding (re-)compiling the string-y parts of your template.

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
    erst::rerun_if_templates_changed().unwrap();
}
```

And one more step: 

    cargo install erst-prepare

`erst-prepare` is a small binary that copies the code part of your templates to `$XDG_CACHE_HOME` so that the build script can re-run on changes only to the code part of your templates.

Then run your project like:

    erst-prepare && cargo run

If you have a unique setup, you may need to use the `--pkg-name` and `--templates-dir` flags to `erst-prepare`:

    erst-prepare --pkg-name my-project --templates-dir /path/to/your/templates/dir

