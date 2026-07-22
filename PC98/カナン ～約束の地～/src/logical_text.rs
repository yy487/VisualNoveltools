use crate::script::{
    encode_cp932_text, instruction_successors, Cfg, ScriptStreamPatch, TextCandidate, TextPart,
};
use crate::text_json::{
    control_marker, encode_page_message, glyph_marker, parse_message, token_line_units,
    tokenize_encoded, EncodedToken, TextJsonError, TranslationEntry, TranslationSegment,
};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundaryAnchor {
    Structural {
        ordinal: usize,
        include_following_newline: bool,
    },
    Newline {
        ordinal: usize,
    },
    BeforeNewline {
        ordinal: usize,
    },
}

#[derive(Debug, Clone)]
struct RawSegment {
    candidate_index: usize,
    part_start: usize,
    part_end: usize,
    page_index: usize,
    page_count: usize,
}

#[derive(Debug, Clone)]
struct LogicalSegment {
    candidate_index: usize,
    part_start: usize,
    part_end: usize,
    visible_part_end: usize,
    source_units: usize,
    anchor_after: Option<BoundaryAnchor>,
    metadata: TranslationSegment,
}

#[derive(Debug, Clone)]
pub struct LogicalPage {
    channel: u16,
    text_type: String,
    scr_msg: String,
    source_controls: Vec<Vec<u8>>,
    source_newlines: usize,
    source_line_units: Vec<usize>,
    segments: Vec<LogicalSegment>,
}

#[derive(Debug, Clone)]
pub struct LogicalScript {
    candidates: Vec<TextCandidate>,
    pub pages: Vec<LogicalPage>,
    pub forced_progressive_chains: usize,
}

enum StreamEvent {
    Segment(RawSegment),
    PageBreak,
}

fn part_bytes(part: &TextPart) -> Result<Vec<u8>, TextJsonError> {
    Ok(match part {
        TextPart::Text(segment) => encode_cp932_text(&segment.text)?,
        TextPart::Control(control) => control.encoded(),
        TextPart::Glyph(glyph) => glyph.bytes.clone(),
    })
}

fn parts_bytes(
    candidate: &TextCandidate,
    start: usize,
    end: usize,
) -> Result<Vec<u8>, TextJsonError> {
    let mut bytes = Vec::new();
    for part in &candidate.stream.parts[start..end] {
        bytes.extend_from_slice(&part_bytes(part)?);
    }
    Ok(bytes)
}

fn render_parts(parts: &[TextPart]) -> String {
    let mut output = String::new();
    for part in parts {
        match part {
            TextPart::Text(segment) => output.push_str(&segment.text),
            TextPart::Control(control) if control.code == 0x0D => output.push('\n'),
            TextPart::Control(control) => output.push_str(&control_marker(control)),
            TextPart::Glyph(glyph) => output.push_str(&glyph_marker(glyph)),
        }
    }
    output
}

fn stream_events(candidate_index: usize, candidate: &TextCandidate) -> Vec<StreamEvent> {
    let page_count = candidate
        .stream
        .parts
        .iter()
        .filter(|part| matches!(part, TextPart::Control(control) if control.code == 0x01))
        .count()
        + 1;
    let mut events = Vec::new();
    let mut start = 0usize;
    let mut page_index = 0usize;
    for (index, part) in candidate.stream.parts.iter().enumerate() {
        if matches!(part, TextPart::Control(control) if control.code == 0x01) {
            if start < index {
                events.push(StreamEvent::Segment(RawSegment {
                    candidate_index,
                    part_start: start,
                    part_end: index,
                    page_index,
                    page_count,
                }));
            }
            events.push(StreamEvent::PageBreak);
            start = index + 1;
            page_index += 1;
        }
    }
    if start < candidate.stream.parts.len() {
        events.push(StreamEvent::Segment(RawSegment {
            candidate_index,
            part_start: start,
            part_end: candidate.stream.parts.len(),
            page_index,
            page_count,
        }));
    }
    events
}

