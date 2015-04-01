extern crate firebase;

use firebase::Firebase;

fn main() {
    let firebase = Firebase::new("https://shining-torch-7752.firebaseio.com");
    let response = firebase.push("/api/messages.json", "{\"name\":\"David\",\"message\":\"Hello from Rust\"}");

    println!("Response body: {:?}", response.body);
}
