use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedText {
    pub clean: String,
    pub parts: Vec<String>,
    pub format_controls: Vec<String>,
    pub ruby_removed: usize,
    pub newline_removed: usize,
}

pub fn unwrap_name(text: &str) -> Option<String> {
    let trimmed = text.trim();
    trimmed
        .strip_prefix('【')
        .and_then(|value| value.strip_suffix('】'))
        .map(ToOwned::to_owned)
}

pub fn normalize_text(text: &str) -> NormalizedText {
    let (without_display_controls, ruby_removed, newline_removed) = strip_display_controls(text);
    let (parts, format_controls) = split_format_controls(&without_display_controls);
    NormalizedText {
        clean: parts.concat(),
        parts,
        format_controls,
        ruby_removed,
        newline_removed,
    }
}

fn strip_display_controls(text: &str) -> (String, usize, usize) {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    let mut ruby_removed = 0usize;
    let mut newline_removed = 0usize;
    while cursor < chars.len() {
        if chars[cursor] == '\\' && chars.get(cursor + 1) == Some(&'n') {
            cursor += 2;
            newline_removed += 1;
            continue;
        }
        if chars[cursor] == '＼' {
            if chars.get(cursor + 1) == Some(&'ｎ') {
                cursor += 2;
                newline_removed += 1;
                continue;
            }
            if chars.get(cursor + 1) == Some(&'＼') && chars.get(cursor + 2) == Some(&'ｎ') {
                cursor += 3;
                newline_removed += 1;
                continue;
            }
        }
        let close = match chars[cursor] {
            '[' => Some(']'),
            '［' => Some('］'),
            _ => None,
        };
        if let Some(close) = close {
            if let Some(relative) = chars[cursor + 1..].iter().position(|&ch| ch == close) {
                cursor += relative + 2;
                ruby_removed += 1;
                continue;
            }
        }
        out.push(chars[cursor]);
        cursor += 1;
    }
    (out, ruby_removed, newline_removed)
}

fn split_format_controls(text: &str) -> (Vec<String>, Vec<String>) {
    let chars: Vec<char> = text.chars().collect();
    let mut parts = vec![String::new()];
    let mut controls = Vec::new();
    let mut cursor = 0usize;
    while cursor < chars.len() {
        let control = if chars[cursor] == '%' {
            match chars.get(cursor + 1) {
                Some('I') => Some("%I"),
                Some('B') => Some("%B"),
                Some('F') => Some("%F"),
                _ => None,
            }
        } else if chars[cursor] == '％' {
            match chars.get(cursor + 1) {
                Some('Ｉ') | Some('I') => Some("%I"),
                Some('Ｂ') | Some('B') => Some("%B"),
                Some('Ｆ') | Some('F') => Some("%F"),
                _ => None,
            }
        } else {
            None
        };
        if let Some(control) = control {
            controls.push(control.to_string());
            parts.push(String::new());
            cursor += 2;
        } else {
            parts
                .last_mut()
                .expect("parts is never empty")
                .push(chars[cursor]);
            cursor += 1;
        }
    }
    (parts, controls)
}

pub fn render_parts(parts: &[String], controls: &[String]) -> Result<String> {
    if parts.len() != controls.len() + 1 {
        return Err(format!(
            "message_parts has {} item(s), expected {} for {} hidden format control(s)",
            parts.len(),
            controls.len() + 1,
            controls.len()
        ));
    }
    let mut out = String::new();
    for (index, part) in parts.iter().enumerate() {
        out.push_str(part);
        if let Some(control) = controls.get(index) {
            out.push_str(control);
        }
    }
    Ok(out)
}

pub fn validate_translated_text(text: &str) -> Result<()> {
    let normalized = normalize_text(text);
    if normalized.ruby_removed != 0 || normalized.newline_removed != 0 {
        return Err(
            "translated text contains a ruby or literal newline marker; remove it from JSON"
                .to_string(),
        );
    }
    if !normalized.format_controls.is_empty() {
        return Err(
            "translated text contains a format placeholder; edit message_parts instead".to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ruby_and_literal_newline() {
        let value = normalize_text("前[よみ]本\\n後");
        assert_eq!(value.clean, "前本後");
        assert_eq!(value.ruby_removed, 1);
        assert_eq!(value.newline_removed, 1);
    }

    #[test]
    fn hides_and_renders_placeholders() {
        let value = normalize_text("%B枚中％Ｆ枚");
        assert_eq!(value.clean, "枚中枚");
        assert_eq!(value.parts, ["", "枚中", "枚"]);
        assert_eq!(value.format_controls, ["%B", "%F"]);
        assert_eq!(
            render_parts(&value.parts, &value.format_controls).unwrap(),
            "%B枚中%F枚"
        );
    }
}
