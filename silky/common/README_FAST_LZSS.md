# Silky ARC LZSS 多进程加速补丁

本补丁只加速 `silky-lzss` 回封时的 LZSS 压缩阶段，不改变 ARC 结构、不关闭压缩，因此输出体积仍应接近原来的 Script.arc。

用法：

```bat
python silky_arc_pack.py "<INPUT_DIR>" Script.arc
```

默认 `--jobs 0`，自动使用 CPU 核心数。也可以手动指定：

```bat
python silky_arc_pack.py "<INPUT_DIR>" Script.arc -j 8
```

如果出问题或想对照旧逻辑：

```bat
python silky_arc_pack.py "<INPUT_DIR>" Script.arc -j 1
```

不要使用 `--store-all`，否则会关闭压缩，体积会明显变大。
