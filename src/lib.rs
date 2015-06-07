extern crate curl;

use std::str;
use curl::http;

mod util;

pub struct Firebase {
    base_uri: String,
}

impl Firebase {
    pub fn new(base_uri: &str) -> Self {
        Firebase {
            base_uri: base_uri.to_string(),
        }
    }

    pub fn authed(base_uri: &str, auth_token: &str) -> Self {
        let uri = util::trim_right(base_uri, "/");
        Firebase {
            base_uri: format!("{}?auth={}", uri, auth_token),
        }
    }

    pub fn at(&self, path: &str) -> Self {
        let mut components = self.base_uri.split('?');

        let base = components.next().unwrap();
        let base = util::trim_right(base, ".json");
        let path = util::trim_right(path, "/");
        let path = util::add_right(path, ".json");
        let url  = util::join(base, &path, "/");

        let url = if let Some(args) = components.next() {
            url + "?" + args
        } else {
            url
        };

        Firebase {
            base_uri: url,
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

    pub fn remove(&self) -> Response {
        self.request(Method::DELETE, None)
    }

    fn request(&self, method: Method, data: Option<&str>) -> Response {
        Firebase::request_url(&self.base_uri, method, data)
    }

    fn request_url(url: &str, method: Method, data: Option<&str>) -> Response {
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
            Err(_) => "Unable to parse"
        };

        Response {
            body: body.to_string(),
            code: res.get_code(),
        }
    }

    pub fn get_url(&self) -> &str {
        return &self.base_uri;
    }

    pub fn order_by(&self, key: &str) -> ParamsRequest {
        self.params_request(order_by_str(key))
    }

    pub fn limit_to_first(&self, count: u32) -> ParamsRequest {
        self.params_request(limit_to_first_str(count))
    }

    pub fn limit_to_last(&self, count: u32) -> ParamsRequest {
        self.params_request(limit_to_last_str(count))
    }

    pub fn start_at(&self, index: u32) -> ParamsRequest {
        self.params_request(start_at_str(index))
    }

    pub fn end_at(&self, index: u32) -> ParamsRequest {
        self.params_request(end_at_str(index))
    }

    pub fn equal_to(&self, value: u32) -> ParamsRequest {
        self.params_request(equal_to_str(value))
    }

    pub fn shallow(&self, flag: bool) -> ParamsRequest {
        self.params_request(shallow_str(flag))
    }

    fn params_request(&self, param: String) -> ParamsRequest {
        ParamsRequest {
            firebase: &self,
            params: vec![param],
        }
    }
}

pub struct ParamsRequest<'l> {
    firebase: &'l Firebase,
    params: Vec<String>,
}

impl<'l> ParamsRequest<'l> {
    pub fn order_by(mut self, key: &str) -> Self {
        self.params.push(order_by_str(key));
        self
    }

    pub fn limit_to_first(mut self, count: u32) -> Self {
        self.params.push(limit_to_first_str(count));
        self
    }

    pub fn limit_to_last(mut self, count: u32) -> Self {
        self.params.push(limit_to_last_str(count));
        self
    }

    pub fn start_at(mut self, index: u32) -> Self {
        self.params.push(start_at_str(index));
        self
    }

    pub fn end_at(mut self, index: u32) -> Self {
        self.params.push(end_at_str(index));
        self
    }

    pub fn equal_to(mut self, value: u32) -> Self {
        self.params.push(equal_to_str(value));
        self
    }

    pub fn shallow(mut self, flag: bool) -> Self {
        self.params.push(shallow_str(flag));
        self
    }

    pub fn get(&self) -> Response {
        Firebase::request_url(&self.get_url(), Method::GET, None)
    }

    pub fn get_url(&self) -> String {
        let params = self.params.connect("&");

        if self.firebase.base_uri.find("?").is_some() {
            format!("{}&{}", &self.firebase.base_uri, params)
        } else {
            format!("{}?{}", &self.firebase.base_uri, params)
        }
    }
}

fn order_by_str(key: &str) -> String {
    format!("orderBy={}", key)
}

fn limit_to_first_str(count: u32) -> String {
    format!("limitToFirst={}", count)
}

fn limit_to_last_str(count: u32) -> String {
    format!("limitToLast={}", count)
}

fn start_at_str(index: u32) -> String {
    format!("startAt={}", index)
}

fn end_at_str(index: u32) -> String {
    format!("endAt={}", index)
}

fn equal_to_str(value: u32) -> String {
    format!("equalTo={}", value)
}

fn shallow_str(flag: bool) -> String {
    format!("shallow={}", flag)
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
        self.code < 400
    }
}
