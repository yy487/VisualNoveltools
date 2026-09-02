# ORETUBAR TOB0 提取/注入工具

这是根据游戏 EXE 的反汇编实现的结构化 Rust 工具，不使用字符串扫描或固定字节前缀启发式。它解析 `TOB0` 的标签表、消息偏移表、两类脚本指令、条件区、参数区和静态跳转，并在文本长度变化后重建所有受影响的长度、标签、消息偏移和 opcode `0..3` 跳转目标。

## 构建

```text
cargo build --release --bin oretubar-tob
```

生成的程序位于 `target/release/oretubar-tob.exe`。

## 使用

```text
oretubar-tob extract --input <TOB_FILE_OR_DIR> --output <JSON_FILE_OR_DIR>
oretubar-tob inject --input <TOB_FILE_OR_DIR> --translation <JSON_FILE_OR_DIR> --output <NEW_FILE_OR_DIR>
oretubar-tob verify --input <TOB_FILE_OR_DIR>
```

输出已存在时必须显式添加 `--overwrite`。目录注入会复制完整源目录，再替换其中的 TOB；不会原地修改游戏文件。无参数启动交互菜单，只给一个路径时仅预填输入，确认前不会写文件。

## 翻译 JSON

- 单槽文本：`scr_msg` 是不可修改的原文，译文写入 `message`。
- 多槽文本和选项：`scr_msg_parts` 是不可修改的原文数组，逐项编辑 `message_parts`；数组长度不能改变。
- `name` 是已确认的 opcode 25 参数 0 说话人名，`_voice` 是参数 1 语音资源 ID。本版将姓名作为只读上下文，不能修改。
- `_file`、`_index`、`_offset`、`_size`、`_type`、`_inst_offset`、`_opcode`、`_table_index`、`_encoding`、`_scr_name` 和 `_voice` 都是校验元数据，不应修改。
- `_type` 可为 `dialogue`、`narration`、`selection`、`title` 或 `summary`。

JSON 使用 UTF-8；TOB 文本按已验证的 CP932 解码和编码。注入拒绝 NUL、CR、非法控制字符、不能用 CP932 表示的字符、被修改的原文/元数据、缺失或重复结构以及越界目标。

## 限制

当前只接受本游戏样本中已验证的 `TOB0`，不猜测 `TOB1`。指令形式的显示控制保持为结构字节，工具只编辑已确认的正文槽。若要直接写入中文，需要另行确认字体和编码补丁方案；未修改注入必须逐字节等同源文件。
