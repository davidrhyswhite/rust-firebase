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

    pub fn authenticated(base_uri: &str, auth_token: &str) -> Self {
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
        self.request(Method::GET, None, None)
    }

    fn get_params(&self, params: String) -> Response {
        self.request(Method::GET, None, Some(params))
    }

    pub fn set(&self, data: &str) -> Response {
        self.request(Method::PUT, Some(data), None)
    }

    pub fn push(&self, data: &str) -> Response {
        self.request(Method::POST, Some(data), None)
    }

    pub fn update(&self, data: &str) -> Response {
        self.request(Method::PATCH, Some(data), None)
    }

    pub fn delete(&self) -> Response {
        self.request(Method::DELETE, None, None)
    }

    fn request(&self, method: Method, data: Option<&str>, params: Option<String>) -> Response {
        let url = if let Some(args) = params {
            if self.base_uri.find("?").is_some() {
                format!("{}&{}", &self.base_uri, args)
            } else {
                format!("{}?{}", &self.base_uri, args)
            }
        } else {
            // TODO: FIX SO THIS DOESN'T HAPPEN
            self.base_uri.clone()
        };

        let url: &str = &url;
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

    pub fn orderBy(&self, key: &str) -> ParamsRequest {
        self.params_request(orderByStr(key))
    }

    pub fn limitToFirst(&self, count: u32) -> ParamsRequest {
        self.params_request(limitToFirstStr(count))
    }

    pub fn limitToLast(&self, count: u32) -> ParamsRequest {
        self.params_request(limitToLastStr(count))
    }

    pub fn startAt(&self, index: u32) -> ParamsRequest {
        self.params_request(startAtStr(index))
    }

    pub fn endAt(&self, index: u32) -> ParamsRequest {
        self.params_request(endAtStr(index))
    }

    pub fn equalTo(&self, value: u32) -> ParamsRequest {
        self.params_request(equalToStr(value))
    }

    pub fn shallow(&self, flag: bool) -> ParamsRequest {
        self.params_request(shallowStr(flag))
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
    pub fn orderBy(&mut self, key: &str) -> &Self {
        self.params.push(orderByStr(key));
        self
    }

    pub fn limitToFirst(&mut self, count: u32) -> &Self {
        self.params.push(limitToFirstStr(count));
        self
    }

    pub fn limitToLast(&mut self, count: u32) -> &Self {
        self.params.push(limitToLastStr(count));
        self
    }

    pub fn startAt(&mut self, index: u32) -> &Self {
        self.params.push(startAtStr(index));
        self
    }

    pub fn endAt(&mut self, index: u32) -> &Self {
        self.params.push(endAtStr(index));
        self
    }

    pub fn equalTo(&mut self, value: u32) -> &Self {
        self.params.push(equalToStr(value));
        self
    }

    pub fn shallow(&mut self, flag: bool) -> &Self {
        self.params.push(shallowStr(flag));
        self
    }

    pub fn get(&self) -> Response {
        Firebase::get_params(self.firebase, self.params.connect("&"))
    }
}

fn orderByStr(key: &str) -> String {
    format!("orderBy={}", key)
}

fn limitToFirstStr(count: u32) -> String {
    format!("limitToFirst={}", count)
}

fn limitToLastStr(count: u32) -> String {
    format!("limitToLast={}", count)
}

fn startAtStr(index: u32) -> String {
    format!("startAt={}", index)
}

fn endAtStr(index: u32) -> String {
    format!("endAt={}", index)
}

fn equalToStr(value: u32) -> String {
    format!("equalTo={}", value)
}

fn shallowStr(flag: bool) -> String {
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
