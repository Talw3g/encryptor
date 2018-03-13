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
use std::fs::OpenOptions;
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
        element.write_file()?;
        Ok(Some(element))
    }

}






struct TarElement {
    name: String,
    mode: usize,
    size: usize,
    typeflag: TypeFlag,
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

        if n == 0 { bail!("Unexpected EOF") }


        // Checking if we have two 512B-blocks of NUL (EOF)
        if buffer.iter().all(|&x| x == 0u8) {
            let mut buffer = [0;512];
            f.read(&mut buffer)
                .chain_err(|| format!("Error reading header in file {:?}",f))?;
            if buffer.iter().all(|&x| x == 0u8) {
                return Ok(None)
            }
        }

        let element = TarElement {
            name: buf_to_string(&buffer[0..100])?,
            mode: buf_to_num(&buffer[100..100+8])?,
            size: buf_to_num(&buffer[124..124+12])?,
            typeflag: typeflag(buffer[156])?,
            magic: buf_to_string(&buffer[257..257+6])?,
            user: buf_to_string(&buffer[265..265+32])?,
            group: buf_to_string(&buffer[297..297+32])?,
            prefix: buf_to_string(&buffer[345..345+155])?,
            data: Vec::new(),
        };

        if element.magic != String::from("ustar") {
            bail!("Not a tar header")
        }

        Ok(Some(element))
    }


    fn read_data(&mut self, f: &mut File) -> Result<()> {
        f.take(self.size as u64)
            .read_to_end(&mut self.data)
            .chain_err(|| format!("Error reading file {}", self.name))?;
        f.seek(
            SeekFrom::Current(
            offset(&self.size, 512)))
            .chain_err(|| "Error seeking position")?;
        Ok(())
    }


    fn write_file(&self) -> Result<()> {
        let path = Path::new("/tmp/")
            .join(&self.prefix)
            .join(&self.name);

        let dirpath = match self.typeflag {
            TypeFlag::RegFile => {
                path.parent().unwrap()
            },
            TypeFlag::Directory => {
                path.as_path()
            },
            TypeFlag::SymLink => {
                println!("Not creating symlinks, skipping {}", path.display());
                return Ok(())
            },
        };

        let _ = fs::create_dir_all(&dirpath)
            .chain_err(|| format!("Could not create directory {}", dirpath.display()))?;

        if self.typeflag == TypeFlag::RegFile {
            let mut fout = OpenOptions::new().write(true)
                .create(true).open(&path)
                .chain_err(|| format!("Could not create file {}", path.display()))?;
            fout.write(&self.data)
                .chain_err(|| format!("Error writing to file {}", path.display()))?;
        }
        Ok(())
    }

}



#[derive(Debug)]
#[derive(PartialEq)]
enum TypeFlag {
    RegFile,
    SymLink,
    Directory,
}




fn typeflag(tf: u8) -> Result<TypeFlag> {
    let tf = tf as char;
    match tf {
        '0'|'\0'|'7' => Ok(TypeFlag::RegFile),
        '2'          => Ok(TypeFlag::SymLink),
        '5'          => Ok(TypeFlag::Directory),
        _            => bail!(format!("Unknown or unmanaged typeflag: {}", tf)),
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

fn buf_to_string(buf: &[u8]) -> Result<String> {
    let vec: Vec<_> = buf.iter()
        .take_while(|&x| x != &0u8)
        .map(|&y| y)
        .collect();

    let string = String::from(
        String::from_utf8(vec)
        .chain_err(|| "Error converting to utf8")?
        .trim());

    Ok(string)
}

fn buf_to_num(buf: &[u8]) -> Result<usize> {
    let string = buf_to_string(buf)?;

    if string.is_empty() {
        bail!("buf_to_num: string is empty")
    }

    let num = usize::from_str_radix(string.as_str(), 8)
        .chain_err(|| "Error parsing string to usize")?;
    Ok(num)
}





fn run() -> Result<()> {
    let archive = TarArch::new(PathBuf::from("/tmp/a.tar"))?;

    for el in archive {
        let el = el?;
        println!("name: {:?}\nsize: {:?}\nmagic: {:?}\ntypeflag: {:?}",el.name, el.size, el.magic, el.typeflag);
//        ::std::process::exit(0);
    }

    println!("EOF, quitting");
    Ok(())
}
