extern crate firebase;
extern crate url;
extern crate rustc_serialize;
extern crate hyper;

use firebase::*;
use url::Url;
use hyper::status::StatusCode;
use std::collections::HashMap;

use std::sync::{Arc, Mutex};

#[test]
fn builds_auth_url() {
    let f = Firebase::authed("https://db.rifebass.com/", "deadbeaf").ok().unwrap();
    assert_eq!(f.get_url(), "https://db.rifebass.com/?auth=deadbeaf");
}

#[test]
fn extends_auth_url() {
    let f = Firebase::authed("https://db.rifebass.com/", "deadbeaf").ok().unwrap();
    let f = f.at("/futurama/SpacePilot3000").ok().unwrap();
    let url_now = "https://db.rifebass.com/futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());
}

#[test]
fn double_extends_url() {
    let f = Firebase::authed("https://db.rifebass.com", "deadbeaf").ok().unwrap();
    let f = f.at("/futurama.json").ok().unwrap();
    let f = f.at("SpacePilot3000").ok().unwrap();
    let url_now = "https://db.rifebass.com/futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());
}

#[test]
fn handle_slashes() {
    let f = Firebase::authed("https://db.rifebass.com", "deadbeaf").ok().unwrap();
    let f = f.at("futurama.json").ok().unwrap();
    let f = f.at("SpacePilot3000.json").ok().unwrap();
    let url_now = "https://db.rifebass.com/futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());

    let f = Firebase::authed("https://db.rifebass.com/", "deadbeaf").ok().unwrap();
    let f = f.at("/futurama/").ok().unwrap();
    let f = f.at("/SpacePilot3000/").ok().unwrap();
    let url_now = "https://db.rifebass.com/futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());
}

#[test]
fn handle_json_suffix() {
    let f = Firebase::new("https://db.rifebass.com").ok().unwrap();
    let f = f.at("0.json").ok().unwrap().at("1.json").ok().unwrap()
             .at("1.json").ok().unwrap().at("8.json").ok().unwrap()
             .at("9.json").ok().unwrap().at("9.json").ok().unwrap()
             .at("9.json").ok().unwrap().at("8.json").ok().unwrap()
             .at("8.json").ok().unwrap().at("1.json").ok().unwrap()
             .at("9.json").ok().unwrap().at("9.json").ok().unwrap()
             .at("9.json").ok().unwrap().at("1.json").ok().unwrap()
             .at("1.json").ok().unwrap().at("9.json").ok().unwrap()
             .at("7.json").ok().unwrap().at("2.json").ok().unwrap()
             .at("5.json").ok().unwrap().at("3.json").ok().unwrap();
    let url_now = "https://db.rifebass.com/0/1/1/8/9/9/9/8/8/1/9/9/9/1/1/9/7/2/5/3.json";
    assert_eq!(url_now, f.get_url());
}

#[test]
fn test_ops() {
    let f = Firebase::new("https://db.fe/").ok().expect("url err");
    let f = f.at("lol").ok().expect("extend err");
    let req = f.end_at(13).limit_to_first(4).equal_to(8).shallow(false);
    let correct = Url::parse("https://db.fe//lol.json?limitToFirst=4&endAt=13&equalTo=8&shallow=false").ok().unwrap();
    let generated = Url::parse(&req.get_url()).ok().unwrap();

    assert_queries_eq(&correct, &generated);
}

#[test]
fn test_auth_ops() {
    let f = Firebase::authed("https://db.fe/", "key").ok().expect("url err").at("lol").ok().unwrap();
    let req = f.order_by("pts").limit_to_last(5).start_at(8);

    let correct = Url::parse("https://db.fe/lol.json?auth=key&orderBy=pts&limitToLast=5&startAt=8").ok().unwrap();
    let generated = Url::parse(&req.get_url()).ok().unwrap();

    assert_queries_eq(&correct, &generated);
}

#[test]
fn test_async_get() {
    let fb = Firebase::new("https://mybd.firebase.com").ok().unwrap();
    let db_ref = fb.at("Profiles/a9sdc8asd99acc/profile_img").ok().unwrap();

    let finished = Arc::new(Mutex::new(false));

    let marker = finished.clone();
    let thread = db_ref.get_async(move |_| {
        let mut finished = marker.lock().unwrap();
        *finished = true;
    });

    assert!(!*finished.lock().unwrap());
    thread.join().ok();
}

#[test]
fn test_ops_ctor() {
    let fb = Firebase::new("https://db.fb.com").ok().unwrap();
    let query = fb.ops(&FbOps {
        order_by:       Some("Hello World"),
        limit_to_first: Some(5),
        end_at:         Some(7),
        equal_to:       Some(3),
        shallow:        Some(true),
        format:         Some(true),
        .. FbOps::default()
    });

    let corr = Url::parse("https://db.fb.com/?limitToFirst=5&orderBy=Hello+World&equalTo=3&format=export&shallow=true&endAt=7").ok().unwrap();
    let this = Url::parse(&query.get_url()).ok().unwrap();
    assert_queries_eq(&corr, &this);
}

#[test]
fn test_resp_json() {
    let response = Response {
        code: StatusCode::Ok,
        body: "{
            \"id\":   \"mongo id\",
            \"data\": \"Hello World!\"
        }".to_string(),
    };

    let record = match response.json().ok().expect("Should've parsed json") {
        Json::Object(o) => o,
        _ => panic!("This shouldv'e been a object!"),
    };

    let data = record.get("data").expect("Should've had a data member");

    match data.clone() {
        Json::String(d) => assert_eq!("Hello World!", d),
        _ => panic!("This shouldv'e been a string!"),
    }
}

#[test]
fn test_resp_struct_easy() {
    let response = Response {
        code: StatusCode::Ok,
        body: "{
            \"fizz\": 3,
            \"buzz\": 5
        }".to_string(),
    };

    let bee: FizzBuzz = response.parse().ok().expect("Should parse into FizzBuzz struct");

    assert_eq!(bee.fizz, 3);
    assert_eq!(bee.buzz, 5);
}

fn assert_queries_eq(a: &Url, b: &Url) {
    let param_a = a.query_pairs().collect::<HashMap<_,_>>();
    let param_b = b.query_pairs().collect::<HashMap<_,_>>();

    assert_eq!(param_a, param_b);
}

#[derive(RustcDecodable)]
struct FizzBuzz {
    fizz: u32,
    buzz: u32,
}
