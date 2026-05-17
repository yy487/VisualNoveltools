# Doukyuusei2 ARC Tool

用于处理《同级生2》/ SilAI 系 ARC 文件的解包与按 manifest 重封包。

## 已确认结构

ARC 文件结构：

```text
u32le count
count * 0x14 encrypted directory entries
raw file bodies...
```

目录项解密后结构：

```text
char name[12]   # NUL padded，通常是 ASCII 8.3 文件名
u32le size
u32le offset
```

目录项加密方式：

```text
name[12] 每字节 xor 0x55
size     xor 0xAA55AA55
offset   xor 0x55AA55AA
```

样本验证：

- `MES.ARC`: 530 个文件，目录结束偏移 `0x296C`，首项 `MAIN.MES` 偏移 `0x296C`。
- `DATA.ARC`: 662 个文件，目录结束偏移 `0x33BC`，首项 `O15EC.A6` 偏移 `0x33BC`。

## 文件说明

- `d2_arc_common.py`：ARC 结构解析、目录项解密/加密、解包、重封包共用逻辑。
- `arc_extract.py`：批量解包/列目录入口。
- `arc_pack.py`：按 `_arc_manifest.json` 顺序重封包入口。

## 解包

单个 ARC：

```bat
python arc_extract.py MES.ARC out_mes
```

批量 ARC：

```bat
python arc_extract.py MES.ARC DATA.ARC out_arc
```

批量模式会输出：

```text
out_arc\MES\...
out_arc\DATA\...
```

只列目录：

```bat
python arc_extract.py --list MES.ARC dummy_out
```

## 重封包

解包时会生成 `_arc_manifest.json`，重封包会按该 manifest 的顺序读取文件并重建目录。

```bat
python arc_pack.py out_mes MES_new.ARC
```

注意：

1. ARC 文件名字段只有 12 字节，文件名过长会报错。
2. 当前样本没有压缩，也没有额外对齐，重封包直接连续拼接文件体。
3. 如果只修改文件内容长度，重封包会自动更新每项 size/offset。
4. 如果新增或删除文件，需要同步修改 `_arc_manifest.json`。

## 反汇编依据摘录

`Doukyuusei2.EXE.c` 中 ARC 读取逻辑要点：

- 初始化时调用 `FUN_0040cea0(this, arc_type, arc_name)` 打开 `MES.ARC`、`DATA.ARC` 等。
- `FUN_0040cea0` 读取 4 字节文件数，然后分配 `count * 0x14` 字节目录缓存。
- `FUN_0040b740` / `FUN_0040be20` 在查找文件前对目录项做异或解密：
  - 目录项前 12 字节文件名逐字节 xor `0x55`；
  - 第 0x0C 字节处 dword xor `0xAA55AA55`；
  - 第 0x10 字节处 dword xor `0x55AA55AA`。
- 查找到文件名后，使用目录项 offset 调 `SetFilePointer`，再按 size 调 `ReadFile`。

