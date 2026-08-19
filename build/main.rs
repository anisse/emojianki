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
                single_locale_translation(ldmls, baselang, lang, parent)
            )
        }
        out += "\0";
    }
    out
}
fn single_locale_translation(ldmls: &Ldmls, baselang: &str, lang: &str, parent: &str) -> String {
    match ldmls[baselang].languages.get(lang).map(String::as_str) {
        Some(UPPER) | None => {
            if let Some(parent_translation) = ldmls[parent].languages.get(lang) {
                if parent == "root" && parent_translation == UPPER {
                    locale_format(baselang, lang, ldmls).into_string()
                } else {
                    "\x1a".to_string()
                }
            } else {
                // Use locale pattern ??
                locale_format(baselang, lang, ldmls).into_string()
            }
        }
        Some(t) => t.to_string(),
    }
}
enum LocaleTranslation {
    None,
    Parent,
    Some(String),
}
impl LocaleTranslation {
    fn into_string(self) -> String {
        match self {
            LocaleTranslation::None => "\x18".to_string(),
            LocaleTranslation::Parent => "\x1a".to_string(),
            LocaleTranslation::Some(s) => s,
        }
    }
}
fn locale_format(baselang: &str, locale: &str, ldmls: &Ldmls) -> LocaleTranslation {
    let x = _locale_format(baselang, locale, ldmls);
    match x {
        LocaleTranslation::None => {
            println!("cargo::warning=No translation for {locale} in {baselang}")
        }
        LocaleTranslation::Parent => {
            println!("cargo::warning=Using parent for {locale} in {baselang}")
        }
        LocaleTranslation::Some(_) => {}
    }
    x
}
fn _locale_format(baselang: &str, locale: &str, ldmls: &Ldmls) -> LocaleTranslation {
    // Nothing to format
    if !locale.contains('_') {
        return LocaleTranslation::None;
    }
    let language = &ldmls[locale].identity.language;
    let mut has_from_item = false;
    let base = match value_or_parent(ldmls, baselang, |l| {
        l.languages.get(language).map(String::as_str)
    })
    .update_is_item(&mut has_from_item)
    {
        None => return LocaleTranslation::None,
        Some(t) => t,
    };
    let locale_pattern = value_or_root(
        ldmls,
        baselang,
        |l| Some(&l.locale_pattern),
        "locale_pattern",
    );
    // Root has no names for language, so there is no fallback, we consider this to be unavailable
    // and return None
    //
    let formatted = locale_pattern.replace("{0}", base);
    let mut script_territory = String::new();
    if let Some(script) = ldmls[locale].identity.script.as_ref() {
        // TODO: check if lang + script have a description
        let script_name = match value_or_parent(ldmls, baselang, |l| {
            l.scripts.get(script).map(String::as_str)
        })
        .update_is_item(&mut has_from_item)
        {
            None => return LocaleTranslation::None,
            Some(t) => t,
        };
        script_territory = script_name.to_string();
    }
    if let Some(territory) = ldmls[locale].identity.territory.as_ref() {
        let territory_name = match value_or_parent(ldmls, baselang, |l| {
            l.territories.get(territory).map(String::as_str)
        })
        .update_is_item(&mut has_from_item)
        {
            None => return LocaleTranslation::None,
            Some(t) => t,
        };
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
    if has_from_item {
        LocaleTranslation::Some(formatted.replace("{1}", &script_territory))
    } else {
        LocaleTranslation::Parent
    }
}
enum ItemOrParent<T> {
    None,
    Parent(T),
    Item(T),
}
impl<T> ItemOrParent<T> {
    // Return true if the value from the item itself
    fn update_is_item(self, has_from_item: &mut bool) -> Option<T> {
        match self {
            ItemOrParent::None => None,
            ItemOrParent::Parent(t) => Some(t),
            ItemOrParent::Item(t) => {
                *has_from_item = true;
                Some(t)
            }
        }
    }
}
fn value_or_parent<'a, 'b>(
    ldmls: &'a Ldmls,
    baselocale: &str,
    field: impl Fn(&'a Ldml) -> Option<&'a str>,
) -> ItemOrParent<&'a str> {
    if let Some(v) = field(&ldmls[baselocale])
        && v != UPPER
    {
        return ItemOrParent::Item(v);
    }
    let parent = &ldmls[baselocale].identity.language;
    if parent != baselocale
        && let Some(v) = field(&ldmls[parent])
        && v != UPPER
    {
        return ItemOrParent::Parent(v);
    }
    ItemOrParent::None
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
