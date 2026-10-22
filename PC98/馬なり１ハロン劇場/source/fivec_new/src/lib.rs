#![forbid(unsafe_code)]

mod container;
mod export;
mod fat;
mod model;
mod view;
mod volume;

pub use export::{ExportFile, ExportManifest, ExportReport, PreparedExport};
pub use model::*;
pub type Result<T> = std::result::Result<T, String>;

use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Immutable, validated snapshot. No GUI, external executable, or IDA dependency.
pub struct Archive {
    source_path: Option<PathBuf>,
    bytes: Arc<[u8]>,
    inspection: Inspection,
}

impl Archive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = fs::canonicalize(path.as_ref())
            .map_err(|e| format!("{}: {e}", path.as_ref().display()))?;
        let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let name = path
            .file_name()
            .ok_or("source has no filename")?
            .to_string_lossy()
            .into_owned();
        let mut archive = Self::from_bytes(name, bytes)?;
        archive.source_path = Some(path);
        Ok(archive)
    }

    pub fn from_bytes(source_name: impl Into<String>, bytes: Vec<u8>) -> Result<Self> {
        let source_name = source_name.into();
        let container = container::decode(&bytes, &source_name)?;
        let volumes = volume::discover(&bytes, &container)?;
        let inspection = Inspection {
            schema: "fivec-new-inspection-v1".into(),
            parser_version: env!("CARGO_PKG_VERSION").into(),
            source_name,
            source_size: bytes.len(),
            source_sha256: sha256(&bytes),
            format: container.format,
            disks: container
                .disks
                .iter()
                .map(|d| DiskReport {
                    index: d.index,
                    name: d.name.clone(),
                    geometry: d.geometry.clone(),
                    sector_count: d.sector_count,
                    payload_bytes: d.payload_bytes,
                    logical_view_available: d.view.is_some(),
                    diagnostics: d.diagnostics.clone(),
                })
                .collect(),
            volumes,
            diagnostics: container.diagnostics,
        };
        Ok(Self {
            source_path: None,
            bytes: bytes.into(),
            inspection,
        })
    }

    pub fn inspection(&self) -> &Inspection {
        &self.inspection
    }
    pub fn volumes(&self) -> &[Volume] {
        &self.inspection.volumes
    }
    pub fn source_path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }

    pub fn read_file(&self, volume_id: &str, entry_id: &str) -> Result<Vec<u8>> {
        let volume = self
            .volumes()
            .iter()
            .find(|v| v.id == volume_id)
            .ok_or_else(|| format!("unknown volume: {volume_id}"))?;
        let fs = volume
            .filesystem
            .as_ref()
            .ok_or_else(|| format!("volume {volume_id} has no supported filesystem"))?;
        let file = fs
            .files
            .iter()
            .find(|f| f.id == entry_id)
            .ok_or_else(|| format!("unknown file entry: {entry_id}"))?;
        let data = read_spans(&self.bytes, &file.source_ranges, file.size)?;
        if sha256(&data) != file.sha256 {
            return Err(format!("file {entry_id}: source mapping hash mismatch"));
        }
        Ok(data)
    }
}

pub(crate) fn read_spans(source: &[u8], spans: &[FileSpan], size: usize) -> Result<Vec<u8>> {
    let mut data = Vec::with_capacity(size.min(source.len()));
    for span in spans {
        if span.file_offset != data.len() {
            return Err("file source ranges have a gap or overlap".into());
        }
        let end = span
            .source_offset
            .checked_add(span.length)
            .ok_or("source range overflow")?;
        data.extend_from_slice(
            source
                .get(span.source_offset..end)
                .ok_or("file source range outside snapshot")?,
        );
    }
    if data.len() != size {
        return Err(format!(
            "file mapping size mismatch: expected {size}, got {}",
            data.len()
        ));
    }
    Ok(data)
}
