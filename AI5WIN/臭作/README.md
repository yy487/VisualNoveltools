# AIWIN/旧 ELF 系 ARC 解包器

本工具基于当前样本 `message.arc`、`graph.arc` 与 `syuusaku.exe.c` 的反编译逻辑整理，用于解包同类 ARC 封包。

## 已确认 ARC 外层结构

```c
struct ArcHeader {
    uint32_t count;       // 文件数量，反编译代码限制 <= 1000
};

struct ArcEntry {
    char     name[16];    // NUL 结尾短文件名
    uint32_t offset;      // 条目数据偏移
    uint32_t size;        // 条目数据大小；是否压缩取决于资源类型
};
```

目录表大小：

```text
4 + count * 0x18
```

## 当前样本结论

### message.arc

`message.arc` 内部是 `.MES`，条目数据为引擎 LZSS-like bitstream，需要解压。

### graph.arc

`graph.arc` 内部是 `.GPX` 图像资源，条目数据不是 LZ 压缩流，需要直接切出。

上一版工具默认对所有条目做 LZ 解压，所以处理 `graph.arc` 时会得到 0 字节或异常文件。本版已改成默认 `auto`：

- `.MES`：自动 LZ 解压；
- 其它扩展名：默认 raw 直接切出。

## 条目压缩格式，仅用于 .MES

- bit 顺序：每字节从高位到低位。
- 控制位为 `1`：读取后续 8 bit 作为 literal 字节。
- 控制位为 `0`：读取后续 12 bit 作为滑窗 offset。
  - offset 为 0：当前压缩流结束。
  - offset 非 0：继续读取 4 bit 长度码，实际复制长度为 `length_code + 2`。
- 滑窗大小 `0x1000`，初始写入位置为 `1`。

## 文件说明

```text
aiwin_arc_common.py   公用 ARC 解析与 LZ 解压模块
aiwin_arc_extract.py  命令行解包入口
```

## 使用方法

列出目录：

```bat
python aiwin_arc_extract.py list graph.arc
python aiwin_arc_extract.py list message.arc
```

自动解包，推荐：

```bat
python aiwin_arc_extract.py extract graph.arc out_graph
python aiwin_arc_extract.py extract message.arc out_mes
```

强制 raw 直接切块：

```bat
python aiwin_arc_extract.py extract graph.arc out_graph --mode raw
```

强制 LZ 解压，仅建议用于确认是压缩流的资源：

```bat
python aiwin_arc_extract.py extract message.arc out_mes --mode decompress
```

兼容旧命令：

```bat
python aiwin_arc_extract.py extract graph.arc out_graph --raw
```

输出目录会生成 `manifest.json`，记录每个条目的索引、文件名、偏移、条目大小、输出大小和实际处理模式。

## 本版验证结果

对本次样本验证：

```text
message.arc
  files = 221
  mode_count = {raw: 0, decompressed: 221}
  total_output_bytes = 8320212

graph.arc
  files = 719
  mode_count = {raw: 719, decompressed: 0}
  total_output_bytes = 47649978
```

# gpx2png_tool

AIWIN / 旧 ELF 系 `GRAPH.ARC` 中 `.gpx` 图像资源转 PNG 的独立工具。

本工具只处理 `.gpx -> .png`，不包含 ARC 解包逻辑。先用 ARC 解包器把 `graph.arc` 切出 `.gpx`，再用本工具批量转换。

## 依赖

```bat
pip install pillow
```

## 用法

单文件：

```bat
python gpx2png.py elf3an.gpx
python gpx2png.py elf3an.gpx elf3an.png
```

批量目录：

```bat
python gpx2png.py out_graph
python gpx2png.py out_graph out_graph_png
```

覆盖已有输出：

```bat
python gpx2png.py out_graph out_graph_png --overwrite
```

输出 manifest：

```bat
python gpx2png.py out_graph out_graph_png --manifest out_graph_png\manifest.json
```

## 透明索引

默认不自动抠透明。

手动指定透明调色板索引：

```bat
python gpx2png.py out_graph out_graph_png --transparent-index 2
```

保守自动判断透明索引：

```bat
python gpx2png.py out_graph out_graph_png --auto-transparent
```

## 色通道说明

当前样本的 GPX 调色板顺序是 `R,G,B`。

上一版误按 `B,G,R` 处理，会出现红蓝通道互换，比如血迹变蓝。本版默认已经修正为 RGB。

如果以后遇到相反的变体，可以临时切换：

```bat
python gpx2png.py out_graph out_graph_png --palette-order bgr
```

默认推荐不要加这个参数。

## GPX 结构

```c
struct GpxHeader {
    uint16_t x;
    uint16_t y;
    uint16_t w;
    uint16_t h;
    uint16_t reverse;       // 0=正常, 1=转置存储
    uint8_t  palette[0xEC * 3]; // R,G,B * 236
    uint8_t  packed[];      // 从 0x2CE 开始
};
```

## 当前验证

已用 `elf3an.gpx` 验证：

```text
640x632, x=0, y=0, reverse=0
```

修正后人物服装、肤色、血迹等通道恢复正常。
