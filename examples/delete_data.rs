extern crate firebase;

use firebase::Firebase;

fn main() {
    let firebase = Firebase::new("https://shining-torch-7752.firebaseio.com").ok().unwrap();
    let david = firebase.at("/users/david_to_be_deleted").ok().unwrap();

    let res = david.set("{\"firstName\":\"Dave\"}").ok().unwrap();

    println!("Response body: {:?}", res.body);
    println!("Response code: {:?}", res.code);
    println!("Response success: {:?}", res.is_success());

    let res = david.remove().ok().unwrap();

    println!("Response body: {:?}", res.body);
    println!("Response code: {:?}", res.code);
    println!("Response success: {:?}", res.is_success());
}
