pub(super) fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    format!(
        "{}{}",
        chars.next().unwrap().to_ascii_lowercase(),
        chars.as_str()
    )
}

pub(super) fn snake(value: &str) -> String {
    let mut result = String::new();
    for (index, character) in value.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 && !result.ends_with('_') {
                result.push('_');
            }
            result.push(character.to_ascii_lowercase());
        } else {
            result.push(character);
        }
    }
    result
}
