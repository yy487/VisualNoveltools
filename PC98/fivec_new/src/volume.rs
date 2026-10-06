use crate::{fat, hex, model::Container, view::ViewReader, Result, Volume};

pub(crate) fn discover(source: &[u8], container: &Container) -> Result<Vec<Volume>> {
    let mut volumes = Vec::new();
    for disk in &container.disks {
        let id = format!("disk-{:03}-whole", disk.index);
        let mut whole = Volume {
            id,
            disk_index: disk.index,
            partition_index: None,
            kind: "whole-disk".into(),
            logical_byte_offset: 0,
            logical_byte_length: disk.view.as_ref().map_or(0, |v| v.len()),
            partition_entry_hex: None,
            filesystem: None,
            diagnostics: Vec::new(),
        };
        let Some(view) = &disk.view else {
            whole
                .diagnostics
                .push("container has no unambiguous logical sector view".into());
            volumes.push(whole);
            continue;
        };
        let reader = ViewReader::new(source, view, 0, view.len())?;
        match fat::probe(&reader) {
            Ok(Some(fs)) => {
                whole.logical_byte_length = fs.fat12.total_sectors * fs.fat12.bytes_per_sector;
                whole.filesystem = Some(fs);
                volumes.push(whole);
                continue;
            }
            Err(error) => {
                whole.diagnostics.push(format!("invalid FAT12: {error}"));
                volumes.push(whole);
                continue;
            }
            Ok(None) => {}
        }
        let Some(g) = &disk.geometry else {
            unreachable!("logical view requires geometry")
        };
        let physical_bps = g.bytes_per_sector as usize;
        if reader.length < physical_bps + 512 {
            whole
                .diagnostics
                .push("no supported FAT12 BPB or PC98 DOS partition table".into());
            volumes.push(whole);
            continue;
        }
        let table = reader.read(physical_bps, 512)?;
        // The supported PC98 DOS profile uses MID 21h / SID 01h; high bits
        // represent boot/active flags. Unknown tables are never guessed as DOS.
        let is_dos = |e: &[u8]| e[0] & 0x7f == 0x21 && e[1] & 0x7f == 1;
        if !table.chunks_exact(32).any(is_dos) {
            whole.diagnostics.push("no supported FAT12 BPB or PC98 DOS partition table (N88 FAT8, CP/M and game-specific layouts are not implemented)".into());
            volumes.push(whole);
            continue;
        }
        let start_of = |e: &[u8]| -> Option<usize> {
            let s = usize::from(e[8]);
            let h = usize::from(e[9]);
            let c = usize::from(u16::from_le_bytes([e[10], e[11]]));
            if s >= g.sectors_per_track as usize
                || h >= g.heads as usize
                || c >= g.cylinders as usize
            {
                return None;
            }
            let start =
                ((c * g.heads as usize + h) * g.sectors_per_track as usize + s) * physical_bps;
            (start >= physical_bps + 512 && start < view.len()).then_some(start)
        };
        let entries: Vec<_> = table
            .chunks_exact(32)
            .enumerate()
            .filter(|(_, e)| e.iter().any(|b| *b != 0))
            .collect();
        for (index, entry) in &entries {
            let mut volume = Volume {
                id: format!("disk-{:03}-partition-{index:02}", disk.index),
                disk_index: disk.index,
                partition_index: Some(*index),
                kind: "pc98-partition".into(),
                logical_byte_offset: 0,
                logical_byte_length: 0,
                partition_entry_hex: Some(hex(entry)),
                filesystem: None,
                diagnostics: Vec::new(),
            };
            let Some(start) = start_of(entry) else {
                volume
                    .diagnostics
                    .push("invalid or unsupported PC98 partition start CHS".into());
                volumes.push(volume);
                continue;
            };
            volume.logical_byte_offset = start;
            let end_c = usize::from(u16::from_le_bytes([entry[14], entry[15]]));
            if entry[12] != 0 || entry[13] != 0 || end_c >= g.cylinders as usize {
                volume.diagnostics.push("unsupported PC98 partition end: only inclusive cylinder ends with H=S=0 are verified".into());
                volumes.push(volume);
                continue;
            }
            let end = (end_c + 1) * g.heads as usize * g.sectors_per_track as usize * physical_bps;
            let next = entries
                .iter()
                .filter(|(other, _)| other != index)
                .filter_map(|(_, e)| start_of(e))
                .filter(|s| *s > start)
                .min()
                .unwrap_or(view.len());
            let overlaps = entries.iter().any(|(other, e)| {
                if other == index {
                    return false;
                }
                let Some(other_start) = start_of(e) else {
                    return false;
                };
                let other_c = usize::from(u16::from_le_bytes([e[14], e[15]]));
                // With an unverified end, any later partition might overlap it.
                let other_end = if e[12] == 0 && e[13] == 0 && other_c < g.cylinders as usize {
                    (other_c + 1) * g.heads as usize * g.sectors_per_track as usize * physical_bps
                } else {
                    view.len()
                };
                start < other_end && other_start < end
            });
            if end <= start || end > view.len() || end > next || overlaps {
                volume
                    .diagnostics
                    .push("invalid or overlapping PC98 partition bounds".into());
                volumes.push(volume);
                continue;
            }
            volume.logical_byte_length = end - start;
            if !is_dos(entry) {
                volume
                    .diagnostics
                    .push("unsupported PC98 partition type".into());
            } else {
                let bounded = ViewReader::new(source, view, start, end - start)?;
                match fat::probe(&bounded) {
                    Ok(Some(fs)) => {
                        volume.logical_byte_length =
                            fs.fat12.bytes_per_sector * fs.fat12.total_sectors;
                        volume.filesystem = Some(fs);
                    }
                    Ok(None) => volume
                        .diagnostics
                        .push("partition has no supported FAT12 BPB".into()),
                    Err(error) => volume.diagnostics.push(format!("invalid FAT12: {error}")),
                }
            }
            volumes.push(volume);
        }
    }
    Ok(volumes)
}
