use csv::Error;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, to_string_pretty, Map, Value};
use std::io;

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
    fn translate_to(&self, lang: &str) -> String;
}

impl FormatTranslation for Translations {
    fn translate_to(&self, lang: &str) -> String {
        let id = &self.id;
        match lang {
            "da_DK" => json!({id: self.languages.da_dk}).to_string(),
            "de_DE" => json!({id: self.languages.de_de}).to_string(),
            "en_US" => json!({id: self.languages.en_us}).to_string(),
            "es_ES" => json!({id: self.languages.es_es}).to_string(),
            "fr_FR" => json!({id: self.languages.fr_fr}).to_string(),
            "it_IT" => json!({id: self.languages.it_it}).to_string(),
            "nl_NL" => json!({id: self.languages.nl_nl}).to_string(),
            "pt_BR" => json!({id: self.languages.pt_br}).to_string(),
            "pt_PT" => json!({id: self.languages.pt_pt}).to_string(),
            "sv_SE" => json!({id: self.languages.sv_se}).to_string(),
            // Don't call this for non-lang fields, e.g. 'id'.
            // panic is intentional here so filter non-langs first.
            &_ => unimplemented!(),
        }
    }
}

struct Translation {
    translations: Vec<String>,
}

fn generate_json() -> Result<(), std::io::Error> {
    let mut reader = csv::Reader::from_reader(io::stdin());
    let headings = reader.headers()?.clone();

    for item in reader.deserialize() {
        let record: Translations = item?;
        println!("The current record is {:?}", record);
        for heading in headings.iter() {
            if heading != "id" && heading != "TextDomain" && !heading.is_empty() {
                println!("Currently processing {}", heading);
                println!("{}", &record.translate_to(heading));
            }
        }
    }
    Ok(())
}

fn main() {
    let mut m = Map::new();
    m.insert("Lorem".to_string(), "ipsum".into());
    let x: Value = m.into();
    println!("{:?}", x);

    if let Err(error) = generate_json() {
        println!("Error running example! {}", error);
    };
}
