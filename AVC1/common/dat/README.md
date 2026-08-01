# ACV1 script.dat 通用工具

本工具统一处理已确认的两种 `script.dat` 归档分支：带明文 `ACV1` magic 的分支，以及没有 magic 的 legacy 分支。程序自动判断分支；用户不需要选择格式或手工计算密钥，只需输入游戏 EXE 使用的游戏名。

## 最简单的用法：拖放

1. 把 `script.dat` 拖到 `acv1_dat_tool.exe` 上。
2. 程序显示自动识别的分支和索引条目数。
3. 按提示输入游戏名。游戏名必须与游戏 EXE 内用于派生密钥的 CP932 标题字符串完全一致。
4. 程序先完整验证全部条目。全部成功后才生成 `script_unpacked`；失败时不会生成半成品目录。
5. 成功或失败后窗口都会等待按 Enter 退出。

把包含 `manifest.json` 和 `files` 的解包目录拖到 EXE 上，会按 manifest 记录的原分支与游戏名回封，默认输出 `script_packed.dat`。

程序拒绝覆盖已有输出。需要重新运行时，请自行改名或移走旧输出。

## 命令行

```powershell
acv1_dat_tool.exe unpack script.dat --game-title "姉小路直子と銀色の死神"
acv1_dat_tool.exe verify script.dat --game-title "姉小路直子と銀色の死神"
acv1_dat_tool.exe roundtrip script.dat --game-title "姉小路直子と銀色の死神"
acv1_dat_tool.exe pack script_unpacked
```

`unpack`、`verify`、`roundtrip` 省略 `--game-title` 时会交互提示输入。使用 `--output` 可以指定单个输出位置。

## 自动分支规则

| 分支 | 文件头 | count | offset 额外处理 |
| --- | --- | --- | --- |
| ACV1 | `41 43 56 31` (`ACV1`) | 第二个 DWORD XOR `0x8B6A4E5F` | XOR `0x8B6A4E5F` |
| legacy | 无 magic | 第一个 DWORD XOR `0x26ACA46E` | 无 |

两个分支共用以下逻辑：

- 每个索引记录固定 21 字节。
- 游戏名按 CP932 编码后计算 CRC64-ECMA；payload XOR 使用 CRC64 低 32 位。
- payload 密钥为 `title_key_low32 XOR entry.key_lo`。
- 仅对 `packed_size / 4` 个完整 DWORD 进行 XOR，末尾 1 至 3 字节不变。
- XOR 后的数据是 zlib 流。
- `flag` 目前只按索引字段原样保存；目标样本的 `flag=2`，但原压缩比对应 zlib 9，因此回封默认使用 zlib 9，不把 flag 猜作压缩级别。

程序会验证条目数、索引边界、payload 越界和重叠、zlib 完整消费以及解压结果不超过 `out_capacity`。不是 `ACV1` magic 的文件只会在整个 legacy 结构也通过验证时被接受。

## manifest.json

manifest 是 UTF-8 JSON，保存：

- 自动识别出的分支与输入文件名；
- 游戏名、完整 CRC64 和低 32 位密钥；
- 每个条目的顺序、key、flag、offset、packed size、capacity 和明文文件名；
- payload 的原始物理顺序；
- 索引后、payload 之间和文件尾的未知字节。

回封必须保留 manifest。程序会重新校验游戏名与记录的 CRC64/密钥，拒绝路径穿越、重复索引、损坏的 hex 字段和条目数量不一致。

## 回封策略

回封会重新压缩全部条目，重新计算每个 payload 的 offset 与 packed size，并默认令 `out_capacity = max(原容量, 新明文长度)`。因此归档层允许条目变长或变短。索引之间和文件尾的未知字节会按原物理布局保留。

Rust 压缩实现可能与原游戏使用的 zlib 版本产生不同压缩字节，所以外层文件不保证 byte-exact；`roundtrip` 要求重新解包后的全部内部文件逐字节一致。目标样本的真实回环为内部明文 exact，外层压缩字节因后端不同而不同。

## 文本、JSON 与限制

这是归档工具，不是文本提取/注入工具。`files` 中的脚本始终以原始字节保存，不转换编码、不解析正文、不修改控制符。文件名中的首个 `*label` 仅是便于识别的 CP932 命名提示；无标签时使用稳定的索引/key 文件名，回封定位始终以 manifest 的 `_index` 等价索引为准。

角色名、正文、选项、控制符和换行规则尚未在本通用归档工具中建档，因此本工具不生成翻译 JSON，也不声明任何 name/message 写回策略。

目前只支持上述两种 21 字节索引布局。其他同扩展名格式、其他 count 常量、不同 CRC 算法或非 zlib payload 不会自动猜测。

## 构建与验证

```powershell
cargo fmt -- --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --release --offline
```

release 文件位于 `target\release\acv1_dat_tool.exe`。
