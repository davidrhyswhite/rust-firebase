#![crate_name = "firebase"]
#![crate_type = "rlib"]

extern crate curl;
extern crate rustc_serialize;

use std::str;
use curl::http;

pub struct Firebase {
    base_uri: String,
}

impl Firebase {
    pub fn new(base_uri: &str) -> Firebase {
        Firebase { base_uri: base_uri.to_string() }
    }

    pub fn get(self, path: &str) -> Response {
        self.request(Method::GET, path, Some(""))
    }

    pub fn set(self, path: &str, data: &str) -> Response {
        self.request(Method::PUT, path, Some(data))
    }

    pub fn push(self, path: &str, data: &str) -> Response {
        self.request(Method::POST, path, Some(data))
    }

    fn request(self, method: Method, path: &str, data: Option<&str>) -> Response {
        let mut url = self.base_uri;
        url.push_str(path);
        let mut handler = http::handle();
         
        let req = match method {
            Method::GET => handler.get(url), 
            Method::POST => handler.post(url, data.unwrap()),
            Method::PUT => handler.put(url, data.unwrap()),
        };
        let res = req.exec().unwrap();

        let body = match str::from_utf8(res.get_body()) {
            Ok(b) => b,
            Err(..) => "Unable to parse"
        };

        return Response {
            body: body.to_string(),
            code: res.get_code(),
        };
    }
}

enum Method {
    GET,
    POST,
    PUT,
}

pub struct Response {
    pub body: String,
    pub code: u32,
}

impl Response {
    pub fn is_success(self) -> bool {
        if self.code < 400 {
            return true;
        }
        return false;
    }
}
