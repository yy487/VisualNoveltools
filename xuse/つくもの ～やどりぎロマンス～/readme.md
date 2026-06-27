# 使用方式

假设当前工作目录有

```
instructions.py
xuse_packer.py
xuse_scrpiler.py
CDFiles.arc   // 游戏里复制过来即可
```

## 解包

```powershell
python xuse_packer.py unpack CDFiles.arc script
python xuse_scrpiler.py decompile script raw
```

## 提取/替换文本

下面是示例代码，读取每个raw文件里的json，然后

```py
insts = json["insts"]
sub_block = json["sub_block"]

inst_i = 0
while inst_i < len(insts):
    inst = insts[inst_i]
    # 选项
    if inst["op"] == "64 00":
        select_count = int(de(inst["args"][0]))
        inst_i += 1
        for i in range(inst_i, inst_i + select_count):
            text_inst = insts[i]
            assert text_inst["op"] == "05 00"
            sub = sub_block[text_inst["meta"]["target"]]
            sub["value"][0] = "中文" + sub["value"][0]

        inst_i += select_count
        continue

    # "01 00"为无名字对话，"3C 00"为有名字对话
    if inst["op"] in ("01 00", "3C 00"):
        totol_count = int(de(inst["args"][0]))
        text_count = int(de(inst["args"][1]))
        extra_03_count = int(de(inst["args"][2]))
        assert totol_count - text_count - extra_03_count == 1

        # 随后的第一个OP一定是 "03 00"
        inst_i += 1
        assert insts[inst_i]["op"] == "03 00"

        # 然后是一系列 "05 00"
        inst_i += 1
        for i in range(inst_i, inst_i + text_count):
            text_inst = insts[i]
            assert text_inst["op"] == "05 00"
            sub = sub_block[text_inst["meta"]["target"]]
            if inst["op"] == "3C 00" and i == inst_i and len(sub["value"][0]) >= 8:
                # 最大名字长度只能是8，否则游戏会报错
                pass
            else:
                sub["value"][0] = "中文" + sub["value"][0]
        inst_i += text_count + extra_03_count
        continue

    inst_i += 1
```

没有测试过GBK，所以译文需要做日繁映射。

注意，这游戏坑比较多，比如名字长度不能太大，不能超过8个字符。
选项可能也有长度限制，但是我没有测试过。
最好按照这个示例提取/替换文本，其他一些文本改了可能报错，或者有什么很难发现的BUG。

游戏支持一定程度上的自动换行，但是超出太多，比如让游戏换太多行，就会报错。

像这样的对话，

```json
{
      "target_op": "05 00",
      "target_op_index": 118,
      "offset": 400,
      "value": [
        "俺は、慌てて身体を起こした。"
      ]
    },
    {
      "target_op": "05 00",
      "target_op_index": 119,
      "offset": 428,
      "value": [
        "まだ半分くらい頭が寝ぼけている。"
      ]
    },
```

改成这样，让它自动换行（这里为了测试换行极限，所以添加了很多字）

```json
{
      "target_op": "05 00",
      "target_op_index": 118,
      "offset": 400,
      "value": [
        "中文俺は、慌てて身体を起こした。中文まだ半分くらい頭が寝ぼけている。中文まだ半分くらい頭が寝ぼけている。中文まだ半分くらい頭が寝ぼけている。中文まだ半分くらい頭が寝ぼけている。中文まだ半分くらい頭が寝ぼけている。"
      ]
    },
    {
      "target_op": "05 00",
      "target_op_index": 119,
      "offset": 428,
      "value": [
        ""
      ]
    },
```

或者也可以自己换行。

## 封包

```powershell
python xuse_scrpiler.py compile raw script
python xuse_packer.py pack script CDFiles2.arc
```

`CDFiles2.arc`就是封包后的文件了。

注意，如果想输出到其他位置，别忘记`script`里面的`触手.vr3`和`__META__.json`，
`xuse_scrpiler.py`会忽略这两个文件，但是`xuse_packer.py`封包的时候需要这两个文件。
