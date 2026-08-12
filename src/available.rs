include!(concat!(env!("OUT_DIR"), "/available.rs"));

pub fn locales() -> Vec<String> {
    AVAILABLE.split("\0").map(|s| s.to_string()).collect()
}

pub fn language_translations(lang: &str) -> Result<impl Iterator<Item = (&str, &str)>, String> {
    let idx = AVAILABLE
        .split("\0")
        .position(|x| x == lang)
        .ok_or(format!("{lang} not found"))?;
    Ok(AVAILABLE
        .split("\0")
        .zip(
            TRANSLATIONS
                .split("\0\0")
                .nth(idx)
                .ok_or(format!("{lang} not in translations"))?
                .split("\0"),
        )
        .filter(|(_, t)| *t != "\x18")) // filter out empty translations
}
