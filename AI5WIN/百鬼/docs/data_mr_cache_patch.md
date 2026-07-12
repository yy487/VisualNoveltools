# AI5WIN / 百鬼 `DATA.MR` 字库扩容缓存区 Patch 说明

本文记录 `DATA.MR` 全量扩容字库为什么会触发 `DATA.MR - ファイル読み込みエラー。`，以及如果要支持扩展后的 `DATA.MR`，exe 内缓存区应如何 patch。

## 1. 当前问题

原版 `DATA.MR`：

```text
压缩态大小: 0x150F93  = 1,380,243 bytes
解压态大小: 0x4F8162  = 5,210,466 bytes
```

全量重绘扩容版 `DATA_MR_chs_full.MR`：

```text
压缩态大小: 0x28258E  = 2,631,054 bytes
解压态大小: 0x77053E  = 7,800,126 bytes
```

原版 exe 内部把 `DATA.MR` 解压输出区和压缩输入临时区切在 `base + 0x500000`：

```asm
0040AA62  mov eax, [esi+000004F8h]     ; eax = base
0040AA68  add eax, 00500000h           ; eax = base + 0x500000
0040AA76  mov [esi+000016D4h], eax     ; compressed input temp pointer
```

也就是说内存布局近似为：

```text
base = [this + 0x4F8]

base + 0x000000 ~ base + 0x4FFFFF    DATA.MR 解压输出区
base + 0x500000 ~ ...                压缩输入临时区 / resource read temp
```

原版解压态 `0x4F8162` 只比 `0x500000` 小 `0x7E9E`，余量约 31.65 KiB。扩容版解压态 `0x77053E` 超过 `0x500000`，解压输出会覆盖压缩输入临时区，导致资源读取或解压阶段失败。

因此，`DATA_MR_chs_full.MR` 报错不是 ARC 封包 size/offset 的问题，而是 exe 内部缓存区边界问题。

## 2. 已确认的关键 Patch 点

### 2.1 解压输出区 / 压缩输入区分界

位置：

```text
VA:          0x0040AA68
File offset: 0x0000AA68
原始字节:    05 00 00 50 00
含义:        add eax, 0x500000
```

可改为：

```text
目标值:      0x800000
新字节:      05 00 00 80 00
含义:        add eax, 0x800000
```

即：

```asm
mov eax, [esi+000004F8h]
add eax, 00800000h
mov [esi+000016D4h], eax
```

这样 `DATA.MR` 解压输出区扩大到 8 MiB，足够容纳当前 full 版：

```text
full 解压态: 0x77053E
新边界:      0x800000
余量:        0x8FAC2 约 574 KiB
```

## 3. 不能只改 0x500000 的原因

只把 `0x500000` 改为 `0x800000` 还不一定安全。原因是 `[this+0x4F8]` 指向的大缓冲区总分配大小也必须足够。

改成 `0x800000` 后，内存需求至少是：

```text
DATA.MR 解压输出区:       0x800000
DATA.MR 压缩输入临时区:   0x28258E
基本合计:                 0xA8258E
建议安全分配:             0xB00000 或 0xC00000
```

如果原始大缓冲区只分配了约 `0x650000` / `0x800000` / 其他不足大小，那么只改 `0040AA68` 会把压缩输入临时区移到未分配内存，仍然会崩溃。

所以 patch 需要两步：

```text
Step 1: 把 split pointer 从 base+0x500000 改为 base+0x800000。
Step 2: 找到 base 缓冲区的分配大小，把它扩大到至少 0xB00000，推荐 0xC00000。
```

## 4. 如何继续定位总分配大小

目前静态上已确认 `[this+0x4F8]` 在初始化函数里被使用，但它不是在 `0x40AA68` 附近分配的，而是在更早的对象/资源缓冲初始化阶段已经准备好。

推荐用 x32dbg 动态定位：

### 4.1 下断点

1. 在 `0040AA62` 下断点。
2. 运行到断点时记录：

```text
this = ESI
base = dword ptr [ESI+0x4F8]
temp = dword ptr [ESI+0x16D4]
```

3. 对 `[ESI+0x4F8]` 设置硬件写入断点，重启游戏，观察是谁写入了 base 指针。

