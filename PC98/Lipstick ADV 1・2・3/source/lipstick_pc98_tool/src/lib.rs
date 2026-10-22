pub mod disk;
pub mod diskx;
pub mod font;
pub mod mes;
pub mod native;
pub mod workflow;

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Game {
    Lipstick1,
    Lipstick2,
    Lipstick3,
}

impl Game {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "1" | "lipstick1" => Ok(Self::Lipstick1),
            "2" | "lipstick2" => Ok(Self::Lipstick2),
            "3" | "lipstick3" => Ok(Self::Lipstick3),
            _ => Err("game 必须是 lipstick1、lipstick2 或 lipstick3".into()),
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Lipstick1 => "lipstick1",
            Self::Lipstick2 => "lipstick2",
            Self::Lipstick3 => "lipstick3",
        }
    }
}

pub(crate) fn word(b: &[u8], p: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(p..p + 2)
            .ok_or("截断的 u16")?
            .try_into()
            .map_err(|_| "u16 长度")?,
    ))
}

pub(crate) fn dword(b: &[u8], p: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(p..p + 4)
            .ok_or("截断的 u32")?
            .try_into()
            .map_err(|_| "u32 长度")?,
    ))
}
