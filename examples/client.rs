extern crate gopher;

use gopher::*;
use gopher::net::*;

use std::env;

fn pretty_print_directory(dir: &Directory) {
    for item in dir.items().iter() {
        if item.is_info() {
            println!("{}", item.name);
        } else {
            println!("{sym} {name:100} {host}:{port} {selector}",
                     sym=item.t.as_char(),
                     name=item.name,
                     selector=item.selector,
                     host=item.host,
                     port=item.port);
        }
    }
}

fn main() {
    let mut args = env::args();

    let host = args.nth(1).unwrap_or(String::from("gopher.quux.org:70"));
    let selector = args.next().unwrap_or(String::from(""));

    let result = read_directory_or_resource(&*host, &selector)
        .expect("could not read resource");

    if let Ok(directory) = result {
        println!("Got Directory:\n");
        pretty_print_directory(&directory);        
    } else {
        if let Err(resource) = result {
            println!("Got Resource:\n");
            println!("{}", resource);
        }
    }
}
