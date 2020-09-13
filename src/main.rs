extern crate rustbox;
extern crate gopher;

use std::env;
use std::io;
use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use gopher::*;

use rustbox::{ Color, Key, RustBox };

pub const MENU_KEYS: &'static str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!@#$%^&*()-+_=";

enum State {
    DisplayDirectory( String, Directory, usize ),
    DisplayResource( String, String, usize ),
    ShowMessage( String ),
    Error( GopherError ),
}

struct Gopher {
    current_host: String,
    current_port: u16,
    current_selector: String,
    states: Vec<State>,
}

impl Gopher {
    pub fn new(host: &str, port: u16, selector: &str) -> Gopher {
        let resource = get_resource(host, port, selector);
        Gopher {
            current_host: host.into(),
            current_port: port,
            current_selector: selector.into(),
            states: vec![
                match resource {
                    Ok(s) => match Directory::from_str(&s) {
                        Ok(directory) => State::DisplayDirectory(format!("{}:{} {}", host, port, selector), directory, 0),
                        Err(e) => State::Error(e) },
                    Err(e) => State::Error(GopherError::Io(e))
                }
            ],
        }
    }

    fn display_directory(rb: &RustBox, dir: &Directory, scroll: usize) {
        let mut line_number = 0;
        let mut item_number = 0;
        for item in dir.items().iter().skip(scroll) {
            if item.is_info() {
                rb.print(0, line_number, rustbox::RB_NORMAL, Color::White, Color::Black, &item.name);
            } else {
                let mut col = 0;
                let button = format!("[{}]", &MENU_KEYS[item_number..item_number+1]);
                rb.print(col, line_number, rustbox::RB_BOLD, Color::White, Color::Black, &button);

                col += button.len()+1;

                match item.t {
                    Type::Unknown(c) => rb.print(col, line_number,
                                                 rustbox::RB_BOLD, Color::White, Color::Red, &format!("{}",c)),
                    Type::Directory => rb.print(col, line_number,
                                                rustbox::RB_BOLD, Color::White, Color::Blue, "/"),
                    _ => {}
                    
                }

                col += 2;
                
                let name = format!("{}", item.name);
                rb.print(col, line_number, rustbox::RB_UNDERLINE, Color::White, Color::Black, &name);
                col += name.len()+1;

                let link = format!("{host}:{port} {selector}",
                                   selector=item.selector,
                                   host=item.host,
                                   port=item.port);
                
                item_number += 1;
            }
            line_number += 1;
        }

        if scroll != 0 {
            let note = format!("[{}/{}]", scroll, dir.items().len());
            rb.print(0, rb.height()-2,
                     rustbox::RB_NORMAL, Color::White, Color::Blue, &note);
        }
    }

    fn display_string(rb: &RustBox, s: &str, scroll: usize) {
        for (i, line) in s.lines().skip(scroll).enumerate() {
            rb.print(0, i,
                     rustbox::RB_NORMAL, Color::White, Color::Black, line);
        }

        if scroll != 0 {
            let note = format!("[{}/{}]", scroll, s.lines().count());
            rb.print(0, rb.height()-2,
                     rustbox::RB_NORMAL, Color::White, Color::Blue, &note);
        }
    }

    fn display_status(rb: &RustBox, s: &str) {
        let line = rb.height()-1;
        rb.print(0, line,
                 rustbox::RB_NORMAL, Color::White, Color::Blue, s);
    }

    pub fn display(&self, rb: &RustBox) {
        match *self.current_state() {
            State::DisplayDirectory(ref location, ref dir, scroll) => {
                Gopher::display_directory(rb, dir, scroll);
                Gopher::display_status(rb, location);
            },
            State::DisplayResource(ref location, ref s, scroll) => {
                Gopher::display_string(rb, s, scroll);
                Gopher::display_status(rb, location);
            },
            State::ShowMessage(ref s) => Gopher::display_string(rb, s, 0),
            State::Error(ref e) => Gopher::display_string(rb, &format!("{:?}",e), 0)
        }
    }

    pub fn current_state(&self) -> &State {
        if self.states.len() > 0 {
            &self.states[self.states.len()-1]
        } else {
            panic!("lost root state")
        }
    }

