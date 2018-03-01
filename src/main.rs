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
use std::path::Path;
//use std::iter::FromIterator;
use std::collections::HashMap;
use std::process::exit;
use std::fmt;
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

#[derive(Debug)]
struct Field {
    value: Option<String>,
    num_value: Option<u64>,
    start: usize,
    len: usize,
    is_num: bool,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"is_num: {}
    value: {:?}
    num_value: {:?}
    start: {}
    len: {}",
            self.is_num, self.value, self.num_value, self.start, self.len)
    }
}

fn header_init() -> HashMap<String,Field> {
    let num_fld_names = ["mode","uid","gid","size","mtime","chksum",
        "typeflag","devmajor","devminor"];
    let num_fld_start: [usize;9] = [100,108,116,124,136,148,156,329,337];
    let num_fld_len: [usize;9] = [8,8,8,12,12,8,1,8,8];
    let str_fld_names = ["name","linkname","magic","uname","gname","prefix"];
    let str_fld_start: [usize;6] = [0,157,257,265,297,345];
    let str_fld_len: [usize;6] = [100,100,6,32,32,155];
    let mut header = HashMap::new();

    for (i,&name) in num_fld_names.iter().enumerate() {
        header.insert(String::from(name),Field {
            value: None,
            num_value: None,
            start: num_fld_start[i],
            len: num_fld_len[i],
            is_num: true,
        });
    }
    for (i,&name) in str_fld_names.iter().enumerate() {
        header.insert(String::from(name),Field {
            value: None,
            num_value: None,
            start: str_fld_start[i],
            len: str_fld_len[i],
            is_num: false,
        });
    }
    header
}

fn read_header(f: &mut File) -> Result<HashMap<String,Field>>{
    let mut header = header_init();
    let mut buffer = [0;512];
    let n = f.read(&mut buffer)
        .chain_err(|| format!("Error reading header in file {:?}",f))?;

    if n == 0 { exit(0) }

    for (_, val) in header.iter_mut() {
        let vec: Vec<_> = buffer[val.start..val.start + val.len]
            .iter()
            .take_while(|&x| x != &0u8)
            .map(|&y| y)
            .collect();
        if !vec.is_empty() {
            let string = String::from(
                String::from_utf8(vec)
                .chain_err(|| "Error converting from utf8")?
                .trim());
            if val.is_num {
                val.num_value = Some(u64::from_str_radix(string.as_str(), 8)
                    .chain_err(|| "Error while parsing string to u64")?);
            }
            val.value = Some(string);
        }
    }

/*
    for (key, val) in header.iter() {
        println!("{} {}",key,val);
    }
*/

    let is_valid = header.get("magic")
        .ok_or("No field 'magic' in header")?
        .value.as_ref()
        .ok_or("No 'magic' value")?
        == &String::from("ustar");
    if is_valid {
        return Ok(header)
    } else {
        bail!("not a tar archive")
    }
}

fn read_data(f: &mut File, header: &HashMap<String,Field>) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let size = match header.get("size") {
        Some(field) => match field.num_value {
            Some(val) => val,
            None => panic!("File has no size value"),
        },
        None => panic!("No field 'size' in header:\n{:?}", header),
    };
    let offset = (size % 512 + 1024) as i64;
    f.take(size).read_to_end(&mut data)
        .chain_err(|| "Error reading data in file")?;
    f.seek(SeekFrom::Current(offset))
        .chain_err(|| "Error while seeking pos")?;
    Ok(data)
}

fn write_file(header: &HashMap<String,Field>, data: Vec<u8>) -> Result<()> {
    let name = header.get("name").ok_or("No field 'name' in header")?
        .value.as_ref().ok_or("No 'name' value")?;
    let prefix = match header.get("prefix").ok_or("No field 'prefix' in header")?.value {
        Some(ref val) => val.as_str(),
        None => "",
    };

    let pathbuf = Path::new("/home/thibault/test").join(&prefix).join(&name);
    let path = pathbuf.as_path();

    let _ = fs::create_dir_all(&path)
        .chain_err(|| format!("Could not create directory {}", path.display()))?;
    if path.is_file() {
        let mut fout = OpenOptions::new().write(true)
            .create(true).open(path)
            .chain_err(|| format!("Could not create file {}",path.display()))?;
        fout.write(&data)
            .chain_err(|| "Error writing to file")?;
    }
    Ok(())

}


fn run() -> Result<()> {
    let f = &mut File::open("/home/thibault/test/a.tar")
        .chain_err(|| "Could not open file")?;
    loop {
        let header = read_header(f)?;
        let data = read_data(f, &header)?;
        let _ = write_file(&header, data)?;
    }
/*    for entry in WalkDir::new("/home/thibault/test") {
        let entry = entry.chain_err(|| "Path not found")?;
        let path = Path::new(entry
            .path().to_str()
            .chain_err(|| "Contains invalid unicode")?);

        if path.is_file() {
            let mut f = File::open(path)
                .chain_err(|| "Could not open file")?;

            let mut buffer = Vec::new();

            let start = f.read_to_end(&mut buffer)
                .chain_err(|| "Error reading file")? as u64;

            let mut fout = OpenOptions::new().write(true)
                .create(true).open("/home/thibault/test/out")
                .chain_err(|| "Could not create file")?;

            fout.write(&buffer)
                .chain_err(|| "Error writing to file")?;

            let mut f = File::open("/home/thibault/test/Ejection parachute.png")
                .chain_err(|| "Could not open file")?;

            let mut buffer = Vec::new();

            let _ = f.read_to_end(&mut buffer)
                .chain_err(|| "Error reading file")?;

            fout.write(&buffer)
                .chain_err(|| "Error writing to file")?;

            let f = &mut File::open("/home/thibault/test/out")
                .chain_err(|| "Could not open file")?;

            f.seek(SeekFrom::Start(start))
                .chain_err(|| "Error while seeking pos")?;
            let mut buffer = Vec::new();
            let _ = f.read_to_end(&mut buffer)
                .chain_err(|| "Error reading file")?;

            let mut fout = OpenOptions::new().write(true)
                .create(true).open("/home/thibault/test/new.png")
                .chain_err(|| "Could not create file")?;

            fout.write(&buffer)
                .chain_err(|| "Error writing to file")?;

        }
    }
*/
}
