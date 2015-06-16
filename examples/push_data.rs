extern crate firebase;

use firebase::Firebase;

fn main() {
    let firebase = Firebase::new("https://shining-torch-7752.firebaseio.com").ok().unwrap();
    let messages = firebase.at("/api/messages").ok().unwrap();

    let res = messages.push("{\"name\":\"David\",\"message\":\"Hello from Rust\"}").ok().unwrap();

    println!("Response body: {:?}", res.body);
    println!("Response code: {:?}", res.code);
    println!("Response success: {:?}", res.is_success());
}
