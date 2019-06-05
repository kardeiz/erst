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

Currently, only the `html` type (or none) is supported, with very basic HTML escaping. To unescape HTML content in your template file, wrap the content in `erst::Raw("<p>Hello</p>")`.