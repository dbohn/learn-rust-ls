use chrono::offset::Utc;
use chrono::DateTime;
use std::time::SystemTime;

/// POSIX mask to get the file type from the st_mode field
const S_IFMT: u32 = 0o170000;

// st_mode values
const S_IFSOCK: u32 = 0o140000;
const S_IFLNK: u32 = 0o120000;
const S_IFREG: u32 = 0o100000;
const S_IFBLK: u32 = 0o060000;
const S_IFDIR: u32 = 0o040000;
const S_IFCHR: u32 = 0o020000;
const S_IFIFO: u32 = 0o010000;

/// Convert a mode unsigned integer in a human readable representation
pub fn stringify_mode(mode: u32) -> String {
    let filetype = match mode & S_IFMT {
        S_IFSOCK => "s",
        S_IFBLK => "b",
        S_IFDIR => "d",
        S_IFREG => "-",
        S_IFLNK => "l",
        S_IFCHR => "c",
        S_IFIFO => "p",
        _ => "?",
    };

    // Only use file permission bits
    let filemode = mode & 0o777;

    let mask: u32 = 0o7;
    let mut output = String::new();

    for i in [2u32, 1u32, 0u32].iter() {
        let permissions = (filemode >> i * 3) & mask;

        output.push_str(if permissions & 0b100u32 > 0u32 {
            "r"
        } else {
            "-"
        });
        output.push_str(if permissions & 0b010u32 > 0u32 {
            "w"
        } else {
            "-"
        });
        output.push_str(if permissions & 0b001u32 > 0u32 {
            "x"
        } else {
            "-"
        });
    }

    format!("{}{}", filetype, output)
}

/// Check if the given Dir Entry is a dotfile, which are hidden by default
pub fn is_dotfile(entry: &std::fs::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn convert_system_time_to_seconds(system_time: SystemTime) -> String {
    let datetime: DateTime<Utc> = system_time.into();
    datetime.format("%b %d %H:%M").to_string()
}