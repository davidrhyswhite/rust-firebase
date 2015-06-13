extern crate curl;
extern crate url;

use std::str;
use std::error::Error;
use std::collections::HashMap;
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

    pub fn from_url(url: &Url) -> Self {
        Firebase {
            url: url.clone(),
        }
    }

    pub fn authed(url: &str, auth_token: &str) -> Result<Self, ParseError> {
        let mut url = try!( Url::parse(url) );
        let opts = vec![ ("auth", auth_token) ];
        url.set_query_from_pairs(opts.into_iter());

        Ok(Firebase {
            url: url,
        })
    }

    /*
     TODO: Concat path and current url
     */
    pub fn at(&self, path: &str) -> Result<Self, ParseError> {
        // let extra = try!( Url::parse(path) );
        // let mut new_url = self.url.clone();
        //
        // // DR. HORRIBLE
        // for component in extra.path().unwrap().iter() {
        //     new_url.path_mut().unwrap().push(component.clone());
        // }
        // panic!();
        Ok(Firebase {
            url: self.url.clone(),
        })
    }

    pub fn ops(&self, opts: &FbOps) -> FirebaseParams {
        FirebaseParams::from_ops(&self.url, opts)
    }

    fn with_params<T: ToString>(&self, key: &'static str, value: T) -> FirebaseParams {
        FirebaseParams::new(&self.url, key, value)
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

    #[inline]
    fn request(&self, method: Method, data: Option<&str>) -> FbResult {
        Firebase::request_url(&self.url, method, data)
    }

    fn request_url(url: &Url, method: Method, data: Option<&str>) -> FbResult {
        let mut handler = http::handle();

        let req = match method {
            Method::GET     => handler.get(   url),
            Method::POST    => handler.post(  url, data.unwrap()),
            Method::PUT     => handler.put(   url, data.unwrap()),
            Method::PATCH   => handler.patch( url, data.unwrap()),
            Method::DELETE  => handler.delete(url),
        };
        let res = try!(req.exec());

        let body = try!(str::from_utf8(res.get_body()));

        Ok(Response {
            body: body.to_string(),
            code: res.get_code(),
        })
    }
}

#[derive(Clone)]
pub struct FirebaseParams {
    url: Url,
}

impl FirebaseParams {
    pub fn order_by(&self, key: &str) -> Self {
        self.add_param(ORDER_BY, key)
    }

    pub fn limit_to_first(&self, count: u32) -> Self {
        self.add_param(LIMIT_TO_FIRST, count)
    }

    pub fn limit_to_last(&self, count: u32) -> Self {
        self.add_param(LIMIT_TO_LAST, count)
    }

    pub fn start_at(&self, index: u32) -> Self {
        self.add_param(START_AT, index)
    }

    pub fn end_at(&self, index: u32) -> Self {
        self.add_param(END_AT, index)
    }

    pub fn equal_to(&self, value: u32) -> Self {
        self.add_param(EQUAL_TO, value)
    }

    pub fn shallow(&self, flag: bool) -> Self {
        self.add_param(SHALLOW, flag)
    }

    pub fn format(&self) -> Self {
        self.add_param(FORMAT, EXPORT)
    }

    fn add_param<T: ToString>(&self, key: &'static str, value: T) -> Self {
        /*
        TODO: add key/val pairs
        */
        self.clone()
    }

    fn new<T: ToString>(url: &Url, key: &'static str, value: T) -> Self {
        /*
         TODO: Set the key/val pairs.
         */
        FirebaseParams {
            url: url.clone(),
        }
    }

    fn from_ops(url: &Url, opts: &FbOps) -> Self {
        let mut params: Vec<(&str, String)> = Vec::new();
        let mut url = url.clone();

        if let Some(order) = opts.order_by {
            params.push((ORDER_BY, order.to_string()));
        }
        if let Some(first) = opts.limit_to_first {
            params.push((LIMIT_TO_FIRST, first.to_string()));
        }
        if let Some(last) = opts.limit_to_last {
            params.push((LIMIT_TO_LAST, last.to_string()));
        }
        if let Some(start) = opts.start_at {
            params.push((START_AT, start.to_string()));
        }
        if let Some(end) = opts.end_at {
            params.push((END_AT, end.to_string()));
        }
        if let Some(equal) = opts.equal_to {
            params.push((EQUAL_TO, equal.to_string()));
        }
        if let Some(shallow) = opts.shallow {
            params.push((SHALLOW, shallow.to_string()));
        }
        if let Some(format) = opts.format {
            if format {
                params.push((FORMAT, EXPORT.to_string()));
            }
        }
        let iter = params.into_iter().map(|(k, v)| (k, &v));
        url.set_query_from_pairs(iter);

        FirebaseParams {
            url: url,
        }
    }

    pub fn get(&self) -> FbResult {
        Firebase::request_url(&self.url, Method::GET, None)
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

pub struct FbOps<'l> {
    order_by:       Option<&'l str>,
    limit_to_first: Option<u32>,
    limit_to_last:  Option<u32>,
    start_at:       Option<u32>,
    end_at:         Option<u32>,
    equal_to:       Option<u32>,
    shallow:        Option<bool>,
    format:         Option<bool>,
}

const DEFAULT: FbOps<'static> = FbOps {
    order_by:       None,
    limit_to_first: None,
    limit_to_last:  None,
    start_at:       None,
    end_at:         None,
    equal_to:       None,
    shallow:        None,
    format:         None,
};

pub struct Response {
    pub body: String,
    pub code: u32,
}

impl Response {
    pub fn is_success(&self) -> bool {
        self.code < 400
    }
}
