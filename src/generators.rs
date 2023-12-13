use crate::get_file_location;
use crate::translations::{FormatTranslation, LangData, Translations};
use csv::{Reader, StringRecord};
use serde_json::{to_string_pretty, Map, Value};
use std::collections::HashMap;
use std::{fs::File, io::Write};

const DUPE_KEY_NOTICE: &str = "translation keys overwritten during conversion.\n";

pub fn generate_json(
    reader: &mut Reader<File>,
    headings: &StringRecord,
    rows: usize,
    output_dir: &str,
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
                let kv = record.format_lang(heading);
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
    println!("\n{times_overwritten} {DUPE_KEY_NOTICE}");

    for lang in dictionary.keys() {
        let mut filename = get_file_location(output_dir)?;
        filename.push(&format!("{lang}.json"));
        let mut file = File::create(filename)?;
        if let Some(json) = dictionary.get(lang) {
            writeln!(
                file,
                "{}",
                to_string_pretty(json).expect("Error writing {lang}.json.")
            )?;
        }
        println!("{lang}.json written to output directory.");
    }

    Ok(())
}

pub fn generate_json_fast(
    reader: &mut Reader<File>,
    headings: &StringRecord,
    rows: usize,
    output_dir: &str,
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
        for (column_idx, heading) in headings.iter().enumerate() {
            // Only process for language headings
            if column_idx != 0 && heading != "TextDomain" && !heading.trim().is_empty() {
                let value = match &record.get(column_idx) {
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
                                &record[0], idx, idx
                            );
                            overwrote_data = true;
                            times_overwritten += 1;
                        }
                    };
                } else {
                    dictionary.insert(heading, Map::with_capacity(rows));
                    // No matter what the parser thinks, we want everything treated as a string
                    let value = match &record.get(column_idx) {
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
    println!("\n{times_overwritten} {DUPE_KEY_NOTICE}");

    for lang in dictionary.keys() {
        let mut filename = get_file_location(output_dir)?;
        filename.push(&format!("{lang}.json"));
        let mut file = File::create(filename)?;
        if let Some(json) = dictionary.get(lang) {
            writeln!(
                file,
                "{}",
                to_string_pretty(json).expect("Error writing {lang}.json.")
            )?;
        }
        println!("{lang}.json written to output directory.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::generate_json_fast;
    use crate::get_file_location;
    use csv::{Reader, ReaderBuilder, StringRecord};
    use std::fs::{self, File};
    use std::io::Write;

    const _CSV_ALL_LANG: &'static str = "\
id,da_DK,de_DE,en_US,es_ES,fr_FR,it_IT,text_domain,nl_NL,pt_BR,pt_PT,sv_SE,
new.translation,ny oversættelse,neue Übersetzung,new translation,nueva traducción,nouvelle traduction,nuova traduzione,,nieuwe vertaling,nova tradução,nova tradução,ny översättning,
";
    const CSV_ONE_ROW: &'static str = "\
id,da_DK,
new.translation,ny oversættelse,
";

    const CSV_TWO_ROW: &'static str = "\
id,da_DK,
new.translation,ny oversættelse,
new.translation,,
";

    fn generate_csv_reader(
        input_filename: &str,
        input_data: &str,
    ) -> (Reader<File>, StringRecord, usize) {
        File::options()
            .write(true)
            .create(true)
            .open(input_filename)
            .unwrap()
            .write_all(input_data.as_bytes())
            .unwrap();
        let file = get_file_location(input_filename).unwrap();
        let mut reader = ReaderBuilder::new().from_path(&file).unwrap();
        let mut reader_count = Reader::from_path(&file).unwrap();

        let headings = reader.headers().unwrap().clone();
        let rows = reader_count.byte_records().count();

        (reader, headings, rows)
    }

    #[test]
    fn it_writes_a_file_to_the_current_directory() {
        let test_file_path = "test_file1.csv";
        let lang_file_path = "da_DK.json";
        let mut test_conf = generate_csv_reader(test_file_path, CSV_ONE_ROW);

        generate_json_fast(&mut test_conf.0, &test_conf.1, test_conf.2, "").unwrap();

        let trans = fs::read_to_string(lang_file_path)
            .unwrap()
            .replace('\n', "")
            .replace("  ", "");
        let trans = trans.trim();
        fs::remove_file(test_file_path).unwrap();
        fs::remove_file(lang_file_path).unwrap();

        assert_eq!(trans, "{\"new.translation\": \"ny oversættelse\"}");
    }

    #[test]
    fn it_does_not_overwrite_existing_key_with_empty_value() {
        let test_file_path = "test_file2.csv";
        let lang_file_path = "da_DK.json";
        let mut test_conf = generate_csv_reader(test_file_path, CSV_TWO_ROW);

        generate_json_fast(&mut test_conf.0, &test_conf.1, test_conf.2, "").unwrap();

        let trans = fs::read_to_string(lang_file_path)
            .unwrap()
            .replace('\n', "")
            .replace("  ", "");
        let trans = trans.trim();
        fs::remove_file(test_file_path).unwrap();
        fs::remove_file(lang_file_path).unwrap();

        assert_eq!(trans, "{\"new.translation\": \"ny oversættelse\"}");
    }

}
