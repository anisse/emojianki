mod annotations;
mod labels;
#[cfg(test)]
mod test;

use genanki_rs_rev::{Deck, Note, Package, basic_model};
use log::{debug, trace};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn generate_set() -> Vec<u8> {
    let labels = labels::get_labels();
    let annotations = annotations::parse_annotations(include_str!(
        "../../unicode/cldr-release-48-2/common/annotationsDerived/fr.xml"
    ));
    // Let's start with the flags

    let mut deck = Deck::new(
        20260717,
        "Drapeaux Emoji",
        "Apprendre les drapeaux avec les emojis",
    );
    for emoji in labels.categories["Flags"].iter() {
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
        } else {
            debug!("Emoji {emoji} has no annotation in french");
        }
    }

    let package = Package::new(vec![deck], std::collections::HashMap::new())
        .expect("Cannot create package for saving");
    let mut out = vec![];
    package.write(&mut out).expect("DB serialization failed");
    trace!("out ({}): {out:?}", out.len());
    out
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
        generate_set();
    }
}
