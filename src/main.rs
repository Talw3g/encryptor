#[macro_use]
extern crate error_chain;
extern crate walkdir;

mod other_error {
    error_chain!{}
}

error_chain!{

    links {
        Another(other_error::Error, other_error::ErrorKind) #[cfg(unix)];
    }

}

use std::fs;
use std::fs::File;
//use std::fs::OpenOptions;
//use std::io;
//use std::io::prelude::*;
//use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::iter::*;
//use std::fmt;
//use walkdir::WalkDir;

fn main() {

    if let Err(ref e) = run() {

        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);
        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

struct TarFile {
    file: Option<File>,

}

impl TarFile {

    fn new(path: PathBuf) -> Result<(TarFile)> {
        Ok(TarFile {
            file: Some(File::open(path)
                .chain_err(|| "Could not open file")?),
        })
    }

    fn next_element(&self) -> Option<TarElement> {
        Some(TarElement {})
    }

}

impl Iterator for TarFile {
    type Item = TarElement;
    fn next(&mut self) -> Option<TarElement> {
        self.next_element()
    }
}

struct TarElement {

}

impl TarElement {

}

fn run() -> Result<()> {
    let tarfile = TarFile::new(PathBuf::from("/tmp/a.tar"))?;

    for el in tarfile {

    }


    Ok(())
}
