extern crate chrono;
extern crate users;

mod util;

use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::time::SystemTime;
use std::{env, fs, io};
use std::collections::BTreeMap;

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
            list_output,
        }
    }
}

struct ListOutput {
    users: BTreeMap<u32, String>,
    groups: BTreeMap<u32, String>
}

impl ListOutput {

    fn new() -> ListOutput {
        ListOutput {
            users: BTreeMap::new(),
            groups: BTreeMap::new(),
        }
    }

    fn resolve_user(uid: u32) -> String {
        if let Some(user) = users::get_user_by_uid(uid) {
            user.name().to_os_string().to_string_lossy().to_string()
        } else {
            uid.to_string()
        }
    }

    fn resolve_group(gid: u32) -> String {
        if let Some(group) = users::get_group_by_gid(gid) {
            group.name().to_os_string().to_string_lossy().to_string()
        } else {
            gid.to_string()
        }
    }

    fn lookup_user(&mut self, uid: u32) -> String {
        self.users.entry(uid).or_insert_with(|| ListOutput::resolve_user(uid)).to_string()
    }

    fn lookup_group(&mut self, gid: u32) -> String {
        self.groups.entry(gid).or_insert_with(|| ListOutput::resolve_group(gid)).to_string()
    }

    fn display_long_file_list(&mut self, entries: &Vec<fs::DirEntry>) {
        for entry in entries {
            if let Ok(metadata) = entry.metadata() {
                self.output_long_list_item(entry.path(), metadata)
            }
        }
    }

    fn output_long_list_item(&mut self, path: std::path::PathBuf, metadata: fs::Metadata) {
        let mode = metadata.permissions().mode();

        let filename = match path.file_name() {
            Some(name) => name.to_os_string().into_string().unwrap_or("".to_string()),
            None => "".to_string(),
        };

        let uid = metadata.uid();
        let gid = metadata.gid();

        if let Ok(link) = path.read_link() {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{} -> {}",
                util::stringify_mode(mode),
                metadata.nlink(),
                self.lookup_user(uid),
                self.lookup_group(gid),
                metadata.len(),
                stringify_date(metadata.modified()),
                filename,
                link.display()
            )
        } else {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                util::stringify_mode(mode),
                metadata.nlink(),
                self.lookup_user(uid),
                self.lookup_group(gid),
                metadata.len(),
                stringify_date(metadata.modified()),
                filename
            );
        }
    }
}

fn get_block_size(entry: &fs::DirEntry) -> u64 {
    if let Ok(metadata) = entry.metadata() {
        metadata.blocks()
    } else {
        0
    }
}

fn total_block_count(entries: &Vec<fs::DirEntry>) -> u64 {
    entries
        .into_iter()
        .map(|entry| get_block_size(entry))
        .fold(0, |acc, x| acc + x)
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

fn stringify_date(time_result: std::io::Result<SystemTime>) -> String {
    match time_result {
        Ok(system_time) => util::convert_system_time_to_seconds(system_time),
        _ => "Unknown".to_string(),
    }
}

fn read_directory(current_directory: &String, config: &Config) -> io::Result<()> {
    match fs::metadata(current_directory) {
        Ok(metadata) if metadata.is_file() => {
            println!("{}", &current_directory);
            return Ok(());
        }
        Err(err) => {
            // ls skips these directories, but continues operation
            return Err(err);
        }
        _ => (),
    };

    let mut entries = fs::read_dir(current_directory)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| !util::is_dotfile(&entry))
        .collect::<Vec<fs::DirEntry>>();

    // Sort entries by path for lexicographical ordering
    entries.sort_by(|a, b| a.path().cmp(&b.path()));

    // Split into directories and files
    let (dirs, files): (Vec<fs::DirEntry>, Vec<fs::DirEntry>) =
        entries.drain(..).partition(|entry| entry.path().is_dir());

    assert_eq!(entries.len(), 0);

    let mut output = ListOutput::new();

    if config.list_output {
        println!("total {}", total_block_count(&files));
        output.display_long_file_list(&files);
        output.display_long_file_list(&dirs);
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
