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
    F: Fn(&Request, &mut Response) -> HttpStatusCode + Send + 'static,
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
                let mut res = create_response(stream, &req);

                let handler = handler.lock().unwrap();
                if let Some(handler) = handler.as_ref() {
                    res.status = handler(&req, &mut res);
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

    use crate::template_vars;
    use crate::types::{Method, Template};

    use http::header::{CONTENT_TYPE, SERVER};
    use http::StatusCode;

    const ADDRESS: &str = "localhost:3004";

    #[test]
    fn it_responds_to_request() {
        let handle = thread::spawn(|| {
            let mut server = Server::new(ADDRESS);
            let template = Template::new("./static/index.html").unwrap();

            server.on_all(move |req, res| match (&req.method, req.path.as_str()) {
                (Method::Get, "/interesting/") => {
                    match template.soak(template_vars! {
                        "title" => "it works",
                        "body" => ("this is cool && ".to_owned() + &req.path)
                    }) {
                        Ok(soaked) => res.body = soaked.into(),
                        Err(err) => {
                            eprintln!("Something went wrong: {:?}", err);
                            return HttpStatusCode::Code500;
                        }
                    }
                    res.headers.set_header("Server", "Rebar");

                    HttpStatusCode::Code200
                }
                (_, "/interesting/") => HttpStatusCode::Code405,
                _ => HttpStatusCode::Code404,
            });

            server.listen_once();
            server.listen_once();
            server.listen_once();
        });

        let response = reqwest::blocking::get(format!("http://{}/interesting", ADDRESS)).unwrap();
        let headers = response.headers();
        assert!(headers.contains_key(CONTENT_TYPE));
        assert!(headers.contains_key(SERVER));
        assert_eq!(headers[CONTENT_TYPE], "text/html; charset=utf-8");
        assert_eq!(headers[SERVER], "Rebar");

        assert_eq!(
            response.text().unwrap(),
            r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta http-equiv="X-UA-Compatible" content="IE=edge" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>it works</title>
        <style>
            html,
            body {
                margin: 0;
            }
        </style>
    </head>
    <body>
        this is cool &amp;&amp; /interesting/
    </body>
</html>
"#
            .to_owned()
        );

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("http://{}/interesting", ADDRESS))
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        let response = reqwest::blocking::get(format!("http://{}/hmm", ADDRESS)).unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        handle.join().unwrap();
    }
}
