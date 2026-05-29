# CPB2PNG 分析流程说明

## 1. 目标

本文档用于记录当前游戏资源中 `.cpb` 图像转换为 `.png` 的分析流程、关键结论、实现路径与后续扩展方向，供后续维护 `cpb2png` 工具时参考。

当前目标聚焦：

- 从 `graphic.arc` 中定位并读取 `.cpb` 资源；
- 还原 `.cpb` 的实际读流解密方式；
- 识别并解析 `TYP1` 图像格式；
- 将 `TYP1 24bpp` 图像转换为 PNG；
- 在耗时的通道交织阶段接入 C 加速。

---

## 2. 输入样本与相关文件

分析过程中使用到的主要样本：

- `graphic.arc`：图像资源包
- `bg002b.cpb`
- `bg005b.cpb`
- `sys.zip`
- `cg.tbl`
- `tvlost.exe`
- `tvlost.exe.c`

其中：

- `graphic.arc` 用于确认 `.cpb` 资源的真实 entry offset / size；
- `tvlost.exe.c` 用于逆向图像读取与解码调用链；
- `cg.tbl` 主要用于资源索引层验证，不是底层 ARC 物理表。

---

## 3. 总体分析路线

整体流程如下：

```text
graphic.arc
→ ARC 文件表解密 / 解压
→ 定位目标 .cpb entry
→ 按游戏 fread-XOR 逻辑解密图像流
→ 识别 TYP1 header
→ 解析图像参数（bpp / width / height / block sizes）
→ 解压三个通道块
→ 通道交织为 RGB
→ 输出 PNG
```

---

## 4. ARC 层分析

### 4.1 先验证 script.arc / graphic.arc 的共性

最早先从 `script.arc` 入手确认该引擎的 ARC 封包框架：

- 文件头不是明文 `ARC`；
- 读取时会先按固定逻辑做 header/filetable 解密；
- 文件表本体使用 zlib 压缩；
- 解压后可得到文件数量、默认后缀、hash、offset、size 等信息。

### 4.2 ARC key

最终确认 ARC 文件表层的 key 为：

```text
0xADD1F4AA
```

该 key 用于 `graphic.arc` / `script.arc` 的文件表解密与资源定位。

### 4.3 graphic.arc 解包结论

已确认 `graphic.arc` 可以稳定解包，文件数为：

```text
482
```

默认内部后缀为：

```text
.cpb
```

并且可以准确定位：

- `bg002b.cpb`
- `bg005b.cpb`
- `black.cpb`
- `red.cpb`
- `white.cpb`

等资源。

---

## 5. 对 `.cpb` 的初始误判与修正

### 5.1 初始误判

一开始直接拿从 `graphic.arc` 中解出的 `.cpb` 数据观察，发现其开头并不是明文：

- 不是 `CPB\x1A`
- 不是 `TYP0`
- 不是 `TYP1`
- 不是 BMP `BM`

因此最初误以为：

- `.cpb` 可能仍然多套了一层整体加密；
- 或者图像尺寸需要通过 header 反推。

在这个阶段，曾经得到过类似 `800x600 / 24bpp / TYP1` 的候选 header，但后续 block 无法正确 zlib 解压，因此这些都只是 **header 假阳性**，不能作为最终结论。

### 5.2 后续修正

继续逆 `tvlost.exe.c` 后发现：

- `.cpb` 不是“整文件一次性 XOR”；
- 而是通过 **fread 包装层** 在读取过程中做流式解密；
- 每次读取会基于当前物理文件位置重新初始化状态。

也就是说：

```text
不是：对整个 cpb 连续 XOR 一次
而是：每次 fread(start_pos, size) 时，按该 start_pos 初始化 XOR 流
```

这个结论是整个 `cpb2png` 成功的关键。

---

## 6. 图像格式识别：TYP1

### 6.1 从 EXE 逆向得到的插件识别结果

图像加载分支中确认存在：

- `TYP0`
- `TYP1`
- `BM`

其中当前这批背景 / EV 图像实际走的是：

```text
TYP1
```