### 4.2 找分配调用

写入 `[this+0x4F8]` 的上游附近通常会出现：

```text
HeapAlloc / GlobalAlloc / LocalAlloc / VirtualAlloc / malloc / new
```

或者封装跳板函数。

需要确认：

```text
分配函数地址
原始分配 size
返回指针保存到哪个 this 字段
[this+0x4F8] 是否只是大池中的一个子区域
```

### 4.3 Patch 分配 size

如果找到类似：

```asm
push 00680000h
call allocator
mov [this+04F8h], eax
```

则改为：

```asm
push 00C00000h
call allocator
mov [this+04F8h], eax
```

如果是：

```asm
mov eax, 00680000h
push eax
call allocator
```

同样把立即数改成 `0x00C00000`。

如果分配大小来自表或配置，则改对应表项。

## 5. 推荐 Patch 参数

针对当前 full 字库，建议参数：

```text
DATA.MR split boundary: 0x800000
resource pool minimum:  0xB00000
resource pool safer:    0xC00000
```

原因：

```text
0x800000 > 0x77053E，能容纳 full 解压数据。
0x800000 + 0x28258E = 0xA8258E，压缩输入临时区也有位置。
0xB00000 留约 0x7DA72 余量。
0xC00000 更安全，留约 0x17DA72 余量。
```

## 6. Patch 后验证顺序

不要直接只测 full 版。建议按下面顺序：

### 6.1 原版 DATA.MR 回归测试

```text
目标：确认 exe patch 没破坏原资源加载。
操作：只 patch exe，不替换 DATA.MR。
结果：游戏应正常启动。
```

### 6.2 existing 字库测试

```text
目标：确认重绘 glyph 格式没问题。
操作：替换不扩容的 DATA_MR_chs_existing.MR。
结果：游戏应正常启动，已有码位的简体字可显示。
```

### 6.3 medium 扩容测试

```text
目标：确认新 split boundary 和 pool size 生效。
操作：构造一个解压态 5.5MB~6.5MB 的 DATA.MR。
结果：正常启动说明超过 0x500000 后仍可读写。
```

### 6.4 full 扩容测试

```text
目标：确认完整扩容可运行。
操作：替换 DATA_MR_chs_full.MR。
结果：游戏应不再出现 DATA.MR 文件读取错误。
```

## 7. 失败现象和判断

### 7.1 仍然报 `DATA.MR - ファイル読み込みエラー。`

优先怀疑：

```text
1. 分配 size 没 patch，base+0x800000 指向未分配区。
2. 仍然有其他读取上限或临时区上限。
3. ARC 内 DATA.MR 压缩流被截断。
4. LZSS 压缩流本身损坏。
```

### 7.2 启动直接崩溃

优先怀疑：

```text
1. 只改了 split pointer，没改总分配。
2. 分配 size patch 错位置。
3. 其他 this 字段依赖原有大池布局。
```

### 7.3 启动正常但文字缺字

这不是缓存区问题，而是：

```text
1. 脚本文本没有做 subs_cn_jp 映射。
2. 映射后的借码字符没进入 DATA.MR 字表。
3. 对应 glyph 没重绘。
4. 当前文本显示路径使用了另一组字表。
```

## 8. 更稳的替代路线：不扩容字库

如果不想 patch exe 缓存区，推荐 no-grow 方案：

```text
1. 统计翻译文本实际用到的简体字。
2. 按 subs_cn_jp.json 映射成 cp932 借码字符。
3. 只处理实际需要的借码字符。
4. 对不在 DATA.MR 字表里的借码字符，从原字表里替换 unused 槽位。
5. 不改变 DATA.MR 总大小，不改变 offset table 大范围布局。
```

这种方案不会碰 `0x500000` 缓冲边界，稳定性高于 exe patch。

## 9. 当前结论

```text
ARC 工具流程本身没问题。
DATA_MR_chs_full.MR 失败的直接原因是解压态超过 exe 内部 0x500000 分界。
最小 patch 点是 VA 0x0040AA68 / file offset 0x0000AA68 的立即数。
完整 patch 还必须找到并扩大 [this+0x4F8] 对应的大缓冲区总分配。
推荐 split=0x800000，pool=0xC00000。
```

