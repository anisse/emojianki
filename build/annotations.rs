// Generate build-time files containing the emoji names for every language - just what is used by emojianki
use std::{collections::HashMap, fs};

use super::ANNOT_DERIVED_DIR;
use super::ANNOT_DIR;
use crate::xml::{ParseEvent, parse_xml_streaming};

pub(crate) type LangAnnotations = HashMap<String, String>;
pub(crate) type Annotations<'a> = HashMap<&'a String, LangAnnotations>;

pub(crate) fn parse_annotations(s: &str) -> LangAnnotations {
    let mut annots = LangAnnotations::new();
    let mut annot_tts: Option<String> = None;

    parse_xml_streaming(s, &["ldml", "annotations", "annotation"], |e| match e {
        ParseEvent::Start(mut attrs) => {
            let cp = attrs.remove("cp").expect("cp should be present");
            if let Some(typ) = attrs.get("type")
                && typ == "tts"
            {
                annot_tts = Some(cp);
            }
        }
        ParseEvent::Text(text) => {
            if let Some(cp) = annot_tts.take() {
                annots.insert(cp, text);
            }
        }
    });
    annots
}

pub(crate) fn load_annotations<'a>(langs: &'a [String]) -> Annotations<'a> {
    let mut annotations = Annotations::new();
    for lang in langs.iter() {
        let annot = fs::read_to_string(format!("{ANNOT_DIR}/{lang}.xml")).expect("annot");
        let annot_derived =
            fs::read_to_string(format!("{ANNOT_DERIVED_DIR}/{lang}.xml")).expect("annot_derived");
        let mut annot_lang = parse_annotations(&annot);
        annot_lang.extend(parse_annotations(&annot_derived));
        //println!("cargo::warning=Annotations for {lang} : {annot_lang:?}");
        annotations.insert(lang, annot_lang);
    }
    annotations
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_annotations() {
        parse_annotations(include_str!("../cldr/common/annotations/fr.xml"));
    }
}
