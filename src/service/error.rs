#[derive(Debug)]
pub enum Error {
    BufferedService {
        source: Box<tower::buffer::error::ServiceError>,
    },
    Buffer {
        source: Box<tower::buffer::error::Closed>,
    },
    Unknown,
}

impl From<tower::BoxError> for Error {
    fn from(err: tower::BoxError) -> Self {
        let err = match err.downcast::<tower::buffer::error::Closed>() {
            Ok(closed) => return Self::Buffer { source: closed },
            Err(err) => err,
        };
        let _ = match err.downcast::<tower::buffer::error::ServiceError>() {
            Ok(err) => return Self::BufferedService { source: err },
            Err(err) => err,
        };

        Self::Unknown
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("service failure")
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::BufferedService { source } => Some(source.as_ref()),
            Error::Buffer { source } => Some(source.as_ref()),
            Error::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    struct IsSized<S: ?Sized> {
        p: PhantomData<S>,
    }

    #[test]
    fn error_sized() {
        let _ = IsSized::<super::Error> { p: PhantomData };
    }
}