### 6.2 TYP1 结构

对当前样本，已确认主要支持的结构为：

```text
00  4 bytes  magic = "TYP1"
04  1 byte   bpp
05  1 byte   flags
06  2 bytes  width   (LE)
08  2 bytes  height  (LE)
0A  4 bytes  block0 compressed size
0E  4 bytes  block1 compressed size
12  4 bytes  block2 compressed size
16  ...      reserved / extra header area
1E  ...      compressed blocks
```

在 `24bpp` 情况下，后面三个 block 分别对应：

- B 通道
- G 通道
- R 通道

每个 block 解压后长度均为：

```text
width * height
```

最终按像素交织成 RGB 即可。

---

## 7. 图像流解密

### 7.1 图像流 key 与 ARC key 不同

这里是一个很关键的点。

ARC 表层使用：

```text
0xADD1F4AA
```

但图像 entry 读入后，真正供 `TYP1` 解码器使用的读流 key 不是这个，而是：

```text
0x4AF3D7A3
```

因此不能把 ARC 解包后得到的 `.cpb` 当成明文 `TYP1`，还必须再走一遍 **图像读流层的 fread-XOR**。

### 7.2 fread-XOR 特点

图像流解密不是简单按 buffer 起点一次性跑完全文件，而是：

- 每次读取一段数据；
- 根据真实物理文件偏移初始化状态；
- 对该段数据解密；
- 再继续下一段读取。

因此对于 `TYP1` 来说，通常要按以下几段分别处理：

```text
header read
block0 read
block1 read
block2 read
```

如果把整个 `.cpb` 一次性连续 XOR，结果会不对。

---

## 8. 压缩块与 adler 校验的修正

### 8.1 早期错误

在最初版本中，把每个压缩块前 4 字节误当成了：

```text
解压后像素数据的 adler32
```

于是校验逻辑错误地写成：

```python
adler32(inflated_pixels)
```

这会导致大量文件虽然已经正确解压，但仍然报：

```text
inflated adler mismatch
```

### 8.2 正确理解

实际应当是：

- 每个 block 的前 4 字节存的是 **压缩 zlib payload 本身的 adler32**；
- 不是解压后像素的 adler32。

正确校验方式为：

```python
stored = block[:4]
calc = adler32(block[4:])
```

即：

```text
校验对象 = 压缩后的 zlib stream
```

这一修正解决了大量 24bpp 文件的误报。

---

## 9. 关于 BLACK / RED / WHITE 资源的判断

曾考虑过如下可能：

- `BLACK.CPB`
- `RED.CPB`
- `WHITE.CPB`

是否是某种额外色板、遮罩图、色通道贴图，需要和 `ev*.cpb` / `bg*.cpb` 叠加后才能恢复最终 PNG。

最终验证结果：

**不是。**

它们本身就是普通的 `TYP1 24bpp` 图像资源，可单独解码输出为 PNG，内容为固定纯色图：

- `black.cpb`：纯黑图
- `white.cpb`：纯白图
- `red.cpb`：固定红色调图

因此：

```text
bg*.cpb / ev*.cpb 是自包含图像，
不需要依赖 BLACK/RED/WHITE 再做额外叠加。
```

---

## 10. 当前工具支持情况

当前 `cpb2png` 工具已完成：

### 10.1 已支持

- `graphic.arc` 文件表解析
- 按文件名提取 `.cpb`
- 图像读流层 fread-XOR 解密
- `TYP1 24bpp` 解码
- 输出 PNG
- 通道交织的 C 加速

### 10.2 暂未支持

- `TYP1 32bpp`
- `TYP1 8bpp`
- 可能存在的其它图像分支 / 变体

工具运行时会对这两类暂未支持的情况给出 `skip`。

---

## 11. C 加速部分

为了提升大图批量转换效率，对最耗时的 RGB 通道交织阶段加入了 C 加速。

### 11.1 加速目标

`TYP1 24bpp` 解压后会得到三块：

- `B[width*height]`
- `G[width*height]`
- `R[width*height]`

