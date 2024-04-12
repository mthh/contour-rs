use std::{error::Error as StdError, fmt, result};

/// A crate private constructor for `Error`.
pub(crate) fn new_error(kind: ErrorKind) -> Error {
    Error(Box::new(kind))
}

/// A type alias for `Result<T, Error>`.
pub type Result<T> = result::Result<T, Error>;

/// An error that can occur when computing contours.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    /// Return the specific type of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    /// Unwrap this error into its underlying type.
    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

/// The specific type of an error.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    BadDimension,
    Unexpected,
    BadCast,
    #[cfg(feature = "geojson")]
    JsonError(serde_json::error::Error),
}

#[cfg(feature = "geojson")]
impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Error {
        new_error(ErrorKind::JsonError(err))
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self.0 {
            ErrorKind::BadDimension => None,
            ErrorKind::Unexpected => None,
            ErrorKind::BadCast => None,
            #[cfg(feature = "geojson")]
            ErrorKind::JsonError(ref err) => Some(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorKind::BadDimension => write!(
                f,
                "The length of provided values doesn't match the (dx, dy) dimensions of the grid"
            ),
            ErrorKind::Unexpected => write!(f, "Unexpected error while computing contours"),
            ErrorKind::BadCast => write!(f, "Failed to cast grid value to Float"),
            #[cfg(feature = "geojson")]
            ErrorKind::JsonError(ref err) => err.fmt(f),
        }
    }
}
