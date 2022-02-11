use crate::types::{
    HeaderMethods, Headers, HttpResponseError, HttpStatusCode, HttpVersion, LogError, Method,
    Request,
};

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
                HttpStatusCode::Code400 => "400 Bad Request",
            }
        )
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut string = String::new();

        for (key, value) in &self.0 {
            string.push_str(format!("{}: {}\r\n", key, value).as_str())
        }

        write!(f, "{}", string)
    }
}

impl Default for Request {
    fn default() -> Request {
        Request {
            method: Method::Get,
            path: "/".into(),
            http_version: HttpVersion::Http1_1,
        }
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

impl<T> HeaderMethods<T> for Headers
where
    T: Into<String>,
{
    fn get_header(&self, name: T) -> Option<&String> {
        self.0.get(&name.into().to_lowercase())
    }

    fn set_header(&mut self, name: T, value: T) -> &mut Self {
        self.0.insert(name.into().to_lowercase(), value.into());

        self
    }

    fn remove_header(&mut self, name: T) -> &mut Self {
        self.0.remove(&name.into().to_lowercase());

        self
    }
}
