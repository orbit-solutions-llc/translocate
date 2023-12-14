mod generators;
mod translations;

use argh::FromArgs;
use csv::{Reader, ReaderBuilder, StringRecord, Terminator, Trim};
use generators::{generate_json, generate_json_fast};
use std::{ffi::OsStr, fs, io, path::PathBuf};
use yansi::Paint;

#[derive(FromArgs)]
/// High performance CSV translation to JSON translation file transformer.
pub struct CliArgs {
    #[argh(option, short = 'd')]
    /// field delimiter to use when parsing. Uses `\t` for TSV and `,` for CSV by default.
    pub delimiter: Option<String>,
    #[argh(option, short = 'e')]
    /// escape character to use for quotes when parsing. Uses `\` for TSV and `"` for CSV by default.
    pub escape_char: Option<String>,
    #[argh(switch, short = 'i')]
    /// flag which determines if the number of fields in records is allowed to change. Parsing is stricter if enabled.
    pub inflexible: bool,
    #[argh(option, short = 'o')]
    /// desired output directory, if different from the current directory. Can be either a relative or absolute file path.
    pub output_dir: Option<String>,
    #[argh(option, short = 't')]
    /// record terminator to use. CSV default is `\r`, `\n` or `\r\n`. TSV default is `\n`.
    pub terminator: Option<String>,
    #[argh(switch, short = 'T')]
    /// flag to determine if non-header fields should be trimmed. Trims leading and trailing whitespace if enabled.
    pub trim: Option<bool>,
    #[argh(switch, short = 'v')]
    /// version information
    pub version: Option<bool>,
    #[argh(positional)]
    #[argh(default = "String::from(\"translations.csv\")")]
    /// relative or absolute path to CSV or TSV. If no file is provided, one called "translations.csv"
    /// is looked for in the current directory.
    pub file: String,
}

pub struct Config<'a> {
    delimiter: u8,
    escape_char: u8,
    flexible: bool,
    output_dir: &'a str,
    terminator_char: Terminator,
    trim_whitespace: Trim,
}

impl<'a> Config<'a> {
    pub fn new(args: &'a CliArgs, file_extension: Option<&OsStr>) -> Config<'a> {
        let is_tsv = if let Some(val) = file_extension {
            val == "tsv"
        } else {
            // If no file extension found, assume csv
            false
        };

        let delimiter = if let Some(delim) = &args.delimiter {
            delim.as_bytes()[0]
        } else if is_tsv {
            b'\t'
        } else {
            b','
        };

        let escape_char = if let Some(esc) = &args.escape_char {
            esc.as_bytes()[0]
        } else if is_tsv {
            b'\\'
        } else {
            b'"'
        };

        let output_dir = if let Some(path) = &args.output_dir {
            path
        } else {
            ""
        };

        let terminator_char = if let Some(terminate) = &args.terminator {
            Terminator::Any(terminate.as_bytes()[0])
        } else if is_tsv {
            Terminator::Any(b'\n')
        } else {
            Terminator::CRLF
        };

        let trim_whitespace = if let Some(trim) = args.trim {
            if trim {
                Trim::Fields
            } else {
                Trim::None
            }
        } else {
            Trim::None
        };

        Config {
            delimiter,
            escape_char,
            flexible: !args.inflexible,
            output_dir: output_dir,
            terminator_char,
            trim_whitespace,
        }
    }
}

fn special_character_check(char: &str, text: &str) {
    if text.contains(char) {
        println!(
            "{} Path {} contained the literal '{}' character.\n{}\n{}\n",
            "Warning:".on_yellow(),
            text.underline(),
            char.bold(),
            "This was not expanded by your shell and will be treated as a filename.",
            "If unintentional, try not wrapping your path inside of quotes.".on_yellow(),
        );
    }
}

/// Takes a path argument and returns a representation of that file system
/// location (`PathBuf`), or an `io::Error` if this representation can't be created.
///
/// * `file` - string slice which is the path to a file system location.
pub fn get_file_location(file: &str) -> Result<PathBuf, io::Error> {
    let cwd = std::env::current_dir()?;
    special_character_check("~", file);
    special_character_check("$", file);

    let full_path = PathBuf::from(file);

    if full_path.has_root() {
        Ok(full_path)
    } else {
        Ok(cwd.join(full_path))
    }
}

pub fn get_file_reader(file_path: &str, config: &Config) -> Result<Reader<fs::File>, csv::Error> {
    let csv_path = get_file_location(file_path).expect("Unable to create path");

    ReaderBuilder::new()
        .delimiter(config.delimiter)
        .escape(Some(config.escape_char))
        .flexible(config.flexible)
        .terminator(config.terminator_char)
        .trim(config.trim_whitespace)
        .from_path(csv_path)
}

pub fn run(
    reader: &mut Reader<fs::File>,
    headings: &StringRecord,
    rows: usize,
    config: &Config,
) -> Result<(), io::Error> {
    if generate_json_fast(reader, headings, rows, config.output_dir).is_err() {
        generate_json(reader, headings, rows, config.output_dir)?
    }
    Ok(())
}

#[cfg(test)]
mod get_file_location_tests {
    use crate::get_file_location;

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn it_turns_a_relative_location_into_a_full_path() {
        let cwd = std::env::current_dir().unwrap();
        let path = get_file_location("").unwrap();
        let path2 = get_file_location("./").unwrap();

        assert!(path.has_root());
        assert!(path2.has_root());
        assert!(path.starts_with(cwd.to_str().unwrap()));
        assert!(path2.starts_with(cwd.to_str().unwrap()));
        assert_eq!(path, path2);
        assert_eq!(path, cwd);
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn it_does_not_append_the_current_directory_to_an_absolute_path() {
        let cwd = std::env::current_dir().unwrap();
        let path = get_file_location("/home").unwrap();

        assert!(!path.starts_with(cwd.to_str().unwrap()));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn it_does_not_append_the_current_directory_to_an_absolute_path_in_windows() {
        let cwd = std::env::current_dir().unwrap();
        let path = get_file_location("A:\\floppy").unwrap();

        assert!(!path.starts_with(cwd.to_str().unwrap()));
    }
}

#[cfg(test)]
mod get_file_reader_tests {
    use csv::{Terminator, Trim};

    use crate::{get_file_reader, Config};

    const CONFIG: Config = Config {
        delimiter: b',',
        escape_char: b'"',
        flexible: true,
        output_dir: "",
        terminator_char: Terminator::CRLF,
        trim_whitespace: Trim::Fields,
    };

    #[test]
    fn it_has_file_reader_that_is_configured_to_have_csv_headers() {
        let path = get_file_reader("package.json", &CONFIG).unwrap();

        assert!(path.has_headers());
    }

}
