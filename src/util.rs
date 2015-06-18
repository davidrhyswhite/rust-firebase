use std::borrow::Cow;

pub fn trim_right<'a, 'b>(uri: &'a str, end: &'b str) -> &'a str {
    let n = uri.len();
    let p = end.len();

    if n >= p && &uri[(n - p)..n] == end {
        &uri[0..(n - p)]
    } else {
        uri
    }
}

pub fn trim_left<'a, 'b>(uri: &'a str, start: &'b str) -> &'a str {
    let n = uri.len();
    let p = start.len();

    if n >= p && &uri[0..p] == start {
        &uri[p..n]
    } else {
        uri
    }
}

pub fn add_right<'l>(uri: &'l str, end: &str) -> Cow<'l, str> {
    let n = uri.len();
    let p = end.len();

    if n >= p && &uri[(n - p)..n] == end {
        Cow::Borrowed(uri)
    } else {
        Cow::Owned(uri.to_string() + end)
    }
}

pub fn add_https<'l>(uri: &'l str) -> Cow<'l, str> {
    let protocol = "https://";
    let other = "http://";
    let p = protocol.len();
    let o = other.len();
    let n = uri.len();

    if n < p || &uri[0..p] != protocol && &uri[0..o] != other {
        Cow::Owned(protocol.to_string() + uri)
    } else {
        Cow::Borrowed(uri)
    }
}
