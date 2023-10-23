use csv::{Error, StringRecord};
use serde::{Deserialize, Serialize};
use serde_json::{to_string, to_string_pretty, Map, Value};
use std::collections::{hash_map, HashMap};
use std::{env, fs::File, io::Write};

#[derive(Debug, Deserialize, Serialize)]
struct Languages {
    // da_DK: String,
    // de_DE: String,
    // en_US: String,
    // es_ES: String,
    // fr_FR: String,
    // it_IT: String,
    // nl_NL: String,
    // pt_BR: String,
    // pt_PT: String,
    // sv_SE: String,
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
    // TextDomain: String,
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

const MSG: &str = "Please give relative path to file as a command line argument!";

fn generate_json<'a>(file: &File) -> Result<(), std::io::Error> {
    // Grab csv data for standard input.
    // TODO: Change to getting from csv file, then read in twice so we can get a count
    // and set HashMap::with_capacity
    let mut reader = csv::Reader::from_reader(file);
    let mut reader_count = csv::Reader::from_reader(file);
    // Copy the data here to avoid borrowing. Allows deserializing (which borrows the
    // reader data) later on.
    let headings = reader.headers()?.clone();

    let rows = reader_count.records().count();

    // HashMap::with_capacity_and_hasher(capacity, hasher) can be used instead, with hasher that is faster
    // https://crates.io/keywords/hasher
    let mut dictionary: HashMap<&str, Map<String, Value>> = HashMap::with_capacity(rows);

    for item in reader.deserialize() {
        let record: Translations = item?;
        println!("The current record is {:?}", record);
        for heading in headings.iter() {
            // Only process for language headings
            if heading != "id" && heading != "TextDomain" && !heading.is_empty() {
                println!("Currently processing {}", heading);
                let kv = record.translate_to(heading);
                if let Some(lang_map) = dictionary.get_mut(heading) {
                    // check value of insert to make sure we're not overwriting anything.
                    // Here into() is used to convert from string references into the key and value types the
                    // json Map needs.
                    lang_map.insert(kv.0.into(), kv.1.into());
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
    println!("{:?}", dictionary);

    for lang in dictionary.keys() {
        let filename = format!("{lang}.json");
        let mut file = File::create(filename)?;
        if let Some(json) = dictionary.get(lang) {
            writeln!(file, "{}", to_string_pretty(json).unwrap())?;
        }
    }

    Ok(())
}

fn main() -> Result<(), csv::Error> {
    let cwd = std::env::current_dir()?;
    // Get cli arguments, then make sure an arg was actually passed
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("{MSG}")
    }
    let csv_path = cwd.join(&args[1]);
    let csv_file = File::open(csv_path)?;

    generate_json(&csv_file);
    Ok(())
}
