use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlParts {
    pub prefix: String,
    pub body: String,
    pub suffix: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoiceMark {
    pub group: usize,
    pub index: usize,
}

pub fn split_boundaries(value: &str) -> ControlParts {
    let mut prefix_end = 0;
    while let Some(end) = consume_control(&value[prefix_end..]) {
        prefix_end += end;
    }

    let mut body_end = value.len();
    loop {
        let Some(start) = value[..body_end].rfind('@') else {
            break;
        };
        let candidate = &value[start..body_end];
        if is_short_control(candidate) || is_name_macro(candidate) {
            body_end = start;
        } else {
            break;
        }
    }

    if body_end < prefix_end {
        body_end = prefix_end;
    }
    ControlParts {
        prefix: value[..prefix_end].to_string(),
        body: value[prefix_end..body_end].to_string(),
        suffix: value[body_end..].to_string(),
    }
}

pub fn control_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut offset = 0;
    while offset < value.len() {
        let slice = &value[offset..];
        if let Some(length) = consume_control(slice) {
            tokens.push(slice[..length].to_string());
            offset += length;
        } else {
            let length = slice.chars().next().map(char::len_utf8).unwrap_or(1);
            offset += length;
        }
    }
    tokens
}

pub fn has_japanese(value: &str) -> bool {
    value.chars().any(|character| {
        matches!(
            character as u32,
            0x3040..=0x30ff
                | 0x3400..=0x4dbf
                | 0x4e00..=0x9fff
                | 0xf900..=0xfaff
        )
    })
}

pub fn is_pure_control(value: &str) -> bool {
    !value.is_empty() && split_boundaries(value).body.is_empty()
}

pub fn is_resource_name(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let extensions = [
        ".png", ".jpg", ".jpeg", ".webp", ".wmv", ".spm", ".bin", ".dat", ".ogg", ".wav", ".mp3",
        ".mp4", ".avi",
    ];
    if extensions
        .iter()
        .any(|extension| lower.ends_with(extension))
    {
        return true;
    }
    if has_japanese(value) || value.contains(' ') || value.contains('「') || value.contains('」')
    {
        return false;
    }
    let prefixes = [
        "title",
        "door",
        "walk",
        "move",
        "pickup",
        "ban",
        "ton",
        "pon",
        "sepia",
        "chime",
        "menu",
        "lime",
        "evcg",
        "siromoya",
        "system",
        "eventgroup",
        "button",
        "water",
        "camera",
        "nuno",
        "binta",
        "cfa",
        "cs",
        "sound",
        "bgm",
        "se",
    ];
    prefixes.iter().any(|prefix| lower.starts_with(prefix))
}

pub fn is_name_candidate(value: &str) -> bool {
    if value.is_empty()
        || !has_japanese(value)
        || is_resource_name(value)
        || is_pure_control(value)
        || value.len() > 48
    {
        return false;
    }
    if value
        .chars()
        .any(|character| "@「」『』！？。．、,，…()（）\r\n".contains(character))
    {
        return false;
    }
    !is_system_text(value)
}

pub fn is_system_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    matches!(
        name,
        "__global.bin"
            | "system.bin"
            | "title.bin"
            | "eventmode.bin"
            | "replaymode.bin"
            | "method.bin"
            | "effecticon.bin"
            | "scriptjump.bin"
            | "test_act.bin"
            | "コンスタグラムテスト.bin"
            | "全bgm開放スクリプト.bin"
            | "全cg開放スクリプト.bin"
            | "機能チュートリアル.bin"
    )
}

pub fn is_system_text(value: &str) -> bool {
    matches!(
        value,
        "最初に戻りました"
            | "ここで体験版が終了した場合"
            | "イベントの最初に戻りました"
            | "イベントモード中は使用できません"
            | "回想の最初に戻りました"
            | "回想モード中は使用できません"
            | "現在は使用できません"
            | "タイトル画面ではセーブできません"
            | "イベント開始時に停止しました"
            | "設定が完了しました。ウィンドウを閉じます。"
    )
}

pub fn choice_marks(values: &[String]) -> HashMap<usize, ChoiceMark> {
    let mut marks = HashMap::new();
    let mut group = 0;
    let mut index = 0;
    while index < values.len() {
        if is_choice_value(&values[index]) {
            let start = index;
            while index < values.len() && is_choice_value(&values[index]) {
                index += 1;
            }
            if index - start >= 2 {
                for (choice_index, slot) in (start..index).enumerate() {
                    marks.insert(
                        slot,
                        ChoiceMark {
                            group,
                            index: choice_index,
                        },
                    );
                }
                group += 1;
            }
        } else {
            index += 1;
        }
    }
    marks
}

pub fn is_choice_value(value: &str) -> bool {
    value.starts_with('『') && value.ends_with('』') && value.len() >= 6
}

pub fn validate_plain_text(value: &str, field: &str) -> Result<(), String> {
    if value.contains('\0') {
        return Err(format!("{field} contains NUL"));
    }
    if value.contains('\r') || value.contains('\n') {
        return Err(format!("{field} contains a physical newline"));
    }
    Ok(())
}

pub fn validate_body_change(_original: &str, replacement: &str) -> Result<(), String> {
    validate_plain_text(replacement, "message")
}

pub fn validate_choice_change(replacement: &str) -> Result<(), String> {
    if !is_choice_value(replacement) {
        return Err("choice message must retain 『...』 delimiters".to_string());
    }
    Ok(())
}

fn consume_control(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'@' {
        return None;
    }
    if bytes[1] == b'*' {
        let end = value.find("@*@")? + 3;
        return Some(end);
    }
    let code = bytes[1] as char;
    if code == 'v' || code == 'h' {
        let mut end = 2;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            end += 1;
        }
        return Some(end);
    }
    if code == 't' {
        let mut end = 2;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        return Some(end);
    }
    if bytes[1].is_ascii_alphabetic() {
        return Some(2);
    }
    None
}

fn is_short_control(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 2 && bytes[0] == b'@' && bytes[1].is_ascii_alphabetic()
}

fn is_name_macro(value: &str) -> bool {
    value.starts_with("@*name@") && value.ends_with("@*@")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_controls_are_separated() {
        let parts = split_boundaries("@v09020004「本文@n行」@k");
        assert_eq!(parts.prefix, "@v09020004");
        assert_eq!(parts.body, "「本文@n行」");
        assert_eq!(parts.suffix, "@k");
    }

    #[test]
    fn scene_controls_are_separated() {
        let parts = split_boundaries("@t0300@hKOGA_A0101BS@n本文」");
        assert_eq!(parts.prefix, "@t0300@hKOGA_A0101BS@n");
        assert_eq!(parts.body, "本文」");
    }

    #[test]
    fn choice_runs_are_marked() {
        let values = vec!["『A』".to_string(), "『B』".to_string(), "普通".to_string()];
        let marks = choice_marks(&values);
        assert_eq!(marks.get(&0).map(|mark| mark.index), Some(0));
        assert_eq!(marks.get(&1).map(|mark| mark.index), Some(1));
        assert!(!marks.contains_key(&2));
    }
}
