/*!
 # Firebase REST API for Rust
 Please have a look at the ```Firebase``` struct to get started.
 */

extern crate hyper;
extern crate url;
extern crate rustc_serialize;

use std::str;
use std::borrow::Cow;
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use hyper::Client;
use hyper::status::StatusCode;
use url::Url;
use std::io::{self, Read};

use rustc_serialize::Decodable;
use rustc_serialize::json;
pub use rustc_serialize::json::{Json, BuilderError, DecoderError};

/// A Firebase instance to manage data.
#[derive(Clone)]
pub struct Firebase {
    url: Url,
}

// TODO: Change all instances of &str to Into<String>
// TODO: Make FB instance from Url and String.
// TODO: Make closure lives 'static or 'fb ?
impl Firebase {
    /// Creates a new Firebase instance from the url of the fb server
    /// The url should be a valid **HTTPS** url, anything else will
    /// result in an error:
    ///
    /// https://docs-examples.firebaseio.com/
    ///
    /// # Failures
    /// - If a url is not specified with the HTTPS scheme, a ```Err(ParseError::UrlIsNotHTTPS)```
    ///   will be returned.
    /// - If a url cannot be parsed into a valid url then a ```Err(ParseError::Parser(url::ParseError)```
    ///   will be returned.
    pub fn new(url: &str) -> Result<Self, ParseError> {
        let url = try!(Url::parse(&url));

        Firebase::from_url(url)
    }

    /// Creates a firebase reference from a borrow of a Url instance.
    /// # Failures
    /// - If a url is not specified with the HTTPS scheme, a ```Err(ParseError::UrlIsNotHTTPS)```
    ///   will be returned.
    /// - If a url cannot be parsed into a valid url then a ```Err(ParseError::Parser(url::ParseError)```
    ///   will be returned.
    pub fn from_url(url: Url) -> Result<Self, ParseError> {
        try!( unwrap_path(&url) );

        if url.scheme() != "https" {
            return Err(ParseError::UrlIsNotHTTPS);
        }

        Ok(Firebase {
            url: url,
        })
    }

    /// Creates a new authenticated Firebase instance from the firebaseio url and an auth token.
    ///
    /// # Examples
    /// ```
    /// # use firebase::Firebase;
    /// let fb = Firebase::authed("https://myfb.firebaseio.com", "deadbeefcafe");
    /// // The url shoud now be: https://myfb.firebaseio.com?auth=deadbeefcafe
    /// ```
    ///
    /// # Failures
    /// - If a url is not specified with the HTTPS scheme, a ```Err(ParseError::UrlIsNotHTTPS)```
    ///   will be returned.
    /// - If a url cannot be parsed into a valid url then a ```Err(ParseError::Parser(url::ParseError)```
    ///   will be returned.
    pub fn authed(url: &str, auth_token: &str) -> Result<Self, ParseError> {
        let mut url = try!(Url::parse(&url));
        if url.scheme() != "https" {
            return Err(ParseError::UrlIsNotHTTPS);
        }
        try!( unwrap_path(&url) );

        url.query_pairs_mut().append_pair(AUTH, auth_token).finish();

        Ok(Firebase {
            url: url,
        })
    }

    /// Creates a new firebase instance that extends the path of an old firebase instance.
    /// Each time a reference is created a clone of the Firebase instance if done, all
    /// Firebase instances follow this immutable style.
    ///
    /// #Examples
    /// ```
    /// # use firebase::Firebase;
    /// // Points to the root of the db ( / )
    /// let fb = Firebase::new("https://myfb.firebaseio.com").unwrap();
    /// // A new reference to /friends/yasha
    /// let yasha = fb.at("/friends/yasha").unwrap();
    /// // A new reference to /friends/yasha/messages
    /// let messages = yasha.at("messages").unwrap();
    pub fn at(&self, add_path: &str) -> Result<Self, ParseError> {
        let mut url = self.url.clone();

        { // Add path to original path, already checked for path.
            // Remove .json from the old path's end.
            let last = url.path_segments().expect("last segment").last().unwrap().to_string();
            let mut path = url.path_segments_mut().expect("path segments");
            path.pop().push(last.trim_right_matches(".json"));

            let add_path = add_path.trim_matches('/');
            let add_path = if !add_path.ends_with(".json") {
                Cow::Owned(add_path.to_string() + ".json")
            } else {
                Cow::Borrowed(add_path)
            };

            for component in add_path.split("/") {
                path.push(component);
            }
        }

        Ok(Firebase {
            url: url,
        })
    }

