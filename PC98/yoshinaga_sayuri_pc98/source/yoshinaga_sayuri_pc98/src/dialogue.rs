use crate::Result;
use serde::Serialize;
use vn_text::Entry;

// Confirmed first-line speaker labels in the original TDH, with padding removed.
const SPEAKERS: &[&str] = &[
    "吉永サユリ",
    "片平なぎさ",
    "スージー・Ｎ",
    "赤貝ヌレ子",
    "三浦みさき",
    "渦巻なると",
    "貝柱帆立",
    "岸壁美潮",
    "美樹本ぱある",
    "磯辺真樹",
    "蘇我野いるか",
    "マルダシ先生",
    "テマキ・Ｎ",
    "用務員",
    "海底牌",
    "韮井カナイ",
    "龍野としこ",
    "電気ウナーギ",
    "Ｓ・マリーネ",
    "シーラ・Ｋ",
    "浜千鳥",
    "海原おはま",
    "トリトン海野",
    "美樹本心の声",
    "西表くまのみ",
    "無修整ヘア子",
    "マリーネ・Ｓ",
    "触手くん",
    "イルカ君３号",
    "イルカ君４号",
    "吉永心の声",
    "全員",
    "熊",
    "生徒全員",
    "三浦＆赤貝",
    "観客",
    "無修正ヘア子",
    "イルカ君軍団",
    "謎の音",
    "吉永さゆり",
    "観客＆吉永",
    "怪しい人物",
    "大観客",
    "ＤＯＮ",
    "あぶく",
    "イルカ君２号",
];

#[derive(Clone, Serialize)]
pub struct Layout {
    pub name_prefix: String,
    pub name_suffix: String,
    pub line_prefixes: Vec<String>,
}

fn padding(ch: char) -> bool {
    ch == ' ' || ch == '　'
}

pub fn project(source: &str) -> (Entry, Option<Layout>) {
    let plain = || {
        (
            Entry {
                name: None,
                message: source.into(),
            },
            None,
        )
    };
    let Some((label, body)) = source.split_once('：') else {
        return plain();
    };
    if label.contains('\n') {
        return plain();
    }
    let identity = label.chars().filter(|&ch| !padding(ch)).collect::<String>();
    if !SPEAKERS.contains(&identity.as_str()) {
        return plain();
    }
    let name = label.trim_matches(padding);
    let prefix = label.len() - label.trim_start_matches(padding).len();
    let mut line_prefixes = Vec::new();
    let mut lines = Vec::new();
    for line in body.split('\n') {
        let clean = line.trim_start_matches(padding);
        line_prefixes.push(line[..line.len() - clean.len()].to_string());
        lines.push(clean);
    }
    (
        Entry {
            name: Some(name.to_string()),
            message: lines.join("\n"),
        },
        Some(Layout {
            name_prefix: label[..prefix].to_string(),
            name_suffix: label[prefix + name.len()..].to_string(),
            line_prefixes,
        }),
    )
}

pub fn merge(entry: &Entry, layout: Option<&Layout>) -> Result<String> {
    match (&entry.name, layout) {
        (None, None) => Ok(entry.message.clone()),
        (Some(name), Some(layout)) => {
            if name.chars().any(char::is_control) {
                return Err("name must be a single line without control characters".into());
            }
            let mut native = format!("{}{}{}：", layout.name_prefix, name, layout.name_suffix);
            for (index, line) in entry.message.split('\n').enumerate() {
                if index != 0 {
                    native.push('\n');
                }
                let prefix = layout
                    .line_prefixes
                    .get(index)
                    .or_else(|| layout.line_prefixes.get(1));
                if let Some(prefix) = prefix {
                    native.push_str(prefix);
                }
                native.push_str(line);
            }
            Ok(native)
        }
        _ => Err("name-field presence does not match the original dialogue layout".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn separates_speaker_and_restores_original_layout() {
        let source = "　用　務　員　：今朝は。\n　　　疲れた。\n　　　　ああ。";
        let (entry, layout) = project(source);
        assert_eq!(entry.name.as_deref(), Some("用　務　員"));
        assert_eq!(entry.message, "今朝は。\n疲れた。\nああ。");
        assert_eq!(merge(&entry, layout.as_ref()).unwrap(), source);
        let edited = Entry {
            name: Some("管理人".into()),
            message: "你好。\n这句变长。\n第三行。\n新增行。".into(),
        };
        assert_eq!(
            merge(&edited, layout.as_ref()).unwrap(),
            "　管理人　：你好。\n　　　这句变长。\n　　　　第三行。\n　　　新增行。"
        );
        assert!(merge(
            &Entry {
                name: Some("A\nB".into()),
                message: String::new()
            },
            layout.as_ref()
        )
        .is_err());
    }
    #[test]
    fn preserves_unnamed_text_and_unconfirmed_colons() {
        for source in [
            "普通正文",
            "１２：３０",
            "備考：本文",
            "\n三浦　みさき：本文",
            "第一行\n　　　续文",
        ] {
            let (entry, layout) = project(source);
            assert!(entry.name.is_none());
            assert_eq!(merge(&entry, layout.as_ref()).unwrap(), source);
        }
        let (entry, layout) = project("三浦　みさき：今朝は。\n　　　疲れた。");
        assert_eq!(entry.name.as_deref(), Some("三浦　みさき"));
        assert_eq!(entry.message, "今朝は。\n疲れた。");
        assert_eq!(
            merge(&entry, layout.as_ref()).unwrap(),
            "三浦　みさき：今朝は。\n　　　疲れた。"
        );
    }
}
