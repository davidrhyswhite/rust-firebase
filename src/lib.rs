extern crate curl;
extern crate url;
extern crate rustc_serialize;

mod util;

use std::str;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use curl::http;
use url::Url;

use rustc_serialize::Decodable;
use rustc_serialize::json;
pub use rustc_serialize::json::{Json, BuilderError, DecoderError};

#[derive(Clone)]
pub struct Firebase {
    url: Arc<Url>,
}

impl Firebase {
    pub fn new(url: &str) -> Result<Self, ParseError> {
        let url = util::add_https(&url);
        let url = try!( parse(&url) );
        try!( unwrap_path(&url) );

        Ok(Firebase {
            url: Arc::new(url),
        })
    }

    pub fn from_url(url: &Url) -> Result<Self, ParseError> {
        let url = url.clone();
        try!( unwrap_path(&url) );

        Ok(Firebase {
            url: Arc::new(url),
        })
    }

    pub fn authed(url: &str, auth_token: &str) -> Result<Self, ParseError> {
        let url = util::add_https(&url);
        let mut url = try!( parse(&url) );
        try!( unwrap_path(&url) );

        let opts = vec![ (AUTH, auth_token) ];
        url.set_query_from_pairs(opts.into_iter());

        Ok(Firebase {
            url: Arc::new(url),
        })
    }

    pub fn at(&self, add_path: &str) -> Result<Self, ParseError> {
        let mut url = (*self.url).clone();

        { // Add path to original path, already checked for path.
            let mut path = url.path_mut().unwrap();
            // Remove .json from the old path's end.
            if let Some(end) = path.pop() {
                let new_end = util::trim_right(&end, ".json").to_string();
                path.push(new_end);
            }
            let add_path = util::trim_right(add_path, "/");
            let add_path = util::trim_left(add_path, "/");
            let add_path = util::add_right(add_path, ".json");

            for component in add_path.split("/").into_iter() {
                path.push(component.to_string());
            }
        }

        Ok(Firebase {
            url: Arc::new(url),
        })
    }

    pub fn ops(&self, opts: &FbOps) -> FirebaseParams {
        FirebaseParams::from_ops(&self.url, opts)
    }

    fn with_params<T: ToString>(&self, key: &'static str, value: T) -> FirebaseParams {
        FirebaseParams::new(&self.url, key, value)
    }

    pub fn get(&self) -> Result<Response, ReqErr> {
        self.request(Method::GET, None)
    }

    pub fn set(&self, data: &str) -> Result<Response, ReqErr> {
        self.request(Method::PUT, Some(data))
    }

    pub fn push(&self, data: &str) -> Result<Response, ReqErr> {
        self.request(Method::POST, Some(data))
    }

    pub fn update(&self, data: &str) -> Result<Response, ReqErr> {
        self.request(Method::PATCH, Some(data))
    }

    pub fn remove(&self) -> Result<Response, ReqErr> {
        self.request(Method::DELETE, None)
    }

