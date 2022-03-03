use crate::types::{
    HeaderMethods, Headers, HttpParseError, HttpResponseError, HttpStatusCode, HttpVersion,
    LogError, Method, NormalizePath, RebarError, Request, TemplateError,
};

use std::collections::HashMap;
use std::error::Error;
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

impl Error for RebarError {}
impl Error for HttpParseError {}
impl Error for TemplateError {}

impl Display for RebarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                RebarError::ParseError(parse_err) => parse_err.to_string(),
                RebarError::TemplateError(template_err) => template_err.to_string(),
            }
        )
    }
}

impl Display for TemplateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                Self::NonexistentPath(p) =>
                    format!("Path {:?} either doesnt exist or is a directory", p),
                Self::ReadErr(err) => err.to_string(),

                Self::InvalidChar(c) => format!("Unexpected char '{}'", c),
                Self::EmptyVariableName => "Cannot have empty variable name".to_string(),
                Self::UnexpectedEof => "Unexpected end of input".to_string(),
                Self::UnterminatedBraces => "Unterminated '{{'".to_string(),

                Self::MissingVariable(varname) => format!("Missing variable `{}`", varname),
            }
        )
    }
}

impl Display for HttpParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidMethod => "Invalid method",
                Self::InvalidPath => "Invalid path",
                Self::InvalidHttpVersion => "Invalid http version",
                Self::InvalidHeaderSyntax => "Invalid header syntax",
                Self::Other(err) => err,
            }
        )
    }
}

impl Display for HttpStatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use HttpStatusCode::*;
        write!(
            f,
            "{}",
            match self {
                Code100 => "100 Continue",
                Code101 => "101 Switching Protocols",
                Code103 => "103 Early Hints",
                Code200 => "200 OK",
                Code201 => "201 Created",
                Code202 => "202 Accepted",
                Code203 => "203 Non-Authoritative Information",
                Code204 => "204 No Content",
                Code205 => "205 Reset Content",
                Code206 => "206 Partial Content",
                Code300 => "300 Multiple Choices",
                Code301 => "301 Moved Permanently",
                Code302 => "302 Found",
                Code303 => "303 See Other",
                Code304 => "304 Not Modified",
                Code307 => "307 Temporary Redirect",
                Code308 => "308 Permanent Redirect",
                Code400 => "400 Bad Request",
                Code401 => "401 Unauthorized",
                Code402 => "402 Payment Required",
                Code403 => "403 Forbidden",
                Code404 => "404 Not Found",
                Code405 => "405 Method Not Allowed",
                Code406 => "406 Not Acceptable",
                Code407 => "407 Proxy Authentication Required",
                Code408 => "408 Request Timeout",
                Code409 => "409 Conflict",
                Code410 => "410 Gone",
                Code411 => "411 Length Required",
                Code412 => "412 Precondition Failed",
                Code413 => "413 Payload Too Large",
                Code414 => "414 URI Too Long",
                Code415 => "415 Unsupported Media Type",
                Code416 => "416 Range Not Satisfiable",
                Code417 => "417 Expectation Failed",
                Code418 => "418 I'm a teapot",
                Code422 => "422 Unprocessable Entity",
                Code425 => "425 Too Early",
                Code426 => "426 Upgrade Required",
                Code428 => "428 Precondition Required",
                Code429 => "429 Too Many Requests",
                Code431 => "431 Request Header Fields Too Large",
                Code451 => "451 Unavailable For Legal Reasons",
                Code500 => "500 Internal Server Error",
                Code501 => "501 Not Implemented",
                Code502 => "502 Bad Gateway",
                Code503 => "503 Service Unavailable",
                Code504 => "504 Gateway Timeout",
                Code505 => "505 HTTP Version Not Supported",
                Code506 => "506 Variant Also Negotiates",
                Code507 => "507 Insufficient Storage",
                Code508 => "508 Loop Detected",
                Code510 => "510 Not Extended",
                Code511 => "511 Network Authentication Required",
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
            query: HashMap::new(),
            fragment: None,
            http_version: HttpVersion::Http1_1,

            headers: Headers(HashMap::new()),

            body: None,
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

impl NormalizePath for String {
    fn normalize(&self) -> (String, Option<String>, Option<String>) {
        fn fix_path(path: &str) -> String {
            if !path.ends_with("/") {
                path.to_owned() + "/"
            } else {
                path.to_owned()
            }
        }

        let mut iter = if self.contains('?') {
            self.splitn(2, '?')
        } else if self.contains('#') {
            self.splitn(2, '#')
        } else {
            return (fix_path(self), None, None);
        };

        let path = match iter.next() {
            Some(s) => fix_path(s),
            None => "/".to_owned(),
        };

        let query = match self.split('?').skip(1).next() {
            Some(s) => match s.split('#').next() {
                Some(s) if s.trim() != "" => Some(s.trim().to_owned()),
                _ => None,
            },
            None => None,
        };

        let fragment = match self.splitn(2, '#').skip(1).next() {
            Some(s) if s.trim() != "" => Some(s.trim().to_owned()),
            _ => None,
        };

        (path, query, fragment)
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

#[macro_export]
macro_rules! template_vars {
    {$($key:expr => $value:expr),* $(,)?} => {
        {
            {

                let mut map = std::collections::HashMap::new();
                $(
                    map.insert($key.into(), $value.into());
                )*
                map
            }
        }
    };
}
