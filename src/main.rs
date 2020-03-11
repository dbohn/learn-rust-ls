use std::{fs, io, env, process};

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

fn display_file_list(entries: Vec<fs::DirEntry>) {
    for entry in entries {
        if let Ok(filename) = entry.file_name().into_string() {
            print!("{}\t", filename);
        }
    }
}

fn read_directory(config: &Config) -> io::Result<()> {
    let mut entries = fs::read_dir(&config.directory)?.filter(|entry| entry.is_ok()).collect::<Result<Vec<fs::DirEntry>, io::Error>>()?;

    // Sort entries by path for lexicographical ordering
    entries.sort_by(|a, b| a.path().cmp(&b.path()));

    // Split into directories and files
    let (dirs, files): (Vec<fs::DirEntry>, Vec<fs::DirEntry>) = entries.drain(..).partition(|entry| entry.metadata().unwrap().is_dir());

    assert_eq!(entries.len(), 0);

    display_file_list(files);
    display_file_list(dirs);

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
