use anyhow::{ensure, Context, Result};
use random_house_d88_tool::text::TextTranslation;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: make_text_probe <input.json> <output.json>")?,
    );
    let output = PathBuf::from(
        args.next()
            .context("usage: make_text_probe <input.json> <output.json>")?,
    );
    ensure!(
        args.next().is_none(),
        "usage: make_text_probe <input.json> <output.json>"
    );

    let bytes = fs::read(&input).with_context(|| format!("failed to read {}", input.display()))?;
    let mut translation: TextTranslation = serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid translation JSON {}", input.display()))?;
    let (index, file, line, message) = {
        let entry = translation
            .entries
            .iter_mut()
            .find(|entry| entry.file == "TITL" && entry.line == 270 && entry.index == 0)
            .context("TITL line 270 probe entry was not found")?;
        ensure!(
            entry.scr_msg == " NEXT ･･･ ｽﾍﾟｰｽ ｷｰ",
            "probe source text changed"
        );
        entry.message = "这是一句完整的汉化测试。".to_owned();
        (
            entry.index,
            entry.file.clone(),
            entry.line,
            entry.message.clone(),
        )
    };

    let json = serde_json::to_vec_pretty(&translation)?;
    fs::write(&output, json).with_context(|| format!("failed to write {}", output.display()))?;
    println!(
        "TEXT_PROBE_JSON pass=yes index={} file={} line={} message={}",
        index, file, line, message
    );
    Ok(())
}
