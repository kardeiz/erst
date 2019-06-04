#[macro_use]
extern crate pest_derive;

pub mod err {
	#[derive(Debug, derive_more::Display, derive_more::From)]
	pub enum Error {
		#[display(fmt = "{}", _0)]
		Io(std::io::Error),
		Syn(syn::Error),
		Fmt(std::fmt::Error),
		Boxed(Box<std::error::Error + Send + Sync>),
	}

	impl Error {
		pub fn boxed<T: std::error::Error + Send + Sync + 'static>(inner: T) -> Self {
			Error::Boxed(Box::new(inner))
		}
	}

	impl std::error::Error for Error {}

	pub type Result<T> = std::result::Result<T, Error>;
}

pub mod parser;

pub mod exp {
	pub use pest::Parser;
}