    /// Creates a FirebaseParams instance, this instance has query parameters
    /// that are associated with it and that are used in every request made.
    /// Since query parameters only affect incomming data from Firebase, you can only
    /// GET data with a FirebaseParams instance.
    ///
    /// This constructor takes in a FbOps struct to define all of its parameters,
    /// all undefined parameters can be omitted by extending the new FbOps struct
    /// by its default.
    ///
    /// # Examples
    /// ```
    /// # use firebase::*;
    /// let fb = Firebase::new("https://db.fb.com").unwrap();
    /// let query = fb.ops(&FbOps {
    ///    order_by:       Some("Hello World"),
    ///    limit_to_first: Some(5),
    ///    end_at:         Some(7),
    ///    equal_to:       Some(3),
    ///    shallow:        Some(true),
    ///    format:         Some(true),
    ///    .. FbOps::default()
    /// });
    /// ```
    pub fn ops(&self, opts: &FbOps) -> FirebaseParams {
        FirebaseParams::from_ops(self.url.clone(), opts)
    }

    /// Returns the current URL as a string that will be used
    /// to make the REST call when talking to Firebase.
    pub fn get_url(&self) -> &str {
        self.url.as_str()
    }

    /// Gets data from Firebase.
    /// # Examples
    /// ```
    /// # use firebase::Firebase;
    /// let firebase = Firebase::new("https://shows.firebaseio.com").unwrap();
    /// let episode = firebase.at("/futurama/episodes/140").unwrap();
    /// let info = episode.get();
    /// ```
    pub fn get(&self) -> Result<Response, ReqErr> {
        self.request(Method::GET, None)
    }

    /// Sets data to Firebase.
    /// # Examples
    /// ```
    /// # use firebase::Firebase;
    /// let firebase = Firebase::new("https://shows.firebaseio.com").unwrap();
    /// let episode = firebase.at("/futurama/episodes/140/description").unwrap();
    /// let info = episode.set("The Last Episode!");
    /// ```
    pub fn set(&self, data: &str) -> Result<Response, ReqErr> {
        self.request(Method::PUT, Some(data))
    }

    /// Pushes data to Firebase.
    /// # Examples
    /// ```
    /// # use firebase::Firebase;
    /// let firebase = Firebase::new("https://shows.firebaseio.com").unwrap();
    /// let episodes = firebase.at("/futurama/episodes").unwrap();
    /// let info = episodes.push("The Lost Episode");
    /// ```
    pub fn push(&self, data: &str) -> Result<Response, ReqErr> {
        self.request(Method::POST, Some(data))
    }

    /// Updates Firebase data.
    /// # Examples
    /// ```
    /// # use firebase::Firebase;
    /// let firebase = Firebase::new("https://shows.firebaseio.com").unwrap();
    /// let desc = firebase.at("/futurama/episodes/140/description").unwrap();
    /// let info = desc.update("The Penultimate Episode!");
    /// ```
    pub fn update(&self, data: &str) -> Result<Response, ReqErr> {
        self.request(Method::PATCH, Some(data))
    }

    /// Removes Firebase data.
    /// # Examples
    /// ```
    /// # use firebase::Firebase;
    /// let firebase = Firebase::new("https://shows.firebaseio.com").unwrap();
    /// let episode = firebase.at("/futurama/episodes/141").unwrap();
    /// episode.remove();
    /// ```
    pub fn remove(&self) -> Result<Response, ReqErr> {
        self.request(Method::DELETE, None)
    }

