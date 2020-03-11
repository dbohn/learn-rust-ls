use std::{fs, io, env, process};

/// Parsed representation of the call configuration
///
/// This contains all parsed options passed to this command
struct Config {
    directory: String
}

impl Config {
    fn new(mut args: std::env::Args) -> Config {
        // Skip application name
        args.next();

        let directory = match args.next() {
            Some(arg) => arg,
            None => ".".to_string()
        };

        Config {
            directory
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

fn is_dotfile(entry: &fs::DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.starts_with(".")).unwrap_or(false)
}

fn read_directory(config: &Config) -> io::Result<()> {
    let mut entries = fs::read_dir(&config.directory)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| !is_dotfile(entry))
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

    let parent_metadata = fs::metadata(&config.directory).unwrap_or_else(|err| {
        eprintln!("ls: {}: {}", &config.directory, err);
        process::exit(1);
    });

    if parent_metadata.is_file() {
        println!("{}", &config.directory);
        process::exit(1);
    }

    read_directory(&config)
}
