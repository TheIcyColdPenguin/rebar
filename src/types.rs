use std::collections::HashMap;
use std::marker::Send;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

pub struct Server<F>
where
    F: Fn(&Request, &mut Response) -> Result<(), HttpStatusCode> + Send + 'static,
{
    pub(crate) listener: TcpListener,
    pub(crate) handler: Arc<Mutex<Option<F>>>,
}

#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Head,
    Delete,
    Options,
    Patch,
}

#[derive(Debug, PartialEq)]
pub(crate) enum HttpParseError {
    InvalidMethod,
    InvalidPath,
    InvalidHttpVersion,
    InvalidHeaderSyntax,

    Other(String),
}

#[derive(Debug, PartialEq)]
pub enum HttpResponseError {
    Other(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum HttpVersion {
    Http1_1,
    Http2_0,
}

#[derive(Debug, PartialEq)]
pub struct Request {
    pub method: Method,
    pub http_version: HttpVersion,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,

    pub headers: Headers,

    pub body: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum HttpStatusCode {
    Code100,
    Code101,
    Code103,
    Code200,
    Code201,
    Code202,
    Code203,
    Code204,
    Code205,
    Code206,
    Code300,
    Code301,
    Code302,
    Code303,
    Code304,
    Code307,
    Code308,
    Code400,
    Code401,
    Code402,
    Code403,
    Code404,
    Code405,
    Code406,
    Code407,
    Code408,
    Code409,
    Code410,
    Code411,
    Code412,
    Code413,
    Code414,
    Code415,
    Code416,
    Code417,
    Code418,
    Code422,
    Code425,
    Code426,
    Code428,
    Code429,
    Code431,
    Code451,
    Code500,
    Code501,
    Code502,
    Code503,
    Code504,
    Code505,
    Code506,
    Code507,
    Code508,
    Code510,
    Code511,
}

#[derive(Debug, PartialEq)]
pub struct Headers(pub HashMap<String, String>);

pub struct Response {
    pub(crate) stream: TcpStream,

    pub headers: Headers,
    pub http_version: HttpVersion,
    pub status: HttpStatusCode,
    pub body: Vec<u8>,
}

pub(crate) trait LogError {
    fn log_error(&self);
}

pub trait HeaderMethods<T>
where
    T: Into<String>,
{
    fn get_header(&self, name: T) -> Option<&String>;
    fn set_header(&mut self, name: T, value: T) -> &mut Self;
    fn remove_header(&mut self, name: T) -> &mut Self;
}

pub(crate) trait NormalizePath {
    fn normalize(&self) -> (String, Option<String>, Option<String>);
}
