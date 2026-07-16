# AVG3216 フロレアール SEEN.TXT 文本工具

本工具用于：

`フロレアール ～すきすきだいすき～` 的 `SEEN.TXT`

提供两个 Rust/Windows 程序：

- `avg3216_extract.exe`：结构化提取 UTF-8 JSON
- `avg3216_inject.exe`：校验 JSON、注入文本并重建 `SEEN.TXT`

两个 EXE 都支持 Windows 文件拖放，也保留完整命令行参数。

## 已确认的文件结构

`SEEN.TXT` 不是普通文本文件，而是四层结构：

1. `PACL`
   - 文件数位于 `0x10`
   - 目录从 `0x20` 开始
   - 每项 `0x20` 字节：16 字节名称、偏移、压缩尺寸、解压尺寸、标志
2. `PACK`
   - 每个 PACL 项均为独立 LZSS 数据块
3. `TPC32`
   - 保存元数据、变量表和 `A/B/C/e[]` 独立子脚本
   - 每个子脚本带独立的 `u32` 长度
4. AVG3216 VM 字节码
   - `0xFF + CP932 双字节串 + NUL`：正文显示
   - `0x58`：选择块，块内的 `0xFF` 是选项正文
   - `0xFE`：单字节字符串
   - `0x60.04`：动态拼接调试/场景标签

对原始文件的严格解码结果：

- PACL 项：45
- VM 指令：42,919
- 可翻译条目：16,834
  - 正文：16,807
  - 选项：27

解析器按真实指令边界行走；不会把数字参数中的 `FF` 字节误识别为文本。

## 项目文本规则

用户已确认本作没有 `name` 标签。

- 不生成 `name`
- 引号内台词不拆说话人
- 台词、旁白和选项都完整放入 `scr_msg` / `message`
- `scr_msg` 是不可变原文
- 翻译只写入 `message`
- 提取时 `message` 默认等于 `scr_msg`

以下内容不进入翻译 JSON：

- `FE` 单字节内部字符串
- `0x60.04` 拼接的窗口标题、调试标签和场景标识，例如 `scene:1 `

完整配置见 `project_profile.json`。

## 拖放使用

### 提取

把 `SEEN.TXT` 拖到：

```text
avg3216_extract.exe
```

默认生成：

```text
SEEN.TXT.json
```

### 注入

编辑 JSON 中的 `message`，不要修改 `scr_msg` 和所有下划线开头的定位字段。

把 `SEEN.TXT.json` 拖到：

```text
avg3216_inject.exe
```

注入器会从 JSON 同目录寻找 `source_file` 指定的原始 `SEEN.TXT`，默认生成：

```text
SEEN_injected.TXT
```

输出已存在时会停止，不会覆盖。

## 命令行

```powershell
avg3216_extract.exe --help
avg3216_inject.exe --help
```

提取：

```powershell
avg3216_extract.exe --no-pause SEEN.TXT
avg3216_extract.exe --no-pause --output translated.json SEEN.TXT
```

注入：

```powershell
avg3216_inject.exe --no-pause SEEN.TXT.json
avg3216_inject.exe --no-pause --source SEEN.TXT --output SEEN_zh.TXT SEEN.TXT.json
```

不带 `--no-pause` 的纯位置参数调用会按拖放模式在结束后等待回车。

## JSON 格式

示例：

```json
{
  "_file": "SEEN101.TXT",
  "_file_index": 1,
  "_subscript": "e0",
  "_index": 0,
  "_offset": 158,
  "_subscript_offset": 59,
  "_size": 32,
  "_type": "message",
  "_opcode": "FF",
  "scr_msg": "　さっきから、ベルが鳴っている。",
  "message": "　さっきから、ベルが鳴っている。"
}
```

选项条目额外包含 `_choice_index`。

注入前会验证：

- JSON 格式版本
- 原始 `SEEN.TXT` 尺寸和 SHA-256
- 条目数量和顺序
- `_file`、`_subscript`、偏移、尺寸、类型和操作码
- `scr_msg` 与源脚本一致
- `message` 可编码

## 编码限制

当前项目配置是 `CP932 -> CP932`。

AVG3216 的 `0xFF` 处理器每次固定读取两个字节。因此 `message` 编码后必须全部是 CP932 双字节字符：

- 可使用日文和全角字符
- 半角 ASCII 字母、数字、空格和标点会被拒绝
- CP932 不可编码字符会被拒绝并列出字符和 Unicode 码点
- 中文注入需要先完成 EXE 编码逻辑和 `FN.DAT` 字库方案，不能直接用当前 CP932 配置硬塞

工具会在写输出前完成这些检查，不会留下半成品。

## 变长注入与重建

文本长度变化时，注入器会按子脚本分别修正：

- `0x15` 条件跳转
- `0x1B` gosub
- `0x1C` goto
- `0x1D/0x1E` 跳转表
- 子脚本 `u32` 长度
- `PACK` 解压/压缩尺寸
- `PACL` 项偏移和尺寸

未修改的 PACK 块保持原始字节。修改过的块使用合法的全字面量 PACK 编码，因此输出文件可能比原文件大。

无翻译改动时直接保留原始归档字节；已验证输出与输入 SHA-256 完全一致。

## 验证

```powershell
cargo fmt -- --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --offline --release --bins
```

真实文件验收包括：

- 全 45 项解压、TPC32 和 VM 严格解析
- 提取 16,834 条
- 无改动注入字节完全一致
- 变长文本注入后全归档重新解析
- 非双字节正文拒绝且不创建输出

## 已知限制

- 当前只支持本作已确认的 AVG3216 指令配置
- 当前只支持 CP932 目标编码
- 尚未处理中文 EXE 编码补丁和 `FN.DAT` 字库生成
- 不会覆盖源文件或已有输出

