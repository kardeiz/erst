pub use erst_derive::Template;
pub use erst_shared as shared;

use std::fmt::{Display, Write};

pub trait Template {
    fn render_into(&self, writer: &mut std::fmt::Write) -> std::fmt::Result;
}

pub struct Html<T>(pub T);
pub struct Raw<T>(pub T);

impl<T> Display for Html<T>
where
    T: Display,
{
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
                c => f.write_char(c),
            }?;
        }
        Ok(())
    }
}

impl<T> Display for Html<Raw<T>>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Html(Raw(ref inner)) = *self;
        Display::fmt(inner, f)
    }
}

#[cfg(feature = "dynamic")]
pub mod dynamic {

    use std::collections::HashMap;

    pub fn get(path: &str, idx: usize) -> Option<String> {
        use std::sync::{Arc, Mutex};

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

    fn parse(path: &str) -> Result<HashMap<usize, String>, Box<std::error::Error>> {
        use erst_shared::{
            exp::Parser as _,
            parser::{ErstParser, Rule},
        };

        let template = std::fs::read_to_string(&path)?;

        let pairs = ErstParser::parse(Rule::template, &template)?;

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
