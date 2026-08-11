# NECRONOMICON MES 引擎档案

## 项目与边界

- 项目：PC-98 `NECRONOMICON`
- 运行时：16 位实模式 OVL 脚本 VM
- 原始样本：从 `NECRONOMICON_A.FDI` 至 `NECRONOMICON_K.FDI` 的 FAT12
  目录重新提取的真实 A–K MES 树
- 开发目录：当前工具目录
- 正式工具目录：用户指定的既有工具链目录
- 同步规则：只修改开发副本；用户明确要求最终更新前不覆盖正式工具。
- 实现：Rust 2024，UTF-8 JSON；`mes_extract` / `mes_inject`

## 脚本结构

- 扩展名：`.MES`
- 编码：CP932，加自定义平假名 `2D..7F -> 82 9F..F1`
- 外层 magic/压缩：无独立 MES 包络；文件由 FDI/FAT12 保存。
- 指令流：`sub_C635` 顺序执行，`sub_D77C` 提供跳过/扫描 token 长度。
- 结构边界：`A0..A4`；`A5` 为显示换行；`A6..D8` 经 `DS:DB5A` 分派。
- 特殊双字节控制：`81 97`、`81 90`、`81 6F`、`81 70`。
- `BA 23`：已确认是 opcode 加 expression，不是终止符。
- 物理 offset/jump：当前样本未发现需要随正文长度修补的序列化 byte offset；
  MES 由顺序 token 和结构标记重建，外层 FDI size/cluster 由 repacker 更新。

证据等级：上述长度与分派为 `confirmed-runtime` + `confirmed-structure`；未命名
handler 的视觉效果不影响长度和无损重建，按 `opaque-preserved` 处理。

## Name / message 规则

- 已确认有名对话：一个直接显示文本段以 `「` 开头，并在同段内出现第一个 `」`。
- `name`：两括号之间；括号属于结构，由注入器恢复。
- `message`：结束括号后的同一物理显示段。
- 旁白/UI/选项：无该前缀时保持无名，不猜 speaker。
- `name` 可写；`_scr_name` 保存并校验源名字。
- `scr_msg` 不可写；实际译文只取 `message`。
- `A5` 不伪装成 JSON 换行；它结束当前 entry，下一显示段另建 entry。

## IR 与注入

- `_tokens` 连续覆盖源文件每个字节，保留 offset、size、type、role、raw。
- A6–D8 全部 51 个 handler 有静态目标和 operand grammar。
- 数值、引号和直接字节 operand 记录 `_owner_opcode_index`。
- 未知/非法 CP932 token 不解码、不删除，按源字节复制。
- 变长注入允许修改 `name/message`，但先重新解析源文件并比较全部 immutable 元数据。
- 中文由既有 `subs_cn_jp.json` 映射后进入 CP932 字位；字体与 MES 工具分离。
- v0.2.1 的编码器按完整 CP932 双字节边界前进，再压缩真正的 `82 9F..F1`
  平假名；不得逐字节扫描并跨两个汉字误识别平假名序列。

## 验证与限制

- 原 FDI 的真实 A–K：208 MES，17,502 entries，12 warning。
- 用户提供的旧拆包树共有 278 MES，其中 `DISK_D/G/H/I/J/K` 混入不属于对应
  FDI 的副本；该树只用于历史分析，不得作为最终封盘文件清单。
- 最终修改回环：208/208 MES，17,502/17,502 名字与正文一致，0 mismatch。
- 最终 FDI：A–K 11 张均通过空替换重封 SHA-256 byte-exact；C 盘最终剩余 0 簇。
- 真实修改：`OPEN1.MES` 两条正文增长/缩短并重新提取成功。
- warning：3 个文件内 12 个未定义 CP932 pair，均无损保留。
- `21 ... 00` 在全样本 496 次出现均为空格 padding，不作为翻译正文。
- `D5` 的运行时自变换未模拟；当前样本未发现因此遗漏的可翻译正文。
