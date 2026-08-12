use std::collections::{BTreeMap, BTreeSet};

use crate::ToolResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMarkup {
    pub plain: String,
    pub links: BTreeMap<u32, (usize, usize)>,
}

pub fn parse_link_markup(input: &str, context: &str) -> ToolResult<ParsedMarkup> {
    let mut plain = String::with_capacity(input.len());
    let mut links = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut open: Vec<(u32, usize)> = Vec::new();
    let mut cursor = 0usize;
    let mut char_pos = 0usize;

    while cursor < input.len() {
        let rest = &input[cursor..];
        if rest.starts_with("[[link:") {
            let end = rest
                .find("]]")
                .ok_or_else(|| format!("{context}: unterminated hyperlink opening tag"))?;
            let id_text = &rest[7..end];
            if id_text.is_empty() || !id_text.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(format!("{context}: invalid hyperlink id {id_text:?}"));
            }
            let id = id_text
                .parse::<u32>()
                .map_err(|_| format!("{context}: hyperlink id is too large"))?;
            if !seen.insert(id) {
                return Err(format!("{context}: duplicate hyperlink id {id}"));
            }
            open.push((id, char_pos));
            cursor += end + 2;
            continue;
        }
        if rest.starts_with("[[/link]]") {
            let (id, start) = open
                .pop()
                .ok_or_else(|| format!("{context}: hyperlink closing tag has no opener"))?;
            let len = char_pos - start;
            if len == 0 {
                return Err(format!("{context}: hyperlink {id} has an empty range"));
            }
            links.insert(id, (start, len));
            cursor += "[[/link]]".len();
            continue;
        }
        let ch = rest
            .chars()
            .next()
            .ok_or_else(|| format!("{context}: invalid UTF-8 cursor"))?;
        plain.push(ch);
        char_pos += 1;
        cursor += ch.len_utf8();
    }
    if let Some((id, _)) = open.last() {
        return Err(format!("{context}: hyperlink {id} is not closed"));
    }
    Ok(ParsedMarkup { plain, links })
}

pub fn insert_link_markup(
    plain: &str,
    spans: &[(u32, usize, usize)],
    context: &str,
) -> ToolResult<String> {
    let char_count = plain.chars().count();
    let mut normalized = Vec::with_capacity(spans.len());
    let mut ids = BTreeSet::new();
    for &(id, start, len) in spans {
        let end = start
            .checked_add(len)
            .ok_or_else(|| format!("{context}: hyperlink {id} range overflow"))?;
        if len == 0 || end > char_count {
            return Err(format!(
                "{context}: hyperlink {id} range {start}..{end} exceeds {char_count} characters"
            ));
        }
        if !ids.insert(id) {
            return Err(format!("{context}: duplicate hyperlink id {id}"));
        }
        normalized.push((id, start, end));
    }
    for (index, &(_, a_start, a_end)) in normalized.iter().enumerate() {
        for &(_, b_start, b_end) in &normalized[index + 1..] {
            let overlaps = a_start < b_end && b_start < a_end;
            let nested =
                (a_start <= b_start && b_end <= a_end) || (b_start <= a_start && a_end <= b_end);
            if overlaps && !nested {
                return Err(format!(
                    "{context}: crossing hyperlink ranges {a_start}..{a_end} and {b_start}..{b_end} cannot be represented safely"
                ));
            }
        }
    }

    let mut opens: BTreeMap<usize, Vec<(u32, usize)>> = BTreeMap::new();
    let mut closes: BTreeMap<usize, Vec<(u32, usize)>> = BTreeMap::new();
    for &(id, start, end) in &normalized {
        opens.entry(start).or_default().push((id, end));
        closes.entry(end).or_default().push((id, start));
    }
    for values in opens.values_mut() {
        values.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    }
    for values in closes.values_mut() {
        values.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));
    }

    let mut out = String::with_capacity(plain.len() + spans.len() * 24);
    for (pos, ch) in plain.chars().enumerate() {
        if let Some(ids) = closes.get(&pos) {
            for _ in ids {
                out.push_str("[[/link]]");
            }
        }
        if let Some(ids) = opens.get(&pos) {
            for (id, _) in ids {
                out.push_str(&format!("[[link:{id}]]"));
            }
        }
        out.push(ch);
    }
    if let Some(ids) = closes.get(&char_count) {
        for _ in ids {
            out.push_str("[[/link]]");
        }
    }
    Ok(out)
}

pub fn absolute_to_line_col(
    text: &str,
    start: usize,
    len: usize,
    context: &str,
) -> ToolResult<(u16, u16, u16)> {
    let chars: Vec<char> = text.chars().collect();
    let end = start
        .checked_add(len)
        .ok_or_else(|| format!("{context}: range overflow"))?;
    if len == 0 || end > chars.len() {
        return Err(format!("{context}: range exceeds translated text"));
    }
    if chars[start..end].contains(&'\n') {
        return Err(format!(
            "{context}: a hyperlink cannot cross a hard newline"
        ));
    }
    let mut line = 1usize;
    let mut col = 1usize;
    for ch in &chars[..start] {
        if *ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    Ok((
        u16::try_from(line).map_err(|_| format!("{context}: line number exceeds u16"))?,
        u16::try_from(col).map_err(|_| format!("{context}: column exceeds u16"))?,
        u16::try_from(len).map_err(|_| format!("{context}: link length exceeds u16"))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markup_roundtrip() {
        let source = "abc交換def";
        let marked = insert_link_markup(source, &[(0, 3, 2)], "test").unwrap();
        assert_eq!(marked, "abc[[link:0]]交換[[/link]]def");
        let parsed = parse_link_markup(&marked, "test").unwrap();
        assert_eq!(parsed.plain, source);
        assert_eq!(parsed.links.get(&0), Some(&(3, 2)));
    }

    #[test]
    fn identical_ranges_are_nested_and_roundtrip() {
        let source = "abc交換def";
        let marked = insert_link_markup(source, &[(0, 3, 2), (1, 3, 2)], "test").unwrap();
        assert_eq!(marked, "abc[[link:0]][[link:1]]交換[[/link]][[/link]]def");
        let parsed = parse_link_markup(&marked, "test").unwrap();
        assert_eq!(parsed.plain, source);
        assert_eq!(parsed.links.get(&0), Some(&(3, 2)));
        assert_eq!(parsed.links.get(&1), Some(&(3, 2)));
    }

    #[test]
    fn line_coordinates() {
        assert_eq!(
            absolute_to_line_col("甲乙\n丙丁", 3, 2, "test").unwrap(),
            (2, 1, 2)
        );
    }
}
