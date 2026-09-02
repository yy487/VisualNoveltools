use encoding_rs::{GBK, SHIFT_JIS};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingChoice {
    Sjis,
    Gbk,
}

impl EncodingChoice {
    pub fn parse(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "sjis" | "shift-jis" | "cp932" => Ok(Self::Sjis),
            "gbk" | "cp936" => Ok(Self::Gbk),
            _ => Err(format!("unknown encoding '{value}', expected sjis or gbk")),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Sjis => "sjis",
            Self::Gbk => "gbk",
        }
    }
}

pub fn decode_with(choice: EncodingChoice, bytes: &[u8]) -> Result<(String, bool)> {
    let (cow, _, had_errors) = match choice {
        EncodingChoice::Sjis => SHIFT_JIS.decode(bytes),
        EncodingChoice::Gbk => GBK.decode(bytes),
    };
    Ok((cow.into_owned(), had_errors))
}

pub fn encode_with(choice: EncodingChoice, text: &str) -> Result<Vec<u8>> {
    let enc = match choice {
        EncodingChoice::Sjis => SHIFT_JIS,
        EncodingChoice::Gbk => GBK,
    };
    let (bytes, _, had_errors) = enc.encode(text);
    if had_errors {
        return Err(format!(
            "text contains characters not representable in {}",
            choice.label()
        ));
    }
    Ok(bytes.into_owned())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: u64,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_name_index", skip_serializing_if = "Option::is_none")]
    pub name_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    #[serde(rename = "_scr_tag", skip_serializing_if = "Option::is_none")]
    pub scr_tag: Option<String>,
    pub scr_msg: String,
    pub message: String,
}

pub fn read_json(path: &Path) -> Result<Vec<Entry>> {
    let data = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&data).map_err(|e| format!("parse {}: {e}", path.display()))
}

pub fn write_json(path: &Path, entries: &[Entry], overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return Err(format!(
            "output exists: {} (use --overwrite)",
            path.display()
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let data = serde_json::to_vec_pretty(entries).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| format!("write {}: {e}", path.display()))
}

pub fn collect_files(input: &Path, extension: &str) -> Result<Vec<(PathBuf, PathBuf)>> {
    if input.is_file() {
        if input
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| x.eq_ignore_ascii_case(extension))
            != Some(true)
        {
            return Err(format!("{} is not a .{extension} file", input.display()));
        }
        return Ok(vec![(
            input.to_path_buf(),
            PathBuf::from(input.file_name().unwrap()),
        )]);
    }
    if !input.is_dir() {
        return Err(format!("input does not exist: {}", input.display()));
    }
    let mut files = Vec::new();
    collect_recursive(input, input, extension, &mut files)?;
    files.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(files)
}

fn collect_recursive(
    root: &Path,
    dir: &Path,
    extension: &str,
    out: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<()> {
    for item in fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))? {
        let path = item.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            collect_recursive(root, &path, extension, out)?;
        } else if path
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| x.eq_ignore_ascii_case(extension))
            == Some(true)
        {
            let rel = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_path_buf();
            out.push((path, rel));
        }
    }
    Ok(())
}

pub fn ensure_output(path: &Path, overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return Err(format!(
            "output exists: {} (use --overwrite)",
            path.display()
        ));
    }
    fs::create_dir_all(path).map_err(|e| format!("create {}: {e}", path.display()))
}

pub fn parse_args(args: &[String]) -> Result<(String, PathBuf, PathBuf, EncodingChoice, bool)> {
    if args.len() < 4 {
        return Err("usage: <extract|inject> --input PATH [--translation PATH] --output PATH --encoding sjis|gbk [--overwrite]".into());
    }
    let mode = args[1].clone();
    let mut input = None;
    let mut translation = None;
    let mut output = None;
    let mut encoding = None;
    let mut overwrite = false;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => { i += 1; input = args.get(i).map(PathBuf::from); }
            "--translation" => { i += 1; translation = args.get(i).map(PathBuf::from); }
            "--output" => { i += 1; output = args.get(i).map(PathBuf::from); }
            "--encoding" => { i += 1; encoding = Some(EncodingChoice::parse(args.get(i).ok_or("missing --encoding value")?)?); }
            "--overwrite" => overwrite = true,
            "--help" | "-h" => return Err("usage: <extract|inject> --input PATH [--translation PATH] --output PATH --encoding sjis|gbk [--overwrite]".into()),
            other => return Err(format!("unknown argument '{other}'")),
        }
        i += 1;
    }
    let input = input.ok_or("missing --input")?;
    let output = output.ok_or("missing --output")?;
    if mode == "inject" && translation.is_none() {
        return Err("inject requires --translation".into());
    }
    Ok((
        mode,
        input,
        output,
        encoding.ok_or("missing --encoding; choose sjis or gbk")?,
        overwrite,
    ))
}

pub fn prompt(label: &str, default: Option<&str>) -> Result<String> {
    match default {
        Some(value) => print!("{label} [{value}]: "),
        None => print!("{label}: "),
    }
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut value = String::new();
    if io::stdin()
        .read_line(&mut value)
        .map_err(|e| e.to_string())?
        == 0
    {
        return Err("end of input".into());
    }
    let value = value.trim();
    if value.is_empty() {
        default
            .map(str::to_owned)
            .ok_or_else(|| format!("{label} is required"))
    } else {
        Ok(value.to_owned())
    }
}

pub fn confirm(label: &str) -> Result<bool> {
    let answer = prompt(&format!("{label}? (y/N)"), Some("N"))?;
    Ok(matches!(answer.to_ascii_lowercase().as_str(), "y" | "yes"))
}
