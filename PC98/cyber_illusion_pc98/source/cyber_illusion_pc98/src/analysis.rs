//! 16-bit control-flow analysis for expanded Cyber Illusion modules.

use crate::Result;
use iced_x86::{
    Decoder, DecoderOptions, FlowControl, Instruction, InstructionInfoFactory, OpAccess, OpKind,
    Register,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};

pub const LOAD_BIAS: usize = 0x0100;
pub const ENTRY_RUNTIME_OFFSET: u16 = 0x0300;
pub const ENTRY_OUTPUT_OFFSET: usize = 0x0200;
pub const SLOT54_RUNTIME_OFFSET: u16 = 0x01E8;
pub const MENU_RENDERER_RUNTIME_OFFSET: u16 = 0x3660;
const PS1SYS_API_TABLE_BASE: u16 = 0x0110;
const PS1SYS_API_TABLE_END: u16 = 0x0230;
/// TEXT00's 0x27A5 near call reaches output 0x4ACE (runtime S:4BCE).
/// The callee pushes SI at 0x4AD2; both returns restore it: `pop si` at
/// 0x4B0F before `leave; ret` at 0x4B11-0x4B12, and `pop si` at 0x4B4D
/// before `leave; ret` at 0x4B4F-0x4B50.
const SI_PRESERVING_NEAR_CALL_RUNTIME: u16 = 0x4ACE + LOAD_BIAS as u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowKind {
    FallThrough,
    NearCall,
    NearJump,
    ConditionalBranch,
    Return,
    IndirectCall,
    IndirectBranch,
    FarCall,
    FarJump,
    Interrupt,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    FallThrough,
    CallTarget,
    BranchTarget,
    IndirectTableTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionRecord {
    /// Root of the reachable near-call context, as an expanded-output offset.
    pub function_entry_output_offset: usize,
    /// Instruction start in the expanded output buffer.
    pub output_offset: usize,
    /// Runtime CS offset after the module is loaded at `S:0100`.
    pub runtime_ip: u16,
    pub len: u8,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub flow: FlowKind,
    /// Direct near target in the 16-bit runtime code segment.
    pub near_target_runtime: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ControlFlowEdge {
    pub function_entry_output_offset: usize,
    pub from_output_offset: usize,
    pub to_output_offset: usize,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AnalysisDiagnostic {
    pub function_entry_output_offset: usize,
    pub output_offset: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PushExpr {
    Immediate(u16),
    Register(String),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushArg {
    pub output_offset: usize,
    pub expression: PushExpr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserCallSite {
    pub function_entry_output_offset: usize,
    pub code_output_offset: usize,
    /// The three adjacent pushes in source order: window, segment, pointer.
    ///
    /// Concrete evidence: TEXT00 output `+0x0F81` has `push 1`, `+0x0F83`
    /// has `push DS`, `+0x0F84` has `push 0x6536`, and the slot-54 far call
    /// begins at `+0x0F87`. `sub_11A49` reads the far pointer at `[BP+6]` and
    /// the window at `[BP+0x0A]`, matching this push order; `S:6536` maps to
    /// expanded-output offset `0x6436` after subtracting the `S:0100` load bias.
    pub abi_pushes: [Option<PushArg>; 3],
    pub window_expr: Option<PushExpr>,
    pub segment_expr: Option<PushExpr>,
    pub pointer_expr: Option<PushExpr>,
    pub runtime_pointer: Option<u16>,
    /// `runtime_pointer - LOAD_BIAS`, only when the segment is proven to be DS.
    pub output_offset: Option<usize>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StaticPointerSource {
    ImmediatePush,
    DsAbsoluteTable {
        register: String,
        definition_output_offset: usize,
        table_runtime_offset: u16,
        table_output_offset: usize,
        table_entry_output_offset: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticTextTargetPath {
    pub push_output_offsets: [usize; 3],
    pub definition_offsets: Vec<usize>,
    /// Instruction output offsets along this concrete route, in code order.
    pub path_offsets: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticTextTarget {
    pub function_entry_output_offset: usize,
    pub code_callsite_output_offset: usize,
    pub window: Option<u16>,
    pub window_expr: Option<PushExpr>,
    pub segment: String,
    pub runtime_pointer: u16,
    pub output_offset: usize,
    pub pointer_expr: PushExpr,
    pub pointer_source: StaticPointerSource,
    /// Canonical evidence from the first route; `evidence_paths` retains all routes.
    pub push_output_offsets: [usize; 3],
    pub definition_offsets: Vec<usize>,
    pub path_offsets: Vec<usize>,
    /// Raw bytes before the NUL terminator. These remain evidence, not translation entries.
    pub raw_text: Vec<u8>,
    pub evidence_paths: Vec<StaticTextTargetPath>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub entry_output_offset: usize,
    pub instructions: Vec<InstructionRecord>,
    pub edges: Vec<ControlFlowEdge>,
    pub parser_calls: Vec<ParserCallSite>,
    pub resolved_indirect_jump_tables: Vec<ResolvedIndirectJumpTable>,
    pub diagnostics: Vec<AnalysisDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedIndirectJumpTable {
    pub function_entry_output_offset: usize,
    pub jump_output_offset: usize,
    /// The segment override used by the jump instruction (`CS` for the supported form).
    pub table_segment: String,
    /// Runtime CS offset encoded by the instruction's disp16.
    pub table_runtime_offset: u16,
    /// Expanded-output location after subtracting the module's `S:0100` load bias.
    pub table_output_offset: usize,
    pub entry_size: u8,
    /// Inclusive upper bound after index normalization; entry_count is bound + 1.
    pub max_index: u16,
    pub entry_count: usize,
    /// Recognized source normalization (`SUB AX,imm` or `DEC AX`), if present.
    pub index_bias: Option<u16>,
    pub bound_compare_output_offset: usize,
    pub bound_branch_output_offset: usize,
    pub default_target_runtime_offset: u16,
    /// Load/normalize, compare, conditional bound, shift, exchange, and jump instruction offsets.
    pub source_instruction_offsets: Vec<usize>,
    pub entries: Vec<ResolvedJumpTableEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedJumpTableEntry {
    pub index: usize,
    pub table_entry_output_offset: usize,
    pub runtime_target_offset: u16,
    pub output_target_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuPointerEntry {
    /// Signed BP displacement encoded by `C7 46 disp8 imm16`.
    pub local_bp_displacement: i8,
    pub runtime_pointer: u16,
    pub output_offset: Option<usize>,
    pub nul_terminated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuTableCandidate {
    pub function_entry_output_offset: usize,
    pub renderer_call_output_offset: usize,
    pub renderer_runtime_target: u16,
    pub entries: Vec<MenuPointerEntry>,
    /// Four pointers with in-range NUL-terminated targets is structural evidence,
    /// not by itself proof that the renderer consumes them as menu text.
    pub complete: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WorkItem {
    function_entry: usize,
    output_offset: usize,
}

/// Analyze reachable 16-bit code starting at runtime `S:0300` (output `+0x200`).
///
/// Direct near calls/branches and verified bounded `CS:[BX+disp16]` near-jump
/// tables are followed. The indirect far call through `DS:01E8` is recorded as
/// PS1SYS slot 54 only when its bytes are exactly `FF 1E E8 01`. Other indirect
/// control transfers remain diagnostic boundaries with no guessed targets.
pub fn analyze_module(expanded: &[u8]) -> Result<AnalysisReport> {
    if expanded.len() <= ENTRY_OUTPUT_OFFSET {
        return Err(format!(
            "expanded module is too short for entry output offset 0x{ENTRY_OUTPUT_OFFSET:X}"
        ));
    }

    let mut instruction_map = BTreeMap::<(usize, usize), InstructionRecord>::new();
    let mut edge_set = BTreeSet::<ControlFlowEdge>::new();
    let mut diagnostic_set = BTreeSet::<AnalysisDiagnostic>::new();
    let mut parser_keys = BTreeSet::<(usize, usize)>::new();
    let mut indirect_jump_tables = BTreeMap::<(usize, usize), ResolvedIndirectJumpTable>::new();
    let mut byte_owners = vec![None::<usize>; expanded.len()];
    let mut worklist = VecDeque::new();
    let mut visited = HashSet::new();

    worklist.push_back(WorkItem {
        function_entry: ENTRY_OUTPUT_OFFSET,
        output_offset: ENTRY_OUTPUT_OFFSET,
    });

    while let Some(item) = worklist.pop_front() {
        if !visited.insert(item) {
            continue;
        }

        if item.output_offset >= expanded.len() {
            diagnostic_set.insert(diagnostic(
                item.function_entry,
                Some(item.output_offset),
                "reachable code offset is outside expanded module".to_owned(),
            ));
            continue;
        }

        if let Some(owner) = byte_owners[item.output_offset] {
            if owner != item.output_offset {
                diagnostic_set.insert(diagnostic(
                    item.function_entry,
                    Some(item.output_offset),
                    format!("control-flow target enters instruction at output 0x{owner:X}"),
                ));
                continue;
            }
        }

        let runtime_ip = match output_to_runtime_ip(item.output_offset) {
            Some(ip) => ip,
            None => {
                diagnostic_set.insert(diagnostic(
                    item.function_entry,
                    Some(item.output_offset),
                    "expanded output offset cannot be represented as a 16-bit runtime IP"
                        .to_owned(),
                ));
                continue;
            }
        };

        let instruction = decode_one(
            expanded,
            item.function_entry,
            item.output_offset,
            runtime_ip,
        );
        let Some((record, decoded)) = instruction else {
            diagnostic_set.insert(diagnostic(
                item.function_entry,
                Some(item.output_offset),
                "invalid or truncated 16-bit instruction".to_owned(),
            ));
            continue;
        };

        let instruction_end = match item.output_offset.checked_add(usize::from(record.len)) {
            Some(end) if end <= expanded.len() => end,
            _ => {
                diagnostic_set.insert(diagnostic(
                    item.function_entry,
                    Some(item.output_offset),
                    "decoded instruction extends past expanded module".to_owned(),
                ));
                continue;
            }
        };

        let overlapping = (item.output_offset..instruction_end).find_map(|offset| {
            byte_owners[offset]
                .filter(|&owner| owner != item.output_offset)
                .map(|owner| (offset, owner))
        });
        if let Some((offset, owner)) = overlapping {
            diagnostic_set.insert(diagnostic(
                item.function_entry,
                Some(item.output_offset),
                format!(
                    "instruction bytes overlap decode at output 0x{owner:X} (overlap at 0x{offset:X})"
                ),
            ));
            continue;
        }

        for offset in item.output_offset..instruction_end {
            byte_owners[offset] = Some(item.output_offset);
        }

        let record_key = (item.function_entry, item.output_offset);
        instruction_map
            .entry(record_key)
            .or_insert_with(|| record.clone());

        let next_offset = instruction_end;
        let direct_near_target = record.near_target_runtime;
        let flow = decoded.flow_control();

        match flow {
            FlowControl::Call if decoded.is_call_near() => {
                if let Some(target) = direct_near_target {
                    enqueue_runtime_target(
                        target,
                        item,
                        item.output_offset,
                        EdgeKind::CallTarget,
                        expanded.len(),
                        &mut edge_set,
                        &mut worklist,
                        &mut diagnostic_set,
                    );
                } else {
                    diagnostic_set.insert(diagnostic(
                        item.function_entry,
                        Some(item.output_offset),
                        "near call has no direct 16-bit target".to_owned(),
                    ));
                }
                enqueue_fallthrough(
                    item,
                    item.output_offset,
                    next_offset,
                    expanded.len(),
                    &mut edge_set,
                    &mut worklist,
                    &mut diagnostic_set,
                );
            }
            FlowControl::Call => {
                if decoded.is_call_far() {
                    diagnostic_set.insert(diagnostic(
                        item.function_entry,
                        Some(item.output_offset),
                        "direct far call target is not followed by this near-CFG analysis"
                            .to_owned(),
                    ));
                } else {
                    diagnostic_set.insert(diagnostic(
                        item.function_entry,
                        Some(item.output_offset),
                        "call target is not a supported direct near address".to_owned(),
                    ));
                }
                enqueue_fallthrough(
                    item,
                    item.output_offset,
                    next_offset,
                    expanded.len(),
                    &mut edge_set,
                    &mut worklist,
                    &mut diagnostic_set,
                );
            }
            FlowControl::IndirectCall => {
                if is_slot54_call(&decoded, &record.bytes) {
                    parser_keys.insert((item.function_entry, item.output_offset));
                } else if !is_ps1sys_api_call(&decoded, &record.bytes) {
                    diagnostic_set.insert(diagnostic(
                        item.function_entry,
                        Some(item.output_offset),
                        "indirect call target is unresolved; no target was guessed".to_owned(),
                    ));
                }
                enqueue_fallthrough(
                    item,
                    item.output_offset,
                    next_offset,
                    expanded.len(),
                    &mut edge_set,
                    &mut worklist,
                    &mut diagnostic_set,
                );
            }
            FlowControl::UnconditionalBranch if decoded.is_jmp_short_or_near() => {
                if let Some(target) = direct_near_target {
                    enqueue_runtime_target(
                        target,
                        item,
                        item.output_offset,
                        EdgeKind::BranchTarget,
                        expanded.len(),
                        &mut edge_set,
                        &mut worklist,
                        &mut diagnostic_set,
                    );
                } else {
                    diagnostic_set.insert(diagnostic(
                        item.function_entry,
                        Some(item.output_offset),
                        "near jump has no direct 16-bit target".to_owned(),
                    ));
                }
            }
            FlowControl::UnconditionalBranch => {
                diagnostic_set.insert(diagnostic(
                    item.function_entry,
                    Some(item.output_offset),
                    if decoded.is_jmp_far() {
                        "far jump target is not followed by this near-CFG analysis".to_owned()
                    } else {
                        "unconditional branch target is not a direct near address".to_owned()
                    },
                ));
            }
            FlowControl::IndirectBranch => {
                match resolve_bounded_near_jump_table(
                    expanded,
                    item,
                    &record,
                    &instruction_map,
                    &edge_set,
                    &byte_owners,
                ) {
                    Ok(table) => {
                        for entry in &table.entries {
                            edge_set.insert(ControlFlowEdge {
                                function_entry_output_offset: item.function_entry,
                                from_output_offset: item.output_offset,
                                to_output_offset: entry.output_target_offset,
                                kind: EdgeKind::IndirectTableTarget,
                            });
                            worklist.push_back(WorkItem {
                                function_entry: item.function_entry,
                                output_offset: entry.output_target_offset,
                            });
                        }
                        indirect_jump_tables
                            .insert((item.function_entry, item.output_offset), table);
                    }
                    Err(reason) => {
                        diagnostic_set.insert(diagnostic(
                            item.function_entry,
                            Some(item.output_offset),
                            format!(
                                "indirect jump target is unresolved; no target was guessed ({reason})"
                            ),
                        ));
                    }
                }
            }
            FlowControl::ConditionalBranch => {
                if let Some(target) = direct_near_target {
                    enqueue_runtime_target(
                        target,
                        item,
                        item.output_offset,
                        EdgeKind::BranchTarget,
                        expanded.len(),
                        &mut edge_set,
                        &mut worklist,
                        &mut diagnostic_set,
                    );
                } else {
                    diagnostic_set.insert(diagnostic(
                        item.function_entry,
                        Some(item.output_offset),
                        "conditional branch has no direct 16-bit target".to_owned(),
                    ));
                }
                enqueue_fallthrough(
                    item,
                    item.output_offset,
                    next_offset,
                    expanded.len(),
                    &mut edge_set,
                    &mut worklist,
                    &mut diagnostic_set,
                );
            }
            FlowControl::Return => {}
            FlowControl::Interrupt => enqueue_fallthrough(
                item,
                item.output_offset,
                next_offset,
                expanded.len(),
                &mut edge_set,
                &mut worklist,
                &mut diagnostic_set,
            ),
            FlowControl::XbeginXabortXend => {
                diagnostic_set.insert(diagnostic(
                    item.function_entry,
                    Some(item.output_offset),
                    "transactional branch is outside the supported 16-bit CFG cases".to_owned(),
                ));
                enqueue_fallthrough(
                    item,
                    item.output_offset,
                    next_offset,
                    expanded.len(),
                    &mut edge_set,
                    &mut worklist,
                    &mut diagnostic_set,
                );
            }
            FlowControl::Exception => {
                diagnostic_set.insert(diagnostic(
                    item.function_entry,
                    Some(item.output_offset),
                    "invalid or exception instruction ends this control-flow path".to_owned(),
                ));
            }
            FlowControl::Next => enqueue_fallthrough(
                item,
                item.output_offset,
                next_offset,
                expanded.len(),
                &mut edge_set,
                &mut worklist,
                &mut diagnostic_set,
            ),
        };
    }

    let instructions: Vec<_> = instruction_map.into_values().collect();
    let edges: Vec<_> = edge_set.into_iter().collect();
    let resolved_indirect_jump_tables = indirect_jump_tables.into_values().collect();
    let mut parser_calls = Vec::with_capacity(parser_keys.len());
    for (function_entry, code_offset) in parser_keys {
        parser_calls.push(resolve_parser_call(
            function_entry,
            code_offset,
            expanded.len(),
            &instructions,
            &edges,
        ));
    }
    parser_calls.sort_by_key(|call| (call.function_entry_output_offset, call.code_output_offset));

    Ok(AnalysisReport {
        entry_output_offset: ENTRY_OUTPUT_OFFSET,
        instructions,
        edges,
        parser_calls,
        resolved_indirect_jump_tables,
        diagnostics: diagnostic_set.into_iter().collect(),
    })
}

/// Resolve every reachable static text target for one slot-54 callsite.
///
/// All predecessor routes must have a proven `window, DS, offset` push sequence.
/// Immediate offsets are accepted directly. A pushed `SI` is resolved only when
/// every reaching definition is a `mov si,[absolute DS address]` table load whose
/// table entry and NUL-terminated target are in the expanded module. Any dynamic,
/// ambiguous, truncated, or out-of-range route returns an error instead of being
/// silently omitted or interpreted as a guessed string.
pub fn resolve_slot54_targets(
    expanded: &[u8],
    report: &AnalysisReport,
    call: &ParserCallSite,
) -> Result<Vec<StaticTextTarget>> {
    let call_instruction = report
        .instructions
        .iter()
        .find(|instruction| {
            instruction.function_entry_output_offset == call.function_entry_output_offset
                && instruction.output_offset == call.code_output_offset
        })
        .ok_or_else(|| {
            format!(
                "slot-54 callsite 0x{:X} is missing from the reachable instruction report",
                call.code_output_offset
            )
        })?;
    let decoded_call = decode_record(call_instruction);
    if !is_slot54_call(&decoded_call, &call_instruction.bytes) {
        return Err(format!(
            "instruction at output 0x{:X} is not the confirmed slot-54 far call",
            call.code_output_offset
        ));
    }

    let push_paths = enumerate_abi_push_paths(report, call)?;
    if push_paths.is_empty() {
        return Err(format!(
            "slot-54 call at output 0x{:X} has no complete ABI push path",
            call.code_output_offset
        ));
    }

    let mut targets = Vec::<StaticTextTarget>::new();
    for path in push_paths {
        let pushes = path.pushes;
        let window_expr = pushes[0].expression.clone();
        let window = match &window_expr {
            PushExpr::Immediate(value) if *value <= 3 => Some(*value),
            PushExpr::Immediate(value) => {
                return Err(format!(
                    "slot-54 window {value} at output 0x{:X} is outside 0..=3",
                    pushes[0].output_offset
                ));
            }
            _ => None,
        };

        if pushes[1].expression != PushExpr::Register("DS".to_owned()) {
            return Err(format!(
                "slot-54 segment push at output 0x{:X} is not proven to be DS",
                pushes[1].output_offset
            ));
        }

        let pointer_expr = pushes[2].expression.clone();
        let pointer_resolutions = match &pointer_expr {
            PushExpr::Immediate(pointer) => vec![PointerResolution {
                runtime_pointer: *pointer,
                source: StaticPointerSource::ImmediatePush,
                definition_offsets: vec![pushes[2].output_offset],
                definition_path_offsets: Vec::new(),
            }],
            PushExpr::Register(register) if register == "SI" => resolve_si_pointer_table(
                expanded,
                report,
                call.function_entry_output_offset,
                pushes[2].output_offset,
            )?,
            PushExpr::Register(register) => {
                return Err(format!(
                    "slot-54 pointer register {register} at output 0x{:X} has no supported static reaching-definition resolver",
                    pushes[2].output_offset
                ));
            }
            PushExpr::Other => {
                return Err(format!(
                    "slot-54 pointer at output 0x{:X} is not an immediate or resolved SI table value",
                    pushes[2].output_offset
                ));
            }
        };

        let push_output_offsets = [
            pushes[0].output_offset,
            pushes[1].output_offset,
            pushes[2].output_offset,
        ];
        for resolution in pointer_resolutions {
            let output_offset =
                runtime_to_output_offset(resolution.runtime_pointer, expanded.len()).ok_or_else(
                    || {
                        format!(
                        "slot-54 pointer S:{:04X} at output 0x{:X} is outside the expanded module",
                        resolution.runtime_pointer, call.code_output_offset
                    )
                    },
                )?;
            let nul_end = expanded
                .get(output_offset..)
                .and_then(|tail| tail.iter().position(|&byte| byte == 0))
                .ok_or_else(|| {
                    format!(
                        "slot-54 target S:{:04X} at output 0x{output_offset:X} has no in-buffer NUL terminator",
                        resolution.runtime_pointer
                    )
                })?;
            let raw_text = expanded[output_offset..output_offset + nul_end].to_vec();

            let mut path_offsets = resolution.definition_path_offsets.clone();
            if path_offsets.is_empty() {
                path_offsets = path.path_offsets.clone();
            } else {
                let pointer_push_index = path
                    .path_offsets
                    .iter()
                    .position(|&offset| offset == pushes[2].output_offset)
                    .ok_or_else(|| "pointer push is absent from ABI path evidence".to_owned())?;
                path_offsets.extend_from_slice(&path.path_offsets[pointer_push_index + 1..]);
            }
            let evidence = StaticTextTargetPath {
                push_output_offsets,
                definition_offsets: resolution.definition_offsets.clone(),
                path_offsets: path_offsets.clone(),
            };

            if let Some(existing) = targets.iter_mut().find(|target| {
                target.function_entry_output_offset == call.function_entry_output_offset
                    && target.code_callsite_output_offset == call.code_output_offset
                    && target.window_expr == Some(window_expr.clone())
                    && target.runtime_pointer == resolution.runtime_pointer
                    && target.pointer_expr == pointer_expr
                    && target.pointer_source == resolution.source
            }) {
                if !existing.evidence_paths.contains(&evidence) {
                    existing.evidence_paths.push(evidence);
                }
                for definition in &resolution.definition_offsets {
                    if !existing.definition_offsets.contains(definition) {
                        existing.definition_offsets.push(*definition);
                    }
                }
                for offset in &path_offsets {
                    if !existing.path_offsets.contains(offset) {
                        existing.path_offsets.push(*offset);
                    }
                }
            } else {
                targets.push(StaticTextTarget {
                    function_entry_output_offset: call.function_entry_output_offset,
                    code_callsite_output_offset: call.code_output_offset,
                    window,
                    window_expr: Some(window_expr.clone()),
                    segment: "DS".to_owned(),
                    runtime_pointer: resolution.runtime_pointer,
                    output_offset,
                    pointer_expr: pointer_expr.clone(),
                    pointer_source: resolution.source,
                    push_output_offsets,
                    definition_offsets: resolution.definition_offsets,
                    path_offsets,
                    raw_text,
                    evidence_paths: vec![evidence],
                });
            }
        }
    }

    targets.sort_by_key(|target| {
        (
            target.code_callsite_output_offset,
            target.runtime_pointer,
            target.output_offset,
        )
    });
    if targets.is_empty() {
        return Err(format!(
            "slot-54 call at output 0x{:X} produced no verified static text targets",
            call.code_output_offset
        ));
    }
    Ok(targets)
}

/// Find local menu-pointer table candidates tied to a reachable direct call to
/// the known renderer target `0x3660`.
///
/// This deliberately reports address evidence and diagnostics only. A
/// NUL-terminated target is not promoted to a translatable string unless a
/// separate script-record parser establishes its role and boundaries.
pub fn find_local_menu_tables(expanded: &[u8], report: &AnalysisReport) -> Vec<MenuTableCandidate> {
    let mut by_function = BTreeMap::<usize, Vec<&InstructionRecord>>::new();
    for instruction in &report.instructions {
        by_function
            .entry(instruction.function_entry_output_offset)
            .or_default()
            .push(instruction);
    }

    let mut candidates = Vec::new();
    for renderer in &report.instructions {
        if renderer.flow != FlowKind::NearCall
            || renderer.near_target_runtime != Some(MENU_RENDERER_RUNTIME_OFFSET)
        {
            continue;
        }

        let function_entry = renderer.function_entry_output_offset;
        let function_instructions = by_function
            .get(&function_entry)
            .cloned()
            .unwrap_or_default();
        let mut stores = BTreeMap::<i8, Vec<MenuPointerEntry>>::new();
        let mut diagnostics = Vec::new();

        for instruction in function_instructions {
            let Some((displacement, runtime_pointer)) = local_menu_pointer_store(instruction)
            else {
                continue;
            };
            let instruction_end = instruction.output_offset + usize::from(instruction.len);
            if !can_reach_in_function(
                report,
                function_entry,
                instruction_end,
                renderer.output_offset,
            ) {
                continue;
            }

            let output_offset = runtime_pointer
                .checked_sub(LOAD_BIAS as u16)
                .map(usize::from)
                .filter(|&offset| offset < expanded.len());
            let nul_terminated = output_offset
                .and_then(|offset| expanded.get(offset..))
                .is_some_and(|tail| tail.contains(&0));

            if output_offset.is_none() {
                diagnostics.push(format!(
                    "local BP[{displacement}] target S:{runtime_pointer:04X} is outside the expanded module"
                ));
            } else if !nul_terminated {
                diagnostics.push(format!(
                    "local BP[{displacement}] target S:{runtime_pointer:04X} has no in-buffer NUL terminator"
                ));
            }

            stores
                .entry(displacement)
                .or_default()
                .push(MenuPointerEntry {
                    local_bp_displacement: displacement,
                    runtime_pointer,
                    output_offset,
                    nul_terminated,
                });
        }

        let expected = [-12i8, -10, -8, -6];
        let mut entries = Vec::new();
        for displacement in expected {
            match stores.remove(&displacement) {
                Some(mut values) if values.len() == 1 => entries.push(values.remove(0)),
                Some(values) if values.len() > 1 => diagnostics.push(format!(
                    "local BP[{displacement}] has {} reachable pointer stores on paths to this renderer; value is ambiguous",
                    values.len()
                )),
                _ => diagnostics.push(format!(
                    "no reachable C7 46 store found for local BP[{displacement}]"
                )),
            }
        }
        for (displacement, values) in stores {
            diagnostics.push(format!(
                "additional local pointer store BP[{displacement}] found ({} candidate values)",
                values.len()
            ));
        }

        let complete = entries.len() == expected.len()
            && entries
                .iter()
                .all(|entry| entry.output_offset.is_some() && entry.nul_terminated);
        if complete {
            diagnostics.push(
                "four in-range NUL-terminated pointer targets are present; renderer dataflow and text boundaries remain unproven".to_owned(),
            );
        }

        candidates.push(MenuTableCandidate {
            function_entry_output_offset: function_entry,
            renderer_call_output_offset: renderer.output_offset,
            renderer_runtime_target: MENU_RENDERER_RUNTIME_OFFSET,
            entries,
            complete,
            diagnostics,
        });
    }

    candidates.sort_by_key(|candidate| {
        (
            candidate.function_entry_output_offset,
            candidate.renderer_call_output_offset,
        )
    });
    candidates
}

fn decode_one(
    expanded: &[u8],
    function_entry_output_offset: usize,
    output_offset: usize,
    runtime_ip: u16,
) -> Option<(InstructionRecord, Instruction)> {
    let code = expanded.get(output_offset..)?;
    let mut decoder = Decoder::with_ip(16, code, u64::from(runtime_ip), DecoderOptions::NONE);
    let instruction = decoder.decode();
    let len = usize::try_from(instruction.len()).ok()?;
    if instruction.is_invalid() || len == 0 || len > code.len() || len > u8::MAX as usize {
        return None;
    }
    let end = output_offset.checked_add(len)?;
    let bytes = expanded.get(output_offset..end)?.to_vec();
    let near_target_runtime = direct_near_target(&instruction);
    let flow = flow_kind(&instruction);

    Some((
        InstructionRecord {
            function_entry_output_offset,
            output_offset,
            runtime_ip,
            len: len as u8,
            bytes,
            mnemonic: format!("{:?}", instruction.mnemonic()),
            flow,
            near_target_runtime,
        },
        instruction,
    ))
}

fn direct_near_target(instruction: &Instruction) -> Option<u16> {
    match instruction.op0_kind() {
        OpKind::NearBranch16 => u16::try_from(instruction.near_branch_target()).ok(),
        OpKind::NearBranch32 => u16::try_from(instruction.near_branch_target()).ok(),
        _ => None,
    }
}

fn flow_kind(instruction: &Instruction) -> FlowKind {
    match instruction.flow_control() {
        FlowControl::Next => FlowKind::FallThrough,
        FlowControl::UnconditionalBranch if instruction.is_jmp_short_or_near() => {
            FlowKind::NearJump
        }
        FlowControl::UnconditionalBranch => FlowKind::FarJump,
        FlowControl::IndirectBranch => FlowKind::IndirectBranch,
        FlowControl::ConditionalBranch => FlowKind::ConditionalBranch,
        FlowControl::Return => FlowKind::Return,
        FlowControl::Call if instruction.is_call_near() => FlowKind::NearCall,
        FlowControl::Call => FlowKind::FarCall,
        FlowControl::IndirectCall => FlowKind::IndirectCall,
        FlowControl::Interrupt => FlowKind::Interrupt,
        FlowControl::XbeginXabortXend | FlowControl::Exception => FlowKind::Other,
    }
}

fn is_slot54_call(instruction: &Instruction, bytes: &[u8]) -> bool {
    instruction.is_call_far_indirect()
        && bytes.len() == 4
        && bytes[0] == 0xFF
        && bytes[1] == 0x1E
        && bytes[2] == SLOT54_RUNTIME_OFFSET as u8
        && bytes[3] == (SLOT54_RUNTIME_OFFSET >> 8) as u8
}

/// The module loader copies 72 PS1SYS far-call pointers to S:0110..S:022F.
/// Calls through aligned words in that range are engine API calls; their body
/// is outside the script module, and the near CFG should continue at fallthrough.
fn is_ps1sys_api_call(instruction: &Instruction, bytes: &[u8]) -> bool {
    if !instruction.is_call_far_indirect()
        || bytes.len() != 4
        || bytes[0] != 0xFF
        || bytes[1] != 0x1E
    {
        return false;
    }
    let slot = u16::from_le_bytes([bytes[2], bytes[3]]);
    (PS1SYS_API_TABLE_BASE..PS1SYS_API_TABLE_END).contains(&slot)
        && (slot - PS1SYS_API_TABLE_BASE) % 4 == 0
}

fn resolve_bounded_near_jump_table(
    expanded: &[u8],
    item: WorkItem,
    jump: &InstructionRecord,
    instructions: &BTreeMap<(usize, usize), InstructionRecord>,
    edges: &BTreeSet<ControlFlowEdge>,
    byte_owners: &[Option<usize>],
) -> Result<ResolvedIndirectJumpTable> {
    if jump.bytes.len() != 5
        || jump.bytes[0] != 0x2E
        || jump.bytes[1] != 0xFF
        || jump.bytes[2] != 0xA7
        || jump.flow != FlowKind::IndirectBranch
    {
        return Err("unsupported indirect jump encoding; expected CS:[BX+disp16]".into());
    }
    let table_runtime_offset = u16::from_le_bytes([jump.bytes[3], jump.bytes[4]]);
    let table_output_offset = runtime_to_output_offset(table_runtime_offset, expanded.len())
        .ok_or_else(|| {
            format!("CS jump table S:{table_runtime_offset:04X} is outside the module")
        })?;

    let xchg = unique_fallthrough_predecessor(
        item.function_entry,
        jump.output_offset,
        instructions,
        edges,
    )?;
    require_single_incoming_edge(
        item.function_entry,
        jump.output_offset,
        xchg.output_offset,
        EdgeKind::FallThrough,
        edges,
    )?;
    if xchg.bytes != [0x93] {
        return Err(format!(
            "CS:[BX+disp16] jump at 0x{:X} is not preceded by xchg AX,BX",
            jump.output_offset
        ));
    }
    let shift = unique_fallthrough_predecessor(
        item.function_entry,
        xchg.output_offset,
        instructions,
        edges,
    )?;
    require_single_incoming_edge(
        item.function_entry,
        xchg.output_offset,
        shift.output_offset,
        EdgeKind::FallThrough,
        edges,
    )?;
    if !is_shift_ax_left_one(&shift) {
        return Err(format!(
            "xchg AX,BX at 0x{:X} is not preceded by shl AX,1",
            xchg.output_offset
        ));
    }

    let conditional_predecessors = edges
        .iter()
        .filter(|edge| {
            edge.function_entry_output_offset == item.function_entry
                && edge.to_output_offset == shift.output_offset
        })
        .filter_map(|edge| {
            let instruction = instructions.get(&(item.function_entry, edge.from_output_offset))?;
            (instruction.flow == FlowKind::ConditionalBranch)
                .then_some((edge.kind, instruction.clone()))
        })
        .collect::<Vec<_>>();
    if conditional_predecessors.len() != 1 {
        return Err(format!(
            "shl AX,1 at 0x{:X} has {} conditional bound branches",
            shift.output_offset,
            conditional_predecessors.len()
        ));
    }
    let (dispatch_edge, bound_branch) = conditional_predecessors.into_iter().next().unwrap();
    require_single_incoming_edge(
        item.function_entry,
        shift.output_offset,
        bound_branch.output_offset,
        dispatch_edge,
        edges,
    )?;
    let (default_target_runtime_offset, default_jump) = match bound_branch.mnemonic.as_str() {
        "Ja" if dispatch_edge == EdgeKind::FallThrough => (
            bound_branch.near_target_runtime.ok_or_else(|| {
                format!(
                    "JA at 0x{:X} has no direct default target",
                    bound_branch.output_offset
                )
            })?,
            None,
        ),
        "Jbe" if dispatch_edge == EdgeKind::BranchTarget => {
            let target = bound_branch.near_target_runtime.ok_or_else(|| {
                format!(
                    "JBE at 0x{:X} has no direct table target",
                    bound_branch.output_offset
                )
            })?;
            if runtime_to_output_offset(target, expanded.len()) != Some(shift.output_offset) {
                return Err(format!(
                    "JBE at 0x{:X} does not branch to shl AX,1 at 0x{:X}",
                    bound_branch.output_offset, shift.output_offset
                ));
            }
            let default_offset = bound_branch.output_offset + usize::from(bound_branch.len);
            if !edges.iter().any(|edge| {
                edge.function_entry_output_offset == item.function_entry
                    && edge.from_output_offset == bound_branch.output_offset
                    && edge.to_output_offset == default_offset
                    && edge.kind == EdgeKind::FallThrough
            }) {
                return Err(format!(
                    "JBE at 0x{:X} has no fall-through default path",
                    bound_branch.output_offset
                ));
            }
            let fallthrough = instructions
                .get(&(item.function_entry, default_offset))
                .cloned()
                .ok_or_else(|| {
                    format!("JBE default instruction at 0x{default_offset:X} is missing")
                })?;
            if fallthrough.flow != FlowKind::NearJump {
                return Err(format!(
                    "JBE default path at 0x{:X} is not an unconditional near JMP",
                    fallthrough.output_offset
                ));
            }
            (
                fallthrough.near_target_runtime.ok_or_else(|| {
                    format!(
                        "default JMP at 0x{:X} has no direct target",
                        fallthrough.output_offset
                    )
                })?,
                Some(fallthrough),
            )
        }
        _ => {
            return Err(format!(
                "unsupported bound branch at 0x{:X}; expected JA-default or JBE-table form",
                bound_branch.output_offset
            ));
        }
    };
    if runtime_to_output_offset(default_target_runtime_offset, expanded.len()).is_none() {
        return Err(format!(
            "bound default target S:{default_target_runtime_offset:04X} is outside module"
        ));
    }

    let compare = unique_fallthrough_predecessor(
        item.function_entry,
        bound_branch.output_offset,
        instructions,
        edges,
    )?;
    require_single_incoming_edge(
        item.function_entry,
        bound_branch.output_offset,
        compare.output_offset,
        EdgeKind::FallThrough,
        edges,
    )?;
    let decoded_compare = decode_record(&compare);
    if decoded_compare.mnemonic() != iced_x86::Mnemonic::Cmp
        || decoded_compare.op0_kind() != OpKind::Register
        || decoded_compare.op0_register() != Register::AX
    {
        return Err(format!(
            "bound branch at 0x{:X} is not preceded by cmp AX,imm",
            bound_branch.output_offset
        ));
    }
    let max_index = immediate_u16(&decoded_compare, 1).ok_or_else(|| {
        format!(
            "cmp AX at 0x{:X} has no supported immediate bound",
            compare.output_offset
        )
    })?;
    let entry_count = usize::from(max_index) + 1;

    let index_adjust = unique_fallthrough_predecessor(
        item.function_entry,
        compare.output_offset,
        instructions,
        edges,
    )
    .ok();
    let index_bias = index_adjust.as_ref().and_then(index_normalization_bias);

    let table_bytes = entry_count
        .checked_mul(2)
        .ok_or_else(|| "jump-table byte length overflow".to_owned())?;
    let table_end = table_output_offset
        .checked_add(table_bytes)
        .ok_or_else(|| "jump-table end offset overflow".to_owned())?;
    if table_end > expanded.len() {
        return Err(format!(
            "jump table at output 0x{table_output_offset:X} with {entry_count} entries extends past module end"
        ));
    }
    if let Some((offset, owner)) = (table_output_offset..table_end).find_map(|offset| {
        byte_owners[offset]
            .map(|owner| (offset, owner))
            .filter(|(_, owner)| *owner != offset)
    }) {
        return Err(format!(
            "jump-table data at output 0x{offset:X} overlaps decoded instruction at 0x{owner:X}"
        ));
    }
    if (table_output_offset..table_end).any(|offset| byte_owners[offset].is_some()) {
        return Err(format!(
            "jump-table range 0x{table_output_offset:X}..0x{table_end:X} overlaps decoded code"
        ));
    }

    let mut entries = Vec::with_capacity(entry_count);
    for index in 0..entry_count {
        let table_entry_output_offset = table_output_offset + index * 2;
        let runtime_target_offset = u16::from_le_bytes([
            expanded[table_entry_output_offset],
            expanded[table_entry_output_offset + 1],
        ]);
        if runtime_target_offset < ENTRY_RUNTIME_OFFSET {
            return Err(format!(
                "jump-table entry {index} points before module entry at S:{runtime_target_offset:04X}"
            ));
        }
        let output_target_offset = runtime_to_output_offset(runtime_target_offset, expanded.len())
            .ok_or_else(|| {
                format!(
                    "jump-table entry {index} target S:{runtime_target_offset:04X} is outside module"
                )
            })?;
        if (table_output_offset..table_end).contains(&output_target_offset) {
            return Err(format!(
                "jump-table entry {index} targets table data at output 0x{output_target_offset:X}"
            ));
        }
        if let Some(owner) = byte_owners[output_target_offset] {
            if owner != output_target_offset {
                return Err(format!(
                    "jump-table target 0x{output_target_offset:X} enters instruction at 0x{owner:X}"
                ));
            }
        }
        if decode_one(
            expanded,
            item.function_entry,
            output_target_offset,
            runtime_target_offset,
        )
        .is_none()
        {
            return Err(format!(
                "jump-table target S:{runtime_target_offset:04X} does not begin a valid 16-bit instruction"
            ));
        }
        entries.push(ResolvedJumpTableEntry {
            index,
            table_entry_output_offset,
            runtime_target_offset,
            output_target_offset,
        });
    }

    let mut source_instruction_offsets = vec![
        compare.output_offset,
        bound_branch.output_offset,
        shift.output_offset,
        xchg.output_offset,
        jump.output_offset,
    ];
    if let Some(adjust) = index_adjust.as_ref().filter(|_| index_bias.is_some()) {
        source_instruction_offsets.push(adjust.output_offset);
    }
    if let Some(default) = &default_jump {
        source_instruction_offsets.push(default.output_offset);
    }
    source_instruction_offsets.sort_unstable();
    source_instruction_offsets.dedup();

    Ok(ResolvedIndirectJumpTable {
        function_entry_output_offset: item.function_entry,
        jump_output_offset: jump.output_offset,
        table_segment: "CS".to_owned(),
        table_runtime_offset,
        table_output_offset,
        entry_size: 2,
        max_index,
        entry_count,
        index_bias,
        bound_compare_output_offset: compare.output_offset,
        bound_branch_output_offset: bound_branch.output_offset,
        default_target_runtime_offset,
        source_instruction_offsets,
        entries,
    })
}

fn unique_fallthrough_predecessor(
    function_entry: usize,
    target: usize,
    instructions: &BTreeMap<(usize, usize), InstructionRecord>,
    edges: &BTreeSet<ControlFlowEdge>,
) -> Result<InstructionRecord> {
    let predecessors = edges
        .iter()
        .filter(|edge| {
            edge.function_entry_output_offset == function_entry
                && edge.to_output_offset == target
                && edge.kind == EdgeKind::FallThrough
        })
        .collect::<Vec<_>>();
    if predecessors.len() != 1 {
        return Err(format!(
            "output 0x{target:X} has {} fall-through predecessors, expected one",
            predecessors.len()
        ));
    }
    instructions
        .get(&(function_entry, predecessors[0].from_output_offset))
        .cloned()
        .ok_or_else(|| {
            format!(
                "fall-through predecessor 0x{:X} has no decoded instruction",
                predecessors[0].from_output_offset
            )
        })
}

fn require_single_incoming_edge(
    function_entry: usize,
    target: usize,
    expected_from: usize,
    expected_kind: EdgeKind,
    edges: &BTreeSet<ControlFlowEdge>,
) -> Result<()> {
    let incoming = edges
        .iter()
        .filter(|edge| {
            edge.function_entry_output_offset == function_entry && edge.to_output_offset == target
        })
        .collect::<Vec<_>>();
    if incoming.len() != 1
        || incoming[0].from_output_offset != expected_from
        || incoming[0].kind != expected_kind
    {
        return Err(format!(
            "indirect-table path at output 0x{target:X} has {} incoming edges instead of the proven {:?} edge from 0x{expected_from:X}",
            incoming.len(), expected_kind
        ));
    }
    Ok(())
}

fn immediate_u16(instruction: &Instruction, operand: u32) -> Option<u16> {
    match instruction.op_kind(operand) {
        OpKind::Immediate8 => Some(u16::from(instruction.immediate8())),
        OpKind::Immediate8to16 => Some(instruction.immediate8to16() as u16),
        OpKind::Immediate16 => Some(instruction.immediate16()),
        _ => None,
    }
}

fn is_shift_ax_left_one(record: &InstructionRecord) -> bool {
    (record.bytes == [0xD1, 0xE0] || record.bytes == [0xC1, 0xE0, 0x01]) && record.mnemonic == "Shl"
}

fn index_normalization_bias(record: &InstructionRecord) -> Option<u16> {
    let instruction = decode_record(record);
    match instruction.mnemonic() {
        iced_x86::Mnemonic::Dec
            if instruction.op0_kind() == OpKind::Register
                && instruction.op0_register() == Register::AX =>
        {
            Some(1)
        }
        iced_x86::Mnemonic::Sub
            if instruction.op0_kind() == OpKind::Register
                && instruction.op0_register() == Register::AX =>
        {
            immediate_u16(&instruction, 1)
        }
        _ => None,
    }
}

fn enqueue_runtime_target(
    runtime_target: u16,
    item: WorkItem,
    from_output_offset: usize,
    kind: EdgeKind,
    module_len: usize,
    edge_set: &mut BTreeSet<ControlFlowEdge>,
    worklist: &mut VecDeque<WorkItem>,
    diagnostic_set: &mut BTreeSet<AnalysisDiagnostic>,
) {
    let Some(target_output_offset) = runtime_to_output_offset(runtime_target, module_len) else {
        diagnostic_set.insert(diagnostic(
            item.function_entry,
            Some(from_output_offset),
            format!(
                "direct target S:{runtime_target:04X} is before the loaded module or outside expanded output"
            ),
        ));
        return;
    };

    edge_set.insert(ControlFlowEdge {
        function_entry_output_offset: item.function_entry,
        from_output_offset,
        to_output_offset: target_output_offset,
        kind,
    });
    let function_entry = if kind == EdgeKind::CallTarget {
        target_output_offset
    } else {
        item.function_entry
    };
    worklist.push_back(WorkItem {
        function_entry,
        output_offset: target_output_offset,
    });
}

fn enqueue_fallthrough(
    item: WorkItem,
    from_output_offset: usize,
    next_offset: usize,
    module_len: usize,
    edge_set: &mut BTreeSet<ControlFlowEdge>,
    worklist: &mut VecDeque<WorkItem>,
    diagnostic_set: &mut BTreeSet<AnalysisDiagnostic>,
) {
    if next_offset >= module_len {
        diagnostic_set.insert(diagnostic(
            item.function_entry,
            Some(from_output_offset),
            "fall-through reaches the end of expanded module".to_owned(),
        ));
        return;
    }
    edge_set.insert(ControlFlowEdge {
        function_entry_output_offset: item.function_entry,
        from_output_offset,
        to_output_offset: next_offset,
        kind: EdgeKind::FallThrough,
    });
    worklist.push_back(WorkItem {
        function_entry: item.function_entry,
        output_offset: next_offset,
    });
}

fn output_to_runtime_ip(output_offset: usize) -> Option<u16> {
    u16::try_from(output_offset.checked_add(LOAD_BIAS)?).ok()
}

fn runtime_to_output_offset(runtime_offset: u16, module_len: usize) -> Option<usize> {
    usize::from(runtime_offset)
        .checked_sub(LOAD_BIAS)
        .filter(|&offset| offset < module_len)
}

fn diagnostic(
    function_entry_output_offset: usize,
    output_offset: Option<usize>,
    message: String,
) -> AnalysisDiagnostic {
    AnalysisDiagnostic {
        function_entry_output_offset,
        output_offset,
        message,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AbiPushPath {
    pushes: [PushArg; 3],
    path_offsets: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PointerResolution {
    runtime_pointer: u16,
    source: StaticPointerSource,
    definition_offsets: Vec<usize>,
    /// For register-derived pointers, the path from a reaching definition to the pointer push.
    definition_path_offsets: Vec<usize>,
}

fn enumerate_abi_push_paths(
    report: &AnalysisReport,
    call: &ParserCallSite,
) -> Result<Vec<AbiPushPath>> {
    fn visit(
        report: &AnalysisReport,
        call: &ParserCallSite,
        cursor: usize,
        next_push: isize,
        pushes: &mut [Option<PushArg>; 3],
        reverse_path: &mut Vec<usize>,
        active: &mut HashSet<(usize, isize)>,
        found: &mut Vec<AbiPushPath>,
        errors: &mut Vec<String>,
        depth: usize,
    ) {
        if next_push < 0 {
            let (Some(window), Some(segment), Some(pointer)) =
                (pushes[0].clone(), pushes[1].clone(), pushes[2].clone())
            else {
                errors.push(format!(
                    "slot-54 call at output 0x{:X} has an incomplete ABI push sequence",
                    call.code_output_offset
                ));
                return;
            };
            let path_offsets = reverse_path.iter().rev().copied().collect();
            let candidate = AbiPushPath {
                pushes: [window, segment, pointer],
                path_offsets,
            };
            if !found.contains(&candidate) {
                found.push(candidate);
            }
            return;
        }
        if depth >= 32 {
            errors.push(format!(
                "slot-54 call at output 0x{:X} exceeds the ABI predecessor path limit",
                call.code_output_offset
            ));
            return;
        }
        if !active.insert((cursor, next_push)) {
            errors.push(format!(
                "slot-54 call at output 0x{:X} has a cycle while resolving ABI pushes",
                call.code_output_offset
            ));
            return;
        }

        let predecessors = report
            .edges
            .iter()
            .filter(|edge| {
                edge.function_entry_output_offset == call.function_entry_output_offset
                    && edge.to_output_offset == cursor
            })
            .collect::<Vec<_>>();
        if predecessors.is_empty() {
            errors.push(format!(
                "slot-54 call at output 0x{:X} has no predecessor for ABI push {}",
                call.code_output_offset,
                next_push + 1
            ));
        }

        for predecessor in predecessors {
            let Some(instruction) = report.instructions.iter().find(|instruction| {
                instruction.function_entry_output_offset == call.function_entry_output_offset
                    && instruction.output_offset == predecessor.from_output_offset
            }) else {
                errors.push(format!(
                    "slot-54 predecessor 0x{:X} has no decoded instruction",
                    predecessor.from_output_offset
                ));
                continue;
            };

            // A direct branch changes control flow but not the argument stack.
            // Follow it to the route that prepared the arguments.
            if matches!(
                instruction.flow,
                FlowKind::NearJump | FlowKind::ConditionalBranch
            ) {
                reverse_path.push(instruction.output_offset);
                visit(
                    report,
                    call,
                    instruction.output_offset,
                    next_push,
                    pushes,
                    reverse_path,
                    active,
                    found,
                    errors,
                    depth + 1,
                );
                reverse_path.pop();
                continue;
            }

            if instruction.mnemonic != "Push" {
                errors.push(format!(
                    "slot-54 path to 0x{:X} crosses unsupported {:?} instruction at 0x{:X} before ABI push {}",
                    call.code_output_offset,
                    instruction.flow,
                    instruction.output_offset,
                    next_push + 1
                ));
                continue;
            }

            let index = next_push as usize;
            pushes[index] = Some(PushArg {
                output_offset: instruction.output_offset,
                expression: push_expression(instruction),
            });
            reverse_path.push(instruction.output_offset);
            visit(
                report,
                call,
                instruction.output_offset,
                next_push - 1,
                pushes,
                reverse_path,
                active,
                found,
                errors,
                depth + 1,
            );
            reverse_path.pop();
            pushes[index] = None;
        }

        active.remove(&(cursor, next_push));
    }

    let mut found = Vec::new();
    let mut errors = Vec::new();
    let mut pushes = [None, None, None];
    let mut reverse_path = vec![call.code_output_offset];
    visit(
        report,
        call,
        call.code_output_offset,
        2,
        &mut pushes,
        &mut reverse_path,
        &mut HashSet::new(),
        &mut found,
        &mut errors,
        0,
    );
    if !errors.is_empty() {
        return Err(format!(
            "slot-54 ABI paths at output 0x{:X} are not all statically resolved: {}",
            call.code_output_offset,
            errors.join("; ")
        ));
    }
    Ok(found)
}

fn resolve_si_pointer_table(
    expanded: &[u8],
    report: &AnalysisReport,
    function_entry: usize,
    pointer_push_offset: usize,
) -> Result<Vec<PointerResolution>> {
    fn walk(
        expanded: &[u8],
        report: &AnalysisReport,
        function_entry: usize,
        cursor: usize,
        reverse_trace: &mut Vec<usize>,
        active: &mut HashSet<usize>,
        found: &mut Vec<PointerResolution>,
        errors: &mut Vec<String>,
        depth: usize,
    ) {
        if depth >= 128 {
            errors.push(format!(
                "SI reaching-definition trace at output 0x{cursor:X} exceeds its path limit"
            ));
            return;
        }
        if !active.insert(cursor) {
            errors.push(format!(
                "cycle while resolving SI reaching definitions at output 0x{cursor:X}"
            ));
            return;
        }

        let predecessors = report
            .edges
            .iter()
            .filter(|edge| {
                edge.function_entry_output_offset == function_entry
                    && edge.to_output_offset == cursor
            })
            .collect::<Vec<_>>();
        if predecessors.is_empty() {
            errors.push(format!(
                "SI at output 0x{cursor:X} has a path with no in-function reaching definition"
            ));
        }

        for predecessor in predecessors {
            let Some(instruction) = report.instructions.iter().find(|instruction| {
                instruction.function_entry_output_offset == function_entry
                    && instruction.output_offset == predecessor.from_output_offset
            }) else {
                errors.push(format!(
                    "SI predecessor 0x{:X} has no decoded instruction",
                    predecessor.from_output_offset
                ));
                continue;
            };
            let decoded = decode_record(instruction);
            let mut next_trace = reverse_trace.clone();
            next_trace.push(instruction.output_offset);

            // PS1SYS slot 54 saves and restores SI. TEXT00's other permitted
            // call is the verified near target S:4BCE (`0x4ACE` output): it
            // pushes SI at 0x4AD2 and restores it on both epilogues (0x4B0F,
            // 0x4B4D). No other call is treated as register-transparent.
            if is_slot54_call(&decoded, &instruction.bytes)
                || (instruction.flow == FlowKind::NearCall
                    && instruction.near_target_runtime == Some(SI_PRESERVING_NEAR_CALL_RUNTIME))
            {
                walk(
                    expanded,
                    report,
                    function_entry,
                    instruction.output_offset,
                    &mut next_trace,
                    active,
                    found,
                    errors,
                    depth + 1,
                );
                continue;
            }

            if matches!(
                instruction.flow,
                FlowKind::NearCall | FlowKind::FarCall | FlowKind::IndirectCall
            ) {
                errors.push(format!(
                    "SI reaching path crosses unresolved call at output 0x{:X}; no register value is guessed",
                    instruction.output_offset
                ));
                continue;
            }

            if instruction_writes_si(&decoded) {
                let Some(table_runtime_offset) = ds_absolute_si_load(&decoded) else {
                    errors.push(format!(
                        "SI definition at output 0x{:X} is not a DS absolute pointer-table load",
                        instruction.output_offset
                    ));
                    continue;
                };
                let table_output_offset =
                    runtime_to_output_offset(table_runtime_offset, expanded.len()).ok_or_else(|| {
                        format!(
                            "SI pointer table address S:{table_runtime_offset:04X} is outside expanded output"
                        )
                    });
                let table_output_offset = match table_output_offset {
                    Ok(offset) => offset,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };
                let Some(entry_bytes) = expanded.get(table_output_offset..table_output_offset + 2)
                else {
                    errors.push(format!(
                        "SI pointer table word at output 0x{table_output_offset:X} is truncated"
                    ));
                    continue;
                };
                let runtime_pointer = u16::from_le_bytes([entry_bytes[0], entry_bytes[1]]);
                let trace_offsets = next_trace.iter().rev().copied().collect();
                let candidate = PointerResolution {
                    runtime_pointer,
                    source: StaticPointerSource::DsAbsoluteTable {
                        register: "SI".to_owned(),
                        definition_output_offset: instruction.output_offset,
                        table_runtime_offset,
                        table_output_offset,
                        table_entry_output_offset: table_output_offset,
                    },
                    definition_offsets: vec![instruction.output_offset],
                    definition_path_offsets: trace_offsets,
                };
                if !found.contains(&candidate) {
                    found.push(candidate);
                }
                continue;
            }

            walk(
                expanded,
                report,
                function_entry,
                instruction.output_offset,
                &mut next_trace,
                active,
                found,
                errors,
                depth + 1,
            );
        }

        active.remove(&cursor);
    }

    let mut found = Vec::new();
    let mut errors = Vec::new();
    walk(
        expanded,
        report,
        function_entry,
        pointer_push_offset,
        &mut vec![pointer_push_offset],
        &mut HashSet::new(),
        &mut found,
        &mut errors,
        0,
    );
    if !errors.is_empty() {
        return Err(format!(
            "SI pointer at output 0x{pointer_push_offset:X} is not fully static: {}",
            errors.join("; ")
        ));
    }
    if found.is_empty() {
        return Err(format!(
            "SI pointer at output 0x{pointer_push_offset:X} has no supported static table definitions"
        ));
    }
    Ok(found)
}

fn instruction_writes_si(instruction: &Instruction) -> bool {
    let mut factory = InstructionInfoFactory::new();
    factory
        .info(instruction)
        .used_registers()
        .iter()
        .any(|used| {
            matches!(
                used.register(),
                Register::SI | Register::ESI | Register::RSI
            ) && matches!(
                used.access(),
                OpAccess::Write
                    | OpAccess::CondWrite
                    | OpAccess::ReadWrite
                    | OpAccess::ReadCondWrite
            )
        })
}

fn ds_absolute_si_load(instruction: &Instruction) -> Option<u16> {
    if instruction.mnemonic() != iced_x86::Mnemonic::Mov
        || instruction.op0_kind() != OpKind::Register
        || instruction.op0_register() != Register::SI
        || instruction.op1_kind() != OpKind::Memory
        || instruction.memory_segment() != Register::DS
        || instruction.memory_base() != Register::None
        || instruction.memory_index() != Register::None
    {
        return None;
    }
    u16::try_from(instruction.memory_displacement64()).ok()
}

fn decode_record(record: &InstructionRecord) -> Instruction {
    let mut decoder = Decoder::with_ip(
        16,
        &record.bytes,
        u64::from(record.runtime_ip),
        DecoderOptions::NONE,
    );
    decoder.decode()
}

fn resolve_parser_call(
    function_entry: usize,
    code_offset: usize,
    module_len: usize,
    instructions: &[InstructionRecord],
    edges: &[ControlFlowEdge],
) -> ParserCallSite {
    let mut diagnostics = Vec::new();
    let mut pushes: [Option<PushArg>; 3] = [None, None, None];
    let mut cursor = code_offset;

    for index in (0..3).rev() {
        let predecessors = edges
            .iter()
            .filter(|edge| {
                edge.function_entry_output_offset == function_entry
                    && edge.to_output_offset == cursor
            })
            .collect::<Vec<_>>();
        if predecessors.len() != 1 {
            diagnostics.push(if predecessors.is_empty() {
                format!("ABI push {} has no unique reachable predecessor", 3 - index)
            } else {
                format!(
                    "ABI push {} has {} possible predecessor edges; arguments are not guessed",
                    3 - index,
                    predecessors.len()
                )
            });
            break;
        }
        let predecessor = predecessors[0];
        if predecessor.kind != EdgeKind::FallThrough {
            diagnostics.push(format!(
                "control-flow edge {:?} enters the ABI push sequence at output 0x{:X}",
                predecessor.kind, cursor
            ));
            break;
        }
        let Some(instruction) = instructions.iter().find(|instruction| {
            instruction.function_entry_output_offset == function_entry
                && instruction.output_offset == predecessor.from_output_offset
        }) else {
            diagnostics.push(format!(
                "predecessor instruction at output 0x{:X} is unavailable",
                predecessor.from_output_offset
            ));
            break;
        };
        if instruction.mnemonic != "Push" {
            diagnostics.push(format!(
                "expected ABI push before slot 54 at output 0x{:X}, found {}",
                instruction.output_offset, instruction.mnemonic
            ));
            break;
        }
        let expression = push_expression(instruction);
        pushes[index] = Some(PushArg {
            output_offset: instruction.output_offset,
            expression,
        });
        cursor = instruction.output_offset;
    }

    let window_expr = pushes[0].as_ref().map(|push| push.expression.clone());
    let segment_expr = pushes[1].as_ref().map(|push| push.expression.clone());
    let pointer_expr = pushes[2].as_ref().map(|push| push.expression.clone());

    if let Some(expression) = &window_expr {
        match expression {
            PushExpr::Immediate(value) if *value <= 3 => {}
            PushExpr::Immediate(value) => diagnostics.push(format!(
                "slot 54 window argument {value} is outside known window range 0..=3"
            )),
            _ => diagnostics.push("slot 54 window argument is not an immediate value".to_owned()),
        }
    }

    let segment_is_ds = matches!(
        segment_expr.as_ref(),
        Some(PushExpr::Register(register)) if register == "DS"
    );
    if segment_expr.is_some() && !segment_is_ds {
        diagnostics.push("slot 54 segment argument is not proven to be DS".to_owned());
    }

    let runtime_pointer = match pointer_expr.as_ref() {
        Some(PushExpr::Immediate(value)) => Some(*value),
        Some(_) => {
            diagnostics.push("slot 54 text offset is not an immediate pointer".to_owned());
            None
        }
        None => {
            diagnostics.push("slot 54 text offset push is missing".to_owned());
            None
        }
    };

    let output_offset = if segment_is_ds {
        runtime_pointer.and_then(|pointer| {
            let offset = runtime_to_output_offset(pointer, module_len);
            if offset.is_none() {
                diagnostics.push(format!(
                    "slot 54 DS pointer S:{pointer:04X} is before S:0100 or outside expanded output"
                ));
            }
            offset
        })
    } else {
        None
    };

    if pushes.iter().any(Option::is_none) {
        diagnostics.push(
            "slot 54 ABI is incomplete; expected adjacent pushes window, DS, offset".to_owned(),
        );
    }

    ParserCallSite {
        function_entry_output_offset: function_entry,
        code_output_offset: code_offset,
        abi_pushes: pushes,
        window_expr,
        segment_expr,
        pointer_expr,
        runtime_pointer,
        output_offset,
        diagnostics,
    }
}

fn push_expression(instruction: &InstructionRecord) -> PushExpr {
    let mut decoder = Decoder::with_ip(
        16,
        &instruction.bytes,
        u64::from(instruction.runtime_ip),
        DecoderOptions::NONE,
    );
    let decoded = decoder.decode();
    match decoded.op0_kind() {
        OpKind::Immediate8to16 => PushExpr::Immediate(decoded.immediate8to16() as u16),
        OpKind::Immediate16 => PushExpr::Immediate(decoded.immediate16()),
        OpKind::Immediate8 => PushExpr::Immediate(u16::from(decoded.immediate8())),
        OpKind::Register => PushExpr::Register(format!("{:?}", decoded.op0_register())),
        _ => PushExpr::Other,
    }
}

fn local_menu_pointer_store(instruction: &InstructionRecord) -> Option<(i8, u16)> {
    let bytes = &instruction.bytes;
    if bytes.len() != 5 || bytes[0] != 0xC7 || bytes[1] != 0x46 {
        return None;
    }
    let displacement = bytes[2];
    if !matches!(displacement, 0xF4 | 0xF6 | 0xF8 | 0xFA) {
        return None;
    }
    let runtime_pointer = u16::from_le_bytes([bytes[3], bytes[4]]);
    Some((displacement as i8, runtime_pointer))
}

fn can_reach_in_function(
    report: &AnalysisReport,
    function_entry: usize,
    start: usize,
    target: usize,
) -> bool {
    if start == target {
        return true;
    }
    let mut queue = VecDeque::from([start]);
    let mut visited = HashSet::new();
    while let Some(offset) = queue.pop_front() {
        if !visited.insert(offset) {
            continue;
        }
        for edge in report.edges.iter().filter(|edge| {
            edge.function_entry_output_offset == function_entry
                && edge.from_output_offset == offset
                && edge.kind != EdgeKind::CallTarget
        }) {
            if edge.to_output_offset == target {
                return true;
            }
            queue.push_back(edge.to_output_offset);
        }
    }
    false
}
