extern crate firebase;

use firebase::Firebase;

fn main() {
    let firebase = Firebase::new("https://shining-torch-7752.firebaseio.com");
    let res = firebase.set("/api/messages.json", "{\"name\":\"David White\",\"message\":\"Hello from Rust\"}");

    println!("Response body: {:?}", res.body);
    println!("Response code: {:?}", res.code);
    println!("Response success: {:?}", res.is_success());
}
