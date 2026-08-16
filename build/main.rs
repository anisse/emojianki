mod categories;
mod charlabels;
mod languages;
mod ldml;
mod xml;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use ldml::Ldml;

const ANNOT_DERIVED_DIR: &str = "cldr/common/annotationsDerived";
const MAIN_LANG_DIR: &str = "cldr/common/main/";
const UPPER: &str = "↑↑↑";

type Ldmls = HashMap<String, Ldml>; // Technically an OrderedMap would help to have a single
// datastructure instead of the Vec + HashMap we use here

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("available.rs");
    let mut langs = languages();
    let ldmls = ldmls(&langs);
    // Now that we have root's ldml, remove it from available languages
    langs.remove(
        langs
            .binary_search(&"root".to_string())
            .expect("root to be present"),
    );
    fs::write(&dest_path, available_languages_file_content(&langs, &ldmls))
        .expect("write available.rs");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", ANNOT_DERIVED_DIR);
}
fn available_languages_file_content(langs: &[String], ldmls: &Ldmls) -> String {
    let mut file_content = "pub(crate) static AVAILABLE: &str = \"".to_string();
    file_content += &langs
        .iter()
        .filter(|s| *s != "root") // Root is only used internally and not exposed to the frontend
        .map(|s| format!("{s}\0"))
        .collect::<String>();
    file_content += "\";\n";
    file_content += &format!(
        "pub(crate) static TRANSLATIONS: &str = \"{}\";\n",
        all_languages_names(langs, ldmls)
    );
    /*
    file_content += &format!(
        "pub(crate) static CATEGORIES: &str = \"{}\";\n",
        all_categories_names(langs, ldmls)
    );
    */
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
// YOLO, load it all in memory. It's only at build time
// it's unnecessary, at a given time we only need the root and the locale's eventual parent
fn ldmls(langs: &[String]) -> Ldmls {
    langs
        .iter()
        // This really shouldn't have been an iterator loop. YOLO
        .map(|lang| (lang, format!("{MAIN_LANG_DIR}/{lang}.xml")))
        .map(|(lang, filename)| {
            (
                lang,
                fs::read(&filename).unwrap_or_else(|e| panic!("cannot read {filename}: {e}")),
            )
        })
        .map(|(lang, content)| {
            (
                lang.to_string(),
                ldml::parse_ldml(
                    str::from_utf8(&content)
                        .unwrap_or_else(|e| panic!("not utf8 in ldml for {lang}: {e}")),
                ),
            )
        })
        .collect()
}

// Generate a string to re-parse at runtime containing all languages's all languages translations
// Order follows the order of languages, which is serialized in the AVAILABLE string
// Format is as-is:
//  - for each language, a \0 is used as separator between language translations
//  -
//  - XXX: if a language has no translation (can happen), it contains the sole \x18 character (cancel), no guessing
//  - if the translation could be available at the parent language, the character \x1a (substitute) will
//  be used
//  - two \0\0 separate languages
//
fn all_languages_names(langs: &[String], ldmls: &Ldmls) -> String {
    let mut out = String::new();
    for baselang in langs.iter() {
        let parent = if baselang.contains('_') {
            &ldmls[baselang].identity.language
        } else {
            "root"
        };
        assert!(ldmls.contains_key(parent));
        for lang in langs.iter() {
            out += &format!(
                "{}\0",
                match ldmls[baselang].languages.get(lang).map(String::as_str) {
                    Some(UPPER) | None => {
                        if let Some(parent_translation) = ldmls[parent].languages.get(lang) {
                            if parent == "root" && parent_translation == UPPER {
                                locale_format(baselang, lang, ldmls).unwrap_or_else(|| {
                                    println!(
                                        "cargo::warning=No translation for {lang} in {baselang}"
                                    );
                                    "\x18".to_string()
                                })
                            } else {
                                "\x1a".to_string()
                            }
                        } else {
                            // Use locale pattern ??
                            locale_format(baselang, lang, ldmls).unwrap_or_else(|| {
                                println!("cargo::warning=No translation for {lang} in {baselang}");
                                "\x18".to_string()
                            })
                        }
                    }
                    Some(t) => t.to_string(),
                }
            );
        }
        out += "\0";
    }
    out
}
fn locale_format(baselang: &str, locale: &str, ldmls: &Ldmls) -> Option<String> {
    // Nothing to format
    locale.contains('_').then_some(())?;
    let locale_pattern = value_or_root(
        ldmls,
        baselang,
        |l| Some(&l.locale_pattern),
        "locale_pattern",
    );
    let language = &ldmls[locale].identity.language;
    // Root has no names for language, so there is no fallback, we consider this to be unavailable
    // and return None
    //
    // TODO: if the locale_format is identitcal to the parent's, do not duplicate it
    let base = value_or_parent(ldmls, baselang, |l| {
        l.languages.get(language).map(String::as_str)
    })?;
    let formatted = locale_pattern.replace("{0}", base);
    let mut script_territory = String::new();
    if let Some(script) = ldmls[locale].identity.script.as_ref() {
        let script_name = value_or_parent(ldmls, baselang, |l| {
            l.scripts.get(script).map(String::as_str)
        })?;
        script_territory = script_name.to_string();
    }
    if let Some(territory) = ldmls[locale].identity.territory.as_ref() {
        let territory_name = value_or_parent(ldmls, baselang, |l| {
            l.territories.get(territory).map(String::as_str)
        })?;
        if script_territory.is_empty() {
            script_territory = territory_name.to_string();
        } else {
            let locale_separator = value_or_root(
                ldmls,
                baselang,
                |l| Some(&l.locale_separator),
                "locale_separator",
            );
            script_territory = locale_separator.replace("{0}", &script_territory);
            script_territory = script_territory.replace("{1}", territory_name);
        }
    }
    Some(formatted.replace("{1}", &script_territory))
}

fn value_or_parent<'a, 'b>(
    ldmls: &'a Ldmls,
    baselocale: &str,
    field: impl Fn(&'a Ldml) -> Option<&'a str>,
) -> Option<&'a str> {
    if let Some(v) = field(&ldmls[baselocale])
        && v != UPPER
    {
        return Some(v);
    }
    let parent = &ldmls[baselocale].identity.language;
    if parent != baselocale
        && let Some(v) = field(&ldmls[parent])
        && v != UPPER
    {
        return Some(v);
    }
    None
}
fn value_or_root<'a, 'b>(
    ldmls: &'a Ldmls,
    baselocale: &str,
    field: impl Fn(&'a Ldml) -> Option<&'a str>,
    err_message: &str,
) -> &'a str {
    if let Some(v) = field(&ldmls[baselocale])
        && v != UPPER
    {
        return v;
    }
    let parent = &ldmls[baselocale].identity.language;
    if parent != baselocale
        && let Some(v) = field(&ldmls[parent])
        && v != UPPER
    {
        return v;
    }
    let v = field(&ldmls["root"]).unwrap_or_else(|| {
        panic!("root should have the field (not in {baselocale} nor {parent}): {err_message}")
    });
    assert!(v != UPPER, "for {v:?}: {err_message}");
    assert!(!v.is_empty());
    v
}
// Generate a string to re-parse at runtime containing all languages's character labels
// Order follows the order of languages, which is serialized in the AVAILABLE string
// Format is as-is:
//  - for each language, a \0 is used as separator between language translations
//  - if a character label has no translation panic
//  - two \0\0 separate languages
fn all_categories_names(langs: &[String]) -> String {
    let categories = categories::get_categories();
    let mut out = String::new();
    for baselang in langs.iter() {
        let filename = format!("{MAIN_LANG_DIR}/{baselang}.xml");
        let content = fs::read(&filename).unwrap_or_else(|e| panic!("cannot read {filename}: {e}"));
        let labels = charlabels::parse_charlabels(
            str::from_utf8(&content).unwrap_or_else(|e| panic!("not utf8 in {filename}: {e}")),
        );
        for label in categories.iter() {
            let cat_lower = label
                .to_ascii_lowercase()
                .replace("&", "_")
                .replace(" ", "");
            out += &format!(
                "{}\0",
                labels
                    .get(&cat_lower)
                    //.unwrap_or(&"\x18".to_string())
                    .unwrap_or_else(|| panic!("Label for {label} in {baselang} not found"))
            );
        }
        out += "\0";
    }
    out
}
