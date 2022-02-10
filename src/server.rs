use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::parse::parse;
use crate::types::Server;

impl Server {
    pub fn new(on: &str) -> Server {
        Server {
            listener: TcpListener::bind(on).unwrap(),
        }
    }

    fn handle_connection(mut stream: TcpStream) {
        thread::spawn(move || match parse(&mut stream) {
            Ok(_) => {
                // temporary setup,
                // TODO: create and use a Response struct
                stream
                    .write(b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\nok")
                    .unwrap();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    const ADDRESS: &str = "localhost:3004";

    #[test]
    fn it_runs_server() {
        let handle = thread::spawn(|| {
            Server::new(ADDRESS).listen_once();
        });

        reqwest::blocking::get(format!("http://{}/ok", ADDRESS)).unwrap();

        handle.join().unwrap();
    }
}
