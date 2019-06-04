pub use erst_derive::Template;
pub use erst_shared::err;

use std::fmt::{Display, Write};

pub trait Template {
	fn render_into(&self, writer: &mut std::fmt::Write) -> err::Result<()>;
}

pub struct Html<T>(pub T);
pub struct Raw<T>(pub T);

impl<T> Display for Html<T> where T: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let inner = self.0.to_string();
        for c in inner.chars() {
        	match c {
        		'<' => f.write_str("&lt;"),
                '>' => f.write_str("&gt;"),
                '&' => f.write_str("&amp;"),
                '"' => f.write_str("&quot;"),
                '\'' => f.write_str("&#x27;"),
                '/' => f.write_str("&#x2f;"),
                c => f.write_char(c)
        	}?;
        }
        Ok(())
    }
}

impl<T> Display for Html<Raw<T>> where T: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Html(Raw(ref inner)) = *self;
        Display::fmt(inner, f)
    }
}