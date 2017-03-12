extern crate curl;

use std::cell::RefCell;

#[derive(Debug)]
pub enum Error {
    Curl(curl::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Curl(ref err) => write!(f, "CURL Error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Curl(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Curl(ref err) => Some(err),
        }
    }
}

impl From<curl::Error> for Error {
    fn from(err: curl::Error) -> Error {
        Error::Curl(err)
    }
}

type Result<T> = std::result::Result<T, Error>;

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
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    fn unsafe_env(key: &str) -> String {
        env::var_os(key).unwrap().into_string().unwrap()
    }

    #[test]
    fn login() {
        let username = unsafe_env("FEEDBIN_USERNAME");
        let password = unsafe_env("FEEDBIN_PASSWORD");
        let feedbin = Feedbin::new(&username, &password);
        assert!(feedbin.is_authenticated().unwrap());
    }
}
