//! # libtranslocate converts a CSV translation file into multiple JSON translation files.
//!
//! ## Overview
//! CSV files are a convenient, and somewhat common format for housing translations. They are
//! plain text files in the [CSV format](https://en.wikipedia.org/wiki/Comma-separated_values).
//! The ease of exporting from a spreadsheet to the CSV format makes it an ideal candidate for
//! non-technical users to use for output in localization tasks. CSV files are also
//! a supported import and export format for many localization services.
//!
//! Unfortunately, though the format [has been standardized](https://www.rfc-editor.org/rfc/rfc4180) there are
//! many non well-formed CSV files in existence, making their direct use for web localization projects somewhat
//! challenging. JSON meanwhile has a very strict format. It is also very popular&mdash;especially in web
//! development&mdash;as a localization format.
//!
//! This crate provides the `translocate` binary, which uses `libtranslocate` to
//! to read an input CSV localization file, and output JSON localization files, with one JSON file
//! being generated for every localization that exists as a column in the input CSV file.
//!
//! The format of the JSON files output will be `{ "localization-key": "localized text" }` e.g.
//!
//! ```json
//! {
//!   "app.title": "Translocate means to move from one place to another."
//! }
//! ```

mod generators;
mod translations;

use argh::FromArgs;
use csv::{Reader, ReaderBuilder, StringRecord, Terminator, Trim};
pub use generators::{generate_json, generate_json_fast};
use std::{ffi::OsStr, fs, io, path::PathBuf};
use yansi::Paint;

#[derive(FromArgs)]
#[argh(description = "High performance CSV translation to JSON translation file transformer.")]
/// Defines the command line arguments that the translocate binary will accept.
pub struct CliArgs {
    #[argh(option, short = 'd')]
    /// column delimiter to use when parsing. Uses `\t` for TSV and `,` for CSV by default.
    pub delimiter: Option<String>,
    #[argh(option, short = 'e')]
    /// escape character to use for quotes when parsing. Uses `\` for TSV and `"` for CSV by default.
    pub escape_char: Option<String>,
    #[argh(option, short = 'I')]
    /// list of csv column header names, separated by commas, to skip when converting to JSON.
    pub ignored_headings: Option<String>,
    #[argh(switch, short = 'i')]
    /// set whether input file parsing should continue if the number of columns in each record differs.
    /// Passing this flag enables stricter (less flexible) parsing.
    pub inflexible: bool,
    #[argh(option, short = 'o')]
    /// desired output directory, if different from the current directory. Can be either a relative or absolute file path.
    pub output_dir: Option<String>,
    #[argh(option, short = 'O')]
    /// if set, saves each file with the name provided inside a directory named by the locale.
    pub output_filename: Option<String>,
    #[argh(option, short = 't')]
    /// character indicating end of each record, passed in as its decimal representation e.g. \n = 10, \r = 13, \t = 9.
    /// CSV default is `\r`, `\n` OR `\r\n`. TSV default is `\n`.
    pub terminator: Option<String>,
    #[argh(switch, short = 'T')]
    /// trim leading and trailing whitespace in non-header columns. Enabled by passing this flag.
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

/// Configures how the CSV file will be read. Defaults are modified after parsing any provided command line options
pub struct Config<'a> {
    /// Delimiter character to use when separating columns. Uses `\t` for TSV and `,` for CSV by default.
    pub delimiter: u8,
    /// Escape character to use for quotes when parsing columns. Uses `\` for TSV and `"` for CSV by default.
    pub escape_char: u8,
    /// list of colum names, separated by commas, to skip when converting to JSON.
    pub ignored_headings: Option<Vec<&'a str>>,
    /// Flag to determine whether processing the input file should continue if the number of columns in records is not always the same.
    /// If true, parsing is less strict. Default is true.
    pub flexible: bool,
    /// Desired output directory, if different from the current directory. Can be either a relative or absolute file path.
    pub output_dir: &'a str,
    /// Unified filename for each localization file. Saves each file inside a directory named by the locale.
    pub output_filename: Option<&'a str>,
    /// Record terminator to use. CSV default is `\r`, `\n` or `\r\n`. TSV default is `\n`.
    pub terminator_char: Terminator,
    /// Flag to determine if non-header columns should be trimmed. Trims leading and trailing whitespace if enabled.
    pub trim_whitespace: Trim,
}

#[doc(hidden)]
impl<'a> Config<'a> {
    /// Parses provided command line arguments and returns a configuration struct
    /// whose options determine how the input CSV file is read or processed.
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

        let ignored_headings = if let Some(list) = &args.ignored_headings {
            let ignored = list.split(',').collect::<Vec<&str>>();
            Some(ignored)
        } else {
            None
        };

        let output_dir = if let Some(path) = &args.output_dir {
            path
        } else {
            ""
        };

        let output_filename = args.output_filename.as_deref();

        let terminator_char = if let Some(terminate_on) = &args.terminator {
            match terminate_on.parse::<u8>() {
                Ok(val) => Terminator::Any(val),
                Err(err) => {
                    eprintln!("{} {}. Try passing the decimal number (less than 256) representing your ascii separator.", "Error:".bold().on_bright_red(), err);
                    std::process::exit(1)
                }
            }
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
            ignored_headings,
            flexible: !args.inflexible,
            output_dir,
            output_filename,
            terminator_char,
            trim_whitespace,
        }
    }
}

/// Checks if special command line shell characters like "~" or "$", which
/// are used for expansions are present anywhere in a text string.
///
/// * `char` - the special character to check for
/// * `text` - the text to search for the special character
fn special_character_check(char: &str, text: &str) {
    if text.contains(char) {
        println!(
            "{} Path {} contained the literal '{}' character and will be treated as a filename.\n{}\n",
            "Warning:".on_yellow().blink(),
            text.underline(),
            char.bold(),
            "If this was not intended, try again without wrapping the path inside of quotes to allow expansion by your shell.".bold(),
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

/// Returns a configured CSV reader for the specified file, or an error.
///
/// * `file_path` - relative or absolute path to file
/// * `config` - parsed command line configuration
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

/// Entry point for library to generate JSON translation files.
///
/// * `reader` - a configured CSV reader
/// * `headings` - heading row for the CSV file
/// * `rows` - number of rows that are in the CSV file
/// * `config` - parsed command line configuration
pub fn run(
    reader: &mut Reader<fs::File>,
    headings: &StringRecord,
    rows: usize,
    config: &Config,
) -> Result<(), io::Error> {
    if generate_json_fast(reader, headings, rows, config).is_err() {
        generate_json(reader, headings, rows, config)?
    }
    Ok(())
}

#[cfg(test)]
mod get_file_location_tests {
    use crate::get_file_location;
    use pretty_assertions::assert_eq;

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
        let path = get_file_location(r"A:\floppy").unwrap();

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
        ignored_headings: None,
        flexible: true,
        output_dir: "",
        output_filename: None,
        terminator_char: Terminator::CRLF,
        trim_whitespace: Trim::Fields,
    };

    #[test]
    fn it_has_file_reader_that_is_configured_to_have_csv_headers() {
        let path = get_file_reader("package.json", &CONFIG).unwrap();

        assert!(path.has_headers());
    }
}
