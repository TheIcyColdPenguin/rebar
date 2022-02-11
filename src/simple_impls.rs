use crate::types::{HttpResponseError, HttpStatusCode, HttpVersion, LogError};

use std::fmt::{Display, Formatter, Result as FmtResult};

impl Display for HttpVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                HttpVersion::Http1_1 => "HTTP/1.1",
                HttpVersion::Http2_0 => "HTTP/2.0",
            }
        )
    }
}

impl Display for HttpStatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                HttpStatusCode::Code200 => "200 OK",
            }
        )
    }
}

impl<T> LogError for Result<T, HttpResponseError> {
    fn log_error(&self) {
        match self {
            Ok(_) => {}
            Err(err) => println!("Error {:?}", err),
        }
    }
}
