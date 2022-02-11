use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};

pub struct Server {
    pub(crate) listener: TcpListener,
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
}

#[derive(Debug, PartialEq, Clone)]
pub enum HttpStatusCode {
    Code200,
    Code400,
    // TODO: add the rest
}

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
