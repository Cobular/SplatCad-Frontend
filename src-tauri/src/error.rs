use generic_error::GenericError;

pub enum Error {
  Generic(GenericError),
}

impl From<GenericError> for Error {
  fn from(error: GenericError) -> Self {
    Error::Generic(error)
  }
}