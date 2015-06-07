# Rust Firebase

Rust based library for interacting with the Firebase REST API.

## Load the crate!

Don't forget to include the library in your project:
```Rust
extern crate firebase;
use firebase::Firebase;
```

## Creating a Firebase reference

### Simple
You can currently create a simple reference to your firebase server:

```Rust
let firebase = Firebase::new("https://<your-firebase>.firebaseio.com");
```

### Authenticated
Or you can create an authenticated connection by supplying your [auth](https://www.firebase.com/docs/rest/guide/user-auth.html) token:

```Rust
let firebase = Firebase::authed("https://<your-firebase>.firebaseio.com", "<token>");
```

## Walking the database

Reference nested objects in your server like so:

```Rust
let show = firebase.at("/shows/futurama"); // points to /shows/futurama
let episode = show.at("s10/meanwhile");    // points to /shows/futurama/s10/meanwhile
```

Slashes and .json extensions will be handled accordingly:

```Rust
// All of the following are equivalent:
let show = firebase.at("/shows/futurama.json");
let show = firebase.at("shows/futurama/");
let show = firebase.at("/shows/futurama/");
```

## Working with data

### Reading data

Reading data can be done with a simple call to ```.get()```
```Rust
let response = show.get();
```

### Writing data

```Rust
let description = episode.at("description");
let response = description.set("the last episode");
```

### Pushing data

```Rust
let episodes = firebase.at("/shows/futurama/episodes");
let response = episodes.push("The Lost Episode!");
```

### Updating data

```Rust
let description = episode.at("description");
let response = description.update("the penultimate episode");
```

### Removing data

```Rust
let episode = firebase.at("/shows/futurama/s10/meanwhile");
let response = episode.remove();
```

## Requests with parameters

```Rust
let episodes = firebase.at("/shows/futurama/episodes");
let top5 = episodes.sort_by("imdb").limit_to_first(5).get();
```

The full list of supported parameters are listed here:

 - ```order_by```
 - ```limit_to_first```
 - ```limit_to_last```
 - ```start_at```
 - ```end_at```
 - ```equal_to```
 - ```shallow```

## Not yet there...

### Working with JSON values

For now JSON is sent and received as a string literal, an easier method is
likely to be implemented in future versions

```Rust
let json = "{ \"name\": \"David Smith\" }"

let people = firebase.at("/earth/us/indiana");
let response = episodes.push(json);
```

### Error handling on requests.

Responses will likely be Result<T, Optional<E>> types to make error handling
more rust-like
