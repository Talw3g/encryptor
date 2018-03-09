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
use std::io::prelude::*;
use std::io::SeekFrom;
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

impl Iterator for TarArch {
    type Item = Result<TarElement>;
    fn next(&mut self) -> Option<Result<TarElement>> {
        //converting Result<Option<>> to Option<Result<>>
        match self.next_element() {
            Ok(option) => match option {
                Some(element) => Some(Ok(element)),
                None => None,
            }
            Err(error) => Some(Err(error)),
        }
    }
}

impl TarArch {

    fn new(path: PathBuf) -> Result<(TarArch)> {
        Ok(TarArch {
            file: Some(File::open(path)
                .chain_err(|| "Could not open file")?),
        })
    }

    fn next_element(&mut self) -> Result<Option<TarElement>> {
        let file = match self.file {
            Some(ref mut file) => file,
            None => bail!("No file opened, aborting."),
        };
        let mut element = match TarElement::read_header(file)? {
            Some(el) => el,
            None => return Ok(None),
        };
        element.read_data(file)?;
        Ok(Some(element))
    }

}






struct TarElement {
    name: String,
    mode: usize,
    size: usize,
    magic: String,
    user: String,
    group: String,
    prefix: String,
    data: Vec<u8>,
}

impl TarElement {

    fn read_header(f: &mut File) -> Result<Option<TarElement>> {
        let mut buffer = [0;512];
        let n = f.read(&mut buffer)
            .chain_err(|| format!("Error reading header in file {:?}",f))?;

        if n == 0 { return Ok(None) }

        let element = TarElement {
            name: match buf_to_string(&buffer[0..100])? {
                Some(s) => s,
                None => return Ok(None),
            },
            mode: match buf_to_num(&buffer[100..100+8])? {
                Some(n) => n,
                None => return Ok(None),
            },
            size: match buf_to_num(&buffer[124..124+12])? {
                Some(n) => n,
                None => return Ok(None),
            },
            magic: match buf_to_string(&buffer[257..257+6])? {
                Some(s) => s,
                None => return Ok(None),
            },
            user: match buf_to_string(&buffer[265..265+32])? {
                Some(s) => s,
                None => return Ok(None),
            },
            group: match buf_to_string(&buffer[297..297+32])? {
                Some(s) => s,
                None => return Ok(None),
            },
            prefix: match buf_to_string(&buffer[345..345+155])? {
                Some(s) => s,
                None => return Ok(None),
            },
            data: Vec::new(),
        };

        if element.magic != String::from("ustar") {
            println!("magic value returns None");
            return Ok(None)
        }

        Ok(Some(element))
    }


    fn read_data(&mut self, f: &mut File) -> Result<()> {
        f.take(self.size as u64).read_to_end(&mut self.data)
            .chain_err(|| format!("Error reading file {}", self.name))?;
        f.seek(SeekFrom::Current(offset(&self.size, 512)))
            .chain_err(|| "Error seeking position")?;
        Ok(())
    }

}




fn offset(size: &usize, block: usize) -> i64 {
    // Takes a data chunk size and a standard block size,
    // and returns the offset necessary to jump to the next
    // block.
    let modulo = (size % block) as i64;

    if modulo == 0 {
        0
    } else {
        block as i64 - modulo
    }
}

fn buf_to_string(buf: &[u8]) -> Result<Option<String>> {
    let vec: Vec<_> = buf.iter()
        .take_while(|&x| x != &0u8)
        .map(|&y| y)
        .collect();

    let string = String::from(
        String::from_utf8(vec)
        .chain_err(|| "Error converting to utf8")?
        .trim());

    if string.is_empty() {
        Ok(None)
    } else {
        Ok(Some(string))
    }
}

fn buf_to_num(buf: &[u8]) -> Result<Option<usize>> {
    let string = match buf_to_string(buf)? {
        Some(s) => s,
        None => return Ok(None),
    };
    let num = usize::from_str_radix(string.as_str(), 8)
        .chain_err(|| "Error parsing string to usize")?;
    Ok(Some(num))
}





fn run() -> Result<()> {
    let archive = TarArch::new(PathBuf::from("/tmp/a.tar"))?;

    for el in archive {
        let el = el?;
        println!("name: {:?}\nsize: {:?}\nmagic: {:?}",el.name, el.size, el.magic);
//        ::std::process::exit(0);
    }


    Ok(())
}
