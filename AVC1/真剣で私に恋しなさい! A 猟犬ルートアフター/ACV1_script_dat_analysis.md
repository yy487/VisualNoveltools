# ACV1 script.dat 解包算法分析

## 结论

本 `script.dat` 的实际格式不是 `avc_codec.py` 中的 SETSUEI/ARCHIVE 格式。`avc_codec.py` 只能作为“索引表 + XOR + 数据区”的设计参照；最终算法按当前 exe 导出的反汇编确定。

当前样本：

- 文件头：`ACV1`，即 little-endian DWORD `0x31564341`。
- 索引数量：`u32[0x04] ^ 0x8B6A4E5F`，样本为 56 项。
- 每项索引大小：`0x15` 字节。
- payload：先按 DWORD XOR，再 zlib inflate。
- 解包验证：56/56 项可成功解出，解出后为 cp932 脚本文本。

## 反汇编依据

### 1. `sub_4CF6E0`：读 `script.dat` 并解析索引

关键逻辑：

```c
n827736897 = sub_4DBC40(fp);
if ( n827736897 == 827736897 ) {       // 0x31564341 == "ACV1"
    v14 = -1955967393;                  // 0x8B6A4E5F
    v5 = sub_4DBC40(fp) ^ 0x8B6A4E5F;   // count
} else {
    v5 = n827736897 ^ 0x26ACA46E;       // legacy/non-ACV1 branch
    v14 = 0;
}

for (; v5; --v5) {
    v6 = sub_4DBC60(fp);                // 8-byte key，拆成 lo/hi
    v11[256] = v6 ^ read_u8(fp);        // flag
    *((DWORD*)v11 + 65) = v14 ^ v6 ^ read_u32(fp); // offset
    *((DWORD*)v11 + 66) = v6 ^ read_u32(fp);       // packed_size
    *((DWORD*)v11 + 67) = v6 ^ read_u32(fp);       // output capacity
}
```

所以 ACV1 分支的表项结构是：

```text
0x00 u64 key
0x08 u8  enc_flag          -> flag = enc_flag ^ low8(key_lo)
0x09 u32 enc_offset        -> offset = enc_offset ^ 0x8B6A4E5F ^ key_lo
0x0D u32 enc_packed_size   -> packed_size = enc_packed_size ^ key_lo
0x11 u32 enc_out_capacity  -> out_capacity = enc_out_capacity ^ key_lo
```

### 2. `sub_4CBC50`：生成 payload XOR 基础 key

`sub_4CBC50` 在成功打开 dat 后，对 rdata 中的标题字符串：

```text
真剣で私に恋しなさい！Ａ猟犬ルートアフター
```

计算 CRC64-ECMA，初值 `0xFFFFFFFFFFFFFFFF`，final XOR 为 `~crc`。样本得到：

```text
crc64 = 0x7e47e47ad95e9c65
crc_low = 0xd95e9c65
```

后续实际用于 payload XOR 的是低 32 位。

### 3. `sub_4CFEF0`：读 payload、XOR、zlib inflate

关键逻辑：

```c
fseek(fp, entry[69], SEEK_SET);     // offset
size = entry[70];                   // packed_size
out_cap = entry[71];                // output capacity
buf = malloc(size);
fread(buf, size, 1, fp);

xor_key = dword_D790D8 ^ entry_key_lo;
for (i = 0; i < size >> 2; i++)
    ((DWORD*)buf)[i] ^= xor_key;

out = malloc(2 * out_cap);
sub_537260(out, &out_cap, buf, size); // zlib inflate, version string "1.2.3"
```

注意：`size >> 2` 说明只 XOR 完整 DWORD，尾部 1~3 字节不处理。

## 与 avc_codec.py 的差异

`avc_codec.py` 描述的是另一种 AVC/SETSUEI 归档：`0x08..0x10` 推导 8 字节 key，`0x10..0x34` 是 XOR 后的 `ARCHIVE\0` header，entry 大小 `0x114` 且包含文件名。

本 `script.dat` 则是：

- `ACV1` magic 在 0x00。
- 没有 `ARCHIVE\0` header。
- 没有明文/加密文件名表。
- index entry 是固定 `0x15` 字节。
- 文件名需要从解出的脚本文本 label 猜测，或保留 entry 编号。

## 使用方式

```bat
python acv1_dat_unpack.py script.dat script_unpacked
```

可选指定标题字符串：

```bat
python acv1_dat_unpack.py script.dat script_unpacked --game-title "真剣で私に恋しなさい！Ａ猟犬ルートアフター"
```

输出：

- `*.txt`：解出的 cp932 脚本文本。
- `manifest.json`：索引项、offset、packed size、解包后 size、CRC key 等元信息。

## 当前样本验证

```text
[unpack] entries=56 packed=192633 unpacked=578655
```

前几项：

```text
000_net.txt
001_flag_format.txt
002_entry.txt
003_local_setting.txt
004_pos_setting.txt
005_DAY.txt
...
009_A1_0000_0.txt
010_A1_0000_1.txt
```

## 后续回包方向

回包也能按同一结构反推：

1. 每个 txt 用 zlib compress。
2. 对压缩流的完整 DWORD 执行 `dword ^= crc_low ^ key_lo`。
3. 重写 payload 区。
4. 更新索引中的 offset / packed_size / out_capacity：
   - `enc_offset = new_offset ^ 0x8B6A4E5F ^ key_lo`
   - `enc_packed_size = packed_size ^ key_lo`
   - `enc_out_capacity = capacity ^ key_lo`
5. 若保持原 key_lo/key_hi 不变，则脚本查找哈希不受影响。

目前我只交付解包器；回包要继续确认 `out_capacity` 是否允许直接写实际解压长度，还是需要保留/扩大到引擎预估容量。
