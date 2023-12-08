use argh::FromArgs;
use csv::{Reader, ReaderBuilder, Terminator, Trim};
use std::io;
use std::path::PathBuf;
use std::process;
use yansi::{Paint};

mod generators;
mod translations;
use generators::{generate_json, generate_json_fast};

const APP_DESC: &str = "trans·lo·cate, verb, to move from one place to another.";
const MISSING_FILE_ERR: &str = "Please give file path as a command line argument!";

#[derive(FromArgs)]
/// High performance CSV translation to JSON translation file transformer.
struct CliArgs {
    #[argh(option, short = 'd')]
    /// field delimiter to use when parsing. Uses `\t` for TSV and `,` for CSV by default.
    delimiter: Option<String>,
    #[argh(option, short = 'e')]
    /// escape character to use for quotes when parsing. Uses `\` for TSV and `"` for CSV by default.
    escape_char: Option<String>,
    #[argh(switch, short = 'i')]
    /// flag which determines if the number of fields in records is allowed to change. Parsing is stricter if enabled.
    inflexible: bool,
    #[argh(option, short = 't')]
    /// record terminator to use. CSV default is `\r`, `\n` or `\r\n`. TSV default is `\n`.
    terminator: Option<String>,
    #[argh(switch, short = 'T')]
    /// flag to determine if non-header fields should be trimmed. Trims leading and trailing whitespace if enabled.
    trim: Option<bool>,
    #[argh(switch, short = 'v')]
    /// version information
    version: Option<bool>,
    #[argh(positional)]
    /// relative or absolute path to CSV or TSV. If no file is provided, one called "translations.csv"
    /// is looked for in the current directory.
    file: Option<String>,
}

/// Checks for and uses first argument as path to file. Prints error if no CLI argument given.
fn get_file_location(file: &str) -> Result<PathBuf, io::Error> {
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

fn main() -> Result<(), std::io::Error> {
    let cli: CliArgs = argh::from_env();
    if cli.version.is_some() {
        return Ok(println!(
            "{} v{}\n{}",
            env!("CARGO_PKG_NAME").underline(),
            env!("CARGO_PKG_VERSION"),
            APP_DESC.italic()
        ));
    }

    let file_path = if let Some(file) = &cli.file {
        file
    } else {
        "translations.csv"
    };
    let csv_path = get_file_location(file_path)?;

    let is_tsv = if let Some(val) = csv_path.extension() {
        val == "tsv"
    } else {
        // If no file extension found, assume csv
        false
    };

    let delimiter = if let Some(delim) = cli.delimiter {
        delim.as_bytes()[0]
    } else if is_tsv {
        b'\t'
    } else {
        b','
    };

    let escape = if let Some(esc) = cli.escape_char {
        esc.as_bytes()[0]
    } else if is_tsv {
        b'\\'
    } else {
        b'"'
    };

    let terminator = if let Some(terminate) = cli.terminator {
        Terminator::Any(terminate.as_bytes()[0])
    } else if is_tsv {
        Terminator::Any(b'\n')
    } else {
        Terminator::CRLF
    };

    let trim_whitespace = if let Some(trim) = cli.trim {
        if trim {
            Trim::Fields
        } else {
            Trim::None
        }
    } else {
        Trim::None
    };

    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .escape(Some(escape))
        .flexible(!cli.inflexible)
        .terminator(terminator)
        .trim(trim_whitespace)
        .from_path(&csv_path)?;
    let mut reader_count = Reader::from_path(&csv_path)?;

    let headings = reader.headers()?.clone();
    let rows = reader_count.byte_records().count();

    if generate_json_fast(&mut reader, &headings, rows).is_err() {
        if let Err(error) = generate_json(&mut reader, &headings, rows) {
            println!("{}", error);
            process::exit(1);
        }
    }
    Ok(println!("\nConversion successful!"))
}
