# Uniform Kanojo NeXAS Project Profile

## 项目与路径

- 项目：Uniform Kanojo（样本目录格式标记为 `uniform-kanojo-2.5-pac`）。
- 引擎/运行时：NeXAS 脚本格式；运行时语义未通过调试器确认。
- 样本目录：`E:\迅雷下载\zfny\mes`。
- 开发/交付工作目录：`E:\迅雷下载\zfny\work`。
- 参考实现：`E:\迅雷下载\zfny\NeXAS_Tool`。
- 当前 Rust 工具源码目录：`uniform_kanojo_nexas_tool`。
- 外层正式工具目录未在本任务中覆盖；只同步用户指定的工作目录。

## 归档与文件表

- `mes` 是已解包目录，共 37 个 `.bin` 和一个 `manifest.json`。
- `manifest.json` 记录外层 `script.pac`、UTF-8 名称和压缩类型 7；外层包络不由本工具写回。
- 36 个 `.bin` 以 `VER-1.00\0` 开头并按结构化表解析；`__global.bin` 是独立全局表，当前作为 opaque 文件原样复制。
- 脚本结构包含全局/函数未知数组、opcode 表、常量字符串表、变量/参数表和 68 字节 opaque block 表。所有未知字段在重建时保留。

## 编码与文本来源

- 样本字符串表经 UTF-8 解码并完成字节级 round-trip；本项目不把 CP932 假设套入已确认的 UTF-8 脚本。
- 可翻译文本来自结构化常量字符串表，不使用全文件启发式扫描。
- 资源名、纯控制字符串、系统文件和已确认系统文本排除。

## 用户确认的 name/message 规则

用户确认的代表样本：

```text
女子生徒Ａ
@v09900000「あっ、古賀先生さようならー！」
```

- 相邻常量字符串中，独立姓名项直接拆为 `name`，后续正文项拆为 `message`。
- `@v09900000` 属于正文外控制前缀；正文从日文引号开始。正文内部控制符（例如 `@n`）保留在 `message`，并允许翻译时增删或调整。
- 旁白/无名文本没有 `name`；连续的 `『...』` 文本作为选项提取。
- 角色名允许随正文一起编辑；注入使用 `_scr_name` 校验原始角色名，防止错位写回。
- 空的正文外控制字段可在 JSON 中省略，反序列化按空字符串处理。

## 注入与变长策略

- 入口按 `_file`、`_scope`、函数索引、字符串索引、`scr_msg`、`_scr_raw` 和控制边界逐项校验。
- `message` 内部控制符可编辑，不要求与 `scr_msg` 的控制符序列一致；NUL 和物理 CR/LF 仍被拒绝。选项必须保留 `『...』` 分隔符。正文外 `_control_prefix` / `_control_suffix` 仍按源脚本校验并自动恢复。
- 文本写回后完整重建常量字符串表和所有已知结构；`_policy` 为 `relocate`。未知区域不移动、不丢弃。
- 外层 PAC 的 offset/size/压缩重建未实现，不能把本工具输出直接当作新的 PAC。

## 证据与限制

- `confirmed-structure`：magic、表边界、UTF-8 round-trip、脚本层 byte-exact rebuild、JSON 定位和控制符边界。
- `confirmed-user-rule`：相邻姓名/正文拆分、系统跳过、选项提取、资源名和纯控制字符串排除、正文内控制符保留、正文外控制符分离。
- `opaque-preserved`：`__global.bin` 语义、未知 block 和未运行时确认的控制符视觉含义。
- 真实样本统计和回环结果详见 `README.md`。发现不符合上述组织规则的格式变体时，应先更新本档案再扩大注入范围。
