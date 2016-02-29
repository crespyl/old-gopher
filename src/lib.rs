//! A simple library for working with the Gopher protocol, as described
//! in [RFC 1436](https://tools.ietf.org/html/rfc1436)
//!
//! # Usage
//!
//! ```
//! use gopher::*;
//!
//! let input = "0About internet Gopher\tStuff:About us\trawBits.micro.umn.edu\t70
//! 1Around University of Minnesota\tZ,5692,AUM\tunderdog.micro.umn.edu\t70
//! 1Microcomputer News & Prices\tPrices/\tpserver.bookstore.umn.edu\t70
//! 1Courses, Schedules, Calendars\t\tevents.ais.umn.edu\t9120
//! 1Student-Staff Directories\t\tuinfo.ais.umn.edu\t70
//! 1Departmental Publications\tStuff:DP:\trawBits.micro.umn.edu\t70
//! .";
//!
//! let directory = Directory::from_str(input).unwrap();
//! let items = directory.items();
//!
//! assert_eq!(items.len(), 6);
//! assert_eq!(items[0].t, Type::File);
//! assert_eq!(items[3].port, 9120);
//! ```
//!
//! # Examples
//!
//! This library includes as an example a simple command-line gopher client,
//! which can be run with `cargo run --example client` or `cargo run --example client -- hostname:port resource`.
//!
//! ```
//! $ cargo run --example client -- cargo run --example client -- gopher.quux.org:70 /Software/Gopher/servers
//!     Running `target/debug/examples/client gopher.quux.org:70 /Software/Gopher/servers`
//! Got Directory:
//! 
//! 1 Aerv.nl                                                      aerv.nl:70
//! 1 Dark Side Of The Net                                         gopher.rp.spb.su:70
//! 1 Floodgap.Com -- featuring a Gopherspace search engine        gopher.floodgap.com:70
//! 1 Gopher Jewels 2 from JumpJet Gopher                          home.jumpjet.info:70 \Gopher_Jewels_2
//! 1 Hal3000.cx                                                   Hal3000.cx:70
//! 1 Heatdeath.Org                                                gopher.heatdeath.org:70
//! 1 Heavything.Com                                               gopher.heavything.com:70
//! 1 Ocean State Free-Net Gopher                                  gopher.osfn.org:70
//! 1 Quux.Org/GopherProject.Org Gopher Archive                    quux.org:70
//! 1 SDF Public Access UNIX                                       freeshell.org:70
//! SDF offers gopher hosting as well!
//! 1 Simple Machines                                              jgw.mdns.org:70
//! 1 Whole Earth 'Lectronic Links                                 gopher.well.sf.ca.us:70
//! 1 Zedah.Org                                                    zedah.org:70
//! 1 shamrockshire.yi.org                                         shamrockshire.yi.org:70
//! 1 xn--ortsvernderlich-6kb.de                                   gopher.xn--ortsvernderlich-6kb.de:70
//! $
//! ```

#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;

use std::io;
use std::fmt;

pub mod net;

#[derive(Debug)]
pub enum GopherError {
    Io(io::Error),
    ParseDirectoryItem(String),
    ParseDirectory(String),
}

impl From<io::Error> for GopherError {
    fn from(io: io::Error) -> GopherError {
        GopherError::Io(io)
    }
}

/// Possible types of Gopher directory items
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    File,
    Directory,
    CSOPhoneBook,
    Error,
    BinHexed,
    BinArchive,
    UUEncoded,
    SearchServer,
    TelnetSession,
    Binary,
    RedundantServer,
    Tn3270Session,
    GIF,
    Image,
    Unknown(char),
}

impl Type {
    /// Utility function for identifying directory items
    pub fn is_directory(&self) -> bool {
        *self == Type::Directory
    }

    /// Utility function for identifying plain file items
    pub fn is_file(&self) -> bool {
        match *self {
            Type::Directory | Type::CSOPhoneBook |
            Type::Error | Type::SearchServer |
            Type::TelnetSession | Type::Tn3270Session |
            Type::RedundantServer | Type::Unknown
                => true,
            _ => false,
        }
    }

    /// Convert a char into a Gopher Type
    pub fn from_char(c: char) -> Type {
        match c {
            '0' => Type::File,
            '1' => Type::Directory,
            '2' => Type::CSOPhoneBook,
            '3' => Type::Error,
            '4' => Type::BinHexed,
            '5' => Type::BinArchive,
            '6' => Type::UUEncoded,
            '7' => Type::SearchServer,
            '8' => Type::TelnetSession,
            '9' => Type::Binary,
            '+' => Type::RedundantServer,
            'T' => Type::Tn3270Session,
            'g' => Type::GIF,
            'I' => Type::Image,
            other => Type::Unknown(other)
        }
    }

    /// Convert back into a char
    pub fn as_char(&self) -> char {
        match *self {
            Type::File => '0',
            Type::Directory => '1',
            Type::CSOPhoneBook => '2',
            Type::Error => '3',
            Type::BinHexed => '4',
            Type::BinArchive => '5',
            Type::UUEncoded => '6',
            Type::SearchServer => '7',
            Type::TelnetSession => '8',
            Type::Binary => '9',
            Type::RedundantServer => '+',
            Type::Tn3270Session => 'T',
            Type::GIF => 'g',
            Type::Image => 'I',
            Type::Unknown(other) => other,
        }
    }
}

