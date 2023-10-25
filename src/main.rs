use serde_json::{to_string_pretty, Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs::File, io, io::Write};

mod translations;
use translations::{FormatTranslation, Translations};

const MSG: &str = "Please give file path as a command line argument!";

fn generate_json(file: &PathBuf) -> Result<(), std::io::Error> {
    let delimiter = if let Some(val) = file.extension() {
      if val == "tsv" {
        b'\t'
      } else {
        b','
      }
    } else {
      // If no file extension found, use comma delimiter
      b','
    };
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .escape(Some(b'\\'))
        .flexible(true)
        .quoting(true)
        .trim(csv::Trim::Fields)
        .from_path(file)?;
    let mut reader_count = csv::Reader::from_path(file)?;
    let headings = reader.headers()?.clone();

    let rows = reader_count.byte_records().count();

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
            if heading != "id" && heading != "TextDomain" && !heading.is_empty() {
                let kv = record.translate_to(heading);
                if let Some(lang_map) = dictionary.get_mut(heading) {
                    // into() used to convert from &str into the key / value types for JSON Map.
                    let old_val = lang_map.insert(kv.0.into(), kv.1.into());
                    if let Some(_val) = old_val {
                        if !overwrote_data {
                            // println!("Overwrite previous entry for \"{}\".\nOld: {:#?}\nNew {:#?}", kv.0, val, kv.1);
                            println!(
                                "Warning: key \"{}\" overwritten by record {idx} (line {}).",
                                kv.0,
                                idx + 1
                            );
                            overwrote_data = true;
                            times_overwritten += 1;
                        }
                    };
                } else {
                    dictionary.insert(heading, Map::with_capacity(rows));

                    dictionary
                        .get_mut(heading)
                        .expect("Unexpected error after creating map")
                        .insert(kv.0.into(), kv.1.into());
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

fn main() {
    let csv_path = if let Ok(path) = get_file_location() {
        path
    } else {
        PathBuf::from("APP_Trans.tsv")
    };

    match generate_json(&csv_path) {
        Ok(_) => println!("\nConversion successful!"),
        Err(err) => println!("{}", err),
    }
}
