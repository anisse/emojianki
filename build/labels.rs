use crate::statuses::{Status, Statuses, statuses};

use icu_experimental::unicodeset_parse;
use itertools::Itertools;

const LABELS: &str = include_str!("../cldr/common/properties/labels.txt");

pub(crate) fn parse_labels(file_content: &str) -> impl Iterator<Item = (String, String)> {
    file_content
        .lines()
        .filter(|l| !(l.starts_with("#") || l.is_empty()))
        .map(|l| {
            let mut parts = l.split(&[';', '\t']).filter(|x| !x.is_empty());
            let sequence = parts.next().expect("first part sequence");
            let category = parts.next().expect("category");
            (category, sequence)
        })
        .flat_map(|(cat, seq)| {
            let (set, _) =
                unicodeset_parse::parse(seq).expect("Built-in data string should always parse");
            set.strings()
                .iter()
                .map(str::to_string)
                .chain(set.code_points().iter_chars().map(|c| format!("{c}")))
                .map(|s| (cat.to_string(), s))
                .collect::<Vec<_>>()
                .into_iter()
        })
}

pub(crate) fn get_categories() -> Vec<String> {
    parse_labels(LABELS).map(|(cat, _)| cat).unique().collect()
}

pub(crate) fn all_emojis_qualified() -> String {
    let statuses = statuses();
    parse_labels(LABELS)
        .chunk_by(|(cat, _)| cat.clone())
        .into_iter()
        .map(|(_, emojis)| {
            emojis
                .filter_map(|(_, e)| qualified(&statuses, &e))
                .map(|e| format!("{e}\0"))
                .collect::<String>()
        })
        .map(|list| format!("{list}\0"))
        .collect()
}
fn qualified(statuses: &Statuses, emoji: &str) -> Option<String> {
    match statuses.get(emoji) {
        Some(Status::Component) => None,
        Some(Status::FullyQualified) => Some(emoji.to_string()),
        Some(Status::MinimallyQualified) => {
            let mut s = String::with_capacity(emoji.len() + 1);
            s.push_str(emoji);
            s.push('\u{FE0F}');
            assert_eq!(statuses.get(&s), Some(&Status::FullyQualified));
            Some(s)
        }
        Some(Status::Unqualified) => {
            let mut s = String::with_capacity(emoji.len() + 1);
            let mut chars = emoji.chars();
            s.push(chars.next().expect("One char"));
            s.push('\u{FE0F}');
            s.extend(chars);
            assert_eq!(statuses.get(&s), Some(&Status::FullyQualified));
            Some(s)
        }
        None => None,
    }
}
