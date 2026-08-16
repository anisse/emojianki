include!(concat!(env!("OUT_DIR"), "/available.rs"));

pub fn locales() -> Vec<String> {
    AVAILABLE.split("\0").map(|s| s.to_string()).collect()
}

pub fn language_translations(lang: &str) -> Result<impl Iterator<Item = (&str, &str)>, String> {
    let parent = lang.split('_').next().ok_or("empty string".to_string())?;
    let (idx, idx_parent) =
        AVAILABLE
            .split("\0")
            .enumerate()
            .fold((None, None), |mut res, (pos, x)| {
                if x == lang {
                    res.0 = Some(pos);
                }
                if x == parent {
                    res.1 = Some(pos);
                }
                res
            });
    let (idx, idx_parent) = (
        idx.ok_or(format!("{lang} not found"))?,
        idx_parent.ok_or(format!("{parent} not found"))?,
    );
    Ok(
        AVAILABLE
            .split("\0")
            .zip(
                TRANSLATIONS
                    .split("\0\0")
                    .nth(idx)
                    .ok_or(format!("{lang} not in translations"))?
                    .split("\0")
                    .zip(
                        TRANSLATIONS
                            .split("\0\0")
                            .nth(idx_parent)
                            .ok_or(format!("{parent} not in translations"))?
                            .split("\0"),
                    ),
            )
            .map(|(l, (tr, par))| if tr != "\x1a" { (l, tr) } else { (l, par) })
            .filter(|(_, t)| *t != "\x1a"), // filter out empty translations
    )
}
