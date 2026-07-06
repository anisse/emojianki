use genanki_rs_rev::{Deck, Error, Note, Package, basic_model};
use log::info;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn generate_set() {
    info!("Hello, world!");

    let mut deck = Deck::new(1234, "Example Deck", "Example Deck containing 2 Flashcards");
    deck.add_note(
        Note::new(
            basic_model(),
            vec!["What is the capital of France?", "Paris"],
        )
        .unwrap(),
    );
    deck.add_note(
        Note::new(
            basic_model(),
            vec!["What is the capital of Germany?", "Berlin"],
        )
        .unwrap(),
    );

    let package = Package::new(vec![deck], std::collections::HashMap::new()).unwrap();
    package.write(NullWriter {}).unwrap();
}

struct NullWriter {}
impl std::io::Write for NullWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl std::io::Seek for NullWriter {
    fn seek(&mut self, _: std::io::SeekFrom) -> std::io::Result<u64> {
        Ok(0)
    }
}

#[wasm_bindgen(start)]
pub(crate) fn web_main() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");
    Ok(())
}
