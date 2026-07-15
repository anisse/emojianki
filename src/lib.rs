mod annotations;
mod available;
mod charlabels;
mod labels;
mod languages;
#[cfg(test)]
mod test;
mod xml;

use std::collections::HashSet;

use annotations::parse_annotations;
use available::LANGUAGES;
use charlabels::parse_charlabels;
use labels::{Labels, get_labels};
use languages::parse_languages;

use genanki_rs_rev::{Deck, Note, Package, basic_model};
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
pub struct Pair {
    pub name: String,
    pub locale_name: String,
}

#[wasm_bindgen]
impl EmojiAnki {
    #[wasm_bindgen]
    pub fn locales(&self) -> Vec<String> {
        LANGUAGES.iter().map(|s| s.to_string()).collect()
    }
    #[wasm_bindgen]
    pub fn languages(&self, main: &[u8]) -> Vec<Pair> {
        let supported = available::LANGUAGES
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<_>>();
        let mut languages = parse_languages(unsafe { str::from_utf8_unchecked(main) })
            .into_iter()
            .filter(|(k, _)| supported.contains(k))
            .map(|(k, v)| Pair {
                name: k,
                locale_name: v,
            })
            .collect::<Vec<_>>();
        languages.sort();
        languages
    }
    #[wasm_bindgen]
    pub fn categories(&self, main: &[u8]) -> Vec<Pair> {
        let s = unsafe { str::from_utf8_unchecked(main) };
        let clabels = parse_charlabels(s);
        debug!("{:?}", self.labels.categories);
        let mut categories = self
            .labels
            .categories
            .keys()
            .inspect(|k| {
                let r = k.to_ascii_lowercase().replace("&", "_").replace(" ", "");
                debug!("{r} = {:?}", clabels.get(&r));
            })
            .map(|k| Pair {
                name: k.clone(),
                locale_name: clabels[&k.to_ascii_lowercase().replace("&", "_").replace(" ", "")]
                    .clone(),
            })
            .collect::<Vec<_>>();
        categories.sort();
        categories
    }

    #[wasm_bindgen]
    pub fn generate_set(
        &self,
        name: String,
        annot: &[u8],
        annot_derived: &[u8],
        categories: Vec<String>,
    ) -> Vec<u8> {
        let annot_s = unsafe { str::from_utf8_unchecked(annot) };
        let annot_derived_s = unsafe { str::from_utf8_unchecked(annot_derived) };
        let mut annotations = parse_annotations(annot_s);
        annotations.extend(parse_annotations(annot_derived_s));

        let mut deck = Deck::new(
            20260717,
            &name,
            "EmojiAnki: https://anisse.github.io/emojianki",
        );
        for category in categories.into_iter() {
            for emoji in self.labels.categories[&category].iter() {
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
    fn test_fr_gen() {
        crate::test::setup();
        let ea = new_emojianki();
        ea.generate_set(
            "Test name".to_string(),
            include_str!("../cldr/common/annotations/fr.xml").as_bytes(),
            include_str!("../cldr/common/annotationsDerived/fr.xml").as_bytes(),
            [
                "Activities",
                "Smileys & People",
                "Objects",
                "Flags",
                "Symbols",
                "Travel & Places",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        );
    }
}
