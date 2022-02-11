use crate::types::{HttpResponseError, HttpStatusCode, Response};

use std::io::Write;

fn to_http_response_error(err: std::io::Error) -> HttpResponseError {
    HttpResponseError::Other(format! {"{:?}", err})
}

impl Response {
    fn write_head(&mut self) -> Result<(), HttpResponseError> {
        let head = format!(
            "{} {}\r\n{}\r\n",
            self.http_version, self.status, self.headers
        );

        self.stream
            .write(head.as_bytes())
            .and(Ok(()))
            .map_err(|err| to_http_response_error(err))
    }

    fn write_body(&mut self) -> Result<(), HttpResponseError> {
        self.stream
            .write(&self.body)
            .and(Ok(()))
            .map_err(|err| to_http_response_error(err))
    }

    pub fn send(mut self) -> Result<HttpStatusCode, HttpResponseError> {
        self.write_head()?;
        self.write_body()?;

        match self.stream.flush() {
            Ok(_) => Ok(self.status.clone()),
            Err(err) => Err(to_http_response_error(err)),
        }
    }
}
