# 制服カノジョ2.5 Script.pac 解包/封包工具

## 依赖

Python 3.10+。

封包/解包原版 `Script.pac` 的 `.bin` 文件块需要 Zstandard：

```bash
pip install zstandard
```

如果不安装 Python 模块，也可以把 `zstd` / `zstd.exe` 放到 PATH。

## 基本命令

查看包结构：

```bash
python uniform_kanojo_pac_tool.py info Script.pac -v
```

解包原包：

```bash
python uniform_kanojo_pac_tool.py unpack Script.pac Script_unpacked
```

封回游戏兼容格式：

```bash
python uniform_kanojo_pac_tool.py pack Script_unpacked Script_new.pac
```

如果目录内有 `manifest.json`，封包会使用原始文件顺序与 UTF-8 文件名；这是推荐流程。

对比原包和你自己封的 chs 包：

```bash
python uniform_kanojo_pac_tool.py diagnose Script.pac Script_chs.pac
```

把本次提供的 `Script_chs.pac` 尽量修回原包布局、原压缩格式：

```bash
python uniform_kanojo_pac_tool.py recover-chs Script.pac Script_chs.pac Script_chs_fixed.pac
```

## 已确认的格式

- 头部 12 字节：`PAC` + 1 字节无效/未初始化 + `uint32 file_count` + `uint32 compression`。
- 原始 `Script.pac`：`file_count=37`，`compression=7`。
- 文件块：本作 `.bin` 基本是 Zstandard frame，签名 `28 b5 2f fd`。
- 末尾：`~custom_huffman(file_table)` + `uint32 table_compressed_size`。
- 文件表记录：每条 76 字节：`name[64] + offset + original_size + compressed_size`。
- 原包文件名编码是 UTF-8，不是 CP932。

## 本次 chs 包的问题

工具检测结果见 `pac_diagnose.txt`。核心问题：

1. 文件数从原包 37 变成 36，缺 `__global.bin`。
2. 压缩 selector 从 7 变成 4。
3. 文件块变成 zlib 流，原包是 Zstandard frame。
4. 文件表日文名使用了 CP932，而原包表名是 UTF-8。
5. `機能チュートリアル.bin` 被改成了 `チュートリアル.bin`。

`Script_chs_fixed.pac` 是按原包布局重封的修复版：保留原包缺失的 `__global.bin`，将 chs 中 `チュートリアル.bin` 覆盖回原名 `機能チュートリアル.bin`，并重新输出为 `compression=7 / UTF-8 name / Zstandard chunks`。
