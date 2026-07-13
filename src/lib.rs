mod annotations;
mod available;
mod charlabels;
mod labels;
mod languages;
#[cfg(test)]
mod test;
mod xml;

use annotations::{Annotations, parse_annotations};
use genanki_rs_rev::{Deck, Note, Package, basic_model};
use labels::{Labels, get_labels};
use log::{debug, trace};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct EmojiAnki {
    labels: Labels,
}

#[wasm_bindgen]
pub fn new_emojianki() -> EmojiAnki {
    EmojiAnki {
        labels: get_labels(),
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Category {
    pub name: String,
    pub locale_name: String,
}

#[wasm_bindgen]
impl EmojiAnki {
    pub fn list_categories(&self, locale: &str) -> Vec<Category> {
        let mut categories = self
            .labels
            .categories
            .keys()
            .map(|k| Category {
                name: k.clone(),
                locale_name: String::new(),
            })
            .collect::<Vec<_>>();
        categories.sort();
        categories
    }

    #[wasm_bindgen]
    pub fn generate_set(&self, locale: &str) -> Vec<u8> {
        let annotations = parse_annotations(include_str!("../cldr/common/annotations/fr.xml"));

        // Let's start with the flags
        let mut deck = Deck::new(
            20260717,
            "Drapeaux Emoji",
            "Apprendre les drapeaux avec les emojis",
        );
        for emoji in self.labels.categories["Flags"].iter() {
            if let Some(annot) = annotations.get(emoji) {
                deck.add_note(
                    Note::new(
                        basic_model(),
                        vec![
                            &format!(
                                "<div style=\"\
                                font-size: 90px; \
                                text-shadow: 0 0 45px white; \
                            \">{emoji}</div>"
                            ),
                            &annot.tts,
                        ],
                    )
                    .expect("Cannot create new note"),
                );
                debug!("Emoji {emoji} TTS is {}", annot.tts);
            } else {
                debug!(
                    "Emoji {{{emoji}}} {:x?} has no annotation",
                    emoji.chars().map(|c| c as u32).collect::<Vec<_>>(),
                );
            }
        }

        let package = Package::new(vec![deck], std::collections::HashMap::new())
            .expect("Cannot create package for saving");
        let mut out = vec![];
        package.write(&mut out).expect("DB serialization failed");
        trace!("out ({}): {out:?}", out.len());
        out
    }
}

#[wasm_bindgen(start)]
pub(crate) fn web_main() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_labels() {
        crate::test::setup();
        let ea = new_emojianki();
        ea.generate_set("fr");
    }
}
