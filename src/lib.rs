extern crate curl;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::cell::RefCell;

#[derive(Debug)]
pub enum Error {
    Curl(curl::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Curl(ref err) => write!(f, "CURL Error: {}", err),
            Error::Json(ref err) => write!(f, "JSON Error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Curl(ref err) => err.description(),
            Error::Json(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Curl(ref err) => Some(err),
            Error::Json(ref err) => Some(err),
        }
    }
}

impl From<curl::Error> for Error {
    fn from(err: curl::Error) -> Error {
        Error::Curl(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize)]
pub struct Subscription {
    id: u64,
    created_at: String,
    feed_id: u64,
    title: String,
    feed_url: String,
    site_url: String,
}

pub struct Feedbin {
    client: RefCell<curl::easy::Easy>,
    endpoint: String,
}

const DEFAULT_ENDPOINT: &'static str = "https://api.feedbin.com";

impl Feedbin {
    pub fn new(username: &str, password: &str) -> Self {
        Feedbin::new_with_endpoint(username, password, DEFAULT_ENDPOINT)
    }

    pub fn new_with_endpoint<S: Into<String>>(username: &str, password: &str, endpoint: S) -> Self {
        let mut client = curl::easy::Easy::new();
        client.username(username).unwrap();
        client.password(password).unwrap();
        Feedbin {
            client: RefCell::new(client),
            endpoint: endpoint.into(),
        }
    }

    pub fn is_authenticated(&self) -> Result<bool> {
        let url = self.endpoint.to_string() + "/v2/authentication.json";
        let mut client = self.client.borrow_mut();
        try!(client.url(&url));
        try!(client.perform());
        Ok(try!(client.response_code()) == 200)
    }

    pub fn get_subscriptions(&self) -> Result<Vec<Subscription>> {
        let url = self.endpoint.to_string() + "/v2/subscriptions.json";
        let mut client = self.client.borrow_mut();
        try!(client.url(&url));
        let mut buf = Vec::new();
        {
            let mut transfer = client.transfer();
            try!(transfer.write_function(|data| {
                                             buf.extend_from_slice(data);
                                             Ok(data.len())
                                         }));
            try!(transfer.perform());
        }
        assert_eq!(try!(client.response_code()), 200); // FIXME
        let subscriptions: Vec<Subscription> = try!(serde_json::from_slice(&buf));
        Ok(subscriptions)
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    fn unsafe_env(key: &str) -> String {
        env::var_os(key).unwrap().into_string().unwrap()
    }

    fn feedbin() -> Feedbin {
        let username = unsafe_env("FEEDBIN_USERNAME");
        let password = unsafe_env("FEEDBIN_PASSWORD");
        Feedbin::new(&username, &password)
    }

    #[test]
    fn authenticated() {
        let feedbin = feedbin();
        assert!(feedbin.is_authenticated().unwrap());
    }

    #[test]
    fn subscriptions() {
        let feedbin = feedbin();
        let subscriptions = feedbin.get_subscriptions();
        assert!(subscriptions.unwrap().len() > 0);
    }
}
