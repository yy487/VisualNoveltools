use crate::{Result, Source};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub struct Prepared {
    sources: Vec<Source>,
    output: PathBuf,
    files: BTreeMap<String, Vec<u8>>,
    pub resources: usize,
    pub translation_files: usize,
    pub translation_entries: usize,
}

fn absent(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
        Ok(_) => Err(format!(
            "output already exists; choose a new directory: {}",
            path.display()
        )),
    }
}

impl Prepared {
    pub(crate) fn new(
        sources: Vec<Source>,
        output: &Path,
        files: BTreeMap<String, Vec<u8>>,
        resources: usize,
        translation_files: usize,
        translation_entries: usize,
    ) -> Result<Self> {
        let absolute = std::path::absolute(output).map_err(|e| e.to_string())?;
        absent(&absolute)?;
        let parent = fs::canonicalize(absolute.parent().ok_or("output has no parent")?)
            .map_err(|e| e.to_string())?;
        if !parent.is_dir() {
            return Err("output parent is not a directory".into());
        }
        let output = parent.join(absolute.file_name().ok_or("output has no filename")?);
        absent(&output)?;
        for source in &sources {
            if output.starts_with(&source.path) || source.path.starts_with(&output) {
                return Err("output overlaps protected input".into());
            }
        }
        for name in files.keys() {
            if name.is_empty()
                || Path::new(name)
                    .components()
                    .any(|c| !matches!(c, Component::Normal(_)))
            {
                return Err(format!("unsafe export path: {name}"));
            }
        }
        Ok(Self {
            sources,
            output,
            files,
            resources,
            translation_files,
            translation_entries,
        })
    }
    pub fn inputs(&self) -> Vec<PathBuf> {
        self.sources.iter().map(|s| s.path.clone()).collect()
    }
    pub fn output(&self) -> &Path {
        &self.output
    }
    fn verify_sources(&self) -> Result<()> {
        for source in &self.sources {
            if crate::workflow::source_hash(&source.path)? != source.hash {
                return Err(format!(
                    "source changed after preparation: {}",
                    source.path.display()
                ));
            }
        }
        Ok(())
    }
    pub fn execute(self) -> Result<PathBuf> {
        self.verify_sources()?;
        absent(&self.output)?;
        let parent = self.output.parent().ok_or("output has no parent")?;
        let stage = Stage::new(parent)?;
        for (name, bytes) in &self.files {
            let path = stage.0.join(name);
            fs::create_dir_all(path.parent().ok_or("file has no parent")?)
                .map_err(|e| e.to_string())?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|e| e.to_string())?;
            file.write_all(bytes).map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            drop(file);
            if fs::read(&path).map_err(|e| e.to_string())? != *bytes {
                return Err(format!("staged file verification failed: {name}"));
            }
        }
        self.verify_sources()?;
        absent(&self.output)?;
        fs::rename(&stage.0, &self.output).map_err(|e| e.to_string())?;
        Ok(self.output)
    }
}

struct Stage(PathBuf);
impl Stage {
    fn new(parent: &Path) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".kohaku-stage-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.to_string()),
            }
        }
        Err("cannot allocate staging directory".into())
    }
}
impl Drop for Stage {
    fn drop(&mut self) {
        // Only this exclusively created staging tree is ever removed.
        let _ = fs::remove_dir_all(&self.0);
    }
}
