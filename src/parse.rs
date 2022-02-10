use std::io::Read;
use std::net::TcpStream;

use crate::types::{HttpParseError, Method, Request};

pub(crate) fn parse(stream: &mut TcpStream) -> Result<Request, HttpParseError> {
    let mut buf = [0u8; 4096];

    match stream.read(&mut buf) {
        Err(err) => Err(HttpParseError::Other(format!("{}", err))),
        Ok(_) => Ok(internal_parse(String::from_utf8_lossy(&buf).into_owned())?),
    }
}

fn internal_parse(req: String) -> Result<Request, HttpParseError> {
    // TODO: split properly on \r\n to read headers
    let mut strings = req.split("\r\n").next().unwrap().split(" ");

    let method = get_method(strings.next())?;
    let path = get_path(strings.next())?;

    validate_http_version(strings.next())?;
    // validate_crlf; adjust tests too

    Ok(Request { method, path })
}

fn validate_http_version(req: Option<&str>) -> Result<(), HttpParseError> {
    if let Some(version) = req {
        match version {
            "HTTP/1.1" | "HTTP/2.0" => Ok(()),
            _ => Err(HttpParseError::InvalidHttpVersion),
        }
    } else {
        Err(HttpParseError::InvalidHttpVersion)
    }
}

fn get_path(req: Option<&str>) -> Result<String, HttpParseError> {
    let string = req.ok_or(HttpParseError::InvalidPath)?.to_owned();

    if string.len() == 0 {
        Err(HttpParseError::InvalidPath)
    } else {
        Ok(string)
    }
}

fn get_method(method: Option<&str>) -> Result<Method, HttpParseError> {
    if let Some(method) = method {
        Ok(match method {
            "GET" => Method::Get,
            "DELETE" => Method::Delete,
            "HEAD" => Method::Head,
            "OPTIONS" => Method::Options,
            "PATCH" => Method::Patch,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            _ => return Err(HttpParseError::InvalidMethod),
        })
    } else {
        Err(HttpParseError::InvalidMethod)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_gets_method() {
        assert_eq!(get_method(None), Err(HttpParseError::InvalidMethod));
        assert_eq!(get_method(Some(" ")), Err(HttpParseError::InvalidMethod));
        assert_eq!(get_method(Some("")), Err(HttpParseError::InvalidMethod));
        assert_eq!(get_method(Some("get")), Err(HttpParseError::InvalidMethod));
        assert_eq!(get_method(Some("pUt")), Err(HttpParseError::InvalidMethod));
        assert_eq!(get_method(Some("rand")), Err(HttpParseError::InvalidMethod));

        assert_eq!(get_method(Some("GET")), Ok(Method::Get));
        assert_eq!(get_method(Some("POST")), Ok(Method::Post));
        assert_eq!(get_method(Some("PATCH")), Ok(Method::Patch));
        assert_eq!(get_method(Some("DELETE")), Ok(Method::Delete));
        assert_eq!(get_method(Some("PUT")), Ok(Method::Put));
        assert_eq!(get_method(Some("OPTIONS")), Ok(Method::Options));
        assert_eq!(get_method(Some("HEAD")), Ok(Method::Head));
    }

    #[test]
    fn it_gets_path() {
        assert_eq!(get_path(None), Err(HttpParseError::InvalidPath));
        assert_eq!(get_path(Some("")), Err(HttpParseError::InvalidPath));

        assert_eq!(get_path(Some("/a/path")), Ok("/a/path".to_owned()));
    }

    #[test]
    fn it_constructs_request() {
        assert_eq!(
            internal_parse("".to_owned()),
            Err(HttpParseError::InvalidMethod)
        );
        assert_eq!(
            internal_parse("g".to_owned()),
            Err(HttpParseError::InvalidMethod)
        );

        assert_eq!(
            internal_parse("GET".to_owned()),
            Err(HttpParseError::InvalidPath)
        );
        assert_eq!(
            internal_parse("GET   ".to_owned()),
            Err(HttpParseError::InvalidPath)
        );
        assert_eq!(
            internal_parse("GET /path".to_owned()),
            Err(HttpParseError::InvalidHttpVersion)
        );
        assert_eq!(
            internal_parse("GET /path HTTP/1.0".to_owned()),
            Err(HttpParseError::InvalidHttpVersion)
        );

        assert_eq!(
            internal_parse("GET /path HTTP/1.1".to_owned()),
            Ok(Request {
                method: Method::Get,
                path: "/path".to_owned()
            })
        )
    }
}