需要将其交织成：

```text
RGBRGBRGB...
```

这一部分纯 Python 可实现，但速度较慢，因此抽出为 C 模块。

### 11.2 C 模块职责

C 加速模块主要负责：

- 输入三个等长通道 buffer；
- 输出交织后的 RGB buffer；
- 供 Python 通过 ctypes / 动态库调用。

该优化不会改变格式逻辑，只改善性能。

---

## 12. 当前实现的标准流程

推荐的标准处理流程如下：

### 12.1 单文件

```text
1. 从 graphic.arc 中按名字找到目标 entry
2. 读取 raw cpb bytes
3. 按图像 fread-XOR 规则解密 header
4. 判断 magic 是否为 TYP1
5. 读取 bpp / width / height / block sizes
6. 分别读取并解密 3 个压缩块
7. 对每个块：校验 adler32(zlib_stream)
8. zlib 解压得到 B/G/R 通道
9. 将三通道交织为 RGB
10. 保存 PNG
```

### 12.2 批量文件

```text
1. 遍历 graphic.arc 中全部 .cpb
2. 对每个 entry 执行单文件流程
3. bpp=24 则转换
4. bpp=32 / 8 等暂不支持则 skip
5. 输出 ok / skip / fail 统计
```

---

## 13. 常见报错解释

### 13.1 `inflated adler mismatch`

含义：

- 旧版本工具的错误校验造成；
- 或者当前压缩块读流起点不对；
- 或者使用了错误的图像 fread-XOR 逻辑。

当前修正后，如果仍然出现，就优先检查：

- 该 block 是否按正确物理 offset 初始化解密；
- 是否把不同 block 连续 XOR 了；
- 是否校验了错误的数据对象。

### 13.2 `supports TYP1 24bpp only`

含义：

- 当前实现明确只支持 `TYP1 24bpp`；
- `32bpp` / `8bpp` 文件暂未实现。

这不是损坏，也不是资源有问题，只是当前工具能力范围限制。

---

## 14. 后续可扩展方向

接下来如果继续扩展，可以按以下顺序进行：

### 14.1 支持 `TYP1 32bpp`

可能用于：

- 立绘
- 带透明通道的 UI 图
- 前景叠加素材

重点关注：

- 是否为 4 通道块；
- alpha 通道是否单独压缩；
- 输出时应转为 RGBA。

### 14.2 支持 `TYP1 8bpp`

可能用于：

- 小地图
- 索引图
- 调色板图像

重点关注：

- palette 存储位置；
- 像素索引与调色板映射；
- 是否仍沿用相同 block/zlib 结构。

### 14.3 回写 PNG -> CPB

如果后续需要做图像汉化或素材替换，可进一步研究：

```text
PNG
→ 拆通道 / 压缩
→ 重建 TYP1
→ 按图像读流层规则封装
→ 写回 graphic.arc
```

不过这一步比单纯 `cpb2png` 难度更高，需要同步解决：

- block 压缩重建；
- adler32 重算；
- 可能的对齐 / 头部字段回填；
- ARC 回封包。

---

## 15. 最终结论

当前已确认：

1. `graphic.arc` 可稳定解析；
2. `.cpb` 资源不是明文图，而是要经过图像读流层的 fread-XOR；
3. 当前主要图像格式为 `TYP1`；
4. `TYP1 24bpp` 可成功解码并输出 PNG；
5. block 前 4 字节校验的是压缩 zlib payload，不是解压后像素；
6. `BLACK/RED/WHITE.CPB` 只是普通纯色图，不是外置色通道叠加资源；
7. 当前工具已足够完成大部分背景 / EV 图像导出；
8. `32bpp` / `8bpp` 仍待后续补完。

---

## 16. 建议的配套文件

建议和 `cpb.md` 一起保留以下文件：

- `cpb2png.py`
- `tvlost_arc_common.py`
- `typ1_interleave.c`
- `README.md`

其中：

- `cpb.md` 记录分析依据与关键结论；
- `README.md` 记录实际命令用法；
- 代码文件负责具体实现。

