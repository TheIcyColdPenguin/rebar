use crate::types::{
    HeaderMethods, Headers, HttpParseError, HttpVersion, Method, NormalizePath, Request,
};

use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;

pub(crate) fn parse(stream: &mut TcpStream) -> Result<Request, HttpParseError> {
    let mut buf = [0u8; 4096];

    match stream.read(&mut buf) {
        Err(err) => Err(HttpParseError::Other(format!("{}", err))),
        Ok(_) => Ok(internal_parse(String::from_utf8_lossy(&buf).into_owned())?),
    }
}

fn internal_parse(req: String) -> Result<Request, HttpParseError> {
    let mut block_iter = req.splitn(2, "\r\n\r\n");

    let head = match block_iter.next() {
        Some(s) => s,

        // it's empty so we simply say the method is invalid as it doesnt exist
        None => return Err(HttpParseError::InvalidMethod),
    };

    let body: Option<String> = match block_iter.next() {
        Some(x) if x.trim() != "" => Some(x.trim().to_owned()),
        _ => None,
    };

    let mut head_iter = head.split("\r\n");

    let mut strings = match head_iter.next() {
        Some(req_line) => req_line.split(" "),
        None => return Err(HttpParseError::InvalidMethod),
    };

    let method = get_method(strings.next())?;
    let (path, query, fragment) = get_path(strings.next())?;
    let http_version = get_http_version(strings.next())?;

    let mut headers = Headers(HashMap::new());

    while let Some(header_line) = head_iter.next() {
        if header_line.len() == 0 {
            break;
        }
        let mut header_iter = header_line.split(':');

        let header_name = match header_iter.next() {
            Some(s) if s.trim() != "" => s.trim(),
            _ => return Err(HttpParseError::InvalidHeaderSyntax),
        };
        let header_value = match header_iter.next() {
            Some(s) if s.trim() != "" => s.trim(),
            _ => return Err(HttpParseError::InvalidHeaderSyntax),
        };

        headers.set_header(header_name, header_value);
    }

    Ok(Request {
        method,
        path,
        query,
        fragment,
        http_version,

        headers,

        body,
    })
}

fn get_http_version(version: Option<&str>) -> Result<HttpVersion, HttpParseError> {
    match version {
        Some("HTTP/1.1") => Ok(HttpVersion::Http1_1),
        Some("HTTP/2.0") => Ok(HttpVersion::Http2_0),
        _ => Err(HttpParseError::InvalidHttpVersion),
    }
}

fn get_path(req: Option<&str>) -> Result<(String, Option<String>, Option<String>), HttpParseError> {
    let string = req.ok_or(HttpParseError::InvalidPath)?.to_owned();

    if string.len() == 0 {
        Err(HttpParseError::InvalidPath)
    } else {
        Ok(string.normalize())
    }
}

fn get_method(method: Option<&str>) -> Result<Method, HttpParseError> {
    Ok(match method {
        Some("GET") => Method::Get,
        Some("DELETE") => Method::Delete,
        Some("HEAD") => Method::Head,
        Some("OPTIONS") => Method::Options,
        Some("PATCH") => Method::Patch,
        Some("POST") => Method::Post,
        Some("PUT") => Method::Put,
        _ => return Err(HttpParseError::InvalidMethod),
    })
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

        assert_eq!(
            get_path(Some("/a/path/")),
            Ok(("/a/path/".to_owned(), None, None))
        );

        assert_eq!(
            get_path(Some("/a/path")),
            Ok(("/a/path/".to_owned(), None, None))
        );

        assert_eq!(
            get_path(Some("/a/path#hmm")),
            Ok(("/a/path/".to_owned(), None, Some("hmm".to_owned())))
        );

        assert_eq!(
            get_path(Some("/a/path?hmm=ok#hmm")),
            Ok((
                "/a/path/".to_owned(),
                Some("hmm=ok".to_owned()),
                Some("hmm".to_owned())
            ))
        );

        assert_eq!(
            get_path(Some("/a/path?hmm=ok")),
            Ok(("/a/path/".to_owned(), Some("hmm=ok".to_owned()), None))
        );
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
                path: "/path/".to_owned(),
                query: None,
                fragment: None,
                http_version: HttpVersion::Http1_1,

                headers: Headers(HashMap::new()),

                body: None,
            })
        );

        assert_eq!(
            internal_parse("GET /path?ok=1 HTTP/1.1".to_owned()),
            Ok(Request {
                method: Method::Get,
                path: "/path/".to_owned(),
                query: Some("ok=1".to_owned()),
                fragment: None,
                http_version: HttpVersion::Http1_1,

                headers: Headers(HashMap::new()),

                body: None,
            })
        );
    }

    #[test]
    fn it_constructs_more_requests_but_with_headers() {
        let mut headers = Headers(HashMap::new());
        headers
            .set_header("Content-Type", "text/html; charset=utf-8")
            .set_header("Host", "www.example.com");

        assert_eq!(
            internal_parse(
                "GET /path?ok=1 HTTP/1.1\r\nContent-Type:text/html; charset=utf-8\r\nHost: www.example.com\r\n\r\n"
                    .to_owned()
            ),
            Ok(Request {
                method: Method::Get,
                path: "/path/".to_owned(),
                query: Some("ok=1".to_owned()),
                fragment: None,
                http_version: HttpVersion::Http1_1,

                headers,

                body: None,
            } )
        );

        let mut headers = Headers(HashMap::new());
        headers
            .set_header("Content-Type", "text/html; charset=utf-8")
            .set_header("Host", "www.example.com");

        assert_eq!(
            internal_parse(
                "POST /path?ok=1 HTTP/1.1\r\nContent-Type:text/html; charset=utf-8\r\nHost: www.example.com\r\n\r\nok\r\n\r\nhmm"
                    .to_owned()
            ),
            Ok(Request {
                method: Method::Post,
                path: "/path/".to_owned(),
                query: Some("ok=1".to_owned()),
                fragment: None,
                http_version: HttpVersion::Http1_1,

                headers,

                body: Some("ok\r\n\r\nhmm".to_owned()),
            } )
        );
    }
}
