use serde::{Deserialize, Serialize};
use serde_json::{to_string, to_string_pretty, Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{
    env,
    fs::File,
    io::{self, Write},
};

#[derive(Debug, Deserialize, Serialize)]
struct Languages {
    #[serde(alias = "da_DK")]
    da_dk: String,
    #[serde(alias = "de_DE")]
    de_de: String,
    #[serde(alias = "en_US")]
    en_us: String,
    #[serde(alias = "es_ES")]
    es_es: String,
    #[serde(alias = "fr_FR")]
    fr_fr: String,
    #[serde(alias = "it_IT")]
    it_it: String,
    #[serde(alias = "nl_NL")]
    nl_nl: String,
    #[serde(alias = "pt_BR")]
    pt_br: String,
    #[serde(alias = "pt_PT")]
    pt_pt: String,
    #[serde(alias = "sv_SE")]
    sv_se: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Translations {
    id: String,
    #[serde(alias = "TextDomain")]
    text_domain: String,
    #[serde(flatten)]
    languages: Languages,
}

trait FormatTranslation {
    fn translate_to(&self, lang: &str) -> (&str, &str);
}

impl FormatTranslation for Translations {
    fn translate_to(&self, lang: &str) -> (&str, &str) {
        match lang {
            "da_DK" => (&self.id, &self.languages.da_dk),
            "de_DE" => (&self.id, &self.languages.de_de),
            "en_US" => (&self.id, &self.languages.en_us),
            "es_ES" => (&self.id, &self.languages.es_es),
            "fr_FR" => (&self.id, &self.languages.fr_fr),
            "it_IT" => (&self.id, &self.languages.it_it),
            "nl_NL" => (&self.id, &self.languages.nl_nl),
            "pt_BR" => (&self.id, &self.languages.pt_br),
            "pt_PT" => (&self.id, &self.languages.pt_pt),
            "sv_SE" => (&self.id, &self.languages.sv_se),
            // Don't call this for non-lang fields, e.g. 'id'.
            // panic is intentional here so filter non-langs first.
            &_ => unimplemented!(),
        }
    }
}

const MSG: &str = "Please give file path as a command line argument!";

fn generate_json(file: &PathBuf) -> Result<(), std::io::Error> {
    // Grab csv data from csv file
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .escape(Some(b'\\'))
        .flexible(true)
        .quoting(true)
        .trim(csv::Trim::Fields)
        .from_path(file)?;
    let mut reader_count = csv::Reader::from_path(file)?;
    // Copy the data here to avoid borrowing. Allows deserializing (which borrows the
    // reader data) later on.
    let headings = reader.headers()?.clone();

    let rows = reader_count.byte_records().count();

    // HashMap::with_capacity_and_hasher(capacity, hasher) can be used instead, with hasher that is faster
    // https://crates.io/keywords/hasher
    let mut dictionary: HashMap<&str, Map<String, Value>> = HashMap::with_capacity(rows);

    let mut idx = 0;
    for item in reader.deserialize() {
        idx += 1;
        let record: Translations = match item {
            Ok(rec) => rec,
            Err(err) => {
                // We could be nice and break here, but there will be a lot of output
                // about duplicates so let's panic here and give user a chance to fix csv file.
                // println!("{}", err);
                // break;

                panic!("{}", err);
            }
        };
        let mut overwrote_data = false;
        for heading in headings.iter() {
            // Only process for language headings
            if heading != "id" && heading != "TextDomain" && !heading.is_empty() {
                let kv = record.translate_to(heading);
                if let Some(lang_map) = dictionary.get_mut(heading) {
                    // check value of insert to make sure we're not overwriting anything.
                    // Here into() is used to convert from string references into the key and value types the
                    // json Map needs.
                    let old_val = lang_map.insert(kv.0.into(), kv.1.into());
                    if let Some(_val) = old_val {
                        if !overwrote_data {
                            // println!("Overwrite previous entry for \"{}\".\nOld: {:#?}\nNew {:#?}", kv.0, val, kv.1);
                            println!(
                                "Overwrite prior entry for \"{}\" in record {idx} (line {}).",
                                kv.0,
                                idx + 1
                            );
                            overwrote_data = true;
                        }
                    };
                } else {
                    dictionary.insert(heading, Map::with_capacity(rows));

                    dictionary
                        .get_mut(heading)
                        .expect("Just created map")
                        .insert(kv.0.into(), kv.1.into());
                }
            }
        }
    }

    for lang in dictionary.keys() {
        let filename = format!("{lang}.json");
        let mut file = File::create(filename)?;
        if let Some(json) = dictionary.get(lang) {
            writeln!(file, "{}", to_string_pretty(json).unwrap())?;
        }
        println!("{lang}.json written to current directory.");
    }

    Ok(())
}

fn get_file() -> Result<PathBuf, io::Error> {
    let cwd = std::env::current_dir()?;
    // Get cli arguments, then make sure an arg was actually passed
    let path = env::args_os().nth(1).expect(MSG);

    // If the path begins with a '/' assume an absolute path. This
    // means windows users can only provide relative paths. 🤷🏾‍♂️
    if path
        .to_str()
        .expect("Invalid path string provided.")
        .starts_with('/')
    {
        Ok(PathBuf::from(path))
    } else {
        Ok(cwd.join(path))
    }
}

fn main() -> Result<(), io::Error> {
    let csv_path = if let Ok(path) = get_file() {
        path
    } else {
        PathBuf::from("APP_Trans.tsv")
    };

    generate_json(&csv_path)
}
