use std::{env, fs, path::Path};
use yox_adv_tool::{
    font::YoxFont,
    renderer::{line_preview_bmp, render_samples, samples_bmp},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 3 {
        return Err("usage: render_preview <inner-font.yox> <output.bmp>".into());
    }
    let font = YoxFont::parse(fs::read(&args[1])?).map_err(std::io::Error::other)?;
    let labels: Vec<char> = "国中文gjpq，。、.,（）「」\"0123ABC".chars().collect();
    let samples = render_samples(&font, &labels).map_err(std::io::Error::other)?;
    fs::write(
        &args[2],
        samples_bmp(font.width, font.height, &samples.glyphs).map_err(std::io::Error::other)?,
    )?;
    let line: Vec<char> = "中文，下一句。ABC,def.gjpq".chars().collect();
    let line_samples = render_samples(&font, &line).map_err(std::io::Error::other)?;
    let line_path =
        Path::new(&args[2]).with_file_name(format!("font-line-{}x{}.bmp", font.width, font.height));
    fs::write(
        &line_path,
        line_preview_bmp(
            font.width,
            font.height,
            &line_samples.glyphs,
            &line_samples.advances,
        )
        .map_err(std::io::Error::other)?,
    )?;
    let meta = serde_json::json!({"font_face": samples.face, "cell": [font.width, font.height], "characters": labels.iter().collect::<String>(), "advances_px": samples.advances, "layout": "8 columns; shared font-metric baseline; punctuation uses its measured advance and bottom-right ink anchor", "line_preview": line_path.file_name().unwrap().to_string_lossy(), "line_text": line.iter().collect::<String>(), "line_font_face": line_samples.face, "line_advances_px": line_samples.advances});
    let meta_path = Path::new(&args[2]).with_extension("json");
    fs::write(meta_path, serde_json::to_vec_pretty(&meta)?)?;
    Ok(())
}
