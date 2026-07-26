use std::collections::HashMap;

#[derive(Debug)]
pub(crate) enum Status {
    Component,
    FullyQualified,
    MinimallyQualified,
    Unqualified,
}

pub(crate) type Statuses = HashMap<String, Status>;

pub(crate) fn statuses() -> Statuses {
    let file_content = include_str!(
        "../cldr/tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/emoji/emoji-test.txt"
    );
    file_content
        .lines()
        .filter(|l| !(l.starts_with("#") || l.is_empty()))
        .map(|l| {
            let mut parts = l
                .split("#")
                .next()
                .expect("non-empty first part")
                .split(";")
                .map(str::trim);
            let emoji = parts
                .next()
                .expect("emoji components")
                .split(" ")
                .map(|s| u32::from_str_radix(s, 16).expect("hex emoji component"))
                .map(char::from_u32)
                .map(|o| o.expect("valid unicode scalar"))
                .collect();
            let qualif = parts
                .next()
                .map(|s| match s {
                    "component" => Status::Component,
                    "fully-qualified" => Status::FullyQualified,
                    "minimally-qualified" => Status::MinimallyQualified,
                    "unqualified" => Status::Unqualified,
                    _ => panic!("unknown qualification {s}"),
                })
                .expect("qualification");
            (emoji, qualif)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_qualifications() {
        crate::test::setup();
        statuses();
    }
}
