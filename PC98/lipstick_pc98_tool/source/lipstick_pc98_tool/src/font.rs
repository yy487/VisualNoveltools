use crate::{Game, Result, mes};
use std::collections::BTreeSet;
use vn_font::font_98::{self, EncodingPlan, EncodingPlanEntry, SubstitutionMap};

pub struct Font {
    game: Game,
    plan: EncodingPlan,
    pub bytes: Vec<u8>,
    pub mapping: Vec<EncodingPlanEntry>,
    pub patched_glyphs: usize,
    reserved: BTreeSet<u16>,
}

pub fn reserved_codes(bytes: &[u8]) -> BTreeSet<u16> {
    bytes
        .windows(2)
        .filter_map(|pair| {
            let text = mes::decode(pair).ok()?;
            let c = text.chars().next()?;
            if text.chars().count() == 1
                && font_98::has_loaded_np2_slot(c)
                && font_98::cp932_for_carrier(c).ok()?.as_slice() == pair
            {
                Some(u16::from_be_bytes([pair[0], pair[1]]))
            } else {
                None
            }
        })
        .collect()
}

fn custom_native(game: Game, c: char) -> Option<[u8; 2]> {
    let pair = font_98::cp932_for_carrier(c).ok()?;
    let code = u16::from_be_bytes(pair);
    (game == Game::Lipstick1 && (0x824f..0x83a0).contains(&code)).then_some(pair)
}

impl Font {
    pub fn prepare(
        game: Game,
        source_font: &[u8],
        reserved: &BTreeSet<u16>,
        texts: &[String],
        face: &str,
    ) -> Result<Self> {
        let requested = texts
            .iter()
            .map(|text| {
                text.chars()
                    .filter(|c| !c.is_ascii() && *c != '\n' && custom_native(game, *c).is_none())
                    .collect::<String>()
            })
            .collect::<Vec<_>>();
        let mut forbidden = mes::controls(game).iter().copied().collect::<BTreeSet<_>>();
        // Protect the entire in-engine KANJI.FNT route, including unassigned but
        // structurally addressable cells. Third-game CG rows 76/77 are already
        // excluded by font_98's NP2 loaded-page contract (pages 56/57).
        if game == Game::Lipstick1 {
            for code in 0x824fu16..0x83a0 {
                if font_98::cp932_to_jis(code.to_be_bytes()).is_ok() {
                    forbidden.insert(code);
                }
            }
        }
        forbidden.insert(0x8196); // NAME.DEF's player-name marker in Adv98V.
        let plan = EncodingPlan::build_with_forbidden_cp932(
            &SubstitutionMap::embedded()?,
            reserved.iter().copied(),
            forbidden,
            requested.iter().map(String::as_str),
        )?;
        let result = font_98::prepare_font(source_font, &plan.requests(), reserved, face)?;
        let mapping = plan.manifest_entries()?;
        Ok(Self {
            game,
            plan,
            bytes: result.bytes,
            patched_glyphs: result.patched_glyphs,
            mapping,
            reserved: reserved.clone(),
        })
    }

    pub fn encode(&self, text: &str, ascii: bool) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        for c in text.chars() {
            if ascii && c.is_ascii() && !c.is_control() {
                out.push(c as u8);
                continue;
            }
            let pair = if let Some(pair) = custom_native(self.game, c) {
                pair
            } else {
                let carrier = self.plan.carrier_for(c).or_else(|e| {
                    if font_98::cp932_for_carrier(c)
                        .is_ok_and(|p| self.reserved.contains(&u16::from_be_bytes(p)))
                    {
                        Ok(c)
                    } else {
                        Err(e)
                    }
                })?;
                font_98::cp932_for_carrier(carrier)?
            };
            let code = u16::from_be_bytes(pair);
            if !mes::is_lead(self.game, pair[0]) || mes::controls(self.game).contains(&code) {
                return Err(format!("字库载体 {code:04X} 与本代指令／显示控制冲突"));
            }
            if !ascii
                && self.game != Game::Lipstick1
                && pair[0] == 0x82
                && (0x9f..=0xf1).contains(&pair[1])
            {
                out.push(pair[1] - 0x72);
            } else {
                out.extend(pair);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_bank_is_not_a_pool_for_new_chinese() {
        assert_eq!(custom_native(Game::Lipstick1, 'あ'), Some([0x82, 0xa0]));
        assert_eq!(custom_native(Game::Lipstick2, 'あ'), None);
        assert!(!reserved_codes(&[0xfa, 0x4c]).contains(&0xfa4c));
    }
}
