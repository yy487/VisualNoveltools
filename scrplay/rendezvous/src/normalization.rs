#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextPolicy {
    FixOrig,
    Raw,
}

impl TextPolicy {
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "fixOrig" => Some(Self::FixOrig),
            "raw" => Some(Self::Raw),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::FixOrig => "fixOrig",
            Self::Raw => "raw",
        }
    }

    pub fn apply(self, text: &str) -> String {
        match self {
            Self::FixOrig => normalize_fix_orig(text),
            Self::Raw => text.to_owned(),
        }
    }
}

pub fn normalize_fix_orig(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '!' => '！',
            '?' => '？',
            '\u{f8f0}' => ' ',
            '｡' => '。',
            '｢' => '「',
            '｣' => '」',
            '､' => '、',
            '･' => '…',
            'ｦ' => 'を',
            'ｧ' => 'ぁ',
            'ｨ' => 'ぃ',
            'ｩ' => 'ぅ',
            'ｪ' => 'ぇ',
            'ｫ' => 'ぉ',
            'ｬ' => 'ゃ',
            'ｭ' => 'ゅ',
            'ｮ' => 'ょ',
            'ｯ' => 'っ',
            'ｰ' => 'ー',
            'ｱ' => 'あ',
            'ｲ' => 'い',
            'ｳ' => 'う',
            'ｴ' => 'え',
            'ｵ' => 'お',
            'ｶ' => 'か',
            'ｷ' => 'き',
            'ｸ' => 'く',
            'ｹ' => 'け',
            'ｺ' => 'こ',
            'ｻ' => 'さ',
            'ｼ' => 'し',
            'ｽ' => 'す',
            'ｾ' => 'せ',
            'ｿ' => 'そ',
            'ﾀ' => 'た',
            'ﾁ' => 'ち',
            'ﾂ' => 'つ',
            'ﾃ' => 'て',
            'ﾄ' => 'と',
            'ﾅ' => 'な',
            'ﾆ' => 'に',
            'ﾇ' => 'ぬ',
            'ﾈ' => 'ね',
            'ﾉ' => 'の',
            'ﾊ' => 'は',
            'ﾋ' => 'ひ',
            'ﾌ' => 'ふ',
            'ﾍ' => 'へ',
            'ﾎ' => 'ほ',
            'ﾏ' => 'ま',
            'ﾐ' => 'み',
            'ﾑ' => 'む',
            'ﾒ' => 'め',
            'ﾓ' => 'も',
            'ﾔ' => 'や',
            'ﾕ' => 'ゆ',
            'ﾖ' => 'よ',
            'ﾗ' => 'ら',
            'ﾘ' => 'り',
            'ﾙ' => 'る',
            'ﾚ' => 'れ',
            'ﾛ' => 'ろ',
            'ﾜ' => 'わ',
            'ﾝ' => 'ん',
            'ﾞ' => '゛',
            'ﾟ' => '゜',
            _ => character,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_fix_orig_character_table() {
        assert_eq!(normalize_fix_orig("｢ﾕｶﾘ!?･ﾞﾟ｣"), "「ゆかり！？…゛゜」");
    }

    #[test]
    fn raw_policy_preserves_text() {
        assert_eq!(TextPolicy::Raw.apply("ｱ!"), "ｱ!");
    }
}
