// Generate build-time files containing the emoji names for every language - just what is used by emojianki
use std::collections::HashMap;

use crate::xml::{ParseEvent, parse_xml_streaming};

pub(crate) type Annotations = HashMap<String, String>;

pub(crate) fn parse_annotations(s: &str) -> Annotations {
    let mut annots = Annotations::new();
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_annotations() {
        parse_annotations(include_str!("../cldr/common/annotations/fr.xml"));
    }
}
