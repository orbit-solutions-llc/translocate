use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LangData {
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Deserialize, Serialize)]
struct Languages {
    #[serde(alias = "da_DK")]
    #[serde(alias = "da-DK")]
    da_dk: LangData,
    #[serde(alias = "de_DE")]
    #[serde(alias = "de-DE")]
    de_de: LangData,
    #[serde(alias = "en_US")]
    #[serde(alias = "en-US")]
    en_us: LangData,
    #[serde(alias = "es_ES")]
    #[serde(alias = "es-ES")]
    es_es: LangData,
    #[serde(alias = "fr_FR")]
    #[serde(alias = "fr-FR")]
    fr_fr: LangData,
    #[serde(alias = "it_IT")]
    #[serde(alias = "it-IT")]
    it_it: LangData,
    #[serde(alias = "nl_NL")]
    #[serde(alias = "nl-NL")]
    nl_nl: LangData,
    #[serde(alias = "pt_BR")]
    #[serde(alias = "pt-BR")]
    pt_br: LangData,
    #[serde(alias = "pt_PT")]
    #[serde(alias = "pt-PT")]
    pt_pt: LangData,
    #[serde(alias = "sv_SE")]
    #[serde(alias = "sv-SE")]
    sv_se: LangData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Translations {
    id: String,
    #[serde(alias = "TextDomain")]
    text_domain: String,
    #[serde(flatten)]
    languages: Languages,
}

pub trait FormatTranslation {
    fn translate_to(&self, lang: &str) -> (&str, &LangData);
}

impl FormatTranslation for Translations {
    fn translate_to(&self, lang: &str) -> (&str, &LangData) {
        match lang {
            "da_DK" => (&self.id, &self.languages.da_dk),
            "da-DK" => (&self.id, &self.languages.da_dk),
            "de_DE" => (&self.id, &self.languages.de_de),
            "de-DE" => (&self.id, &self.languages.de_de),
            "en_US" => (&self.id, &self.languages.en_us),
            "en-US" => (&self.id, &self.languages.en_us),
            "es_ES" => (&self.id, &self.languages.es_es),
            "es-ES" => (&self.id, &self.languages.es_es),
            "fr_FR" => (&self.id, &self.languages.fr_fr),
            "fr-FR" => (&self.id, &self.languages.fr_fr),
            "it_IT" => (&self.id, &self.languages.it_it),
            "it-IT" => (&self.id, &self.languages.it_it),
            "nl_NL" => (&self.id, &self.languages.nl_nl),
            "nl-NL" => (&self.id, &self.languages.nl_nl),
            "pt_BR" => (&self.id, &self.languages.pt_br),
            "pt-BR" => (&self.id, &self.languages.pt_br),
            "pt_PT" => (&self.id, &self.languages.pt_pt),
            "pt-PT" => (&self.id, &self.languages.pt_pt),
            "sv_SE" => (&self.id, &self.languages.sv_se),
            "sv-SE" => (&self.id, &self.languages.sv_se),
            // Don't call this for non-lang fields, e.g. 'id'.
            // panic is intentional here so filter non-langs first.
            &_ => unimplemented!(),
        }
    }
}
