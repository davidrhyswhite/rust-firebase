#![crate_name = "firebase"]
#![crate_type = "rlib"]

extern crate curl;
extern crate "rustc-serialize" as rustc_serialize;

use std::str;
use curl::http;

pub struct Firebase {
    base_uri: String 
}

impl Firebase {
    pub fn new(base_uri: &str) -> Firebase {
        Firebase { base_uri: base_uri.to_string() }
    }

    pub fn push(self, path: &str, data: &str) -> Response {
        let mut url = self.base_uri;
        url.push_str(path);
        
        let res = http::handle()
            .post(url, data)
            .exec().unwrap();
                                                    
        let body = match str::from_utf8(res.get_body()) {
            Ok(b) => b,
            Err(..) => "Unable to parse"
        };

        return Response { body: body.to_string() };
    }
}

pub struct Response {
    pub body: String
}
