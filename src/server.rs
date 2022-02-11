use crate::parse::parse;
use crate::types::{HeaderMethods, Headers, HttpStatusCode, LogError, Request, Response, Server};

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::thread;

impl Server {
    pub fn new(on: &str) -> Server {
        Server {
            listener: TcpListener::bind(on).unwrap(),
        }
    }

    fn handle_connection(mut stream: TcpStream) {
        thread::spawn(move || match parse(&mut stream) {
            Ok(req) => {
                // temporary setup,
                // TODO: use closures
                let mut response = Server::create_response(stream, &req);
                response.body = "ok".into();
                response.headers.set_header("Server", "Rebar");
                response.send().log_error();
            }
            Err(_) => {
                let mut response = Server::create_response(stream, &Default::default());
                response.status = HttpStatusCode::Code400;
                response.send().log_error();
            }
        });
    }

    pub fn listen_once(&mut self) {
        match self.listener.accept() {
            Ok((stream, _)) => Server::handle_connection(stream),
            Err(err) => println!("Error: {:?}", err),
        }
    }

    pub fn listen(&mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => Server::handle_connection(stream),
                Err(err) => println!("Error: {:?}", err),
            }
        }
    }

    fn create_response(stream: TcpStream, req: &Request) -> Response {
        let mut headers = Headers(HashMap::new());

        headers.set_header("Content-Type", "text/html; charset=utf-8");

        Response {
            stream: stream,

            headers,
            status: HttpStatusCode::Code200,
            http_version: req.http_version.clone(),
            body: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use http::header::{CONTENT_TYPE, SERVER};

    const ADDRESS: &str = "localhost:3004";

    #[test]
    fn it_responds_to_request() {
        let handle = thread::spawn(|| {
            Server::new(ADDRESS).listen_once();
        });

        let response = reqwest::blocking::get(format!("http://{}/ok", ADDRESS)).unwrap();
        let headers = response.headers();
        assert!(headers.contains_key(CONTENT_TYPE));
        assert!(headers.contains_key(SERVER));
        assert_eq!(headers[CONTENT_TYPE], "text/html; charset=utf-8");
        assert_eq!(headers[SERVER], "Rebar");

        assert_eq!(response.text().unwrap(), "ok".to_owned());

        handle.join().unwrap();
    }
}
