mod annotations;
mod available;
mod charlabels;
mod labels;
mod statuses;
#[cfg(test)]
mod test;
mod xml;

use annotations::parse_annotations;
use charlabels::parse_charlabels;
use labels::{Labels, get_labels};
use statuses::{Status, Statuses, statuses};

use genanki_rs_rev::{Deck, Field, Model, Note, Package, Template};
use log::{debug, trace};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct EmojiAnki {
    labels: Labels,
    statuses: Statuses,
}

#[wasm_bindgen]
pub fn new_emojianki() -> EmojiAnki {
    EmojiAnki {
        labels: get_labels(),
        statuses: statuses(),
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
        available::locales()
    }
    #[wasm_bindgen]
    pub fn languages(&self, lang: String) -> Vec<Pair> {
        available::language_translations(&lang)
            .expect("Non-empty list of languages")
            .map(|(k, t)| Pair {
                name: k.to_string(),
                locale_name: t.to_string(),
            })
            .collect::<Vec<_>>()
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
                if let Some(annot) = annotations.get(emoji).or_else(|| {
                    /* Match without variant selectors in case the annotation is without it
                     */
                    annotations.get(
                        &emoji
                            .chars()
                            // This character is a variant selector (color emoji vs text) and is
                            // not removed by classic unicode normalization
                            .filter(|c| *c != '\u{FE0F}')
                            .collect::<String>(),
                    )
                }) && let Some(emoji_qual) = self.qualified(emoji)
                {
                    deck.add_note(
                        Note::new(Self::anki_model(), vec![&emoji_qual, &annot.tts])
                            .expect("Cannot create new note"),
                    );
                    debug!(
                        "Emoji {emoji_qual} ({:?}) TTS is {}",
                        self.statuses.get(&emoji_qual),
                        annot.tts
                    );
                } else {
                    debug!(
                        "Emoji {{{emoji}}} {:x?} has no annotation or is {:?}",
                        emoji.chars().map(|c| c as u32).collect::<Vec<_>>(),
                        self.statuses.get(emoji),
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

    fn anki_model() -> Model {
        Model::with_options(
            1784196000,
            "EmojiAnki Base card",
            vec![
                Field::new("Front").font("Arial"),
                Field::new("Back").font("Arial"),
            ],
            vec![
                Template::new("Card 1")
                    .qfmt("{{Front}}")
                    .qfmt("<div class=\"emoji\">{{Front}}</div>")
                    .afmt("{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}"),
            ],
            Some(
                ".card {
                    font-family: arial;
                    font-size: 20px;
                    line-height: 1.5;
                    text-align: center;
                    color: black;
                    background-color: white;
                }
                .emoji {
                    font-size: 90px; \
                    text-shadow: 0 0 45px #ffffff; \
                }
            ",
            ),
            None,
            None,
            None,
            None,
        )
    }
    fn qualified(&self, emoji: &str) -> Option<String> {
        if let Some(status) = self.statuses.get(emoji) {
            debug!("{emoji} is {status:?}");
            match status {
                Status::Component => None,
                Status::FullyQualified => Some(emoji.to_string()),
                Status::MinimallyQualified => {
                    let mut s = String::with_capacity(emoji.len() + 1);
                    s.push_str(emoji);
                    s.push('\u{FE0F}');
                    assert_eq!(self.statuses.get(&s), Some(&Status::FullyQualified));
                    Some(s)
                }
                Status::Unqualified => {
                    let mut s = String::with_capacity(emoji.len() + 1);
                    let mut chars = emoji.chars();
                    s.push(chars.next().expect("One char"));
                    s.push('\u{FE0F}');
                    s.extend(chars);
                    assert_eq!(self.statuses.get(&s), Some(&Status::FullyQualified));
                    Some(s)
                }
            }
        } else {
            None
        }
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
