use csv::{Reader, ReaderBuilder, Terminator};
use std::path::PathBuf;
use std::process;
use std::{env, io};

mod generators;
mod translations;
use generators::{generate_json, generate_json_fast};

const MSG: &str = "Please give file path as a command line argument!";

/// Checks for and uses first argument as path to file. Prints error if no CLI argument given.
fn get_file_location() -> Result<PathBuf, io::Error> {
    let cwd = std::env::current_dir()?;
    // Get cli arguments, then make sure an arg was actually passed
    let path = env::args_os().nth(1).expect(MSG);

    let full_path = PathBuf::from(path);

    if full_path.has_root() {
        Ok(full_path)
    } else {
        Ok(cwd.join(full_path))
    }
}

fn main() -> Result<(), std::io::Error> {
    let csv_path = if let Ok(path) = get_file_location() {
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
    let delimiter = if is_tsv { b'\t' } else { b',' };
    let escape = if is_tsv { Some(b'\\') } else { Some(b'"') };
    let terminator = if is_tsv {
        Terminator::Any(b'\n')
    } else {
        Terminator::CRLF
    };

    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .escape(escape)
        .flexible(true)
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
