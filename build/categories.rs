pub(crate) fn get_categories() -> Vec<String> {
    let file_content = include_str!("../cldr/common/properties/labels.txt");
    let mut categories: Vec<String> = vec![];
    for l in file_content.lines() {
        if l.starts_with("#") || l.is_empty() {
            continue;
        }
        let parts: Vec<&str> = l.split(&[';', '\t']).filter(|x| !x.is_empty()).collect();
        if !categories.contains(&parts[1].to_string()) {
            categories.push(parts[1].to_string());
        }
    }
    categories
}
