extern crate firebase;

use firebase::Firebase;

#[test]
fn builds_auth_url() {
    let f = Firebase::authed("http://db.rifebass.com/", "deadbeaf").ok().unwrap();
    assert_eq!(f.get_url(), "http://db.rifebass.com/?auth=deadbeaf");
}

#[test]
fn extends_auth_url() {
    let f = Firebase::authed("http://db.rifebass.com/", "deadbeaf").ok().unwrap();
    let f = f.at("/futurama/SpacePilot3000").ok().unwrap();
    let url_now = "http://db.rifebass.com//futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());
}

#[test]
fn double_extends_url() {
    let f = Firebase::authed("http://db.rifebass.com", "deadbeaf").ok().unwrap();
    let f = f.at("/futurama.json").ok().unwrap();
    let f = f.at("SpacePilot3000").ok().unwrap();
    let url_now = "http://db.rifebass.com//futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());
}

#[test]
fn handle_slashes() {
    let f = Firebase::authed("http://db.rifebass.com", "deadbeaf").ok().unwrap();
    let f = f.at("futurama.json").ok().unwrap();
    let f = f.at("SpacePilot3000.json").ok().unwrap();
    let url_now = "http://db.rifebass.com//futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());

    let f = Firebase::authed("http://db.rifebass.com/", "deadbeaf").ok().unwrap();
    let f = f.at("/futurama/").ok().unwrap();
    let f = f.at("/SpacePilot3000/").ok().unwrap();
    let url_now = "http://db.rifebass.com//futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());
}

#[test]
fn handle_json_suffix() {
    let f = Firebase::new("http://db.rifebass.com").ok().unwrap();
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
    let url_now = "http://db.rifebass.com//0/1/1/8/9/9/9/8/8/1/9/9/9/1/1/9/7/2/5/3.json";
    assert_eq!(url_now, f.get_url());
}

#[test]
fn test_ops() {
    let f = Firebase::new("http://db.fe/").ok().expect("url err");
    let f = f.at("lol").ok().expect("extend err");
    let req = f.end_at(13).limit_to_first(4).equal_to(8).shallow(false);
    let a = "http://db.fe//lol.json?limitToFirst=4&endAt=13&equalTo=8&shallow=false";
    assert_eq!(a, req.get_url());
}

#[test]
fn test_auth_ops() {
    let f = Firebase::authed("db.fe/", "key").ok().unwrap().at("lol").ok().unwrap();
    let req = f.order_by("pts").limit_to_last(5).start_at(8);
    let a = "db.fe/lol.json?auth=key&orderBy=pts&limitToLast=5&startAt=8";
    assert_eq!(a, req.get_url());
}
