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
        let bytes_len = bytes.len();
        let mut frm = 0;

        let mut escape = |frm: &mut usize, to: usize, rep| -> std::io::Result<()> {
            if *frm < to {
                self.write_slice(&bytes[*frm..to])
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            }
            self.0
                .write_str(rep)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            *frm = to + 1;
            Ok(())
        };

        for (idx, byte) in bytes.iter().enumerate() {
            match byte {
                b'<' => escape(&mut frm, idx, "&lt;")?,
                b'>' => escape(&mut frm, idx, "&gt;")?,
                b'&' => escape(&mut frm, idx, "&amp;")?,
                b'"' => escape(&mut frm, idx, "&quot;")?,
                b'\'' => escape(&mut frm, idx, "&#x27;")?,
                b'/' => escape(&mut frm, idx, "&#x2f;")?,
                _ => {}
            }
        }

        if frm < bytes_len {
            self.write_slice(&bytes[frm..bytes_len])
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        }

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
    crate::dynamic::rerun_if_templates_changed()
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
