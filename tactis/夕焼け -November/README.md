# 夕焼け -November- MES 文本工具

本目录提供 `extract.exe` 和 `inject.exe`，用于从游戏的无扩展名 MES 脚本提取正文，并在不移动任何指令或跳转目标的前提下注入 CP932 文本。

## 使用方法

```powershell
.\extract.exe '..\mes'
.\inject.exe '..\mes' '..\mes_json'
```

也可以把一个 MES 文件或整个 `mes` 目录拖到 `extract.exe`。`inject.exe` 需要两个位置参数，推荐在 PowerShell 或命令提示符中调用：第一个是原 MES 文件/目录，第二个是 JSON 文件/目录。

显式指定输出：

```powershell
.\extract.exe '..\mes' --output '.\translation_json'
.\inject.exe '..\mes' '.\translation_json' --output '.\mes_injected'
```

默认输出：

- 单文件提取：`ha01-1` -> `ha01-1.json`
- 目录提取：`mes` -> `mes_json\`
- 单文件注入：`ha01-1` -> `ha01-1_injected`
- 目录注入：`mes` -> `mes_injected\`

工具绝不覆盖已有文件或目录。目录注入会复制全部源文件，仅修改存在对应 JSON 的脚本。

## JSON 合同

JSON 为 UTF-8、每个源脚本一个文件、每句话一条记录。只修改 `message`：

```json
{
  "_format": "yuyake-mes-text-v2",
  "_file": "ha01-1",
  "entries": [
    {
      "_file": "ha01-1",
      "_index": 0,
      "_inst_offset": 80,
      "_offset": 84,
      "_size": 94,
      "_payload_offset": 83,
      "_payload_size": 217,
      "_type": "message",
      "_opcode": "0x15",
      "_page": 21,
      "_part_index": 0,
      "_encoding": "CP932",
      "_policy": "in_place",
      "_control_before": "01",
      "_control_after": "05 01",
      "_payload_sha256": "...",
      "scr_msg": "　降り注ぐ秋の太陽は少し前の季節の狂暴な光でなく、やわらかく優しいぬくもりを与えてくれていた。",
      "message": "　降り注ぐ秋の太陽は少し前の季節の狂暴な光でなく、やわらかく優しいぬくもりを与えてくれていた。"
    }
  ]
}
```

- `scr_msg` 是不可修改的源文校验字段。
- `message` 是唯一可编辑字段。
- 游戏没有独立角色名，因此没有 `name` 字段。
- 同一页中的每句话分别编号；`05`、`05 01` 等边界保存在 `_control_before/_control_after`，不会混入正文。
- `message` 内不能加入 CR/LF；需要新增、删除或合并句子会改变控制结构，当前工具不支持。
- 所有下划线字段是定位或验证元数据，不要修改。

## 控制码与编码

反汇编确认 `0x15` 指令后跟小端 `u16` 页号和一个 NUL 结尾字符串。页字符串中的 `01`、`02`、`05` 是运行时显示控制码；工具把它们保存在 `_layout` 中并在注入时原样恢复。正文采用 CP932，无法编码的字符会列出并拒绝写出，不会替换成问号。

纯控制页不提取。资源名、声音、背景、BGM、跳转及其他 opcode 数据均按 opaque 字节保留。

## 注入限制

当前只支持 `in_place`，文件长度永远不变：

- 新正文编码后不能超过原页槽。
- 短文本产生的空余字节必须是偶数；工具用渲染器明确忽略的 `03 03` 控制对填充。
- 奇数字节差会报错，可通过调整一个可编码的单字节字符来改变奇偶性。
- 不支持新增、删除或合并句子，也不支持变长重定位或修改控制码。

这些限制是因为脚本包含按字节位置跳转；在完整重建所有跳转与引用前不能移动后续数据。

## 验证

源码构建：

```powershell
cargo fmt -- --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --release --offline --bins
```

无修改回环：

```powershell
.\extract.exe '..\mes' --output '.\roundtrip_json'
.\inject.exe '..\mes' '.\roundtrip_json' --output '.\roundtrip_mes'
```

然后逐文件比较 SHA-256。工具的格式证据、角色名规则和剩余限制见 `PROJECT_PROFILE.md`。
