extern crate chrono;

use std::{fs, io, env};
use std::os::unix::fs::{PermissionsExt, MetadataExt};
use std::time::SystemTime;
use chrono::DateTime;
use chrono::offset::Utc;

/// POSIX mask to get the file type from the st_mode field
const S_IFMT : u32 = 0o170000;

// st_mode values
const S_IFSOCK : u32 = 0o140000;
const S_IFLNK : u32 = 0o120000;
const S_IFREG : u32 = 0o100000;
const S_IFBLK : u32 = 0o060000;
const S_IFDIR : u32 = 0o040000;
const S_IFCHR : u32 = 0o020000;
const S_IFIFO : u32 = 0o010000;

/// Parsed representation of the call configuration
///
/// This contains all parsed options passed to this command
struct Config {
    /// The list of all directories to ls
    directories: Vec<String>,
    /// Determines if the name of the currently ls'ed directory should be displayed before the contents
    show_directory_name: bool,
    list_output: bool,
}

impl Config {
    fn new(mut args: std::env::Args) -> Config {
        // Skip application name
        args.next();

        let mut directories: Vec<String> = Vec::new();

        let mut list_output = false;

        while let Some(directory) = args.next() {
            if directory == "-l" {
                list_output = true;
            } else {
                directories.push(directory);
            }
        }

        // If no directory has been passed, use the current directory
        if directories.len() == 0 {
            directories.push(".".to_string())
        }

        Config {
            show_directory_name: directories.len() > 1,
            directories,
            list_output
        }
    }
}

/// Output a list of DirEntry objects.
///
/// As we are not mutating the list in here, we do not need ownership of the list
fn display_file_list(entries: &Vec<fs::DirEntry>) {
    for entry in entries {
        if let Ok(filename) = entry.file_name().into_string() {
            print!("{}\t", filename);
        }
    }
}

fn display_long_file_list(entries: &Vec<fs::DirEntry>) {
    for entry in entries {
        if let Ok(filename) = entry.file_name().into_string() {
            if let Ok(metadata) = entry.metadata() {
                let permissions = metadata.permissions();

                println!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    stringify_mode(permissions.mode()),
                    metadata.nlink(),
                    metadata.uid(),
                    metadata.gid(),
                    metadata.len(),
                    stringify_date(metadata.modified()),
                    filename
                );
            }
        }
    }
}

fn stringify_date(time_result: std::io::Result<SystemTime>) -> String {
    match time_result {
        Ok(system_time) => convert_system_time_to_seconds(system_time),
        _ => "Unknown".to_string()
    }
}

fn convert_system_time_to_seconds(system_time: SystemTime) -> String {
    let datetime: DateTime<Utc> = system_time.into();
    datetime.format("%b %d %H:%M").to_string()
}

fn stringify_mode(mode: u32) -> String {
    let filetype = match mode & S_IFMT {
        S_IFSOCK => "s",
        S_IFBLK => "b",
        S_IFDIR => "d",
        S_IFREG => "-",
        S_IFLNK => "l",
        S_IFCHR => "c",
        S_IFIFO => "p",
        _ => "?"
    };

    // Only use file permission bits
    let filemode = mode & 0o777;

    let mask : u32 = 0o7;
    let mut output = String::new();

    for i in [2u32, 1u32, 0u32].iter() {
        let permissions = (filemode >> i*3) & mask;

        output.push_str(if permissions & 0b100u32 > 0u32 {"r"} else {"-"});
        output.push_str(if permissions & 0b010u32 > 0u32 {"w"} else {"-"});
        output.push_str(if permissions & 0b001u32 > 0u32 {"x"} else {"-"});
    }

    return format!("{}{}", filetype, output)
}

/// Check if the given Dir Entry is a dotfile, which are hidden by default
fn is_dotfile(entry: &fs::DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.starts_with(".")).unwrap_or(false)
}

fn read_directory(current_directory: &String, config: &Config) -> io::Result<()> {
    match fs::metadata(current_directory) {
        Ok(metadata) if metadata.is_file() => {
            println!("{}", &current_directory);
            return Ok(())
        },
        Ok(metadata) if metadata.is_dir() && config.list_output => {
            // TODO: Sadly this does not work as we have to calculate this for all the files in the dir
            println!("total {}", metadata.blocks());
        },
        Err(err) => {
            // ls skips these directories, but continues operation
            return Err(err)
        },
        _ => ()
    };

    let mut entries = fs::read_dir(current_directory)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| !is_dotfile(&entry))
        .collect::<Vec<fs::DirEntry>>();

    // Sort entries by path for lexicographical ordering
    entries.sort_by(|a, b| a.path().cmp(&b.path()));

    // Split into directories and files
    let (dirs, files): (Vec<fs::DirEntry>, Vec<fs::DirEntry>) = entries.drain(..)
        .partition(|entry| entry.path().is_dir());

    assert_eq!(entries.len(), 0);

    if config.list_output {
        display_long_file_list(&files);
        display_long_file_list(&dirs);
    } else {
        display_file_list(&files);
        display_file_list(&dirs);
    }

    print!("\n");

    Ok(())
}

fn main() {
    let config = Config::new(env::args());

    for directory in &config.directories {
        if config.show_directory_name {
            println!("{}:", directory);
        }

        if let Err(e) = read_directory(directory, &config) {
            eprintln!("ls: {}: {}", directory, e);
        }
    }
}