fn first_text_outcomes(cfg: &Cfg, start: usize) -> (BTreeSet<usize>, bool) {
    let text_offsets: HashSet<_> = cfg
        .instructions
        .values()
        .filter(|instruction| matches!(instruction.opcode, 0x15 | 0x16))
        .map(|instruction| instruction.offset)
        .collect();
    let mut queue = VecDeque::from([start]);
    let mut seen = HashSet::new();
    let mut outcomes = BTreeSet::new();
    let mut terminated = false;
    while let Some(offset) = queue.pop_front() {
        if !seen.insert(offset) {
            continue;
        }
        if text_offsets.contains(&offset) {
            outcomes.insert(offset);
            continue;
        }
        let Some(instruction) = cfg.instructions.get(&offset) else {
            terminated = true;
            continue;
        };
        let successors = instruction_successors(instruction);
        if successors.is_empty() {
            terminated = true;
        }
        queue.extend(successors);
    }
    (outcomes, terminated)
}

fn ends_at_page_boundary(candidate: &TextCandidate) -> bool {
    candidate
        .stream
        .parts
        .iter()
        .rev()
        .find(|part| !matches!(part, TextPart::Control(control) if control.code == 0x02))
        .is_some_and(|part| matches!(part, TextPart::Control(control) if control.code == 0x01))
}

fn progressive_fragment(candidate: &TextCandidate) -> Option<String> {
    if candidate.operands.first() != Some(&9) {
        return None;
    }
    let visible_end =
        trailing_hidden_start(&candidate.stream.parts, 0, candidate.stream.parts.len());
    let visible_parts = &candidate.stream.parts[..visible_end];
    if visible_parts
        .iter()
        .any(|part| matches!(part, TextPart::Control(_) | TextPart::Glyph(_)))
    {
        return None;
    }
    let message = render_parts(visible_parts);
    (message.chars().count() <= 2 && !message.contains('\n')).then_some(message)
}

fn mark_progressive_quote_chains(candidates: &[TextCandidate], edges: &mut [bool]) -> usize {
    let mut chains = 0usize;
    let mut start = 0usize;
    while start < candidates.len() {
        let Some(first) = progressive_fragment(&candidates[start]) else {
            start += 1;
            continue;
        };
        if first != "「" {
            start += 1;
            continue;
        }
        let mut merged = first;
        let mut end = start + 1;
        while end < candidates.len() && end <= start + 15 {
            let Some(fragment) = progressive_fragment(&candidates[end]) else {
                break;
            };
            if candidates[end].instruction_offset <= candidates[end - 1].instruction_offset {
                break;
            }
            merged.push_str(&fragment);
            if merged.matches('「').count() == 1 && merged.matches('」').count() == 1 {
                if merged.ends_with('」') && end - start + 1 >= 4 {
                    for edge in &mut edges[start..end] {
                        *edge = true;
                    }
                    chains += 1;
                    start = end;
                }
                break;
            }
            if fragment.contains(['「', '」']) {
                break;
            }
            end += 1;
        }
        start += 1;
    }
    chains
}

fn trailing_hidden_start(parts: &[TextPart], start: usize, end: usize) -> usize {
    let mut visible_end = end;
    while visible_end > start
        && matches!(
            &parts[visible_end - 1],
            TextPart::Control(control) if matches!(control.code, 0x01 | 0x02)
        )
    {
        visible_end -= 1;
    }
    visible_end
}

fn is_structural_control(token: &EncodedToken) -> bool {
    token
        .control
        .is_some_and(|code| !matches!(code, 0x01 | 0x0D))
}