    pub fn get_async<F>(&self, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static {
        Firebase::request_url_async(&self.url, Method::GET, None, callback)
    }

    pub fn set_async<S, F>(&self, data: S, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static, S: Into<String> {
        Firebase::request_url_async(&self.url, Method::PUT, Some(data.into()), callback)
    }

    pub fn push_async<S, F>(&self, data: S, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static, S: Into<String> {
        Firebase::request_url_async(&self.url, Method::POST, Some(data.into()), callback)
    }

    pub fn update_async<S, F>(&self, data: S, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static, S: Into<String> {
        Firebase::request_url_async(&self.url, Method::PATCH, Some(data.into()), callback)
    }

    pub fn remove_async<F>(&self, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static {
        Firebase::request_url_async(&self.url, Method::DELETE, None, callback)
    }

    pub fn order_by(&self, key: &str) -> FirebaseParams {
        self.with_params(ORDER_BY, key)
    }

    pub fn limit_to_first(&self, count: u32) -> FirebaseParams {
        self.with_params(LIMIT_TO_FIRST, count)
    }

    pub fn limit_to_last(&self, count: u32) -> FirebaseParams {
        self.with_params(LIMIT_TO_LAST, count)
    }

    pub fn start_at(&self, index: u32) -> FirebaseParams {
        self.with_params(START_AT, index)
    }

    pub fn end_at(&self, index: u32) -> FirebaseParams {
        self.with_params(END_AT, index)
    }

    pub fn equal_to(&self, value: u32) -> FirebaseParams {
        self.with_params(EQUAL_TO, value)
    }

    pub fn shallow(&self, flag: bool) -> FirebaseParams {
        self.with_params(SHALLOW, flag)
    }

    pub fn format(&self) -> FirebaseParams {
        self.with_params(FORMAT, EXPORT)
    }

    pub fn get_url(&self) -> String {
        self.url.serialize()
    }

    #[inline]
    fn request(&self, method: Method, data: Option<&str>) -> Result<Response, ReqErr> {
        Firebase::request_url(&self.url, method, data)
    }

    fn request_url(url: &Url, method: Method, data: Option<&str>) -> Result<Response, ReqErr> {
        let mut handler = http::handle();

        let req = match method {
            Method::GET     => handler.get(   url),
            Method::POST    => handler.post(  url, data.unwrap()),
            Method::PUT     => handler.put(   url, data.unwrap()),
            Method::PATCH   => handler.patch( url, data.unwrap()),
            Method::DELETE  => handler.delete(url),
        };

        let res = match req.exec() {
            Ok(r)  => r,
            Err(e) => return Err(ReqErr::NetworkErr(e)),
        };

        let body = match str::from_utf8(res.get_body()) {
            Ok(b)  => b,
            Err(e) => return Err(ReqErr::RespNotUTF8(e)),
        };

        Ok(Response {
            body: body.to_string(),
            code: res.get_code(),
        })
    }

    fn request_url_async<F>(url: &Arc<Url>, method: Method, data: Option<String>, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static {
        // Fast, because its in an arc.
        let url = url.clone();

        thread::spawn(move || {
            callback(Firebase::request_url(&url, method, data.as_ref().map(|s| s as &str)));
        })
    }
}

#[derive(Clone)]
pub struct FirebaseParams {
    url: Arc<Url>,
    params: HashMap<&'static str, String>,
}

impl FirebaseParams {
    pub fn get(&self) -> Result<Response, ReqErr> {
        Firebase::request_url(&self.url, Method::GET, None)
    }

    pub fn get_async<F>(&self, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static {
        Firebase::request_url_async(&self.url, Method::GET, None, callback)
    }

    pub fn order_by(self, key: &str) -> Self {
        self.add_param(ORDER_BY, key)
    }

    pub fn limit_to_first(self, count: u32) -> Self {
        self.add_param(LIMIT_TO_FIRST, count)
    }

    pub fn limit_to_last(self, count: u32) -> Self {
        self.add_param(LIMIT_TO_LAST, count)
    }

    pub fn start_at(self, index: u32) -> Self {
        self.add_param(START_AT, index)
    }

    pub fn end_at(self, index: u32) -> Self {
        self.add_param(END_AT, index)
    }

    pub fn equal_to(self, value: u32) -> Self {
        self.add_param(EQUAL_TO, value)
    }

    pub fn shallow(self, flag: bool) -> Self {
        self.add_param(SHALLOW, flag)
    }

    pub fn format(self) -> Self {
        self.add_param(FORMAT, EXPORT)
    }

    pub fn get_url(&self) -> String {
        self.url.serialize()
    }

    fn add_param<T: ToString>(mut self, key: &'static str, value: T) -> Self {
        let value = value.to_string();
        self.params.insert(key, value);
        self.set_params();
        self
    }

    fn set_params(&mut self) {
        // Only clones the url when edited. This is CoW
        // Many threads can run requests without ever cloning the url.
        let mut url = (*self.url).clone();
        url.set_query_from_pairs(self.params.iter().map(|(&k, v)| (k, v as &str)));
        self.url = Arc::new(url);
    }

    fn get_auth(url: &Url) -> HashMap<&'static str, String> {
        let mut pair: HashMap<&'static str, String> = HashMap::new();

        if let Some(queries) = url.query_pairs() {
            for &(ref k, ref v) in queries.iter() {
                if k == AUTH {
                    pair.insert(AUTH, v.to_string());
                }
            }
        }
        pair
    }

    fn new<T: ToString>(url: &Url, key: &'static str, value: T) -> Self {
        let me = FirebaseParams {
            url: Arc::new(url.clone()),
            params: FirebaseParams::get_auth(&url),
        };
        me.add_param(key, value)
    }

    fn from_ops(url: &Url, opts: &FbOps) -> Self {
        let mut me = FirebaseParams {
            url: Arc::new(url.clone()),
            params: FirebaseParams::get_auth(&url),
        };
        if let Some(order) = opts.order_by {
            me.params.insert(ORDER_BY, order.to_string());
        }
        if let Some(first) = opts.limit_to_first {
            me.params.insert(LIMIT_TO_FIRST, first.to_string());
        }
        if let Some(last) = opts.limit_to_last {
            me.params.insert(LIMIT_TO_LAST, last.to_string());
        }
        if let Some(start) = opts.start_at {
            me.params.insert(START_AT, start.to_string());
        }
        if let Some(end) = opts.end_at {
            me.params.insert(END_AT, end.to_string());
        }
        if let Some(equal) = opts.equal_to {
            me.params.insert(EQUAL_TO, equal.to_string());
        }
        if let Some(shallow) = opts.shallow {
            me.params.insert(SHALLOW, shallow.to_string());
        }
        if let Some(format) = opts.format {
            if format {
                me.params.insert(FORMAT, EXPORT.to_string());
            }
        }
        // Copy all of the params into the url.
        me.set_params();
        me
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
const AUTH:           &'static str = "auth";

#[derive(Debug)]
pub struct FbOps<'l> {
    pub order_by:       Option<&'l str>,
    pub limit_to_first: Option<u32>,
    pub limit_to_last:  Option<u32>,
    pub start_at:       Option<u32>,
    pub end_at:         Option<u32>,
    pub equal_to:       Option<u32>,
    pub shallow:        Option<bool>,
    pub format:         Option<bool>,
}

impl<'l> Default for FbOps<'l> {
    fn default() -> Self {
        FbOps {
            order_by:       None,
            limit_to_first: None,
            limit_to_last:  None,
            start_at:       None,
            end_at:         None,
            equal_to:       None,
            shallow:        None,
            format:         None,
        }
    }
}

#[derive(Debug)]
pub enum ReqErr {
    RespNotUTF8(str::Utf8Error),
    NetworkErr(curl::ErrCode),
}

#[derive(Debug)]
pub enum ParseError {
    UrlHasNoPath,
    Parser(url::ParseError),
}

#[derive(Debug)]
pub struct Response {
    pub body: String,
    pub code: u32,
}

impl Response {
    pub fn is_success(&self) -> bool {
        self.code == 200
    }

    pub fn json(&self) -> Result<Json, BuilderError> {
        Json::from_str(&self.body)
    }

    pub fn parse<D>(&self) -> Result<D, DecoderError> where D: Decodable {
        json::decode(&self.body)
    }
}

fn parse(url: &str) -> Result<Url, ParseError> {
    match Url::parse(&url) {
        Ok(u)  => Ok(u),
        Err(e) => Err(ParseError::Parser(e)),
    }
}

fn unwrap_path(url: &Url) -> Result<&[String], ParseError> {
    match url.path() {
        None    => return Err(ParseError::UrlHasNoPath),
        Some(p) => return Ok(p),
    }
}
