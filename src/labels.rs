use std::collections::{HashMap, HashSet};

use icu_experimental::unicodeset_parse;
use log::info;

#[derive(Debug)]
pub(crate) struct Labels {
    categories: HashSet<String>,
    emojis: HashMap<String, String>,
}

pub(crate) fn get_labels() -> Labels {
    let file_content = include_str!("../../unicode/cldr-release-48-2/common/properties/labels.txt");
    let mut labels = Labels {
        categories: HashSet::new(),
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
            if !labels.categories.contains(cat) {
                labels.categories.insert(cat.to_string());
            }
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
    labels
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_labels() {
        init_log().unwrap();
        get_labels();
    }
    use log::{Level, Metadata, Record};
    use std::error::Error;
    struct TestLogger;
    fn init_log() -> Result<(), Box<dyn Error>> {
        log::set_logger(&LOGGER)?;
        log::set_max_level(Level::Debug.to_level_filter());
        Ok(())
    }

    impl log::Log for TestLogger {
        fn enabled(&self, _metadata: &Metadata) -> bool {
            true
        }

        fn log(&self, record: &Record) {
            println!("{}", record.args());
        }

        fn flush(&self) {}
    }

    static LOGGER: TestLogger = TestLogger;
}
