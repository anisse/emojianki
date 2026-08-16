// CLDR Common main files parsing
use std::collections::HashMap;

use crate::xml::{Matcher, ParseEvent, parse_xml_multiple};

pub(crate) type Languages = HashMap<String, String>;
pub(crate) type Territories = HashMap<String, String>;
pub(crate) type Scripts = HashMap<String, String>;
pub(crate) type CharLabels = HashMap<String, String>;

// Only contains what's needed by emojianki
#[derive(Default, Debug)]
pub(crate) struct Ldml {
    pub(crate) identity: Identity,
    pub(crate) locale_pattern: String,
    pub(crate) locale_separator: String,
    pub(crate) languages: Languages,
    pub(crate) territories: Territories,
    pub(crate) scripts: Scripts,
    pub(crate) charlabels: CharLabels,
}

#[derive(Default, Debug)]
pub(crate) struct Identity {
    pub(crate) language: String,
    pub(crate) script: Option<String>,
    pub(crate) territory: Option<String>,
    pub(crate) variant: Option<String>,
}

pub(crate) fn parse_ldml(s: &str) -> Ldml {
    let mut ldml = Ldml::default();
    parse_xml_multiple(
        s,
        &mut [
            Matcher {
                path: &["ldml", "identity", "language"],
                cb_fn: &mut |e| match e {
                    ParseEvent::Start(mut attrs) => {
                        ldml.identity.language = attrs
                            .remove("type")
                            .expect("type should be present for language")
                    }
                    ParseEvent::Text(_) => {}
                },
            },
            Matcher {
                path: &["ldml", "identity", "script"],
                cb_fn: &mut xml_single_type_optional(&mut ldml.identity.script),
            },
            Matcher {
                path: &["ldml", "identity", "territory"],
                cb_fn: &mut xml_single_type_optional(&mut ldml.identity.territory),
            },
            Matcher {
                path: &["ldml", "identity", "variant"],
                cb_fn: &mut xml_single_type_optional(&mut ldml.identity.variant),
            },
            Matcher {
                path: &[
                    "ldml",
                    "localeDisplayNames",
                    "localeDisplayPattern",
                    "localePattern",
                ],
                cb_fn: &mut |e| match e {
                    ParseEvent::Start(_) => {}
                    ParseEvent::Text(text) => ldml.locale_pattern = text,
                },
            },
            Matcher {
                path: &[
                    "ldml",
                    "localeDisplayNames",
                    "localeDisplayPattern",
                    "localeSeparator",
                ],
                cb_fn: &mut |e| match e {
                    ParseEvent::Start(_) => {}
                    ParseEvent::Text(text) => ldml.locale_separator = text,
                },
            },
            Matcher {
                path: &["ldml", "characterLabels", "characterLabel"],
                /* Technically charlabels have no "alt" attributes, but re-using the same parsing
                 * does not hurt here */
                cb_fn: &mut xml_attr_type_and_value(&mut ldml.charlabels),
            },
            Matcher {
                path: &["ldml", "localeDisplayNames", "languages", "language"],
                cb_fn: &mut xml_attr_type_and_value(&mut ldml.languages),
            },
            Matcher {
                path: &["ldml", "localeDisplayNames", "territories", "territory"],
                cb_fn: &mut xml_attr_type_and_value(&mut ldml.territories),
            },
            Matcher {
                path: &["ldml", "localeDisplayNames", "scripts", "script"],
                cb_fn: &mut xml_attr_type_and_value(&mut ldml.scripts),
            },
        ],
    );
    assert!(
        !ldml.identity.language.is_empty(),
        "Lang {s} has empty identity language"
    );
    ldml
}

fn xml_attr_type_and_value(map: &mut HashMap<String, String>) -> impl FnMut(ParseEvent) {
    let mut typ: Option<String> = None;
    move |e| match e {
        ParseEvent::Start(mut attrs) => {
            if !attrs.contains_key("alt") {
                // Ignore alts and take main option
                typ = Some(attrs.remove("type").expect("type should be present"))
            }
        }
        ParseEvent::Text(text) => {
            if let Some(label_type) = typ.take() {
                map.insert(label_type, text);
            }
        }
    }
}

fn xml_single_type_optional(typ: &mut Option<String>) -> impl FnMut(ParseEvent) {
    |e| match e {
        ParseEvent::Start(mut attrs) => *typ = Some(attrs.remove("type").expect("type missing")),
        ParseEvent::Text(_) => {}
    }
}