fn finalize_page(
    candidates: &[TextCandidate],
    raw_segments: Vec<RawSegment>,
) -> Result<Option<LogicalPage>, TextJsonError> {
    if raw_segments.is_empty() {
        return Ok(None);
    }
    let channel = candidates[raw_segments[0].candidate_index].operands[0];
    if raw_segments
        .iter()
        .any(|segment| candidates[segment.candidate_index].operands[0] != channel)
    {
        return Err(TextJsonError::Invalid(
            "logical page combines different display channels".to_owned(),
        ));
    }

    let mut scr_msg = String::new();
    let mut visible_bytes = Vec::new();
    let mut prepared = Vec::with_capacity(raw_segments.len());
    for (index, raw) in raw_segments.iter().enumerate() {
        let candidate = &candidates[raw.candidate_index];
        let visible_part_end = if index + 1 == raw_segments.len() {
            trailing_hidden_start(&candidate.stream.parts, raw.part_start, raw.part_end)
        } else {
            raw.part_end
        };
        scr_msg.push_str(&render_parts(
            &candidate.stream.parts[raw.part_start..visible_part_end],
        ));
        let bytes = parts_bytes(candidate, raw.part_start, visible_part_end)?;
        visible_bytes.extend_from_slice(&bytes);
        prepared.push((raw, visible_part_end, bytes));
    }
    if scr_msg.trim().is_empty() {
        return Ok(None);
    }
    let parsed = parse_message(&scr_msg)?;
    if parsed.page_count != 0 || parsed.bytes != visible_bytes {
        return Err(TextJsonError::Invalid(
            "logical page rendering is not byte-exact".to_owned(),
        ));
    }
    let source_line_units = token_line_units(&tokenize_encoded(&visible_bytes)?);

    let mut structural_ordinal = 0usize;
    let mut newline_ordinal = 0usize;
    let mut segments = Vec::with_capacity(prepared.len());
    for (index, (raw, visible_part_end, bytes)) in prepared.iter().enumerate() {
        let tokens = tokenize_encoded(bytes)?;
        structural_ordinal += tokens
            .iter()
            .filter(|token| is_structural_control(token))
            .count();
        newline_ordinal += tokens
            .iter()
            .filter(|token| token.control == Some(0x0D))
            .count();
        let anchor_after = if index + 1 == raw_segments.len() {
            None
        } else if tokens
            .last()
            .is_some_and(|token| token.control == Some(0x02))
        {
            Some(BoundaryAnchor::Structural {
                ordinal: structural_ordinal,
                include_following_newline: false,
            })
        } else if tokens.len() >= 2
            && tokens[tokens.len() - 2].control == Some(0x02)
            && tokens
                .last()
                .is_some_and(|token| token.control == Some(0x0D))
        {
            Some(BoundaryAnchor::Structural {
                ordinal: structural_ordinal,
                include_following_newline: true,
            })
        } else if channel == 8
            && tokens
                .last()
                .is_some_and(|token| token.control == Some(0x0D))
        {
            Some(BoundaryAnchor::Newline {
                ordinal: newline_ordinal,
            })
        } else if prepared
            .get(index + 1)
            .is_some_and(|(_, _, next_bytes)| next_bytes.first() == Some(&0x0D))
        {
            Some(BoundaryAnchor::BeforeNewline {
                ordinal: newline_ordinal + 1,
            })
        } else {
            None
        };
        let candidate = &candidates[raw.candidate_index];
        segments.push(LogicalSegment {
            candidate_index: raw.candidate_index,
            part_start: raw.part_start,
            part_end: raw.part_end,
            visible_part_end: *visible_part_end,
            source_units: tokens.iter().map(|token| token.units).sum(),
            anchor_after,
            metadata: TranslationSegment {
                instruction_offset: candidate.instruction_offset,
                offset: candidate.stream.offset,
                size: candidate.stream.end_offset - candidate.stream.offset,
                opcode: format!("0x{:02X}", candidate.opcode),
                part_start: raw.part_start,
                part_end: raw.part_end,
                page_index: raw.page_index,
                page_count: raw.page_count,
            },
        });
    }

    Ok(Some(LogicalPage {
        channel,
        text_type: if channel == 8 { "choice" } else { "text" }.to_owned(),
        scr_msg,
        source_controls: parsed.structural_controls,
        source_newlines: parsed.newline_count,
        source_line_units,
        segments,
    }))
}

