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

struct TarArch {
    file: Option<File>,

}

impl TarArch {

    fn new(path: PathBuf) -> Result<(TarArch)> {
        Ok(TarArch {
            file: Some(File::open(path)
                .chain_err(|| "Could not open file")?),
        })
    }

    fn next_element(&mut self) -> Option<Result<TarElement>> {
        let mut file = match self.file {
            Some(ref mut f) => f,
            None => return None,
        };
        let mut element = match TarElement::read_header(file) {
            Some(Ok(el)) => el,
            Some(Err(e)) => return Some(Err(e)),
            None => return None,
        };
        Some(Ok(element))
    }

}

impl Iterator for TarArch {
    type Item = Result<TarElement>;
    fn next(&mut self) -> Option<Result<TarElement>> {
        self.next_element()
    }
}

struct TarElement {
    filename: String,
    size: usize,
}

impl TarElement {

    fn read_header(f: &mut File) -> Option<Result<TarElement>> {
        let mut element = TarElement { filename: String::from("tata"), size: 0 };
        bail!("tata !");
        Some(Ok(element))
    }

}

fn run() -> Result<()> {
    let archive = TarArch::new(PathBuf::from("/tmp/a.tar"))?;

    for el in archive {
        ::std::process::exit(0);

    }


    Ok(())
}
