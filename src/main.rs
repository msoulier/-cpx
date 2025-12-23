use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

const QUIET: bool = false;

fn die<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("Fatal: {}", msg.as_ref());
    process::exit(1);
}

fn copy_with_progress<P, Q, F>(
    source: P,
    dest: Q,
    buffer_size: usize,
    progress_hook: F,
    dexist: &bool,
    ddir: &bool) -> io::Result<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    F: Fn(u64, u64),
{
    let source_path = source.as_ref();
    let mut dest_path = dest.as_ref().to_path_buf();

    // If the destination exists and is a directory, we should append the
    // original filename to the path.
    if *dexist && *ddir {
        if let Some(filename) = source_path.file_name() {
            dest_path = dest_path.join(filename);
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "source has no filename"));
        }
    }

    println!("copying {} to {}", source_path.display(), dest_path.display());

    let mut srcfile = File::open(source_path)?;
    let mut dstfile = File::create(&dest_path)?;
    let mut buffer = vec![0u8; buffer_size];
    let mut total_bytes = 0u64;

    let file_size = srcfile.metadata()?.len();

    loop {
        let bytes_read = srcfile.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        dstfile.write_all(&buffer[..bytes_read])?;
        total_bytes += bytes_read as u64;

        progress_hook(total_bytes, file_size);
    }
    Ok(total_bytes)
}

// This is a primitive that will copy one file
fn quiet_copy(source: &str, dest: &str, dexist: &bool, ddir: &bool) -> io::Result<u64>
{
    let dest_string = String::from(dest);
    let source_string = String::from(source);
    let mut dbuf = PathBuf::from(&dest_string);
    let sbuf = PathBuf::from(&source_string);
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
    println!("copying {} to {}", sbuf.display(), dbuf.display());

    match fs::copy(sbuf, dbuf) {
        Ok(bytes_copied) => { return Ok(bytes_copied); }
        Err(e) => { return Err(e); }
    }
}

fn progress(current: u64, total: u64) {
    let percentage: f64 = ( current as f64 / total as f64 ) * 100.0;
    print!("                                                 ");
    print!("\r");
    print!("copied {} bytes of {} total: {:.2}%", current, total, percentage);
    if percentage >= 100.0 {
        println!();
    }
    // flush
    io::stdout().flush().unwrap();
}

fn main() {

    // FIXME: handle panics here more nicely
    let source = std::env::args().nth(1).expect("No source given");
    let dest = std::env::args().nth(2).expect("No dest given");

    let source_exists = fs::exists(&source);
    let source_exists_b: bool;
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

    let dest_exists_b: bool;
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

    if QUIET {
        match quiet_copy(&source, &dest, &dest_exists_b, &ddir) {
            Ok(_) => { println!("done"); }
            Err(e) => { die(format!("error in copy: {}", e)); }
        }
    } else {
        match copy_with_progress(&source, &dest, 40960, &progress, &dest_exists_b, &ddir) {
            Ok(_) => { println!("done"); }
            Err(e) => { die(format!("error in copy: {}", e)); }
        }
    }
}
