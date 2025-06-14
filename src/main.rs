use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::process;

fn die<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("Fatal: {}", msg.as_ref());
    process::exit(1);
}

// This is a primitive that will copy one file
fn do_copy(source: &str, dest: &str, dexist: &bool, ddir: &bool) {
    let dest_string = String::from(dest);
    let source_string = String::from(source);
    let mut dbuf = PathBuf::from(&dest_string);
    let mut sbuf = PathBuf::from(&source_string);
    // If the destination exists and is a directory, we should append the
    // original filename to the path.
    if *dexist && *ddir {
        // Need the basename of the source path.
        let sname = sbuf.file_name();
        match sname {
            Some(_os_str) => { dbuf = Path::new(dest).join(sname.unwrap()); },
            None => { die("don't know how to handle this source"); }
        }
    }
    println!("need to copy {} to {}", sbuf.display(), dbuf.display());

    match fs::copy(sbuf, dbuf) {
        Ok(_) => { println!("good copy"); },
        Err(e) => { die(format!("copy failed: {}", e)) }
    }
}

fn main() {

    // FIXME: handle panics here more nicely
    let source = std::env::args().nth(1).expect("No source given");
    let dest = std::env::args().nth(2).expect("No dest given");

    let source_exists = fs::exists(&source);
    let mut source_exists_b: bool = false;
    match source_exists {
        Ok(true) => { source_exists_b = true; },
        Ok(false) => { source_exists_b = false; },
        Err(_) => { die("error attempting to check source file existence"); }
    }

    if source_exists_b {
        // Is it a file? We don't copy anything else yet.
        println!("source exists");
        let attr: Metadata = fs::metadata(&source).unwrap();
        if !attr.is_file() {
            die("source must be a file at this time");
        }
    } else {
        die ("source does not exist");
    }

    // The destination should be a pre-existing directory, or a file that may
    // or may not exist.
    // Does the destination exist?
    let dest_exists = fs::exists(&dest);

    let mut dest_exists_b: bool = false;
    match dest_exists {
        Ok(true) => { dest_exists_b = true; },
        Ok(false) => { dest_exists_b = false; },
        Err(_) => { die("error attempting to check destination existence"); }
    }

    let mut ddir: bool = false;

    if dest_exists_b {
        println!("destination exists");
        let attr = fs::metadata(&dest).unwrap();
        if attr.is_dir() {
            println!("{} is a directory", dest);
            ddir = true;
        }
    } else {
        println!("destination does not exist");
    }

    do_copy(&source, &dest, &dest_exists_b, &ddir);
}
