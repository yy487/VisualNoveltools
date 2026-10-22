use crate::{codec::Record, Result};
use encoding_rs::SHIFT_JIS;

pub const SLOT_SIZES: [usize; 3] = [8, 12, 14];

pub fn records(bytes: &[u8], count: usize, width: usize) -> Result<Vec<Record>> {
    if !SLOT_SIZES.contains(&width) {
        return Err(format!("不支持 {width} 字节的 XOR 固定槽"));
    }
    let expected = count.checked_mul(width).ok_or("词典长度溢出")?;
    if bytes.len() != expected {
        return Err(format!(
            "词典大小应为 {expected} 字节，实际 {}",
            bytes.len()
        ));
    }
    bytes
        .chunks_exact(width)
        .enumerate()
        .map(|(index, raw)| {
            let decoded: Vec<_> = raw.iter().map(|b| b ^ 0xff).collect();
            let end = decoded
                .iter()
                .rposition(|b| *b != b' ')
                .map_or(0, |p| p + 1);
            let mut message = SHIFT_JIS
                .decode_without_bom_handling_and_without_replacement(&decoded[..end])
                .ok_or_else(|| format!("词典槽 {index} 含无效 CP932"))?
                .into_owned();
            if width == 8 {
                message = message.trim_end().to_owned();
            }
            validate_source(&message).map_err(|e| format!("词典槽 {index}: {e}"))?;
            Ok(Record {
                offset: index * width,
                raw: raw.to_vec(),
                message,
                expression_lines: Vec::new(),
            })
        })
        .collect()
}

fn validate_source(text: &str) -> Result<()> {
    if text.chars().any(char::is_control) {
        return Err("词条含控制字符".into());
    }
    Ok(())
}

pub fn validate_translation(text: &str) -> Result<()> {
    validate_source(text)?;
    if text.chars().any(|ch| matches!(ch, '<' | '>' | '%' | '$')) {
        return Err("词条不能含标记或格式指令字符".into());
    }
    Ok(())
}

pub fn rebuild(
    record: &Record,
    translated: &str,
    plan: &vn_font::font_98::EncodingPlan,
    width: usize,
) -> Result<Vec<u8>> {
    if !SLOT_SIZES.contains(&width) {
        return Err(format!("不支持 {width} 字节的 XOR 固定槽"));
    }
    if record.raw.len() != width {
        return Err(format!(
            "词典槽原始长度应为 {width} 字节，实际 {}",
            record.raw.len()
        ));
    }
    if record.offset == 0 && record.message.is_empty() && !translated.is_empty() {
        return Err("词典 id 0 是空槽，不能填充".into());
    }
    if translated == record.message {
        return Ok(record.raw.clone());
    }
    validate_translation(translated)?;
    let encoded = crate::codec::encode(translated, plan)?;
    if encoded.len() > width {
        return Err(format!(
            "词条占 {} 字节，{width} 字节槽位无法容纳",
            encoded.len(),
        ));
    }
    let mut decoded = encoded;
    decoded.resize(width, b' ');
    decoded.iter_mut().for_each(|byte| *byte ^= 0xff);
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vn_font::font_98::{EncodingPlan, SubstitutionMap};

    fn plan(text: &str) -> EncodingPlan {
        let display = text
            .chars()
            .filter(|character| {
                crate::codec::font_text(&character.to_string())
                    .is_ok_and(|display| !display.is_empty())
            })
            .collect::<String>();
        EncodingPlan::build(
            &SubstitutionMap::embedded().unwrap(),
            [],
            [display.as_str()],
        )
        .unwrap()
    }

    #[test]
    fn xor14_capacity_padding_and_controls_are_enforced() {
        let raw: Vec<_> = b"abc           ".iter().map(|byte| byte ^ 0xff).collect();
        let record = records(&raw, 1, 14).unwrap().remove(0);
        assert_eq!(record.message, "abc");
        let rebuilt = rebuild(&record, "xy", &plan("xy"), 14).unwrap();
        let decoded: Vec<_> = rebuilt.iter().map(|byte| byte ^ 0xff).collect();
        assert_eq!(decoded, b"xy            ");
        let leading = rebuild(&record, "  xy", &plan("  xy"), 14).unwrap();
        assert_eq!(
            leading.iter().map(|byte| byte ^ 0xff).collect::<Vec<_>>(),
            b"  xy          "
        );
        assert!(rebuild(&record, "123456789012345", &plan("123456789012345"), 14).is_err());
        assert!(validate_translation("a\tb").is_err());

        let empty = records(&[0xdf; 14], 1, 14).unwrap().remove(0);
        assert!(empty.message.is_empty());
        assert!(rebuild(&empty, "filled", &plan("filled"), 14).is_err());
    }

    #[test]
    fn chinese_dictionary_text_uses_and_decodes_the_font_plan_carrier() {
        let plan = plan("蓝调");
        let record = records(&[0xdf; 28], 2, 14).unwrap().pop().unwrap();
        let encoded = rebuild(&record, "蓝调", &plan, 14).unwrap();
        let decoded: Vec<_> = encoded.iter().map(|byte| byte ^ 0xff).collect();
        let carriers = SHIFT_JIS
            .decode_without_bom_handling_and_without_replacement(&decoded[..4])
            .unwrap();
        assert_eq!(plan.decode_carriers(&carriers), "蓝调");
        assert_eq!(encoded.len(), 14);
    }

    #[test]
    fn eight_byte_jname_slots_trim_source_padding_and_fit_chinese_plan() {
        let (source, _, errors) = SHIFT_JIS.encode("オカベ　");
        assert!(!errors);
        assert_eq!(source.len(), 8);
        let raw: Vec<_> = source.iter().map(|byte| byte ^ 0xff).collect();
        let record = records(&raw, 1, 8).unwrap().remove(0);
        assert_eq!(record.message, "オカベ");
        assert_eq!(
            rebuild(&record, &record.message, &plan("オカベ"), 8).unwrap(),
            raw
        );

        let translated = records(&[0xdf; 16], 2, 8).unwrap().pop().unwrap();
        let chinese_plan = plan("蓝调");
        let encoded = rebuild(&translated, "蓝调", &chinese_plan, 8).unwrap();
        let decoded: Vec<_> = encoded.iter().map(|byte| byte ^ 0xff).collect();
        let carriers = SHIFT_JIS
            .decode_without_bom_handling_and_without_replacement(&decoded[..4])
            .unwrap();
        assert_eq!(chinese_plan.decode_carriers(&carriers), "蓝调");
        assert!(rebuild(&translated, "蓝调蓝调蓝", &plan("蓝调蓝调蓝"), 8).is_err());
    }

    #[test]
    fn twelve_byte_race_name_slots_fit_six_chinese_characters() {
        let translated = records(&[0xdf; 24], 2, 12).unwrap().pop().unwrap();
        let six = "蓝调蓝调蓝调";
        let encoded = rebuild(&translated, six, &plan(six), 12).unwrap();
        let decoded: Vec<_> = encoded.iter().map(|byte| byte ^ 0xff).collect();
        let carriers = SHIFT_JIS
            .decode_without_bom_handling_and_without_replacement(&decoded[..12])
            .unwrap();
        let six_plan = plan(six);
        assert_eq!(six_plan.decode_carriers(&carriers), six);
        assert!(rebuild(&translated, "蓝调蓝调蓝调蓝", &plan("蓝调蓝调蓝调蓝"), 12).is_err());
    }
}
