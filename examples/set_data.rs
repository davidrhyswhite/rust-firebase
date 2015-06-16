extern crate firebase;

use firebase::Firebase;

fn main() {
    let messages = Firebase::new("https://shining-torch-7752.firebaseio.com/api/messages.json").ok().unwrap();

    let res = messages.set("{\"name\":\"David White\",\"message\":\"Hello from Rust\"}").ok().unwrap();

    println!("Response body: {:?}", res.body);
    println!("Response code: {:?}", res.code);
    println!("Response success: {:?}", res.is_success());
}
