//! A tiny module for testing Rocket applications.
//!
//! # Enabling
//!
//! The `testing` module is only available when Rocket is compiled with the
//! `testing` feature flag. The suggested way to enable the `testing` module is
//! through Cargo's `[dev-dependencies]` feature which allows features (and
//! other dependencies) to be enabled exclusively when testing/benchmarking your
//! application.
//!
//! To compile Rocket with the `testing` feature for testing/benchmarking, add
//! the following to your `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! rocket = { version = "*", features = ["testing"] }
//! ```
//!
//! Then, in your testing module, `use` the testing types. This typically looks
//! as follows:
//!
//! ```rust,ignore
//! #[cfg(test)]
//! mod test {
//!   use super::rocket;
//!   use rocket::testing::MockRequest;
//!   use rocket::http::Method::*;
//! }
//! ```
//!
//! # Usage
//!
//! The testing methadology is simple:
//!
//!   1. Construct a `Rocket` instance
//!   2. Construct a request.
//!   3. Dispatch the request using the Rocket instance.
//!   4. Inspect, validate, and verify the response.
//!
//! ## Construct a `Rocket` Instance
//!
//! Constructing a `Rocket` instance for testing is identical to constructing
//! one for launching, except you should not call the `launch` method. That is,
//! use `rocket::ignite`, then mount routes and catchers. That's it!
//!
//! ## Construct a (Mock)Request
//!
//! The [MockRequest](struct.MockRequest.html) object enables the creation of an
//! HTTP request without using any networking. A `MockRequest` object is
//! constructed using the builder pattern. For example, the following code
//! builds a request for submitting a login form with three fields:
//!
//! ```rust
//! # use rocket::http::Method::*;
//! # use rocket::testing::MockRequest;
//! # use rocket::http::ContentType;
//! let (username, password, age) = ("user", "password", 32);
//! MockRequest::new(Post, "/login")
//!     .header(ContentType::Form)
//!     .body(&format!("username={}&password={}&age={}", username, password, age));
//! ```
//!
//! ## Dispatch and Validate
//!
//! Finally, requests can be dispatched using the
//! [dispatch_with](struct.MockRequest.html#method.dispatch_with) method on the
//! contructed `MockRequest` instance. The method returns the body of the
//! response. At present, the API does not allow for headers in the response to
//! be examined.
//!
//! # Example
//!
//! The following is an example of a complete application with testing.
//!
//! ```rust
//! #![feature(plugin)]
//! #![plugin(rocket_codegen)]
//!
//! extern crate rocket;
//!
//! #[get("/")]
//! fn hello() -> &'static str {
//!     "Hello, world!"
//! }
//!
//! # fn main() {  }
//! #[cfg(test)]
//! mod test {
//!     use super::rocket;
//!     use rocket::testing::MockRequest;
//!     use rocket::http::Method::*;
//!
//!     #[test]
//!     fn test_hello_world() {
//!         let rocket = rocket::ignite().mount("/", routes![super::hello]);
//!         let mut req = MockRequest::new(Get, "/");
//!         let mut response = req.dispatch_with(&rocket);
//!
//!         // Write the body out as a string.
//!         let body_str = response.body().unwrap().to_string().unwrap();
//!
//!         // Check that the body contains what we expect.
//!         assert_eq!(body_str, "Hello, world!".to_string());
//!     }
//! }
//! ```

use http::{Method, Header, Cookie};
use ::{Rocket, Request, Response, Data};

/// A type for mocking requests for testing Rocket applications.
pub struct MockRequest {
    request: Request,
    data: Data
}

impl MockRequest {
    /// Constructs a new mocked request with the given `method` and `uri`.
    #[inline]
    pub fn new<S: AsRef<str>>(method: Method, uri: S) -> Self {
        MockRequest {
            request: Request::mock(method, uri.as_ref()),
            data: Data::new(vec![])
        }
    }

    /// Add a header to this request.
    ///
    /// # Examples
    ///
    /// Add the Content-Type header:
    ///
    /// ```rust
    /// use rocket::http::Method::*;
    /// use rocket::testing::MockRequest;
    /// use rocket::http::ContentType;
    ///
    /// let req = MockRequest::new(Get, "/").header(ContentType::JSON);
    /// ```
    #[inline]
    pub fn header<'h, H: Into<Header<'static>>>(mut self, header: H) -> Self {
        self.request.add_header(header.into());
        self
    }

    /// Add a cookie to this request.
    ///
    /// # Examples
    ///
    /// Add `user_id` cookie:
    ///
    /// ```rust
    /// use rocket::http::Method::*;
    /// use rocket::testing::MockRequest;
    /// use rocket::http::Cookie;
    ///
    /// let req = MockRequest::new(Get, "/")
    ///     .cookie(Cookie::new("user_id".into(), "12".into()));
    /// ```
    #[inline]
    pub fn cookie(self, cookie: Cookie) -> Self {
        self.request.cookies().add(cookie);
        self
    }

    /// Set the body (data) of the request.
    ///
    /// # Examples
    ///
    /// Set the body to be a JSON structure; also sets the Content-Type.
    ///
    /// ```rust
    /// use rocket::http::Method::*;
    /// use rocket::testing::MockRequest;
    /// use rocket::http::ContentType;
    ///
    /// let req = MockRequest::new(Post, "/")
    ///     .header(ContentType::JSON)
    ///     .body(r#"{ "key": "value", "array": [1, 2, 3], }"#);
    /// ```
    #[inline]
    pub fn body<S: AsRef<str>>(mut self, body: S) -> Self {
        self.data = Data::new(body.as_ref().as_bytes().into());
        self
    }

    /// Dispatch this request using a given instance of Rocket.
    ///
    /// Returns the body of the response if there was a response. The return
    /// value is `None` if any of the following occurs:
    ///
    ///   1. The returned body was not valid UTF8.
    ///   2. The application failed to respond.
    ///
    /// # Examples
    ///
    /// Dispatch to a Rocket instance with the `"Hello, world!"` example
    /// mounted:
    ///
    /// ```rust
    /// # #![feature(plugin)]
    /// # #![plugin(rocket_codegen)]
    /// # extern crate rocket;
    /// #
    /// #[get("/")]
    /// fn hello() -> &'static str {
    ///     "Hello, world!"
    /// }
    ///
    /// use rocket::testing::MockRequest;
    /// use rocket::http::Method::*;
    ///
    /// # fn main() {
    /// let rocket = rocket::ignite().mount("/", routes![hello]);
    /// let mut req = MockRequest::new(Get, "/");
    /// let mut response = req.dispatch_with(&rocket);
    ///
    /// let body_str = response.body().unwrap().to_string().unwrap();
    /// assert_eq!(body_str, "Hello, world!".to_string());
    /// # }
    /// ```
    pub fn dispatch_with<'r>(&'r mut self, rocket: &Rocket) -> Response<'r> {
        let data = ::std::mem::replace(&mut self.data, Data::new(vec![]));
        match rocket.dispatch(&self.request, data) {
            Ok(response) => response,
            Err(status) => rocket.handle_error(status, &self.request)
        }
    }
}
