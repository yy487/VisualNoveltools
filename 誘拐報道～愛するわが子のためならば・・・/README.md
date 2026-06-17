# Yuukai `system.dat` 解包 / 封包工具

## 已确认格式

该样本的 `system.dat` 不是常见的表头前置封包，而是：

```text
4 字节目录表偏移，大端序 u32
raw-deflate stream 0
raw-deflate stream 1
...
raw-deflate catalog stream
```

最后一个 raw-deflate 流解压后是 AMF3 动态对象，结构等价于：

```text
{
  "文件名": [压缩流偏移, 压缩流大小],
  ...
}
```

目录中的 offset / size 指向 `.dat` 内的压缩流位置和压缩大小。实际文件内容是对应 raw-deflate 流的解压结果。

其中第 1 个 4 字节不是密钥；对本样本：`01 13 7D F7` 按大端序解析为 `0x01137DF7`，正好等于最后一个目录表压缩流的起始偏移。修改资源重新封包时，该值必须随目录表新位置一起重算。

当前样本统计：

```text
streams = 233
files   = 232
zero-size entries = 1
```

其中最后 1 个 stream 是目录表本身，不作为普通文件导出。`saveload.dat` 是目录中的 0 字节占位项。

## 命令

### 验证

```bash
python yuukai_dat_tool.py verify system.dat
```

### 查看目录

```bash
python yuukai_dat_tool.py list system.dat --limit 30
```

### 解包

```bash
python yuukai_dat_tool.py unpack system.dat system_unpack
```

输出结构：

```text
system_unpack/
├─ _manifest.json        # 封包重建用清单，不要删除
├─ _catalog.amf3.bin     # 原目录表解压结果，调试用
└─ files/                # 实际导出的资源文件
   ├─ config.dat
   ├─ title.dat
   ├─ *.png
   └─ *.mp3
```

### 封包

```bash
python yuukai_dat_tool.py pack system_unpack system_new.dat
```

工具会读取 `system_unpack/_manifest.json`，按原始物理流顺序重建 raw-deflate 流，并重新生成 AMF3 目录表。

### 零修改回环测试

```bash
python yuukai_dat_tool.py roundtrip system.dat rt_work
```

本样本已经验证：未修改任何导出文件时，`roundtrip` 生成的 `rebuilt.dat` 与原始 `system.dat` 字节完全一致。

## 加密 / 压缩说明

严格说该 `.dat` 没有额外 XOR、AES 或自定义加密层。它的“看起来像密文”的原因是每个文件都是独立的 raw-deflate 压缩流，且目录表也被 raw-deflate 压缩后放在文件末尾。

读取流程是：

```text
1. 读取前 4 字节，大端序得到 catalog_offset
2. 跳到 catalog_offset，按 raw-deflate 解出 AMF3 目录表
3. 从目录表取 filename -> [compressed_offset, compressed_size]
4. 跳到 compressed_offset，读取 compressed_size 字节
5. 对该段做 raw-deflate 解压，得到真实 PNG/MP3/DAT 内容
```

封包流程反过来：

```text
1. 每个文件用 raw-deflate 重新压缩
2. 顺序写入所有压缩流
3. 记录每个流的新 offset/size
4. 生成 AMF3 目录表
5. raw-deflate 压缩目录表并追加到末尾
6. 把目录表压缩流起始偏移写回文件开头 4 字节，大端序
```

## 修改资源时的注意点

1. 修改文件放在 `system_unpack/files/` 下，不要改 `_manifest.json` 的结构。
2. 文件名由目录表索引，建议不要随意改名；新增文件目前不作为常规工作流支持。
3. 修改 PNG/MP3/JSON 后直接 `pack` 即可；目录中的压缩 offset/size 会自动重算。
4. `--level` 默认是 9。该样本原始流使用 raw-deflate level 9；零修改回包可 byte-exact。
5. 修改资源后目录表位置通常会变，工具会自动重写开头 4 字节目录偏移。

## 已知限制

- 目前只实现了本样本确认到的 AMF3 目录子集：动态对象，值为 `[offset, size]` 两整数数组。
- 未实现新增文件的自动插入；需要新增资源时，应扩展 `_manifest.json` 和目录编码逻辑。
- 第 1 个 4 字节已确认是目录表压缩流偏移，大端序 u32。
