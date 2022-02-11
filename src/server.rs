use crate::parse::parse;
use crate::types::{HttpStatusCode, LogError, Request, Response, Server};

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
                response.send().log_error();
            }
            Err(err) => println!("Error: {:?}", err),
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
        Response {
            stream: stream,

            status: HttpStatusCode::Code200,
            http_version: req.http_version.clone(),
            headers: HashMap::new(),
            body: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ADDRESS: &str = "localhost:3004";

    #[test]
    fn it_responds_to_request() {
        let handle = thread::spawn(|| {
            Server::new(ADDRESS).listen_once();
        });

        reqwest::blocking::get(format!("http://{}/ok", ADDRESS)).unwrap();

        handle.join().unwrap();
    }
}
