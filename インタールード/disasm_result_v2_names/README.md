# output_script.zip 全量保守反汇编结果

## 结果摘要
- 文件数：203
- 总字节数：2747039
- 文本条目：41251
- 地址空间覆盖率：100.00% 全部通过
- 零突变回环：203/203 文件通过

## 目录结构
- `sources/<源文件名>/ir_compact.json`：紧凑 IR 元数据
- `sources/<源文件名>/instructions.jsonl`：逐字节覆盖的反汇编原子/记录
- `sources/<源文件名>/disassembly.asm`：人类可读反汇编清单
- `sources/<源文件名>/region_map.json`：区域图与覆盖率
- `sources/<源文件名>/text_entries.jsonl`：由 IR 投影得到的可本地化文本
- `texts/*.dsat.txt`：分源 DSAT 双行翻译文本
- `rebuilt/`：由 IR raw_bytes 拼接回封的零修改重建文件
- `coverage_summary.csv`：全文件覆盖/回环汇总
- `roundtrip_report.txt`：Smoke round-trip 总报告

## 解析说明
由于没有目标 VM 的官方 opcode 表，本次采用保守 typed-atom 反汇编：识别 `00 xx`、`F1/F2/F3`、`80 len payload`、`FF` 等可见字节码原子；无法确认语义的字节以 opaque/raw 原子保留。该策略优先满足全域覆盖与零突变回环，不臆造控制流语义。