    /// Asynchronous version of the get method, takes a callback
    /// and returns a handle to the thread making the request to Firebase.
    /// # Examples
    /// ```
    /// # use firebase::Firebase;
    /// let firebase = Firebase::new("https://shows.firebaseio.com").unwrap();
    /// let desc = firebase.at("/futurama/episodes/141/description").unwrap();
    /// let original = "The Lost Episode";
    /// desc.get_async(move |result| {
    ///     if result.unwrap().body != original {
    ///         println!("The description changed!");
    ///     }
    /// });
    pub fn get_async<F>(&self, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static {
        Firebase::request_url_async(self.url.clone(), Method::GET, None, callback)
    }

    /// Asynchronous version of the set method, takes a callback
    /// and returns a handle to the thread making the request to Firebase.
    pub fn set_async<S, F>(&self, data: S, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static, S: Into<String> {
        Firebase::request_url_async(self.url.clone(), Method::PUT, Some(data.into()), callback)
    }

    /// Asynchronous version of the push method, takes a callback
    /// and returns a handle to the thread making the request to Firebase.
    pub fn push_async<S, F>(&self, data: S, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static, S: Into<String> {
        Firebase::request_url_async(self.url.clone(), Method::POST, Some(data.into()), callback)
    }

    /// Asynchronous version of the update method, takes a callback
    /// and returns a handle to the thread making the request to Firebase.
    pub fn update_async<S, F>(&self, data: S, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static, S: Into<String> {
        Firebase::request_url_async(self.url.clone(), Method::PATCH, Some(data.into()), callback)
    }

    /// Asynchronous version of the remove method, takes a callback
    /// and returns a handle to the thread making the request to Firebase.
    pub fn remove_async<F>(&self, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static {
        Firebase::request_url_async(self.url.clone(), Method::DELETE, None, callback)
    }

    /// Creates a ```FirebaseParams``` instance, a Firebase struct that only
    /// knows how to GET data, and sorts this data by the key provided.
    pub fn order_by(&self, key: &str) -> FirebaseParams {
        self.with_params(ORDER_BY, key)
    }

    /// Creates a ```FirebaseParams``` instance, a Firebase struct that only
    /// knows how to GET data, and limits the number of entries returned
    /// on each request to the first ```count```. Often used with ```order_by```.
    pub fn limit_to_first(&self, count: u32) -> FirebaseParams {
        self.with_params(LIMIT_TO_FIRST, count)
    }

    /// Creates a ```FirebaseParams``` instance, a Firebase struct that only
    /// knows how to GET data, and limits the number of entries returned
    /// on each request to the last ```count```. Often used with ```order_by```.
    pub fn limit_to_last(&self, count: u32) -> FirebaseParams {
        self.with_params(LIMIT_TO_LAST, count)
    }

    /// Creates a ```FirebaseParams``` instance, a Firebase struct that only
    /// knows how to GET data, and only returns entries starting after
    /// the specified index. Often used with ```order_by```.
    pub fn start_at(&self, index: u32) -> FirebaseParams {
        self.with_params(START_AT, index)
    }

    /// Creates a ```FirebaseParams``` instance, a Firebase struct that only
    /// knows how to GET data, and only returns entries appearing before
    /// the specified index. Often used with ```order_by```.
    pub fn end_at(&self, index: u32) -> FirebaseParams {
        self.with_params(END_AT, index)
    }

    /// Creates a ```FirebaseParams``` instance, a Firebase struct that only
    /// knows how to GET data, and returns only the entry at the specified
    /// index. Often used with ```order_by```.
    pub fn equal_to(&self, index: u32) -> FirebaseParams {
        self.with_params(EQUAL_TO, index)
    }

    /// Creates a ```FirebaseParams``` instance, a Firebase struct that only
    /// knows how to GET data, and only returns a shallow copy of the db
    /// in every request.
    pub fn shallow(&self, flag: bool) -> FirebaseParams {
        self.with_params(SHALLOW, flag)
    }

    /// Creates a ```FirebaseParams``` instance, a Firebase struct that only
    /// knows how to GET data, and formats the data to be exported in every
    /// request. (e.g. includes a priority field).
    pub fn format(&self) -> FirebaseParams {
        self.with_params(FORMAT, EXPORT)
    }

    #[inline]
    fn request(&self, method: Method, data: Option<&str>) -> Result<Response, ReqErr> {
        Firebase::request_url(self.url.clone(), method, data)
    }

    fn request_url(url: Url, method: Method, data: Option<&str>) -> Result<Response, ReqErr> {
        let client = Client::new();

        let req = match method {
            Method::GET     => client.get(url),
            Method::POST    => client.post(url).body(data.expect("data")),
            Method::PUT     => client.put(url).body(data.expect("data")),
            Method::PATCH   => client.patch(url).body(data.expect("data")),
            Method::DELETE  => client.delete(url),
        };

        let mut res = try!(req.send());

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        Ok(Response {
            body: body,
            code: res.status,
        })
    }

    fn request_url_async<F>(url: Url, method: Method, data: Option<String>, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static {
        thread::spawn(move || {
            callback(Firebase::request_url(url, method, data.as_ref().map(|s| s as &str)));
        })
    }

    fn with_params<T: ToString>(&self, key: &'static str, value: T) -> FirebaseParams {
        FirebaseParams::new(self.url.clone(), key, value)
    }
}

/// The FirebaseParams struct is a Firebase reference with attatched
/// query parameters that allow you to sort, limit, and format the data
/// received from Firebase.
///
/// It has been made into its own struct because the Firebase API specifies
/// that you can only GET data with query parameters. And so taking advantage of
/// type systems, this struct can only GET data.
///
/// You can add any number of parameters to this struct by chaining calls together:
///
/// ```
/// # use firebase::*;
/// let episodes = Firebase::new("https://futurama.firebaseio.com/episodes/").unwrap();
/// let alphabetic = episodes.order_by("\"title\"").limit_to_first(5);
/// let first5 = alphabetic.get();
/// ```
///
/// Setting the same parameter overwrites the previous parameter:
///
/// ```
/// # use firebase::*;
/// let episodes = Firebase::new("https://arrdev.firebaseio.com/episodes/").unwrap();
/// // This will create a request that gets entries starting at the 0th index.
/// let skip10 = episodes.start_at(10).start_at(0);
/// ```
#[derive(Clone)]
pub struct FirebaseParams {
    url: Url,
    params: HashMap<&'static str, String>,
}

impl FirebaseParams {
    /// Gets data from Firebase.
    /// # Examples
    /// ```
    /// # use firebase::Firebase;
    /// let episodes = Firebase::new("https://futurama.firebaseio.com/episodes/").unwrap();
    /// let alphabetic = episodes.order_by("\"title\"").limit_to_first(5);
    /// let first5 = alphabetic.get();
    /// ```
    pub fn get(&self) -> Result<Response, ReqErr> {
        Firebase::request_url(self.url.clone(), Method::GET, None)
    }

    /// Asynchronous version of the get method, takes a callback
    /// and returns a handle to the thread making the request to Firebase.
    pub fn get_async<F>(&self, callback: F) -> JoinHandle<()>
    where F: Fn(Result<Response, ReqErr>) + Send + 'static {
        Firebase::request_url_async(self.url.clone(), Method::GET, None, callback)
    }

    /// Returns the current URL as a string that will be used
    /// to make the REST call when talking to Firebase.
    pub fn get_url(&self) -> &str {
        self.url.as_str()
    }

    // TODO: Wrap in quotes if not already. Or always wrap in quotes.
    /// Modifies the current ```FirebaseParams``` instance
    /// and sorts this data by the key provided.
    pub fn order_by(self, key: &str) -> Self {
        self.add_param(ORDER_BY, key)
    }

    /// Modifies the current ```FirebaseParams``` instance
    /// and limits the number of entries returned
    /// on each request to the first ```count```. Often used with ```order_by```.
    pub fn limit_to_first(self, count: u32) -> Self {
        self.add_param(LIMIT_TO_FIRST, count)
    }

    /// Modifies the current ```FirebaseParams``` instance
    /// and limits the number of entries returned
    /// on each request to the last ```count```. Often used with ```order_by```.
    pub fn limit_to_last(self, count: u32) -> Self {
        self.add_param(LIMIT_TO_LAST, count)
    }

    /// Modifies the current ```FirebaseParams``` instance
    /// and only returns entries starting after
    /// the specified index. Often used with ```order_by```.
    pub fn start_at(self, index: u32) -> Self {
        self.add_param(START_AT, index)
    }

    /// Modifies the current ```FirebaseParams``` instance
    /// and only returns entries appearing before
    /// the specified index. Often used with ```order_by```.
    pub fn end_at(self, index: u32) -> Self {
        self.add_param(END_AT, index)
    }

    /// Modifies the current ```FirebaseParams``` instance
    /// and returns only the entry at the specified
    /// index. Often used with ```order_by```.
    pub fn equal_to(self, value: u32) -> Self {
        self.add_param(EQUAL_TO, value)
    }

    /// Modifies the current ```FirebaseParams``` instance
    /// and only returns a shallow copy of the db
    /// in every request.
    pub fn shallow(self, flag: bool) -> Self {
        self.add_param(SHALLOW, flag)
    }

    /// Modifies the current ```FirebaseParams``` instance
    /// and formats the data to be exported in every
    /// request. (e.g. includes a priority field)
    pub fn format(self) -> Self {
        self.add_param(FORMAT, EXPORT)
    }

    fn add_param<T: ToString>(mut self, key: &'static str, value: T) -> Self {
        let value = value.to_string();
        self.params.insert(key, value);
        self.set_params();
        self
    }

    fn set_params(&mut self) {
        self.url.query_pairs_mut()
            .extend_pairs(self.params.iter().map(|(&k, v)| (k, v as &str)))
            .finish();
    }

    fn get_auth(url: &Url) -> HashMap<&'static str, String> {
        url.query_pairs()
            .filter(|&(ref k,_)| k == AUTH)
            .map(|(_, v)| (AUTH, v.into_owned()))
            .collect()
    }

    fn new<T: ToString>(url: Url, key: &'static str, value: T) -> Self {
        let me = FirebaseParams {
            params: FirebaseParams::get_auth(&url),
            url: url,
        };
        me.add_param(key, value)
    }

    fn from_ops(url: Url, opts: &FbOps) -> Self {
        let mut me = FirebaseParams {
            params: FirebaseParams::get_auth(&url),
            url: url,
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
    ReqNotJSON,
    RespNotUTF8(str::Utf8Error),
    Network(hyper::Error),
    Io(io::Error),
}

impl From<hyper::Error> for ReqErr {
    fn from(e: hyper::Error) -> Self {
        ReqErr::Network(e)
    }
}

impl From<io::Error> for ReqErr {
    fn from(e: io::Error) -> Self {
        ReqErr::Io(e)
    }
}


#[derive(Debug)]
pub enum ParseError {
    UrlHasNoPath,
    UrlIsNotHTTPS,
    Parser(url::ParseError),
}

impl From<url::ParseError> for ParseError {
    fn from(e: url::ParseError) -> Self {
        ParseError::Parser(e)
    }
}

#[derive(Debug)]
pub struct Response {
    pub body: String,
    pub code: StatusCode,
}

impl Response {
    /// Returns true if the status code is 200
    pub fn is_success(&self) -> bool {
        self.code == StatusCode::Ok
    }

    /// Turns the response body into a Json enum.
    pub fn json(&self) -> Result<Json, BuilderError> {
        Json::from_str(&self.body)
    }

    /// Encodes the data received into a struct matching the data.
    /// # Examples
    ///
    /// ```
    /// extern crate firebase;
    /// extern crate hyper;
    /// use hyper::status::StatusCode;
    /// use firebase::Response;
    ///
    /// fn main() {
    ///     let response = Response {
    ///         body: "324567898".to_string(),
    ///         code: StatusCode::Ok,
    ///     };
    ///
    ///     let parsed: u32 = response.parse().unwrap();
    ///     println!("Data is: {}", parsed);
    /// }
    /// ```
    pub fn parse<D>(&self) -> Result<D, DecoderError> where D: Decodable {
        json::decode(&self.body)
    }
}

fn unwrap_path(url: &Url) -> Result<str::Split<char>, ParseError> {
    match url.path_segments() {
        None    => return Err(ParseError::UrlHasNoPath),
        Some(p) => return Ok(p),
    }
}

// This code will happen when Trait Specialization becomes available
// in rust.
// pub trait ToJsonStr {
//     fn to_json_str(&self) -> Result<Cow<str>, /* TODO */ Box<Error>>;
// }
//
// impl<'l> ToJsonStr for &'l str {
//     fn to_json_str(&self) -> Result<Cow<str>, /* TODO */ Box<Error>> {
//         Ok(Cow::Borrowed(self))
//     }
// }
//
// impl<S> ToJsonStr for S where S: Encodable {
//     fn to_json_str(&self) -> Result<Cow<str>, /* TODO */ Box<Error>> {
//         match json::encode(self) {
//             Ok(encoded) => Ok(Cow::Owned(encoded)),
//             Err(e)      => Err(Box::new(e)),
//         }
//     }
// }
