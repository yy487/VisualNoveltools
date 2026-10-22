use crate::{model::FileSpan, Result};
use vn_sector_map::LinearView;

/// A bounded byte view. FAT logical sectors need not equal physical sector sizes.
pub(crate) struct ViewReader<'a> {
    pub source: &'a [u8],
    pub view: &'a LinearView,
    pub base: usize,
    pub length: usize,
}

impl<'a> ViewReader<'a> {
    pub fn new(source: &'a [u8], view: &'a LinearView, base: usize, length: usize) -> Result<Self> {
        if base.checked_add(length).is_none_or(|end| end > view.len()) {
            return Err("volume range exceeds the logical disk view".into());
        }
        Ok(Self {
            source,
            view,
            base,
            length,
        })
    }

    pub fn spans(&self, offset: usize, length: usize) -> Result<Vec<FileSpan>> {
        if offset
            .checked_add(length)
            .is_none_or(|end| end > self.length)
        {
            return Err(format!(
                "volume read outside bounds: offset={offset:#x}, length={length:#x}, volume={:#x}",
                self.length
            ));
        }
        let start = self.base + offset;
        let mapped = self
            .view
            .map_range(start..start + length)
            .map_err(|e| e.to_string())?;
        let mut spans = Vec::with_capacity(mapped.len());
        let mut cursor = 0;
        for span in mapped {
            let size = span.source_range.len();
            if span.source_range.end > self.source.len() {
                return Err("mapped source range exceeds the input snapshot".into());
            }
            spans.push(FileSpan {
                file_offset: cursor,
                source_offset: span.source_range.start,
                length: size,
            });
            cursor += size;
        }
        if cursor != length {
            return Err("logical view does not cover the complete requested range".into());
        }
        Ok(spans)
    }

    pub fn read(&self, offset: usize, length: usize) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(length.min(self.length));
        for span in self.spans(offset, length)? {
            data.extend_from_slice(
                &self.source[span.source_offset..span.source_offset + span.length],
            );
        }
        Ok(data)
    }
}
