use csv::{Reader, ReaderBuilder, StringRecord, Terminator};
use serde_json::{to_string_pretty, Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;
use std::{env, fs::File, io, io::Write};

mod translations;
use translations::{FormatTranslation, LangData, Translations};

const MSG: &str = "Please give file path as a command line argument!";

fn generate_json(
    reader: &mut Reader<File>,
    headings: &StringRecord,
    rows: usize,
) -> Result<(), std::io::Error> {
    // HashMap::with_capacity_and_hasher(capacity, hasher) can be used instead, with hasher
    // that is faster https://crates.io/keywords/hasher
    let mut dictionary: HashMap<&str, Map<String, Value>> = HashMap::with_capacity(rows);
    let mut times_overwritten = 0;

    for (idx, item) in reader.deserialize().enumerate() {
        let record: Translations = item?;
        let mut overwrote_data = false;

        // Loop in a loop? Incredibly inefficient? Who cares!? Optimize when it matters.
        for heading in headings.iter() {
            // Only process for language headings
            if heading != "id" && heading != "TextDomain" && !heading.trim().is_empty() {
                let kv = record.translate_to(heading);
                if let Some(lang_map) = dictionary.get_mut(heading) {
                    // No matter what the parser thinks, we want everything treated as a string
                    let value = match kv.1 {
                        LangData::Float(v) => format!("{v}"),
                        LangData::Integer(v) => format!("{v}"),
                        LangData::String(v) => v.to_owned(),
                    };

                    let old_val = lang_map.insert(kv.0.into(), value.into());
                    if let Some(_val) = old_val {
                        if !overwrote_data {
                            // println!("Overwrite previous entry for \"{}\".\nOld: {:#?}\nNew {:#?}", kv.0, val, kv.1);
                            println!(
                                "Warning: key \"{}\" overwritten by record {} (line {}).",
                                kv.0,
                                idx + 1,
                                idx + 1
                            );
                            overwrote_data = true;
                            times_overwritten += 1;
                        }
                    };
                } else {
                    dictionary.insert(heading, Map::with_capacity(rows));
                    // No matter what the parser thinks, we want everything treated as a string
                    let value = match kv.1 {
                        LangData::Float(v) => format!("{v}"),
                        LangData::Integer(v) => format!("{v}"),
                        LangData::String(v) => v.to_owned(),
                    };

                    dictionary
                        .get_mut(heading)
                        .expect("Unexpected error after creating map")
                        .insert(kv.0.into(), value.into());
                }
            }
        }
    }
    println!("\n{times_overwritten} translation keys overwritten during conversion.");

    for lang in dictionary.keys() {
        let filename = format!("{lang}.json");
        let mut file = File::create(filename)?;
        if let Some(json) = dictionary.get(lang) {
            writeln!(
                file,
                "{}",
                to_string_pretty(json).expect("Error writing {lang}.json.")
            )?;
        }
        println!("{lang}.json written to current directory.");
    }

    Ok(())
}

fn generate_json_fast(
    reader: &mut Reader<File>,
    headings: &StringRecord,
    rows: usize,
) -> Result<(), std::io::Error> {
    // HashMap::with_capacity_and_hasher(capacity, hasher) can be used instead, with hasher
    // that is faster https://crates.io/keywords/hasher
    let mut dictionary: HashMap<&str, Map<String, Value>> = HashMap::with_capacity(rows);
    let mut times_overwritten = 0;

    let mut record = StringRecord::new();
    let mut idx = 0;

    while reader.read_record(&mut record)? {
        let mut overwrote_data = false;
        idx += 1;

        // Loop in a loop? Incredibly inefficient? Who cares!? Optimize when it matters.
        for (i, heading) in headings.iter().enumerate() {
            // Only process for language headings
            if heading != "id" && heading != "TextDomain" && !heading.trim().is_empty() {
                let value = match &record.get(i) {
                    Some(head) => head,
                    None => "",
                };

                if let Some(lang_map) = dictionary.get_mut(heading) {
                    let old_val = lang_map.insert(record[0].into(), value.into());
                    if let Some(_val) = old_val {
                        if !overwrote_data {
                            // println!("Overwrite previous entry for \"{}\".\nOld: {:#?}\nNew {:#?}", kv.0, val, kv.1);
                            println!(
                                "Warning: key \"{}\" overwritten by record {} (line {}).",
                                &record[0],
                                idx + 0,
                                idx + 0
                            );
                            overwrote_data = true;
                            times_overwritten += 1;
                        }
                    };
                } else {
                    dictionary.insert(heading, Map::with_capacity(rows));
                    // No matter what the parser thinks, we want everything treated as a string
                    let value = match &record.get(i) {
                        Some(head) => head,
                        None => "",
                    };

                    dictionary
                        .get_mut(heading)
                        .expect("Unexpected error after creating map")
                        .insert(record[0].into(), value.into());
                }
            }
        }
    }
    println!("\n{times_overwritten} translation keys overwritten during conversion.");

    for lang in dictionary.keys() {
        let filename = format!("{lang}.json");
        let mut file = File::create(filename)?;
        if let Some(json) = dictionary.get(lang) {
            writeln!(
                file,
                "{}",
                to_string_pretty(json).expect("Error writing {lang}.json.")
            )?;
        }
        println!("{lang}.json written to current directory.");
    }

    Ok(())
}

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
