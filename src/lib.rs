extern crate curl;
extern crate url;

use std::str;
use std::error::Error;
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;

use curl::http;
use url::{Url, ParseError};

mod util;

pub type FbResult = Result<Response, Box<Error>>;

#[derive(Clone)]
pub struct Firebase {
    url: Url,
}

impl Firebase {
    pub fn new(url: &str) -> Result<Self, ParseError> {
        Ok(Firebase {
            url: try!( Url::parse(url) ),
        })
    }

    pub fn authed(url: &str, auth_token: &str) -> Result<Self, ParseError> {
        let url = util::trim_right(url, "/");
        let url = try!( Url::parse(url) );
        let opts = [ ("auth", auth_token) ];
        url.set_query_from_pairs(opts.iter());

        Ok(Firebase {
            url: url,
        })
    }

    pub fn from_url(url: Url) -> Self {
        Firebase {
            url: url,
        }
    }

    pub fn at(&self, path: &str) -> Result<Self, ParseError> {
        let extra = try!( Url::parse(path) );
        let new_url = self.url.clone();

        // HORRIBLE
        // for &component in extra.path().unwrap().into_iter() {
        //     new_url.path_mut().unwrap().push(component);
        // }
        panic!();

        Ok(Firebase {
            url: new_url,
        })
    }

    pub fn get(&self) -> FbResult {
        self.request(Method::GET, None)
    }

    pub fn set(&self, data: &str) -> FbResult {
        self.request(Method::PUT, Some(data))
    }

    pub fn push(&self, data: &str) -> FbResult {
        self.request(Method::POST, Some(data))
    }

    pub fn update(&self, data: &str) -> FbResult {
        self.request(Method::PATCH, Some(data))
    }

    pub fn remove(&self) -> FbResult {
        self.request(Method::DELETE, None)
    }

    pub fn order_by(&self, key: &str) -> FirebaseParams {
        self.with_params(ORDER_BY, key.to_string())
    }

    pub fn limit_to_first(&self, count: u32) -> FirebaseParams {
        self.with_params(LIMIT_TO_FIRST, count.to_string())
    }

    pub fn limit_to_last(&self, count: u32) -> FirebaseParams {
        self.with_params(LIMIT_TO_LAST, count.to_string())
    }

    pub fn start_at(&self, index: u32) -> FirebaseParams {
        self.with_params(START_AT, index.to_string())
    }

    pub fn end_at(&self, index: u32) -> FirebaseParams {
        self.with_params(END_AT, index.to_string())
    }

    pub fn equal_to(&self, value: u32) -> FirebaseParams {
        self.with_params(EQUAL_TO, value.to_string())
    }

    pub fn shallow(&self, flag: bool) -> FirebaseParams {
        self.with_params(SHALLOW, flag.to_string())
    }

    pub fn format(&self) -> FirebaseParams {
        self.with_params(FORMAT, EXPORT.to_string())
    }

    fn with_params(&self, key: &'static str, value: String) -> FirebaseParams {
        FirebaseParams {
            url: &self.url,
            params: vec![(key, value)],
        }
    }

    fn request(&self, method: Method, data: Option<&str>) -> FbResult {
        let mut handler = http::handle();

        let req = match method {
            Method::GET     => handler.get(   &self.url),
            Method::POST    => handler.post(  &self.url, data.unwrap()),
            Method::PUT     => handler.put(   &self.url, data.unwrap()),
            Method::PATCH   => handler.patch( &self.url, data.unwrap()),
            Method::DELETE  => handler.delete(&self.url),
        };
        let res = try!(req.exec());

        let body = try!(str::from_utf8(res.get_body()));

        Ok(Response {
            body: body.to_string(),
            code: res.get_code(),
        })
    }
}

struct FirebaseParams<'l> {
    url: &'l Url,
    params: Vec<(&'static str, String)>,
}

impl<'l> FirebaseParams<'l> {
    pub fn order_by(mut self, key: &str) -> Self {
        self.params.push((ORDER_BY, key.to_string()));
        self
    }

    pub fn limit_to_first(mut self, count: u32) -> Self {
        self.params.push((LIMIT_TO_FIRST, count.to_string()));
        self
    }

    pub fn limit_to_last(mut self, count: u32) -> Self {
        self.params.push((LIMIT_TO_LAST, count.to_string()));
        self
    }

    pub fn start_at(mut self, index: u32) -> Self {
        self.params.push((START_AT, index.to_string()));
        self
    }

    pub fn end_at(mut self, index: u32) -> Self {
        self.params.push((END_AT, index.to_string()));
        self
    }

    pub fn equal_to(mut self, value: u32) -> Self {
        self.params.push((EQUAL_TO, value.to_string()));
        self
    }

    pub fn shallow(mut self, flag: bool) -> Self {
        self.params.push((SHALLOW, flag.to_string()));
        self
    }

    pub fn format(mut self) -> Self {
        self.params.push((FORMAT, EXPORT.to_string()));
        self
    }

    pub fn get(&self) -> FbResult {
        self.get_fb().get()
    }

    fn get_fb(&self) -> Firebase {
        let url = self.url.clone();
        let all_refs = self.params.iter().map(|&(k, v)| (k, &v));

        url.set_query_from_pairs(all_refs);
        Firebase::from_url(url)
    }
}

enum Method {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

const ORDER_BY:       &'static str = "orderBy";
const LIMIT_TO_FIRST: &'static str = "limitToFirst";
const LIMIT_TO_LAST:  &'static str = "limitToLast";
const START_AT:       &'static str = "startAt";
const END_AT:         &'static str = "endAt";
const EQUAL_TO:       &'static str = "equalTo";
const SHALLOW:        &'static str = "shallow";
const FORMAT:         &'static str = "format";
const EXPORT:         &'static str = "export";

pub struct Response {
    pub body: String,
    pub code: u32,
}

impl Response {
    pub fn is_success(&self) -> bool {
        self.code < 400
    }
}
