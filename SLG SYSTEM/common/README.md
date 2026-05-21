# SZS100__ 自动解包/封包工具

适用范围：已确认支持同系 `SZS100__` 封包，包括：

- 三国姫1 `script.szs`
- 三国姫2 `script2.szs`
- 天極姫1 `script*.szs`
- 天極姫2 `sys.szs`

## 已确认的封包结构

```text
magic: SZS100__
0x0C: file_count u32le

entry size = 0x110
+0x000 name[0x100]，cp932，\0 结尾
+0x100 offset u64le
+0x108 size   u64le
```

## 已支持的成员加密模式

所有样本的成员数据都先经过 SZS 成员层 XOR：

```text
stored_xor = stored ^ 0x90
```

之后根据 EXE 版本分为两类：

### 1. full_lcg_sub

三国姫1/2 样本使用。

```text
state = state * 0x343FD + 0x269EC3
rand8 = ((state >> 16) & 0x7FFF) & 0xFF
plain[i] = ((stored[i] ^ 0x90) - rand8) & 0xFF
stored[i] = ((plain[i] + rand8) & 0xFF) ^ 0x90
```

已确认 seed：

```text
三国姫1: 0x7f501e37
三国姫2: 0x3e9f9d19
```

### 2. reseed_lcg_xor

天極姫1/2 样本使用。

这个版本不是保存完整 LCG state，而是把上一轮返回的 `rand` 当作下一轮参数：

```text
x = seed
for each byte:
    x = (((x * 0x343FD + 0x269EC3) signed>> 16) & 0x7FFF)
    plain[i] = (stored[i] ^ 0x90) ^ (x & 0xFF)
```

加密和解密同构。

已确认 seed：

```text
天極姫1 script天極姫1.szs: 0x15ec7646
天極姫2 sys.szs:          0x13194ff5
```

## 自动检测逻辑

工具会从 EXE / `.exe.c` 中扫描常见 `push imm32`、`mov reg, imm32`、`mov [esp+xx], imm32` 常量，然后对 SZS 成员进行试解密评分。

评分特征包括：

- `main.txt`
- `start`
- `mode`
- `Data\...`
- `.txt` / `.bin` / `.dat` / `.wma`
- 大量 NUL 填充的 ASCII 表
- 小整数/偏移表结构

最终输出：

```text
seed
archive_xor
crypto mode
score
```

## 用法

列文件表：

```bat
python szs_auto.py list script.szs
```

自动检测：

```bat
python szs_auto.py detect script.szs --exe game.exe -v
```

解包：

```bat
python szs_auto.py unpack script.szs script_unpacked --exe game.exe
```

重封包：

```bat
python szs_auto.py pack script_unpacked script_repack.szs
```

解包目录会生成 `manifest.json`，其中保存文件表和 crypto 信息。正常情况下，pack 时不需要再传 `--seed`。

## 手动指定参数

```bat
python szs_auto.py unpack script.szs out --seed 0x15ec7646 --mode reseed_lcg_xor --xor 0x90
```

可选 mode：

```text
full_lcg_sub
reseed_lcg_xor
```

## 验证结果

当前版本已做 roundtrip 验证：

```text
script.szs           -> unpack -> pack -> 与原文件 bit-exact 一致
script2.szs          -> unpack -> pack -> 与原文件 bit-exact 一致
script天極姫1.szs    -> unpack -> pack -> 与原文件 bit-exact 一致
sys.szs              -> unpack -> pack -> 与原文件 bit-exact 一致
```

## 注意

这个工具只负责 SZS 解包/封包和成员解密/加密。解出来的 `.sd/.tko/.lb/.lbn/.ev/.bl/.sb/.sbn` 仍然是引擎脚本、标签、事件表和索引结构；文本提取/变长注入需要继续分析这些内部格式。
