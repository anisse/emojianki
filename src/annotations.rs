use std::collections::HashMap;

use crate::xml::{ParseEvent, parse_xml_streaming};
use log::trace;

#[derive(Default, Debug)]
pub(crate) struct Annotation {
    pub(crate) tts: String,
    pub(crate) notes: Vec<String>,
}
pub(crate) type Annotations = HashMap<String, Annotation>;

pub(crate) fn parse_annotations(s: &str) -> Annotations {
    let mut annots = Annotations::new();
    let mut annot: Option<String> = None;
    let mut annot_tts: Option<String> = None;

    parse_xml_streaming(s, &["ldml", "annotations", "annotation"], |e| match e {
        ParseEvent::Start(mut attrs) => {
            let cp = attrs.remove("cp").expect("cp should be present");
            if let Some(typ) = attrs.get("type")
                && typ == "tts"
            {
                trace!("cp {} is tts", cp);
                annot_tts = Some(cp);
            } else {
                trace!("attributes: {attrs:?}",);
                annot = Some(cp);
            }
        }
        ParseEvent::Text(text) => {
            if let Some(cp) = annot_tts.take() {
                annots.entry(cp).or_default().tts = text;
            } else if let Some(cp) = annot.take() {
                annots
                    .entry(cp)
                    .or_default()
                    .notes
                    .extend(text.split("|").map(|s| s.trim()).map(str::to_string));
            }
        }
    });
    trace!("Annotations: {annots:?}");
    annots
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_annotations() {
        crate::test::setup();
        parse_annotations(include_str!("../cldr/common/annotations/fr.xml"));
    }
}
