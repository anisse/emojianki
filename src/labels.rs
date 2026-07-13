use std::collections::HashMap;

use icu_experimental::unicodeset_parse;
use log::trace;

#[derive(Debug)]
pub(crate) struct Labels {
    pub(crate) categories: HashMap<String, Vec<String>>,
    pub(crate) emojis: HashMap<String, String>,
}

pub(crate) fn get_labels() -> Labels {
    let file_content = include_str!("../../unicode/cldr-release-48-2/common/properties/labels.txt");
    let mut labels = Labels {
        categories: HashMap::new(),
        emojis: HashMap::new(),
    };
    for l in file_content.lines() {
        if l.starts_with("#") || l.is_empty() {
            continue;
        }
        let parts: Vec<&str> = l.split(&[';', '\t']).filter(|x| !x.is_empty()).collect();
        let (set, _) =
            unicodeset_parse::parse(parts[0]).expect("Built-in data string should always parse");
        // Way too many allocs!
        let mut insert = |em: &str, cat: &str| {
            labels
                .categories
                .entry(cat.to_string())
                .or_default()
                .push(em.to_string());
            // excessive alloc, should point to an enum / ID
            labels.emojis.insert(em.to_string(), cat.to_string());
        };
        if set.has_strings() {
            for s in set.strings().iter() {
                insert(s, parts[1]);
            }
        }
        for cp in set.code_points().iter_chars() {
            insert(&format!("{}", cp), parts[1]); // excessive alloc
        }
    }
    trace!("labels: {labels:?}");
    labels
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_labels() {
        crate::test::setup();
        get_labels();
    }
}
