use argh::FromArgs;
use csv::{Reader, ReaderBuilder, Terminator};
use std::io;
use std::path::PathBuf;
use std::process;

mod generators;
mod translations;
use generators::{generate_json, generate_json_fast};

const APP_DESC: &str = "trans·lo·cate, verb, to move from one place to another.";

#[derive(FromArgs)]
/// High performance CSV translation to JSON translation file transformer.
struct CliArgs {
    #[argh(option, short = 'd')]
    /// field delimiter to use when parsing. Uses `\t` for TSV and `,` for CSV by default.
    delimiter: Option<u8>,
    #[argh(option, short = 'e')]
    /// escape character to use for quotes when parsing. Uses `\` for TSV and `"` for CSV by default.
    escape_char: Option<u8>,
    #[argh(switch, short = 'i')]
    /// if the number of fields in records is allowed to change. Enabling makes parsing more strict.
    inflexible: bool,
    #[argh(option, short = 't')]
    /// record terminator to use. CSV default is `\r`, `\n` or `\r\n`. TSV default is `\n`.
    terminator: Option<u8>,
    #[argh(switch, short = 'v')]
    /// version info
    version: Option<bool>,
    #[argh(positional)]
    /// relative or absolute path to CSV or TSV.
    file: String,
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
    if let Some(_) = cli.version {
        return Ok(println!(
            "{} v{}\n{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            APP_DESC
        ));
    }

    let csv_path = if let Ok(path) = get_file_location(&cli.file) {
        path
    } else {
        PathBuf::from("translations.csv")
    };

    let is_tsv = if let Some(val) = csv_path.extension() {
        if val == "tsv" {
            true
        } else {
            false
        }
    } else {
        // If no file extension found, assume csv
        false
    };

    let delimiter = if let Some(delim) = cli.delimiter {
        delim
    } else if is_tsv {
        b'\t'
    } else {
        b','
    };

    let escape = if let Some(esc) = cli.escape_char {
        Some(esc)
    } else if is_tsv {
        Some(b'\\')
    } else {
        Some(b'"')
    };

    let terminator = if let Some(terminate) = cli.terminator {
        Terminator::Any(terminate)
    } else if is_tsv {
        Terminator::Any(b'\n')
    } else {
        Terminator::CRLF
    };

    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .escape(escape)
        .flexible(!cli.inflexible)
        .terminator(terminator)
        .from_path(&csv_path)?;
    let mut reader_count = Reader::from_path(&csv_path)?;

    let headings = reader.headers()?.clone();
    let rows = reader_count.byte_records().count();

    if let Err(_) = generate_json_fast(&mut reader, &headings, rows) {
        if let Err(error) = generate_json(&mut reader, &headings, rows) {
            println!("{}", error.to_string());
            process::exit(1);
        }
    }
    Ok(println!("\nConversion successful!"))
}
