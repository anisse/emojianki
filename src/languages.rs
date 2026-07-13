use std::collections::HashMap;

use crate::xml::{ParseEvent, parse_xml_streaming};

pub(crate) type Languages = HashMap<String, String>;

pub(crate) fn parse_languages(s: &str) -> Languages {
    let mut langs = Languages::new();
    let mut typ: Option<String> = None;
    parse_xml_streaming(
        s,
        &["ldml", "localeDisplayNames", "languages", "language"],
        |e| match e {
            ParseEvent::Start(mut attrs) => {
                typ = Some(attrs.remove("type").expect("type should be present"))
            }
            ParseEvent::Text(text) => {
                if let Some(label_type) = typ.take() {
                    langs.insert(label_type, text);
                }
            }
        },
    );
    langs
}
