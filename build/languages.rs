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
                if !attrs.contains_key("alt") {
                    // Ignore alts and take main option
                    typ = Some(attrs.remove("type").expect("type should be present"))
                }
            }
            ParseEvent::Text(text) => {
                if let Some(label_type) = typ.take()
                // ignore continuations
                    && text != "↑↑↑"
                {
                    langs.insert(label_type, text);
                }
            }
        },
    );
    langs
}
