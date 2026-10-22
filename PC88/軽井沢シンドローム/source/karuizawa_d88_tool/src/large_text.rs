use crate::text_compression::TextDictionary;
use crate::Result;

pub fn compact_menu(message: &str, rows: usize) -> Result<String> {
    let normalized = message
        .trim()
        .trim_end_matches('.')
        .to_ascii_uppercase()
        .replace("SCREEN, MODE", "SCREEN MODE")
        .replace("GAME, SAVE", "GAME SAVE");
    let aliases = [
        ("ASK ABOUT SOMETHING", "ASK"),
        ("TALK TO SOMEONE", "TALK"),
        ("EXAMINE SOMETHING", "EXAMINE"),
        ("SHOW SOMETHING", "SHOW ITEM"),
        ("LOCATION: MOVE", "MOVE"),
        ("ASK SOMETHING", "ASK"),
        ("MOVE LOCATION", "MOVE"),
        ("ENTER INSIDE", "ENTER"),
        ("LOOK AROUND", "LOOK AREA"),
        ("CALL PERSON", "CALL OUT"),
        ("SCREEN MODE", "DISPLAY"),
        ("GAME SAVE", "SAVE GAME"),
        ("HOLD CLOSE", "EMBRACE"),
        ("PULL CLOSE", "EMBRACE"),
        ("HER: LOOK", "LOOK HER"),
        ("GO INSIDE", "ENTER"),
        ("INTERROGATE", "QUESTION"),
        ("INVESTIGATE", "EXAMINE"),
        ("TAKE SOMETHING", "TAKE ITEM"),
        ("TAKE ITEM", "TAKE ITEM"),
        ("SHOW ITEM", "SHOW ITEM"),
        ("ROUGH UP", "ROUGH UP"),
        ("PUSH HIT", "PUSH/HIT"),
        ("REASON", "DEDUCE"),
    ];
    let mut labels = Vec::new();
    for separator in ["  ", "|", ","] {
        let parts = normalized
            .split(separator)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>();
        if parts.len() == rows {
            labels = parts.into_iter().map(str::to_owned).collect();
            break;
        }
    }
    if labels.is_empty() {
        let mut remaining = normalized.as_str();
        while !remaining.is_empty() {
            let end = aliases
                .iter()
                .filter_map(|(phrase, _)| {
                    remaining
                        .strip_prefix(phrase)
                        .filter(|tail| tail.is_empty() || tail.starts_with(' '))
                        .map(|_| phrase.len())
                })
                .max()
                .unwrap_or_else(|| remaining.find(' ').unwrap_or(remaining.len()));
            labels.push(remaining[..end].to_owned());
            remaining = remaining[end..].trim_start();
        }
    }
    if labels.len() != rows {
        return Err(format!(
            "大字菜单需要 {rows} 个双空格分隔标签，当前识别为 {} 个：{message}",
            labels.len()
        ));
    }
    for label in &mut labels {
        if let Some((_, short)) = aliases.iter().find(|(phrase, _)| *phrase == label) {
            *label = (*short).into();
        }
        if label.len() > 9 {
            return Err(format!("大字菜单标签超过 9 字符，请缩短：{label}"));
        }
    }
    Ok(labels.join("  "))
}

pub fn wrap_literal(
    raw: &[u8],
    initial_column: Option<usize>,
    dictionary: &TextDictionary,
) -> Result<(Vec<u8>, usize)> {
    let mut column = initial_column;
    let mut wrapped = Vec::new();
    let mut offset = 0;
    while offset < raw.len() {
        let byte = raw[offset];
        if byte == b'+' {
            column = Some(0);
            wrapped.push(byte);
            offset += 1;
        } else if byte == b' ' {
            if column.is_some_and(|column| column >= 40) {
                wrapped.push(b'+');
                column = Some(0);
            } else {
                wrapped.push(byte);
                column = column.map(|value| value + 1);
            }
            offset += 1;
        } else if is_glyph(byte) {
            let end = (offset..raw.len())
                .find(|index| !is_glyph(raw[*index]))
                .unwrap_or(raw.len());
            let word = &raw[offset..end];
            if word.len() > 40 {
                return Err("大字正文包含超过 40 字符的连续单词".into());
            }
            if column.is_none_or(|value| value + word.len() > 40) {
                wrapped.push(b'+');
                column = Some(0);
            }
            wrapped.extend_from_slice(word);
            column = column.map(|value| value + word.len());
            offset = end;
        } else {
            wrapped.push(byte);
            offset += 1;
        }
    }
    let mut output = Vec::new();
    let mut text = String::new();
    for byte in wrapped {
        if (0x86..=0xE3).contains(&byte) {
            text.push(char::from(byte - 0x86 + b'!'));
        } else if byte == b' ' {
            text.push(' ');
        } else {
            output.extend(dictionary.encode_text(&text)?);
            text.clear();
            output.push(byte);
        }
    }
    output.extend(dictionary.encode_text(&text)?);
    Ok((output, column.unwrap_or(0)))
}

fn is_glyph(byte: u8) -> bool {
    (0x86..=0xFE).contains(&byte) || byte.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_labels_keep_their_order_and_fit() {
        let text = "HER: LOOK  TALK  EMBRACE  KISS  ASK SOMETHING  GO INSIDE  LOOK AROUND  CALL";
        assert_eq!(
            compact_menu(text, 8).unwrap(),
            "LOOK HER  TALK  EMBRACE  KISS  ASK  ENTER  LOOK AREA  CALL"
        );
        assert_eq!(
            compact_menu("Move | Look Around | Investigate", 3).unwrap(),
            "MOVE  LOOK AREA  EXAMINE"
        );
        assert_eq!(
            compact_menu(
                "Pull Close Push Hit Rough Up Sound Screen Mode Game Save Other",
                7
            )
            .unwrap(),
            "EMBRACE  PUSH/HIT  ROUGH UP  SOUND  DISPLAY  SAVE GAME  OTHER"
        );
    }

    #[test]
    fn wrapping_accounts_for_a_name_prefix() {
        let dictionary = TextDictionary::default();
        let raw = dictionary
            .encode_text("I was speeding toward the villa")
            .unwrap();
        let (wrapped, column) = wrap_literal(&raw, Some(26), &dictionary).unwrap();
        assert!(wrapped.contains(&b'+'));
        assert!(column <= 40);
        let visible = |bytes: &[u8]| {
            bytes
                .iter()
                .copied()
                .filter(|byte| !matches!(*byte, b' ' | b'+'))
                .collect::<Vec<_>>()
        };
        assert_eq!(visible(&wrapped), visible(&raw));
    }
}
