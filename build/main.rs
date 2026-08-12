mod languages;
mod xml;

use std::env;
use std::fs;
use std::path::Path;

const ANNOT_DERIVED_DIR: &str = "cldr/common/annotationsDerived";
const MAIN_LANG_DIR: &str = "cldr/common/main/";

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("available.rs");
    let langs = languages();
    fs::write(&dest_path, available_languages_file_content(&langs)).expect("write available.rs");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", ANNOT_DERIVED_DIR);
}
fn available_languages_file_content(langs: &[String]) -> String {
    let mut file_content = "pub(crate) static AVAILABLE: &str = \"".to_string();
    file_content += &langs.iter().map(|s| format!("{s}\0")).collect::<String>();
    file_content += "\";\n";
    file_content += &format!(
        "pub(crate) static TRANSLATIONS: &str = \"{}\";\n",
        all_languages_names(langs)
    );
    file_content
}
fn languages() -> Vec<String> {
    let mut langs = fs::read_dir(ANNOT_DERIVED_DIR)
        .expect("cldr submodule to be initialized")
        .map(|res| res.expect("reading dir entry"))
        .map(|entry| {
            entry
                .path()
                .file_stem()
                .expect("basename")
                .to_str()
                .expect("utf8 filenames")
                .to_string()
        })
        .collect::<Vec<_>>();
    langs.sort();
    langs
}
// Generate a string to re-parse at runtime containing all languages's all languages translations
// Order follows the order of languages, which is serialized in the AVAILABLE string
// Format is as-is:
//  - for each language, a \0 is used as separator between language translations
//  - if a language has no translation (quite common), it contains the sole \x18 character (cancel), no guessing
//  - two \0\0 separate languages
//
fn all_languages_names(langs: &[String]) -> String {
    let mut out = String::new();
    for baselang in langs.iter() {
        let filename = format!("{MAIN_LANG_DIR}/{baselang}.xml");
        let content = fs::read(&filename).unwrap_or_else(|e| panic!("cannot read {filename}: {e}"));
        let translations = languages::parse_languages(
            str::from_utf8(&content).unwrap_or_else(|e| panic!("not utf8 in {filename}: {e}")),
        );
        for lang in langs.iter() {
            out += &format!(
                "{}\0",
                translations.get(lang).unwrap_or(&"\x18".to_string())
            );
        }
        out += "\0";
    }
    out
}
