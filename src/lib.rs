mod annot;
mod available;
#[cfg(test)]
mod test;

use annot::parse_annots;
use available::emojis;

use genanki_rs_rev::{Deck, Field, Model, Note, Package, Template};
use log::trace;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Default)]
pub struct EmojiAnki {}

#[wasm_bindgen(getter_with_clone)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Pair {
    pub name: String,
    pub locale_name: String,
}

#[wasm_bindgen]
impl EmojiAnki {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }
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
    pub fn categories(&self, lang: String) -> Vec<Pair> {
        let mut categories = available::categories(&lang)
            .expect("Non-empty list of categories")
            .map(|(k, v)| Pair {
                name: k.to_string(),
                locale_name: v.to_string(),
            })
            .collect::<Vec<_>>();

        categories.sort();
        categories
    }

    #[wasm_bindgen]
    pub fn generate_set(
        &self,
        name: String,
        annotations: &[u8],
        categories: Vec<String>,
    ) -> Vec<u8> {
        let annotations = unsafe { str::from_utf8_unchecked(annotations) };

        let mut deck = Deck::new(
            20260717,
            &name,
            "EmojiAnki: https://anisse.github.io/emojianki",
        );
        for ((category, emojis), annots) in emojis().zip(parse_annots(annotations)) {
            if !categories.contains(&category.to_string()) {
                continue;
            }
            for (emoji, annot) in emojis.zip(annots).filter(|(_, a)| !a.is_empty()) {
                deck.add_note(
                    Note::new(Self::anki_model(), vec![emoji, annot])
                        .expect("Cannot create new note"),
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
        let ea = EmojiAnki::default();
        ea.generate_set(
            "Test name".to_string(),
            include_str!("../web/data/fr.txt").as_bytes(),
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
