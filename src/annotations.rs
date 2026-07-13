use std::collections::HashMap;

use log::{debug, trace};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

#[derive(Default, Debug)]
pub(crate) struct Annotation {
    pub(crate) tts: String,
    pub(crate) notes: Vec<String>,
}
type Annotations = HashMap<String, Annotation>;

pub(crate) fn parse_annotations(s: &str) -> Annotations {
    let mut annots = Annotations::new();
    let mut reader = Reader::from_str(s);
    reader.config_mut().trim_text(true);

    let mut path = vec![];
    let mut annot_tts: Option<String> = None;
    let mut annot: Option<String> = None;
    // Hand rolled parser, not much better than DOM, but that will have to do
    loop {
        match reader.read_event() {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,

            Ok(Event::End(e)) => {
                let tag_qname = e.name();
                let tag_name = str::from_utf8(tag_qname.as_ref()).expect("utf-8 tag name");
                // Just in case, because they should have been consumed as we got the text
                annot = None;
                annot_tts = None;
                assert_eq!(&path.pop().expect("something to pop"), tag_name);
            }
            Ok(Event::Start(e)) => {
                let tag_name = str::from_utf8(e.name().as_ref())
                    .expect("utf-8 tag name")
                    .to_string(); // alloc gallore
                path.push(tag_name);
                trace!("Tag path: {path:?}");
                if path == ["ldml", "annotations", "annotation"] {
                    let mut attrs = e
                        .attributes()
                        .map(|a| {
                            let attr = a.expect("Annotation tag should have attributes");
                            (
                                str::from_utf8(attr.key.as_ref())
                                    .expect("utf-8 str in attr key")
                                    .to_string(),
                                str::from_utf8(&(attr.value))
                                    .expect("utf-8 str in attr value")
                                    .to_string(),
                            )
                        })
                        .collect::<HashMap<_, _>>();
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
            }
            Ok(Event::Text(e)) => {
                let text = e
                    .decode()
                    .expect("utf-8 content in text of tag")
                    .into_owned();
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

            // There are several other `Event`s we do not consider here because this is minimal
            // streaming parsing
            _ => (),
        }
    }
    debug!("Annotations: {annots:?}");
    annots
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_labels() {
        crate::test::setup();
        parse_annotations(include_str!(
            "../../unicode/cldr-release-48-2/common/annotations/fr.xml"
        ));
    }
}
