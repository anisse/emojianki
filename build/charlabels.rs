use std::collections::HashMap;

use crate::xml::{ParseEvent, parse_xml_streaming};

pub(crate) type CharLabels = HashMap<String, String>;

pub(crate) fn parse_charlabels(s: &str) -> CharLabels {
    let mut clabels = CharLabels::new();
    let mut typ: Option<String> = None;
    parse_xml_streaming(
        s,
        &["ldml", "characterLabels", "characterLabel"],
        |e| match e {
            ParseEvent::Start(mut attrs) => {
                typ = Some(attrs.remove("type").expect("type should be present"))
            }
            ParseEvent::Text(text) => {
                if let Some(label_type) = typ.take() {
                    clabels.insert(label_type, text);
                }
            }
        },
    );
    clabels
}
