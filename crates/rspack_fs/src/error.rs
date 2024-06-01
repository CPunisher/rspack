use std::fmt::Display;

#[cfg(feature = "rspack-error")]
use rspack_error::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
};

#[cfg(feature = "rspack-error")]
#[derive(Debug, Error, Diagnostic)]
#[error("Rspack FS Error: {0}")]
struct FsError(#[source] std::io::Error);

#[derive(Debug, Error)]
pub enum Error {
  /// Generic I/O error
  Io(std::io::Error),
}

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
    Self::Io(value)
  }
}

#[cfg(feature = "rspack-error")]
impl From<Error> for rspack_error::Error {
  fn from(value: Error) -> Self {
    match value {
      Error::Io(err) => FsError(err).into(),
    }
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Io(err) => write!(f, "IO error: {err}"),
    }
  }
}

pub type Result<T> = std::result::Result<T, Error>;