pub fn build_logical_script(
    cfg: &Cfg,
    all_candidates: &[TextCandidate],
    excluded: &BTreeSet<usize>,
) -> Result<LogicalScript, TextJsonError> {
    let candidates: Vec<_> = all_candidates
        .iter()
        .filter(|candidate| !excluded.contains(&candidate.instruction_offset))
        .cloned()
        .collect();
    for candidate in &candidates {
        if candidate.opcode != 0x15 || !matches!(candidate.operands.first(), Some(8 | 9)) {
            return Err(TextJsonError::Invalid(format!(
                "unsupported main-story text instruction 0x{:04X}",
                candidate.instruction_offset
            )));
        }
    }

    let mut same_page_edges = vec![false; candidates.len().saturating_sub(1)];
    for index in 0..same_page_edges.len() {
        if candidates[index].operands[0] != candidates[index + 1].operands[0]
            || ends_at_page_boundary(&candidates[index])
        {
            continue;
        }
        let instruction = cfg
            .instructions
            .get(&candidates[index].instruction_offset)
            .ok_or_else(|| {
                TextJsonError::Invalid(format!(
                    "missing CFG instruction 0x{:04X}",
                    candidates[index].instruction_offset
                ))
            })?;
        let (outcomes, terminated) =
            first_text_outcomes(cfg, instruction.offset + instruction.size);
        same_page_edges[index] = !terminated
            && outcomes.len() == 1
            && outcomes.contains(&candidates[index + 1].instruction_offset);
    }
    let forced_progressive_chains =
        mark_progressive_quote_chains(&candidates, &mut same_page_edges);

    let mut pages = Vec::new();
    let mut current = Vec::new();
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        if candidate_index > 0 && !same_page_edges[candidate_index - 1] {
            if let Some(page) = finalize_page(&candidates, std::mem::take(&mut current))? {
                pages.push(page);
            }
        }
        for event in stream_events(candidate_index, candidate) {
            match event {
                StreamEvent::Segment(segment) => current.push(segment),
                StreamEvent::PageBreak => {
                    if let Some(page) = finalize_page(&candidates, std::mem::take(&mut current))? {
                        pages.push(page);
                    }
                }
            }
        }
        if candidate_index + 1 == candidates.len() || !same_page_edges[candidate_index] {
            if let Some(page) = finalize_page(&candidates, std::mem::take(&mut current))? {
                pages.push(page);
            }
        }
    }

    Ok(LogicalScript {
        candidates,
        pages,
        forced_progressive_chains,
    })
}

impl LogicalPage {
    pub fn translation_entry(&self, file: &str, index: usize) -> TranslationEntry {
        TranslationEntry {
            file: file.to_owned(),
            index,
            text_type: self.text_type.clone(),
            channel: self.channel,
            encoding: "CP932".to_owned(),
            segments: self
                .segments
                .iter()
                .map(|segment| segment.metadata.clone())
                .collect(),
            scr_msg: self.scr_msg.clone(),
            message: self.scr_msg.clone(),
        }
    }

    fn validate_entry(
        &self,
        file: &str,
        index: usize,
        entry: &TranslationEntry,
    ) -> Result<(), TextJsonError> {
        let expected_segments: Vec<_> = self
            .segments
            .iter()
            .map(|segment| segment.metadata.clone())
            .collect();
        if entry.file != file
            || entry.index != index
            || entry.text_type != self.text_type
            || entry.channel != self.channel
            || entry.encoding != "CP932"
            || entry.segments != expected_segments
        {
            return Err(TextJsonError::Invalid(format!(
                "entry {index} metadata does not match the source logical page"
            )));
        }
        if entry.scr_msg != self.scr_msg {
            return Err(TextJsonError::Invalid(format!(
                "entry {index} immutable scr_msg does not match the source logical page"
            )));
        }
        Ok(())
    }
}

