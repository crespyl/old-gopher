//! Network Utilities
//!
//! This module defines a handful of helper functions for talking to remote
//! Gopher servers.  These can be useful for proof-of-concept or getting for
//! getting started, but probably shouldn't be used for anything more serious.

use std::io;
use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use GopherError;
use Directory;

/// Utility function to read a resource from a server
fn read_string<T: ToSocketAddrs>(address: T, selector: &str) -> Result<String, io::Error> {
    let mut stream = try!(TcpStream::connect(address));

    // set default timeouts to 5 seconds
    try!(stream.set_read_timeout(Some(Duration::new(5, 0))));
    try!(stream.set_write_timeout(Some(Duration::new(5, 0))));

    // send the directory selector
    try!(write!(stream, "{}\n", selector));

    let mut buffer = String::new();
    try!(stream.read_to_string(&mut buffer));

    Ok(buffer)
}

/// Connect to a Gopher server and read the specified directory
pub fn read_directory<T: ToSocketAddrs>(address: T, selector: &str) -> Result<Directory, GopherError> {
    let buffer = try!(read_string(address, selector));
    Directory::from_str(&buffer)
}

/// Connect to a Gopher server and read the specified resource
/// If the result can be parsed as a Directory, return the result, otherwise
/// return the plain string
pub fn read_directory_or_resource<T: ToSocketAddrs>(address: T, selector: &str) -> Result<Result<Directory, String>, GopherError> {
    let buffer = try!(read_string(address, selector));
    if let Ok(directory) = Directory::from_str(&buffer) {
        Ok(Ok(directory))
    } else {
        Ok(Err(buffer))
    }
}