/// An item in a Gopher Directory
#[derive(Debug)]
pub struct DirectoryItem {
    pub t: Type,
    pub name: String,
    pub selector: String,
    pub host: String,
    pub port: usize,
}

impl DirectoryItem {
    /// Parse a &str into a DirectoryItem
    pub fn from_str(s: &str) -> Result<DirectoryItem, GopherError> {
        if s.len() <= 1 {
            return Err(GopherError::ParseDirectoryItem(s.into()));
        }

        let re = regex!(r"(?P<t>.)(?P<name>[^\t]*)\t(?P<selector>[^\t]*)\t(?P<host>[^\t]*)\t(?P<port>\d*)");

        if let Some(captures) = re.captures(s) {
            Ok(
                DirectoryItem {
                    t: Type::from_char(s.chars().nth(0).unwrap()),
                    name: captures.name("name").unwrap().into(),
                    selector: captures.name("selector").unwrap().into(),
                    host: captures.name("host").unwrap().into(),
                    port: captures.name("port").unwrap().parse().unwrap_or(70),
                }
            )
        } else {
            Err(GopherError::ParseDirectoryItem(s.into()))
        }
    }

    /// Many Gopher servers use "fake" items to provide human readable text in
    /// directory listings.
    /// This function is a simple heuristic, and shouldn't really be relied upon
    pub fn is_fake(&self) -> bool {
        self.selector.ends_with("fake") ||
            self.name == "fake" ||
            self.host == "fake"
    }
}

impl fmt::Display for DirectoryItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{sym}{name}\t{selector}\t{host}\t{port}",
                sym = self.t.as_char(),
                name = self.name,
                selector = self.selector,
                host = self.host,
                port = self.port)
    }
}

/// A Gopher Directory
#[derive(Debug)]
pub struct Directory {
    items: Vec<DirectoryItem>
}

impl Directory {
    /// Parse a &str into a Directory
    pub fn from_str(s: &str) -> Result<Directory, GopherError> {
        let mut items = Vec::new();
        for line in s.lines() {
            if line == "." { break; }
            else if let Ok(item) = DirectoryItem::from_str(line) {
                items.push(item);
            } else {
                return Err(GopherError::ParseDirectoryItem(line.into()));
            }

        }
        Ok(Directory { items: items })
    }

    /// Returns the list of DirectoryItems
    pub fn items(&self) -> &[DirectoryItem] {
        &self.items
    }
}

impl fmt::Display for Directory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for item in &self.items {
            try!(write!(f, "{}\n", item));
        }
        write!(f, ".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_directory() {
        let input = "0About internet Gopher\tStuff:About us\trawBits.micro.umn.edu\t70
1Around University of Minnesota\tZ,5692,AUM\tunderdog.micro.umn.edu\t70
1Microcomputer News & Prices\tPrices/\tpserver.bookstore.umn.edu\t70
1Courses, Schedules, Calendars\t\tevents.ais.umn.edu\t9120
1Student-Staff Directories\t\tuinfo.ais.umn.edu\t70
1Departmental Publications\tStuff:DP:\trawBits.micro.umn.edu\t70
.";
        let directory = Directory::from_str(input).expect("failed to parse sample directory");
        assert_eq!(directory.items.len(), 6);

        let item0 = &directory.items[0];
        assert_eq!(item0.t, Type::File);
        assert_eq!(item0.name, "About internet Gopher");
        assert_eq!(item0.selector, "Stuff:About us");
        assert_eq!(item0.host, "rawBits.micro.umn.edu");
        assert_eq!(item0.port, 70);

        let item3 = &directory.items[3];
        assert_eq!(item3.t, Type::Directory);
        assert_eq!(item3.name, "Courses, Schedules, Calendars");
        assert_eq!(item3.selector, "");
        assert_eq!(item3.host, "events.ais.umn.edu");
        assert_eq!(item3.port, 9120);
    }

    #[test]
    fn format_directory() {
        let input = "0About internet Gopher\tStuff:About us\trawBits.micro.umn.edu\t70
1Around University of Minnesota\tZ,5692,AUM\tunderdog.micro.umn.edu\t70
1Microcomputer News & Prices\tPrices/\tpserver.bookstore.umn.edu\t70
1Courses, Schedules, Calendars\t\tevents.ais.umn.edu\t9120
1Student-Staff Directories\t\tuinfo.ais.umn.edu\t70
1Departmental Publications\tStuff:DP:\trawBits.micro.umn.edu\t70
.";
        let directory = Directory::from_str(input).expect("failed to parse sample directory");
        let output = format!("{}", directory);
        assert_eq!(input, output);
    }
    
    #[test]
    fn parse_directory_item() {
        let input = "0A Sample Text File\t/sample.txt\tgopher.example.net\t70";
        let item = DirectoryItem::from_str(input).expect("failed to parse sample directory item");
        assert_eq!(item.t, Type::File);
        assert_eq!(item.name, "A Sample Text File");
        assert_eq!(item.selector, "/sample.txt");
        assert_eq!(item.host, "gopher.example.net");
        assert_eq!(item.port, 70);
    }

    #[test]
    fn format_directory_item() {
        let item = DirectoryItem {
            t: Type::File,
            name: String::from("A Sample Text File"),
            selector: String::from("/sample.txt"),
            host: String::from("gopher.example.net"),
            port: 70,
        };
        let output = format!("{}",item);
        assert_eq!(output, "0A Sample Text File\t/sample.txt\tgopher.example.net\t70");
    }
}