fn anchor_token_position(
    tokens: &[EncodedToken],
    anchor: BoundaryAnchor,
) -> Result<usize, TextJsonError> {
    match anchor {
        BoundaryAnchor::Structural {
            ordinal,
            include_following_newline,
        } => {
            let mut current = 0usize;
            for (index, token) in tokens.iter().enumerate() {
                if is_structural_control(token) {
                    current += 1;
                    if current == ordinal {
                        let mut position = index + 1;
                        if include_following_newline
                            && tokens
                                .get(position)
                                .is_some_and(|next| next.control == Some(0x0D))
                        {
                            position += 1;
                        }
                        return Ok(position);
                    }
                }
            }
        }
        BoundaryAnchor::Newline { ordinal } => {
            let mut current = 0usize;
            for (index, token) in tokens.iter().enumerate() {
                if token.control == Some(0x0D) {
                    current += 1;
                    if current == ordinal {
                        return Ok(index + 1);
                    }
                }
            }
        }
        BoundaryAnchor::BeforeNewline { ordinal } => {
            let mut current = 0usize;
            for (index, token) in tokens.iter().enumerate() {
                if token.control == Some(0x0D) {
                    current += 1;
                    if current == ordinal {
                        return Ok(index);
                    }
                }
            }
        }
    }
    Err(TextJsonError::Invalid(
        "translated message no longer contains a required segment anchor".to_owned(),
    ))
}

fn best_proportional_position(
    tokens: &[EncodedToken],
    origin: usize,
    start: usize,
    end: usize,
    target_units: usize,
) -> usize {
    let mut units = tokens[origin..start]
        .iter()
        .map(|token| token.units)
        .sum::<usize>();
    let mut best = start;
    let mut best_difference = target_units;
    for (position, token) in tokens.iter().enumerate().take(end).skip(start) {
        let difference = units.abs_diff(target_units);
        if difference < best_difference {
            best = position;
            best_difference = difference;
        }
        units += token.units;
    }
    if units.abs_diff(target_units) < best_difference {
        best = end;
    }
    best
}

fn split_page_tokens(
    page: &LogicalPage,
    tokens: &[EncodedToken],
) -> Result<Vec<Vec<u8>>, TextJsonError> {
    let count = page.segments.len();
    if count == 1 {
        return Ok(vec![tokens
            .iter()
            .flat_map(|token| token.bytes.iter().copied())
            .collect()]);
    }
    let mut fixed = vec![None; count + 1];
    fixed[0] = Some(0);
    fixed[count] = Some(tokens.len());
    let translated_newlines = tokens
        .iter()
        .filter(|token| token.control == Some(0x0D))
        .count();
    for (index, segment) in page.segments.iter().enumerate().take(count - 1) {
        if let Some(anchor) = segment.anchor_after {
            if matches!(anchor, BoundaryAnchor::BeforeNewline { .. })
                && page.channel == 9
                && translated_newlines != page.source_newlines
            {
                continue;
            }
            fixed[index + 1] = Some(anchor_token_position(tokens, anchor)?);
        }
    }
    let fixed_points: Vec<_> = fixed
        .iter()
        .enumerate()
        .filter_map(|(index, position)| position.map(|position| (index, position)))
        .collect();
    for pair in fixed_points.windows(2) {
        if pair[0].1 > pair[1].1 {
            return Err(TextJsonError::Invalid(
                "translated segment anchors are out of order".to_owned(),
            ));
        }
    }

    let mut boundaries = vec![0usize; count + 1];
    boundaries[0] = 0;
    boundaries[count] = tokens.len();
    for pair in fixed_points.windows(2) {
        let (left_boundary, left_token) = pair[0];
        let (right_boundary, right_token) = pair[1];
        boundaries[left_boundary] = left_token;
        boundaries[right_boundary] = right_token;
        if right_boundary == left_boundary + 1 {
            continue;
        }
        let source_total: usize = page.segments[left_boundary..right_boundary]
            .iter()
            .map(|segment| segment.source_units)
            .sum();
        let translated_total: usize = tokens[left_token..right_token]
            .iter()
            .map(|token| token.units)
            .sum();
        let mut source_prefix = 0usize;
        let mut previous = left_token;
        for (boundary, slot) in boundaries
            .iter_mut()
            .enumerate()
            .take(right_boundary)
            .skip(left_boundary + 1)
        {
            source_prefix += page.segments[boundary - 1].source_units;
            let target_units = if source_total == 0 {
                (right_token - left_token) * (boundary - left_boundary)
                    / (right_boundary - left_boundary)
            } else {
                (translated_total * source_prefix + source_total / 2) / source_total
            };
            let position = if source_total == 0 {
                left_token + target_units
            } else {
                best_proportional_position(tokens, left_token, previous, right_token, target_units)
            };
            *slot = position;
            previous = position;
        }
    }

    let mut output = Vec::with_capacity(count);
    for range in boundaries.windows(2) {
        output.push(
            tokens[range[0]..range[1]]
                .iter()
                .flat_map(|token| token.bytes.iter().copied())
                .collect(),
        );
    }
    Ok(output)
}

