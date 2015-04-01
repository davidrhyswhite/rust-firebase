# Rust Firebase

Rust based library for interacting with the Firebase REST API.


## Pushing data

Generates a new child location using a unique key and returns a Firebase reference to it.

```rust
extern crate firebase;

use firebase::Firebase;

fn main() {
    let firebase = Firebase::new("https://<your-firebase>.firebaseio.com");
    let res = firebase.push("/api/messages.json", "{\"name\":\"David\",\"message\":\"Hello from Rust\"}");

    println!("Response body: {:?}", res.body);
    println!("Response code: {:?}", res.code);
    println!("Response success: {:?}", res.is_success());
}
```
