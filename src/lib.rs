mod generators;
mod translations;

use argh::FromArgs;
use csv::{Reader, ReaderBuilder, StringRecord, Terminator, Trim};
use generators::{generate_json, generate_json_fast};
use std::{ffi::OsStr, fs, io, path::PathBuf, process};
use yansi::Paint;

const MISSING_FILE_ERR: &str = "Try again with the absolute (full) path to the file.";

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

pub struct Config {
    delimiter: u8,
    escape_char: u8,
    flexible: bool,
    output_dir: String,
    terminator_char: Terminator,
    trim_whitespace: Trim,
}

impl Config {
    pub fn new(args: &CliArgs, file_extension: Option<&OsStr>) -> Config {
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
            output_dir: output_dir.to_owned(),
            terminator_char,
            trim_whitespace,
        }
    }
}

/// Checks for and uses first argument as path to file. Prints error if no CLI argument given.
pub fn get_file_location(file: &str) -> Result<PathBuf, io::Error> {
    let cwd = std::env::current_dir()?;
    // Get cli arguments, then make sure an arg was actually passed
    let path = file;

    let full_path = PathBuf::from(path);

    if full_path.has_root() {
        Ok(full_path)
    } else {
        Ok(cwd.join(full_path))
    }
}

pub fn get_file_reader(file_path: &str, config: &Config) -> Reader<fs::File> {
    let csv_path = get_file_location(file_path).expect("Unable to create path");

    match ReaderBuilder::new()
        .delimiter(config.delimiter)
        .escape(Some(config.escape_char))
        .flexible(config.flexible)
        .terminator(config.terminator_char)
        .trim(config.trim_whitespace)
        .from_path(csv_path)
    {
        Ok(res) => res,
        Err(err) => match err.kind() {
            csv::ErrorKind::Io(io_error) => {
                if io_error.kind() == io::ErrorKind::NotFound {
                    eprintln!(
                        "{} file `{}` not found. {}",
                        "Error:".bold().on_bright_red(),
                        file_path.bold(),
                        MISSING_FILE_ERR.underline()
                    );
                    process::exit(1)
                } else {
                    eprintln!("{}", io_error);
                    process::exit(1)
                }
            }
            _ => {
                eprintln!("{}", err);
                process::exit(1)
            }
        },
    }
}

pub fn run(
    reader: &mut Reader<fs::File>,
    headings: &StringRecord,
    rows: usize,
    config: &Config,
) -> Result<(), io::Error> {
    if generate_json_fast(reader, headings, rows, &config.output_dir).is_err() {
        generate_json(reader, headings, rows, &config.output_dir)?
    }
    Ok(())
}
