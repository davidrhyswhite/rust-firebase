extern crate curl;

use std::str;
use curl::http;

pub struct Firebase {
    base_uri: String,
}

impl Firebase {
    pub fn new(base_uri: &str) -> Firebase {
        Firebase {
            base_uri: base_uri.to_string(),
        }
    }

    pub fn at(&self, path: &str) -> Firebase {
        Firebase {
            base_uri: self.base_uri.clone() + path,
        }
    }

    pub fn get(&self) -> Response {
        self.request(Method::GET, None)
    }

    pub fn set(&self, data: &str) -> Response {
        self.request(Method::PUT, Some(data))
    }

    pub fn push(&self, data: &str) -> Response {
        self.request(Method::POST, Some(data))
    }

    pub fn update(&self, data: &str) -> Response {
        self.request(Method::PATCH, Some(data))
    }

    pub fn delete(&self) -> Response {
        self.request(Method::DELETE, None)
    }

    fn request(&self, method: Method, data: Option<&str>) -> Response {
        let url: &str = &self.base_uri;
        let mut handler = http::handle();

        let req = match method {
            Method::GET     => handler.get(url),
            Method::POST    => handler.post(url, data.unwrap()),
            Method::PUT     => handler.put(url, data.unwrap()),
            Method::PATCH   => handler.patch(url, data.unwrap()),
            Method::DELETE  => handler.delete(url)
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
    PATCH,
    DELETE,
}

pub struct Response {
    pub body: String,
    pub code: u32,
}

impl Response {
    pub fn is_success(&self) -> bool {
        if self.code < 400 {
            return true;
        }
        return false;
    }
}
