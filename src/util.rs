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

pub fn add_right(uri: &str, end: &str) -> String {
    let n = uri.len();
    let p = end.len();

    if n >= p && &uri[(n - p)..n] == end {
        uri.to_string()
    } else {
        uri.to_string() + end
    }
}

pub fn add_https(uri: &str) -> String {
    let protocol = "https://";
    let other = "http://";
    let p = protocol.len();
    let o = other.len();
    let n = uri.len();

    if n >= p && &uri[0..p] != protocol && &uri[0..o] != other {
        protocol.to_string() + uri
    } else {
        uri.to_string()
    }
}
//
// pub fn join(left: &str, right: &str, delim: &str) -> String {
//     trim_right(left, delim).to_string() + delim + trim_left(right, delim)
// }
