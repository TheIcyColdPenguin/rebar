use crate::parse::parse;
use crate::types::{HeaderMethods, Headers, HttpStatusCode, LogError, Request, Response, Server};

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

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

impl<F> Server<F>
where
    F: Fn(&Request, &mut Response) -> Result<(), HttpStatusCode> + Send,
{
    pub fn new(on: &str) -> Server<F> {
        Server {
            listener: TcpListener::bind(on).unwrap(),
            handler: Arc::new(Mutex::new(None)),
        }
    }

    pub fn on_all(&mut self, handler: F) {
        self.handler = Arc::new(Mutex::new(Some(handler)));
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let handler = self.handler.clone();
        thread::spawn(move || match parse(&mut stream) {
            Ok(req) => {
                // temporary setup,
                // TODO: use closures
                let mut res = create_response(stream, &req);

                let handler = handler.lock().unwrap();
                if let Some(handler) = handler.as_ref() {
                    match handler(&req, &mut res) {
                        Ok(_) => {}
                        Err(code) => res.status = code,
                    }
                }

                res.send().log_error();
            }
            Err(_) => {
                let mut res = create_response(stream, &Default::default());
                res.status = HttpStatusCode::Code400;
                res.send().log_error();
            }
        });
    }

    pub fn listen_once(&mut self) {
        match self.listener.accept() {
            Ok((stream, _)) => self.handle_connection(stream),
            Err(err) => println!("Error: {:?}", err),
        }
    }

    pub fn listen(&mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => self.handle_connection(stream),
                Err(err) => println!("Error: {:?}", err),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::Method;

    use http::header::{CONTENT_TYPE, SERVER};

    const ADDRESS: &str = "localhost:3004";

    #[test]
    fn it_responds_to_request() {
        let handle = thread::spawn(|| {
            let mut server = Server::new(ADDRESS);

            server.on_all(|req, res| {
                if req.method == Method::Get && req.path == "/" {
                    res.body = "ok".into();
                    res.headers.set_header("Server", "Rebar");

                    Ok(())
                } else {
                    Err(HttpStatusCode::Code400)
                }
            });

            server.listen_once();
        });

        let response = reqwest::blocking::get(format!("http://{}", ADDRESS)).unwrap();
        let headers = response.headers();
        assert!(headers.contains_key(CONTENT_TYPE));
        assert!(headers.contains_key(SERVER));
        assert_eq!(headers[CONTENT_TYPE], "text/html; charset=utf-8");
        assert_eq!(headers[SERVER], "Rebar");

        assert_eq!(response.text().unwrap(), "ok".to_owned());

        handle.join().unwrap();
    }
}
