use std::{fs, io, env};

/// Parsed representation of the call configuration
///
/// This contains all parsed options passed to this command
struct Config {
    directories: Vec<String>
}

impl Config {
    fn new(mut args: std::env::Args) -> Config {
        // Skip application name
        args.next();

        let mut directories: Vec<String> = Vec::new();

        while let Some(directory) = args.next() {
            directories.push(directory);
        }

        // If no option has been passed, use the current directory
        if directories.len() == 0 {
            directories.push(".".to_string())
        }

        Config {
            directories
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

/// Check if the given Dir Entry is a dotfile, which are hidden by default
fn is_dotfile(entry: &fs::DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.starts_with(".")).unwrap_or(false)
}

fn read_directory(current_directory: &String, _config: &Config) -> io::Result<()> {

    match fs::metadata(current_directory) {
        Ok(metadata) if metadata.is_file() => {
            println!("{}", &current_directory);
            return Ok(())
        },
        Err(err) => {
            // ls skips these directories, but continues operation
            return Err(err)
        },
        _ => true
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

    display_file_list(&files);
    display_file_list(&dirs);

    print!("\n");

    Ok(())
}

fn main() -> io::Result<()> {
    let config = Config::new(env::args());

    if config.directories.len() == 1 {
        return read_directory(&config.directories[0], &config);
    }

    for directory in &config.directories {
        println!("{}:", directory);
        if let Err(e) = read_directory(directory, &config) {
            eprintln!("ls: {}: {}", directory, e);
        }
    }
    Ok(())
}