impl LogicalScript {
    pub fn translation_entries(&self, file: &str) -> Vec<TranslationEntry> {
        self.pages
            .iter()
            .enumerate()
            .map(|(index, page)| page.translation_entry(file, index))
            .collect()
    }

    pub fn prepare_patches(
        &self,
        file: &str,
        entries: &[TranslationEntry],
    ) -> Result<Vec<ScriptStreamPatch>, TextJsonError> {
        if entries.len() != self.pages.len() {
            return Err(TextJsonError::Invalid(format!(
                "JSON has {} page entries, source has {} logical pages",
                entries.len(),
                self.pages.len()
            )));
        }
        let mut assignments: HashMap<(usize, usize, usize), Vec<u8>> = HashMap::new();
        for (index, (page, entry)) in self.pages.iter().zip(entries).enumerate() {
            page.validate_entry(file, index, entry)?;
            let split = if entry.message == entry.scr_msg {
                page.segments
                    .iter()
                    .map(|segment| {
                        parts_bytes(
                            &self.candidates[segment.candidate_index],
                            segment.part_start,
                            segment.visible_part_end,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                let encoded = encode_page_message(
                    &page.source_controls,
                    page.source_newlines,
                    &page.source_line_units,
                    &entry.message,
                    page.channel,
                )
                .map_err(|error| TextJsonError::Invalid(format!("entry {index}: {error}")))?;
                split_page_tokens(page, &encoded.tokens)?
            };
            for (segment, bytes) in page.segments.iter().zip(split) {
                let key = (
                    segment.candidate_index,
                    segment.part_start,
                    segment.part_end,
                );
                if assignments.insert(key, bytes).is_some() {
                    return Err(TextJsonError::Invalid(
                        "logical page segment was assigned more than once".to_owned(),
                    ));
                }
            }
        }

        let mut by_candidate: Vec<Vec<&LogicalSegment>> = vec![Vec::new(); self.candidates.len()];
        for page in &self.pages {
            for segment in &page.segments {
                by_candidate[segment.candidate_index].push(segment);
            }
        }
        let mut patches = Vec::with_capacity(self.candidates.len());
        for (candidate_index, candidate) in self.candidates.iter().enumerate() {
            by_candidate[candidate_index].sort_by_key(|segment| segment.part_start);
            let by_start: HashMap<_, _> = by_candidate[candidate_index]
                .iter()
                .map(|segment| (segment.part_start, *segment))
                .collect();
            let mut rebuilt = Vec::new();
            let mut part_index = 0usize;
            while part_index < candidate.stream.parts.len() {
                if let Some(segment) = by_start.get(&part_index) {
                    let key = (
                        segment.candidate_index,
                        segment.part_start,
                        segment.part_end,
                    );
                    rebuilt.extend_from_slice(assignments.get(&key).ok_or_else(|| {
                        TextJsonError::Invalid("logical page segment has no assignment".to_owned())
                    })?);
                    rebuilt.extend_from_slice(&parts_bytes(
                        candidate,
                        segment.visible_part_end,
                        segment.part_end,
                    )?);
                    part_index = segment.part_end;
                } else {
                    rebuilt.extend_from_slice(&part_bytes(&candidate.stream.parts[part_index])?);
                    part_index += 1;
                }
            }
            rebuilt.push(0);
            patches.push(ScriptStreamPatch {
                instruction_offset: candidate.instruction_offset,
                expected_text_offset: candidate.stream.offset,
                expected_size: candidate.stream.end_offset - candidate.stream.offset,
                expected_stream: candidate.stream.encoded(),
                replacement_stream: rebuilt,
            });
        }
        Ok(patches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{build_cfg, cfg_text_candidates, SCRIPT_BODY_OFFSET};

    fn encoded_stream(parts: &[(&str, &[u8])]) -> Vec<u8> {
        let mut stream = Vec::new();
        for (text, controls) in parts {
            stream.extend_from_slice(&encode_cp932_text(text).unwrap());
            stream.extend_from_slice(controls);
        }
        stream.push(0);
        stream
    }

    fn fixture_on_channel(streams: &[Vec<u8>], channel: u8) -> (Cfg, Vec<TextCandidate>) {
        let mut decoded = vec![0u8; 0x800];
        let mut text_offset = 0x200usize;
        for (index, stream) in streams.iter().enumerate() {
            let instruction_offset = SCRIPT_BODY_OFFSET + index * 6;
            decoded[instruction_offset..instruction_offset + 6].copy_from_slice(&[
                0x15,
                0,
                channel,
                0,
                text_offset as u8,
                (text_offset >> 8) as u8,
            ]);
            decoded[text_offset..text_offset + stream.len()].copy_from_slice(stream);
            text_offset += stream.len();
        }
        let terminator = SCRIPT_BODY_OFFSET + streams.len() * 6;
        decoded[terminator..terminator + 2].copy_from_slice(&[0, 0]);
        let cfg = build_cfg(&decoded);
        assert!(cfg.warnings.is_empty(), "{:?}", cfg.warnings);
        let candidates = cfg_text_candidates(&decoded, &cfg).unwrap();
        (cfg, candidates)
    }

    fn fixture(streams: &[Vec<u8>]) -> (Cfg, Vec<TextCandidate>) {
        fixture_on_channel(streams, 9)
    }

    fn logical(streams: &[Vec<u8>]) -> LogicalScript {
        let (cfg, candidates) = fixture(streams);
        build_logical_script(&cfg, &candidates, &BTreeSet::new()).unwrap()
    }

    fn logical_on_channel(streams: &[Vec<u8>], channel: u8) -> LogicalScript {
        let (cfg, candidates) = fixture_on_channel(streams, channel);
        build_logical_script(&cfg, &candidates, &BTreeSet::new()).unwrap()
    }

    #[test]
    fn internal_page_splits_but_final_subpage_joins_the_next_instruction() {
        let model = logical(&[
            encoded_stream(&[("一頁目", &[0x01]), ("二頁目前半", &[])]),
            encoded_stream(&[("後半", &[0x01])]),
        ]);
        assert_eq!(model.pages.len(), 2);
        assert_eq!(model.pages[0].scr_msg, "一頁目");
        assert_eq!(model.pages[1].scr_msg, "二頁目前半後半");
        assert_eq!(model.pages[1].segments.len(), 2);
    }

    #[test]
    fn reflow_can_remove_a_source_layout_newline_inside_a_name() {
        let model = logical(&[encoded_stream(&[
            ("ヴィアンカ＝Ａ＝イジュ", &[0x0D]),
            ("ウイン", &[0x01]),
        ])]);
        let mut entries = model.translation_entries("cs00_00.s");
        assert_eq!(entries[0].scr_msg, "ヴィアンカ＝Ａ＝イジュ\nウイン");
        entries[0].message = "ヴィアンカ＝Ａ＝イジュウイン".to_owned();
        let patches = model.prepare_patches("cs00_00.s", &entries).unwrap();
        assert!(!patches[0].replacement_stream.contains(&0x0D));
        assert_eq!(patches[0].replacement_stream.last(), Some(&0));
    }

    #[test]
    fn unanchored_segments_split_proportionally_without_empty_tail() {
        let model = logical(&[
            encoded_stream(&[("AA", &[])]),
            encoded_stream(&[("BB", &[])]),
            encoded_stream(&[("CC", &[0x01])]),
        ]);
        assert_eq!(model.pages.len(), 1);
        let mut entries = model.translation_entries("cs00_00.s");
        entries[0].message = "123456789".to_owned();
        let patches = model.prepare_patches("cs00_00.s", &entries).unwrap();
        assert_eq!(patches[0].replacement_stream, b"123\0");
        assert_eq!(patches[1].replacement_stream, b"456\0");
        assert_eq!(patches[2].replacement_stream, b"789\x01\0");
    }

    #[test]
    fn choice_segment_boundary_before_newline_is_preserved() {
        let model = logical_on_channel(
            &[
                encoded_stream(&[("      箱", &[])]),
                encoded_stream(&[("", &[0x0D]), (" コンピュータ", &[0x01])]),
            ],
            8,
        );
        let mut entries = model.translation_entries("cs00_00.s");
        entries[0].message = "      箱子\n 電脳".to_owned();
        let patches = model.prepare_patches("cs00_00.s", &entries).unwrap();

        let mut first = encode_cp932_text("      箱子").unwrap();
        first.push(0);
        assert_eq!(patches[0].replacement_stream, first);
        assert_eq!(patches[1].replacement_stream.first(), Some(&0x0D));
    }

    #[test]
    fn story_segment_newline_anchor_is_optional_during_reflow() {
        let model = logical(&[
            encoded_stream(&[("      箱", &[])]),
            encoded_stream(&[("", &[0x0D]), (" コンピュータ", &[0x01])]),
        ]);
        let mut entries = model.translation_entries("cs00_00.s");
        entries[0].message = "箱子 電脳".to_owned();
        assert!(model.prepare_patches("cs00_00.s", &entries).is_ok());
    }

    #[test]
    fn wait_marker_is_a_fixed_segment_anchor() {
        let model = logical(&[
            encoded_stream(&[("質問", &[0x02])]),
            encoded_stream(&[("回答", &[0x01])]),
        ]);
        let mut entries = model.translation_entries("cs00_00.s");
        assert_eq!(entries[0].scr_msg, "質問[[WAIT]]回答");
        entries[0].message = "問い[[WAIT]]答え".to_owned();
        let patches = model.prepare_patches("cs00_00.s", &entries).unwrap();
        assert!(patches[0].replacement_stream.ends_with(&[0x02, 0]));
        assert!(patches[1].replacement_stream.ends_with(&[0x01, 0]));
    }

    #[test]
    fn progressive_quote_animation_is_forced_into_one_page() {
        let fragments = ["「", "・", "・", "・", "」"];
        let mut streams: Vec<_> = fragments
            .iter()
            .map(|fragment| encoded_stream(&[(fragment, &[])]))
            .collect();
        streams[4] = encoded_stream(&[("」", &[0x01])]);
        let (_, candidates) = fixture(&streams);
        let mut edges = vec![false; candidates.len() - 1];
        assert_eq!(mark_progressive_quote_chains(&candidates, &mut edges), 1);
        assert!(edges.into_iter().all(|edge| edge));
    }

    #[test]
    fn changed_metadata_and_page_markers_are_rejected() {
        let model = logical(&[encoded_stream(&[("原文", &[0x01])])]);
        let mut entries = model.translation_entries("cs00_00.s");
        entries[0].segments[0].instruction_offset += 1;
        assert!(model
            .prepare_patches("cs00_00.s", &entries)
            .unwrap_err()
            .to_string()
            .contains("metadata"));

        let mut entries = model.translation_entries("cs00_00.s");
        entries[0].message = "前[[PAGE]]後".to_owned();
        assert!(model
            .prepare_patches("cs00_00.s", &entries)
            .unwrap_err()
            .to_string()
            .contains("page boundaries"));
    }
}
