# RealLive Seen.txt 工具
这好像还不能随便hook，容易出问题奇妙
本工具针对当前项目的 RealLive `Seen.txt`。文本流程为结构化解析：读取 10000 项索引表，解 SEEN 块，使用内置 `crypt_template.py` 中的 256 字节 XOR key 解密 packed code，再按 RealLive LZ 解压 VM code。提取和注入都不依赖裸扫 SJIS 字符串。

## 命令

提取：

```bat
python extract.py Seen.txt json --clean
```

注入：

```bat
python inject.py Seen.txt json\new chs\Seen.txt
```

指定 SEEN：

```bat
python extract.py Seen.txt json --seen 2 106 --clean
python inject.py Seen.txt json\new chs\Seen.txt --seen 2 106
```

可选字符映射：

```bat
python inject.py Seen.txt json\new chs\Seen.txt --map-json subs_cn_jp.json
```

## JSON

对话中 `【】` 内角色名拆为 `name`：

```json
{
  "name": "葵",
  "scr_msg": "「……」",
  "message": "「……」",
  "_scr_name": "葵",
  "_name_source": "bracket_prefix"
}
```

`_scr_name` 用于校验原始人名；翻译 `name` 不会导致 `scr_msg mismatch`。写回时工具会组合成 `【name】message`。

## 本版关键修复

1. 修复 `#00:01` 流程命令解析。RealLive 表达式字节码中的 `0x28 '('` 可能是运算符，不是嵌套括号；旧版把它当括号平衡，导致跳过位置错误。
2. 注入时重定位以下 VM 内联跳转目标：
   - `#00:01:0000 <u32 target>`
   - `#00:01:0002(expr) <u32 target>`
   - `#00:01:0005 <u32 target>`
   - `#00:01:0003(expr) { <u32 target>... }`
   - `#00:01:0008(expr) { <u32 target>... }`
   - `#00:01:0004(expr) { (case) <u32 target>... }`
3. 提取时跳过上述 flow jump table，不再把目标 offset 字节误提取成 `祐`、`弖`、`)就` 这类伪文本。
4. 零修改回环：`extract -> inject` 不改变原始 `Seen.txt`，byte-exact。



## Verification

Basic structural check:

```bat
python verify.py Seen_chs.txt
```

Check a patched file against the original Seen.txt and the JSON used for injection:

```bat
python verify.py chs\seen.txt --original seen.txt --json json\new
```

If injection used a character map, pass the same map:

```bat
python verify.py chs\seen.txt --original seen.txt --json json\new --map-json subs_cn_jp.json
```
