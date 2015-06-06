extern crate firebase;

use firebase::Firebase;

#[test]
fn builds_auth_url() {
    let f = Firebase::authenticated("http://db.rifebass.com/", "deadbeaf");
    assert_eq!(f.get_url(), "http://db.rifebass.com?auth=deadbeaf");
}

#[test]
fn extends_auth_url() {
    let f = Firebase::authenticated("http://db.rifebass.com/", "deadbeaf");
    let f = f.at("/futurama/SpacePilot3000");
    let url_now = "http://db.rifebass.com/futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());
}

#[test]
fn double_extends_url() {
    let f = Firebase::authenticated("http://db.rifebass.com/", "deadbeaf");
    let f = f.at("/futurama.json");
    let f = f.at("SpacePilot3000");
    let url_now = "http://db.rifebass.com/futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());
}

#[test]
fn handle_slashes() {
    let f = Firebase::authenticated("http://db.rifebass.com", "deadbeaf");
    let f = f.at("futurama.json");
    let f = f.at("SpacePilot3000.json");
    let url_now = "http://db.rifebass.com/futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());

    let f = Firebase::authenticated("http://db.rifebass.com/", "deadbeaf");
    let f = f.at("/futurama/");
    let f = f.at("/SpacePilot3000/");
    let url_now = "http://db.rifebass.com/futurama/SpacePilot3000.json?auth=deadbeaf";
    assert_eq!(url_now, f.get_url());
}

#[test]
fn handle_json_suffix() {
    let f = Firebase::new("http://db.rifebass.com");
    let f = f.at("0.json").at("1.json").at("1.json").at("8.json")
             .at("9.json").at("9.json").at("9.json").at("8.json")
             .at("8.json").at("1.json").at("9.json").at("9.json")
             .at("9.json").at("1.json").at("1.json").at("9.json")
             .at("7.json").at("2.json").at("5.json").at("3.json");
    let url_now = "http://db.rifebass.com/0/1/1/8/9/9/9/8/8/1/9/9/9/1/1/9/7/2/5/3.json";
    assert_eq!(url_now, f.get_url());
}
