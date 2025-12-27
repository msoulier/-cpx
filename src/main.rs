use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use clap::Parser;
use indicatif::{ProgressBar,ProgressStyle};
use clonetree::{clone_tree,Options};

////////////////////////////////////////////////////
/// The progress indicator.
trait Progress {
    fn init(&mut self, total: u64);
    fn tick(&mut self, current: u64);
    fn finish(&mut self);
}

struct ProgressIndicator {
    current: u64,
    total: u64,
    bar: ProgressBar,
    oldcurrent: u64,
    percentage: f64,
}

impl Progress for ProgressIndicator {
    fn init(&mut self, total: u64) {
        self.total = total;
        let bar = ProgressBar::new(total);
        bar.set_style(ProgressStyle::with_template("[{bytes_per_sec}] [ETA {eta}] {bar:40.cyan/blue} {binary_bytes} of {binary_total_bytes} {percent}% Complete")
            .unwrap()
            .progress_chars("##-"));
        self.bar = bar;
    }

    fn tick (&mut self, current: u64) {
        self.current = current;
        self.percentage = ( self.current as f64 / self.total as f64 ) * 100.0;
        // flush
        //io::stdout().flush().unwrap();
        self.bar.inc(self.current-self.oldcurrent);
        self.oldcurrent = self.current;
    }

    fn finish(&mut self) {
        self.bar.finish();
    }
}

impl ProgressIndicator {
    fn new(current: u64, total: u64) -> Self {
        // FIXME: we're going to throw this object away in init - need to refactor
        let bar = ProgressBar::new(total);
        ProgressIndicator { current: current, total: total, bar: bar, oldcurrent: 0, percentage: 0.0 }
    }
}

////////////////////////////////////////////////////
/// Command-line options.

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // quiet option
    #[arg(short, long)]
    quiet: bool,

    /// Number of times to greet
    #[arg(short, long)]
    progress: bool,

    files: Vec<String>, // captures all positional arguments
}

////////////////////////////////////////////////////
/// The main program.

fn die<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("Fatal: {}", msg.as_ref());
    process::exit(1);
}

fn progress_copy_dir(_source: &str, _dest: &str) -> Result<(), Box<dyn std::error::Error>>
{
    die("progress copy not implemented yet");
}

fn quiet_copy_dir(source: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>>
{
    let options = Options::new();
    clone_tree(source, dest, &options)?;
    Ok(())
}

fn copy_with_progress<P, Q, F>(
    source: P,
    dest: Q,
    buffer_size: usize,
    mut progress_hook: F,
    dexist: &bool,
    ddir: &bool) -> io::Result<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    F: Progress,
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
    progress_hook.init(file_size);

    loop {
        let bytes_read = srcfile.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        dstfile.write_all(&buffer[..bytes_read])?;
        total_bytes += bytes_read as u64;

        progress_hook.tick(total_bytes);
    }
    progress_hook.finish();
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


fn main() {
    let args = Args::parse();
    if args.files.len() < 2 {
        eprintln!("Usage: cpx [--quiet|--progress] <source> <dest>");
        std::process::exit(1);
    }
    let mut quiet = args.quiet;
    let progress = args.progress;
    if !quiet && !progress {
        quiet = true;
    }
    if quiet && progress {
        eprintln!("ERROR: The --quiet and --progress options are mutually exclusive");
        eprintln!("Usage: cpx [--quiet|--progress] <source> <dest>");
        std::process::exit(1);
    }

    let nsfiles = args.files.len() - 1;

    let dest = args.files.get(args.files.len()-1).unwrap();
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
        let attr = fs::metadata(&dest).unwrap();
        if attr.is_dir() {
            ddir = true;
        } else {
            if nsfiles > 1 {
                die("multiple source files require a directory as destination");
            }
        }
    }

    // With multiple sources, destination must be a directory.
    if &args.files.len()-1 > 1 && !ddir {
        die("with multiple sources, the destination must be a directory");
    }

    // Loop over all sources before copying anything. Rules to enforce.
    for source in &args.files[0..&args.files.len()-1] {
        let source_exists = fs::exists(&source);
        let source_exists_b: bool;
        match source_exists {
            Ok(true) => { source_exists_b = true; },
            Ok(false) => { source_exists_b = false; },
            Err(_) => { die("error attempting to check source file existence"); }
        }

        if source_exists_b {
            // Is it a file? We don't copy anything else yet.
            let attr: Metadata = fs::metadata(&source).unwrap();
            if !attr.is_file() {
                if quiet {
                    let _ = quiet_copy_dir(source, dest);
                } else {
                    let _ = progress_copy_dir(source, dest);
                }
            }
        } else {
            die(format!("source does not exist: {}", &source));
        }
    }

    for source in &args.files[0..&args.files.len()-1] {
        if quiet {
            match quiet_copy(&source, &dest, &dest_exists_b, &ddir) {
                Ok(_) => { println!("done"); }
                Err(e) => { die(format!("error in copy: {}", e)); }
            }
        } else {
            let p = ProgressIndicator::new(0, 0);
            match copy_with_progress(&source, &dest, 40960, p, &dest_exists_b, &ddir) {
                Ok(_) => { println!("done"); }
                Err(e) => { die(format!("error in copy: {}", e)); }
            }
        }
    }
}
