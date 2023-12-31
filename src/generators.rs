use crate::translations::{FormatTranslation, LangData, Translations};
use crate::{get_file_location, Config};
use csv::{Reader, StringRecord};
use serde_json::{to_string_pretty, Map, Value};
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::Write;
use yansi::Paint;

const DUPE_KEY_NOTICE: &str = "translation keys overwritten during conversion.\n";

/// Generate JSON files from CSV using structured deserialization
///
/// * `reader` - a configured CSV reader
/// * `headings` - heading row for the CSV file
/// * `rows` - number of rows that are in the CSV file
/// * `config` - parsed command line configuration
pub fn generate_json(
    reader: &mut Reader<File>,
    headings: &StringRecord,
    rows: usize,
    config: &Config,
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
                    // Only replace value below if there's an actual value.
                    if value.is_empty() {
                        continue;
                    };

                    let old_val = lang_map.insert(kv.0.into(), value.into());
                    if let Some(_val) = old_val {
                        if !overwrote_data {
                            println!(
                                "{} key \"{}\" overwritten by record {} (line {}).",
                                "Warning:".on_yellow().italic(),
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
        let mut filename = get_file_location(config.output_dir)?;

        if let Some(outfile) = config.output_filename {
            filename.push(lang);
            create_dir_all(&filename)?;
            filename.push(format!("{outfile}.json"));
        } else {
            filename.push(format!("{lang}.json"));
        }

        if let Some(json) = dictionary.get(lang) {
            writeln!(
                File::create(&filename)?,
                "{}",
                to_string_pretty(json).expect("Error writing {lang}.json.")
            )?;
        }
        println!(
            "{} written to {}.",
            filename.file_name().unwrap().to_string_lossy(),
            filename.parent().unwrap().to_string_lossy()
        );
    }

    Ok(())
}

/// Generate JSON files from CSV using StringRecord
///
/// * `reader` - a configured CSV reader
/// * `headings` - heading row for the CSV file
/// * `rows` - number of rows that are in the CSV file
/// * `config` - parsed command line configuration
pub fn generate_json_fast(
    reader: &mut Reader<File>,
    headings: &StringRecord,
    rows: usize,
    config: &Config,
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

                // Checking if there's already a language map entry for this translation key
                // and replace it if there is.
                if let Some(lang_map) = dictionary.get_mut(heading) {
                    // but only replace if there's an actual value.
                    if value.is_empty() {
                        continue;
                    };

                    let old_val = lang_map.insert(record[0].into(), value.into());
                    if let Some(_val) = old_val {
                        if !overwrote_data {
                            println!(
                                "{} key \"{}\" overwritten by record {} (line {}).",
                                "Warning:".on_yellow().italic(),
                                &record[0],
                                idx,
                                idx
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
        let mut filename = get_file_location(config.output_dir)?;

        if let Some(outfile) = config.output_filename {
            filename.push(lang);
            create_dir_all(&filename)?;
            filename.push(format!("{outfile}.json"));
        } else {
            filename.push(format!("{lang}.json"));
        }

        if let Some(json) = dictionary.get(lang) {
            writeln!(
                File::create(&filename)?,
                "{}",
                to_string_pretty(json).expect("Error writing {lang}.json.")
            )?;
        }
        println!(
            "{} written to {}.",
            filename.file_name().unwrap().to_string_lossy(),
            filename.parent().unwrap().to_string_lossy()
        );
    }

    Ok(())
}

#[cfg(test)]
mod generator_tests {
    use super::generate_json_fast;
    use crate::{get_file_location, get_file_reader, Config};
    use csv::{Reader, StringRecord, Terminator, Trim};
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    const CONFIG: Config = Config {
        delimiter: b',',
        escape_char: b'"',
        flexible: true,
        output_dir: "",
        output_filename: None,
        terminator_char: Terminator::CRLF,
        trim_whitespace: Trim::Fields,
    };

    const CSV_ALL_LANG: &str = "\
id,da_DK,de_DE,en_US,es_ES,fr_FR,it_IT,TextDomain,nl_NL,pt_BR,pt_PT,sv_SE,
new.translation,ny oversættelse,neue Übersetzung,new translation,nueva traducción,nouvelle traduction,nuova traduzione,,nieuwe vertaling,nova tradução,nova tradução,ny översättning,
";

    const CSV_ROW_A: &str = "\
id,da_DK_a,
new.translation,,
";

    const CSV_ROW_0: &str = "\
id,da_DK_0,
new.translation,,
";

    const CSV_ROW_1: &str = "\
id,da_DK_1,
new.translation,ny oversættelse,
";

    const CSV_ROW_2: &str = "\
id,da_DK_2,
new.translation,ny oversættelse,
new.translation,,
";

    const CSV_ROW_3: &str = "\
id,da_DK_3,
new.translation,ny oversættelse,
new.translation,,
new.translation,nyoversættelse,
";

    const CSV_ROW_4: &str = "\
id,da_DK_4,
new.translation,,
new.translation,ny oversættelse,
";

    const SSV_ROW_1: &str = "\
id;da_DK_s;
new.translation;ny oversættelse;
";

    const TSV_ROW_1: &str = "\
id\tda_DK_t\t
new.translation\tny oversættelse\t
";

    const DA_JSON_0: &str = "{\n  \"new.translation\": \"\"\n}";
    const DA_JSON_1: &str = "{\n  \"new.translation\": \"ny oversættelse\"\n}";
    const DA_JSON_2: &str = "{\n  \"new.translation\": \"nyoversættelse\"\n}";

    fn generate_csv_reader(
        input_filename: &str,
        input_data: &str,
        config: &Config,
    ) -> (Reader<File>, StringRecord, usize) {
        File::options()
            .write(true)
            .create(true)
            .open(input_filename)
            .unwrap()
            .write_all(input_data.as_bytes())
            .unwrap();
        let mut reader = get_file_reader(input_filename, config).unwrap();
        let file = get_file_location(input_filename).unwrap();
        let mut reader_count = Reader::from_path(file).unwrap();

        let headings = reader.headers().unwrap().clone();
        let rows = reader_count.byte_records().count();

        (reader, headings, rows)
    }

    #[test]
    fn it_writes_all_columns_except_textdomain_to_a_file() {
        let test_file_path = "test_file0.csv";
        let illegal_file = "TextDomain.json";
        let lang_file_list = [
            "da_DK.json",
            "de_DE.json",
            "en_US.json",
            "es_ES.json",
            "fr_FR.json",
            "it_IT.json",
            "nl_NL.json",
            "pt_BR.json",
            "pt_PT.json",
            "sv_SE.json",
        ];
        let translations = [
            "ny oversættelse",
            "neue Übersetzung",
            "new translation",
            "nueva traducción",
            "nouvelle traduction",
            "nuova traduzione",
            "nieuwe vertaling",
            "nova tradução",
            "nova tradução",
            "ny översättning",
        ];
        let mut test_conf = generate_csv_reader(test_file_path, CSV_ALL_LANG, &CONFIG);

        generate_json_fast(&mut test_conf.0, &test_conf.1, test_conf.2, &CONFIG).unwrap();

        for (idx, file) in lang_file_list.iter().enumerate() {
            let trans = fs::read_to_string(file).unwrap();
            let trans = trans.trim();
            fs::remove_file(file).unwrap();

            assert!(File::open(illegal_file).is_err());
            assert_eq!(
                trans,
                format!("{{\n  \"new.translation\": \"{}\"\n}}", translations[idx])
            );
        }
        fs::remove_file(test_file_path).unwrap();
    }

    #[test]
    fn it_creates_a_new_json_file_for_the_given_language_from_tsv() {
        let test_file_path = "test_file_1.tsv";
        let lang_file_path = "da_DK_t.json";
        let config = &Config {
            delimiter: b'\t',
            ..CONFIG
        };
        let mut test_conf = generate_csv_reader(test_file_path, TSV_ROW_1, config);

        generate_json_fast(&mut test_conf.0, &test_conf.1, test_conf.2, config).unwrap();

        let trans = fs::read_to_string(lang_file_path).unwrap();
        let trans = trans.trim();
        fs::remove_file(test_file_path).unwrap();
        fs::remove_file(lang_file_path).unwrap();

        assert_eq!(trans, DA_JSON_1);
    }

    #[test]
    fn it_creates_a_json_file_with_specific_name() {
        let test_file_path = "test_file_a.csv";
        let lang_file_path = "da_DK_a/locales.json";
        let config = &Config {
            output_filename: Some("locales"),
            ..CONFIG
        };

        let mut test_conf = generate_csv_reader(test_file_path, CSV_ROW_A, config);
        generate_json_fast(&mut test_conf.0, &test_conf.1, test_conf.2, config).unwrap();

        assert!(Path::new(lang_file_path).exists());
        fs::remove_file(lang_file_path).unwrap();

        let config = &Config {
            output_dir: "custom",
            ..*config
        };
        let lang_file_path = "custom/da_DK_a/locales.json";
        let mut test_conf = generate_csv_reader(test_file_path, CSV_ROW_A, config);
        generate_json_fast(&mut test_conf.0, &test_conf.1, test_conf.2, config).unwrap();

        assert!(Path::new(lang_file_path).exists());

        fs::remove_file(test_file_path).unwrap();
        fs::remove_file(lang_file_path).unwrap();
    }

    #[test]
    fn it_creates_a_new_json_file_for_the_given_language_from_ssv() {
        let test_file_path = "test_file_1.ssv";
        let lang_file_path = "da_DK_s.json";
        let config = &Config {
            delimiter: b';',
            ..CONFIG
        };
        let mut test_conf = generate_csv_reader(test_file_path, SSV_ROW_1, config);

        generate_json_fast(&mut test_conf.0, &test_conf.1, test_conf.2, config).unwrap();

        let trans = fs::read_to_string(lang_file_path).unwrap();
        let trans = trans.trim();
        fs::remove_file(test_file_path).unwrap();
        fs::remove_file(lang_file_path).unwrap();

        assert_eq!(trans, DA_JSON_1);
    }

    #[test]
    fn it_creates_a_new_json_file_for_the_given_language_from_csv() {
        // Empty translation
        let test_file_0 = "test_file0.csv";
        let lang_file_0 = "da_DK_0.json";
        let mut test_conf_0 = generate_csv_reader(test_file_0, CSV_ROW_0, &CONFIG);
        generate_json_fast(&mut test_conf_0.0, &test_conf_0.1, test_conf_0.2, &CONFIG).unwrap();

        // Actual translation
        let test_file_1 = "test_file1.csv";
        let lang_file_1 = "da_DK_1.json";
        let mut test_conf_1 = generate_csv_reader(test_file_1, CSV_ROW_1, &CONFIG);
        generate_json_fast(&mut test_conf_1.0, &test_conf_1.1, test_conf_1.2, &CONFIG).unwrap();

        let trans_0 = fs::read_to_string(lang_file_0).unwrap();
        let trans_0 = trans_0.trim();
        fs::remove_file(test_file_0).unwrap();
        fs::remove_file(lang_file_0).unwrap();

        let trans_1 = fs::read_to_string(lang_file_1).unwrap();
        let trans_1 = trans_1.trim();
        fs::remove_file(test_file_1).unwrap();
        fs::remove_file(lang_file_1).unwrap();

        assert_eq!(trans_0, DA_JSON_0);
        assert_eq!(trans_1, DA_JSON_1);
    }

    #[test]
    fn it_does_not_overwrite_existing_key_with_empty_value() {
        let test_file_path = "test_file2.csv";
        let lang_file_path = "da_DK_2.json";
        let mut test_conf = generate_csv_reader(test_file_path, CSV_ROW_2, &CONFIG);

        generate_json_fast(&mut test_conf.0, &test_conf.1, test_conf.2, &CONFIG).unwrap();

        let trans = fs::read_to_string(lang_file_path).unwrap();
        let trans = trans.trim();
        fs::remove_file(test_file_path).unwrap();
        fs::remove_file(lang_file_path).unwrap();

        assert_eq!(trans, DA_JSON_1);
    }

    #[test]
    fn it_overwrites_existing_key_with_new_value() {
        let test_file_path = "test_file3.csv";
        let lang_file_path = "da_DK_3.json";
        let mut test_conf = generate_csv_reader(test_file_path, CSV_ROW_3, &CONFIG);

        generate_json_fast(&mut test_conf.0, &test_conf.1, test_conf.2, &CONFIG).unwrap();

        let trans = fs::read_to_string(lang_file_path).unwrap();
        let trans = trans.trim();
        fs::remove_file(test_file_path).unwrap();
        fs::remove_file(lang_file_path).unwrap();

        assert_eq!(trans, DA_JSON_2);
    }

    #[test]
    fn it_overwrites_empty_value_with_new_value() {
        let test_file_path = "test_file4.csv";
        let lang_file_path = "da_DK_4.json";
        let mut test_conf = generate_csv_reader(test_file_path, CSV_ROW_4, &CONFIG);

        generate_json_fast(&mut test_conf.0, &test_conf.1, test_conf.2, &CONFIG).unwrap();

        let trans = fs::read_to_string(lang_file_path).unwrap();
        let trans = trans.trim();
        fs::remove_file(test_file_path).unwrap();
        fs::remove_file(lang_file_path).unwrap();

        assert_eq!(trans, DA_JSON_1);
    }
}