    pub fn current_state_mut(&mut self) -> &mut State {
        let len = self.states.len();
        if len > 0 {
            &mut self.states[len-1]
        } else {
            panic!("lost root state")
        }
    }

    pub fn scroll(&mut self, amount: isize) {
        match *self.current_state_mut() {
            State::DisplayDirectory(_, _, ref mut scroll) |
            State::DisplayResource(_, _, ref mut scroll) => {
                let new_scroll = *scroll as isize + amount;
                if new_scroll < 0 {
                    *scroll = 0
                } else {
                    *scroll = new_scroll as usize;
                }
            }
            _ => {}
        }
    }

    /// Return to the previous state
    pub fn pop_state(&mut self) {
        if self.states.len() > 1 {
            self.states.pop();
        }
    }

    /// Choose the nth item in the current directory
    /// Shows an error if not already in a directory
    pub fn activate_item(&mut self, n: usize) {
        let new_state = match *self.current_state() {
            State::DisplayDirectory(_, ref dir, scroll) => {
                if let Some(item) = dir.items()
                    .iter()
                    .skip(scroll)
                    .filter(|&item| !item.is_info())
                    .nth(n) {
                        match get_resource(&*item.host, item.port, &*item.selector) {
                            Ok(resource) => match Directory::from_str(&resource) {
                                Ok(directory) => State::DisplayDirectory(
                                    format!("{}:{} {}", &*item.host, item.port, &*item.selector),
                                    directory, 0
                                ),
                                Err(e) => State::DisplayResource(
                                    format!("{}:{} {}", &*item.host, item.port, &*item.selector),
                                    resource, 0
                                )
                            },
                            Err(e) => State::Error(GopherError::Io(e))
                        }
                    } else {
                        State::ShowMessage("No such item".into())
                    }
            },
            _ => State::ShowMessage("Not in a directory".into())
        };
        self.states.push(new_state);
    }
}

fn get_resource(host: &str, port: u16, selector: &str) -> Result<String, io::Error> {
    let address = (host, port);
    let mut stream = TcpStream::connect(address)?;

    // set default timeouts to 5 seconds
    stream.set_read_timeout(Some(Duration::new(5, 0)))?;
    stream.set_write_timeout(Some(Duration::new(5, 0)))?;

    // send the directory selector
    write!(stream, "{}\n", selector)?;

    let mut buffer = String::new();
    stream.read_to_string(&mut buffer)?;

    Ok(buffer)
}

fn get_resource_from_item(item: &DirectoryItem) -> Result<String, io::Error> {
    get_resource(&item.host, item.port, &item.selector)
}


fn main() {
    let rustbox = match RustBox::init(rustbox::InitOptions {
        input_mode: rustbox::InputMode::Current,
        output_mode: rustbox::OutputMode::Current,
        buffer_stderr: true,
    }) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("rustbox err: {}", e),
    };
    
    let mut args = env::args();

    let host = args.nth(1).unwrap_or(String::from("gopher.quux.org"));
    let port = 70;
    let selector = args.next().unwrap_or(String::from(""));

    let mut gopher = Gopher::new(&host, port, &selector);

    rustbox.clear();

    loop {
        rustbox.clear();
        gopher.display(&rustbox);
        rustbox.present();

        match rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    // quit
                    Key::Ctrl('c') | Key::Ctrl('q') => { break; }

                    // back
                    Key::Esc | Key::Tab => { gopher.pop_state(); }

                    // menu entries
                    Key::Char(pressed) => {
                        for (n, c) in MENU_KEYS.chars().enumerate() {
                            if pressed == c {
                                gopher.activate_item(n)
                            }
                        }
                    }

                    // scroll
                    Key::Up => {
                        gopher.scroll(-1);
                    }
                    Key::Down => {
                        gopher.scroll(1);
                    }
                    Key::PageUp => {
                        gopher.scroll(-10);
                    }
                    Key::PageDown => {
                        gopher.scroll(10);
                    }

                    _ => { }
                }
            },
            Err(e) => panic!("{:?}", e),
            _ => { }
        }
    }
}
