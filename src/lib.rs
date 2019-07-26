pub use erst_derive::Template;

use std::fmt::{Display, Write};

pub trait Template {
    fn render_into(&self, writer: &mut std::fmt::Write) -> std::fmt::Result;

    fn size_hint() -> usize;

    fn render(&self) -> Result<String, std::fmt::Error> {
        let mut buffer = String::with_capacity(Self::size_hint());
        self.render_into(&mut buffer)?;
        Ok(buffer)
    }
}

pub struct Html<T>(pub T);
pub struct Raw<T>(pub T);

pub struct HtmlWriter<'a, 'b: 'a>(&'a mut std::fmt::Formatter<'b>);

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
            if byte < &b'"' || byte > &b'>' { continue; }

            let rep_opt = match byte {
                b'<' => Some("&lt;"),
                b'>' => Some("&gt;"),
                b'&' => Some("&amp;"),
                b'"' => Some("&quot;"),
                b'\'' => Some("&#x27;"),
                b'/' => Some("&#x2f;"),
                _ => { None }
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

#[cfg(feature = "dynamic")]
pub fn rerun_if_templates_changed() ->  erst_shared::err::Result<()> {
    dynamic::rerun_if_templates_changed()
}

#[cfg(not(feature = "dynamic"))]
pub fn rerun_if_templates_changed() ->  erst_shared::err::Result<()> {
    Ok(())
}

#[cfg(feature = "dynamic")]
pub mod dynamic {

    pub use erst_shared::dynamic::*;

    use std::collections::HashMap;

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
            .map_err(|e| erst_shared::err::Error::Parse(e.to_string()) )?;

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
