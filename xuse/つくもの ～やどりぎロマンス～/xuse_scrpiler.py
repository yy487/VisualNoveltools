from pathlib import Path
import sys
from typing import cast

from instructions import (
    Instruction,
    assemble_one_inst,
    h,
    parse_data,
    u16,
    u32,
    U32,
    BinaryReader,
    BinaryWriter,
    Bytes,
    String,
    de,
    retype_like,
    se,
    collect_files,
    read_json,
    write_json,
)


def crc16_xmodem(data: bytes | bytearray, crc=0) -> int:
    """
    XMODEM CRC16 算法。
    """
    for b in data:
        crc ^= b << 8
        for _ in range(8):
            if crc & 0x8000:
                crc = ((crc << 1) ^ 0x1021) & 0xFFFF
            else:
                crc = (crc << 1) & 0xFFFF
    return crc


def crc16_check(data_list: list[bytes]) -> bool:
    crc = 0
    for data in data_list:
        crc = crc16_xmodem(data, crc)
    return crc == 0


def calc_zero_crc16(data_list: list[bytes]) -> bytes:
    crc = 0
    for data in data_list:
        crc = crc16_xmodem(data, crc)
    return crc.to_bytes(2, byteorder="big")


# 注意，每条指令都是定长的，均为8字节
# 额外数据都放在block5里面，并且注意，
# 每个数据块的偏移均相对于某个块，而不是整个文件
INST_MAP = {
    h("03 00"): [u16, u32],
    h("05 00"): [u16, u32],
    # =========================================
    # 上面指令有额外数据（指向block5）
    # =========================================
    h("01 00"): [u16, u16, u16],
    h("02 00 FF FF FF FF FF FF"): [],
    h("04 00 FF FF"): [u32],
    h("06 00"): [u16, u32],
    h("07 00 FF FF"): [u32],
    h("09 00 FF FF"): [u32],
    h("0A 00 FF FF FF FF FF FF"): [],
    h("0B 00 FF FF FF FF FF FF"): [],
    h("0C 00"): [u16, u32],
    h("0D 00"): [u16, u32],
    h("0E 00"): [u16, u32],
    h("0F 00"): [u16, u32],
    h("10 00"): [u16, u32],
    h("13 00"): [u16, u32],
    h("14 00"): [u16, u32],
    h("16 00"): [u16, u32],
    h("17 00"): [u16, u32],
    h("19 00"): [u16, u32],
    h("20 00"): [u16, u32],
    h("21 00 FF FF FF FF FF FF"): [],
    h("22 00 FF FF FF FF FF FF"): [],
    h("26 01"): [u32, u16.eq(0xFFFF)],
    h("27 00 FF FF FF FF FF FF"): [],
    h("29 00"): [u16, u32.eq(0xFFFFFFFF)],
    h("2A 00"): [u16, u32.eq(0xFFFFFFFF)],
    h("36 00"): [u32, u16.eq(0xFFFF)],
    h("37 00"): [u32, u16.eq(0xFFFF)],
    h("3C 00"): [u16, u16, u16],
    h("3E 00"): [u32, u16.eq(0xFFFF)],
    h("42 00"): [u32, u16.eq(0xFFFF)],
    h("43 00 FF FF FF FF FF FF"): [],
    h("4E 00"): [u32, u16.eq(0xFFFF)],
    h("55 00"): [u32, u16.eq(0xFFFF)],
    h("56 00"): [u32, u16.eq(0xFFFF)],
    h("57 00"): [u32, u16.eq(0xFFFF)],
    h("58 00"): [u32, u16.eq(0xFFFF)],
    h("59 00"): [u32, u16.eq(0xFFFF)],
    h("5A 00"): [u32, u16.eq(0xFFFF)],
    h("5B 00"): [u32, u16.eq(0xFFFF)],
    h("61 00"): [u32, u16.eq(0xFFFF)],
    h("62 00"): [u32, u16.eq(0xFFFF)],
    h("63 00"): [u32, u16.eq(0xFFFF)],
    h("64 00"): [u16, u16, u16.eq(0xFFFF)],
    h("66 00"): [u32, u16.eq(0xFFFF)],
    h("6C 00"): [u32, u16.eq(0xFFFF)],
    h("6D 00"): [u32, u16.eq(0xFFFF)],
    h("6E 00"): [u32, u16.eq(0xFFFF)],
    h("6F 00"): [u32, u16.eq(0xFFFF)],
    h("70 00"): [u32, u16.eq(0xFFFF)],
    h("71 00"): [u32, u16.eq(0xFFFF)],
    h("73 00"): [u32, u16.eq(0xFFFF)],
    h("74 00"): [u32, u16.eq(0xFFFF)],
    h("75 00"): [u32, u16.eq(0xFFFF)],
    h("76 00"): [u32, u16.eq(0xFFFF)],
    h("78 00"): [u32, u16.eq(0xFFFF)],
    h("7C 00"): [u32, u16.eq(0xFFFF)],
    h("7D 00"): [u32, u16.eq(0xFFFF)],
    h("7E 00"): [u32, u16.eq(0xFFFF)],
    h("80 00"): [u32, u16.eq(0xFFFF)],
    h("84 00"): [u32, u16.eq(0xFFFF)],
    h("85 00"): [u32, u16.eq(0xFFFF)],
    h("86 00"): [u32, u16.eq(0xFFFF)],
    h("88 00"): [u32, u16.eq(0xFFFF)],
    h("89 00"): [u32, u16.eq(0xFFFF)],
    h("8A 00"): [u32, u16.eq(0xFFFF)],
    h("8B 00"): [u32, u16.eq(0xFFFF)],
    h("8C 00"): [u32, u16.eq(0xFFFF)],
    h("8D 00"): [u32, u16.eq(0xFFFF)],
    h("8E 00"): [u32, u16.eq(0xFFFF)],
    h("8F 00"): [u32, u16.eq(0xFFFF)],
    h("91 00"): [u32, u16.eq(0xFFFF)],
    h("94 00"): [u32, u16.eq(0xFFFF)],
    h("95 00"): [u32, u16.eq(0xFFFF)],
    h("96 00"): [u32, u16.eq(0xFFFF)],
    h("97 00"): [u32, u16.eq(0xFFFF)],
    h("98 00"): [u32, u16.eq(0xFFFF)],
    h("A6 00"): [u32, u16.eq(0xFFFF)],
    h("A8 00"): [u32, u16.eq(0xFFFF)],
    h("A9 00"): [u32, u16.eq(0xFFFF)],
    h("AA 00"): [u32, u16.eq(0xFFFF)],
    h("AB 00"): [u32, u16.eq(0xFFFF)],
    h("0C 01"): [u32, u16.eq(0xFFFF)],
    h("0D 01"): [u32, u16.eq(0xFFFF)],
    h("0F 01"): [u32, u16.eq(0xFFFF)],
    h("3D 01"): [u32, u16.eq(0xFFFF)],
    h("53 01"): [u32, u16.eq(0xFFFF)],
}

OFFSET_OPS = {
    "03 00": [1],
    "05 00": [1],
}


def get_sub_block_offset_from_inst(inst: Instruction) -> U32:
    op = inst["op"]
    if op not in OFFSET_OPS:
        raise ValueError(f"非法的OP {op}")
    idx = OFFSET_OPS[op][0]
    return cast(U32, de(inst["args"][idx]))


def decompile_sub_block(inst: Instruction, sub_block_data: Bytes) -> list[str]:
    value = []

    reader = BinaryReader(sub_block_data)

    match inst["op"]:
        # "05 00"是文本，包含了名字，对话，还有各种ID什么的，需要筛选一下
        # 判断是不是名字需要根据"03 00"块的参数（应该）
        case "05 00":
            str_len = int(de(inst["args"][0]))
            assert len(sub_block_data) == str_len
            value.append(se(String(xor_0x53(sub_block_data).decode("cp932"))))
            return value
        case "03 00":
            for _ in range(2):
                value.append(se(reader.read_u32()))
            rest_bytes = reader.read_rest_bytes()
            if len(rest_bytes) > 0:  # 注音？
                sub_reader = BinaryReader(xor_0x53(rest_bytes))
                str_len = sub_reader.read_u32()
                s = sub_reader.read_bytes(str_len).decode("cp932")
                value.append(s)
            return value
        case _:
            raise ValueError("不可能")


def compile_sub_block(inst: Instruction, value: list[str]) -> bytes:
    """根据指令反解并编译对应的 sub_block 数据。"""
    match inst["op"]:
        case "05 00":
            s = cast(String, de(value[0]))
            s_bytes = xor_0x53(s.encode("cp932"))
            inst["args"][0] = se(retype_like(de(inst["args"][0]), len(s_bytes)))
            return s_bytes
        case "03 00":
            assert len(value) in (2, 3)
            writer = BinaryWriter()
            writer.write_u32(int(de(value[0])))
            writer.write_u32(int(de(value[1])))
            if len(value) == 3:
                s_bytes = xor_0x53(cast(String, de(value[2])).encode("cp932"))
                s_len_bytes = xor_0x53(len(s_bytes).to_bytes(4, "little"))
                writer.write_bytes(s_len_bytes)
                writer.write_bytes(s_bytes)
            return writer.to_bytes()
        case _:
            raise ValueError("不可能")


def xor_0x53(bs: bytes) -> bytes:
    return bytes([b ^ 0x53 for b in bs])


def decompile(input_root: Path, output_root: Path) -> None:
    """反编译：将二进制文件转换为JSON"""
    cd_files = collect_files(input_root, ".cd3")

    for file in cd_files:
        reader = BinaryReader(file.read_bytes())

        # 解析header------------------------------------------
        magic = reader.read_bytes(4)
        assert magic == b"SMK "

        version = reader.read_u32()
        assert version == 0x10000

        block1_count = reader.read_u32()
        block1_size = reader.read_u32()

        block2_count = reader.read_u32()
        block2_size = reader.read_u32()

        block3_count = reader.read_u32()
        block3_size = reader.read_u32()

        block4_size = reader.read_u32()
        block5_size = reader.read_u32()

        header_crc = reader.read_bytes(2)

        assert crc16_check([
            magic,
            version.to_bytes(4, "little"),
            block1_count.to_bytes(4, "little"),
            block1_size.to_bytes(4, "little"),
            block2_count.to_bytes(4, "little"),
            block2_size.to_bytes(4, "little"),
            block3_count.to_bytes(4, "little"),
            block3_size.to_bytes(4, "little"),
            block4_size.to_bytes(4, "little"),
            block5_size.to_bytes(4, "little"),
            header_crc,
        ])

        # 解析block123------------------------------------------
        sub_reader = BinaryReader(reader.read_bytes(block1_size))
        block1 = []
        crc = 0
        for _ in range(block1_count):
            offset = sub_reader.read_u32()
            raw_size = xor_0x53(sub_reader.read_bytes(4))
            size = int.from_bytes(raw_size, "little")
            raw_payload = xor_0x53(sub_reader.read_bytes(size))
            payload = raw_payload.decode("cp932")

            crc = crc16_xmodem(offset.to_bytes(4, "little"), crc)
            crc = crc16_xmodem(raw_size, crc)
            crc = crc16_xmodem(raw_payload, crc)

            block1.append({"offset": offset, "size": size, "payload": payload})
        assert sub_reader.is_eof()
        block1_crc = reader.read_bytes(2)
        assert crc16_xmodem(block1_crc, crc) == 0

        sub_reader = BinaryReader(reader.read_bytes(block2_size))
        block2 = []
        crc = 0
        for _ in range(block2_count):
            offset = sub_reader.read_u32()
            raw_size = xor_0x53(sub_reader.read_bytes(4))
            size = int.from_bytes(raw_size, "little")
            raw_payload = xor_0x53(sub_reader.read_bytes(size))
            payload = raw_payload.decode("cp932")

            crc = crc16_xmodem(offset.to_bytes(4, "little"), crc)
            crc = crc16_xmodem(raw_size, crc)
            crc = crc16_xmodem(raw_payload, crc)

            block2.append({"offset": offset, "size": size, "payload": payload})
        assert sub_reader.is_eof()
        block2_crc = reader.read_bytes(2)
        assert crc16_xmodem(block2_crc, crc) == 0

        sub_reader = BinaryReader(reader.read_bytes(block3_size))
        block3 = []
        crc = 0
        for _ in range(block3_count):
            offset = sub_reader.read_u32()
            raw_size = xor_0x53(sub_reader.read_bytes(4))
            size = int.from_bytes(raw_size, "little")
            raw_payload = xor_0x53(sub_reader.read_bytes(size))
            payload = raw_payload.decode("cp932")

            crc = crc16_xmodem(offset.to_bytes(4, "little"), crc)
            crc = crc16_xmodem(raw_size, crc)
            crc = crc16_xmodem(raw_payload, crc)

            block3.append({"offset": offset, "size": size, "payload": payload})
        assert sub_reader.is_eof()
        block3_crc = reader.read_bytes(2)
        assert crc16_xmodem(block3_crc, crc) == 0

        block4 = reader.read_bytes(block4_size)
        block4_crc = reader.read_bytes(2)
        assert crc16_check([block4, block4_crc])

        block5 = reader.read_bytes(block5_size)
        block5_crc = reader.read_bytes(2)
        assert crc16_check([block5, block5_crc])

        assert reader.is_eof()

        # 解析block4------------------------------------
        inst_reader = BinaryReader(block4)
        insts = parse_data(
            {
                "file_name": str(file),
                "offset": 0,
            },
            inst_reader,
            INST_MAP,
        )

        # 解析block5----------------------------------
        last_offset = None
        sub_block = []
        for inst_i, inst in enumerate(insts):
            if inst["op"] in OFFSET_OPS:
                offset = get_sub_block_offset_from_inst(inst)
                if last_offset is None:
                    assert offset == 0
                    last_offset = offset
                    inst["meta"]["target"] = len(sub_block)
                    sub_block.append({
                        "target_op": inst["op"],
                        "target_op_index": inst_i,
                        "offset": offset,
                    })
                    continue

                assert last_offset < offset
                sub_block[-1]["value"] = decompile_sub_block(
                    insts[sub_block[-1]["target_op_index"]],
                    Bytes(block5[last_offset:offset]),
                )
                last_offset = offset
                inst["meta"]["target"] = len(sub_block)
                sub_block.append({
                    "target_op": inst["op"],
                    "target_op_index": inst_i,
                    "offset": offset,
                })

        if last_offset is not None:
            sub_block[-1]["value"] = decompile_sub_block(
                insts[sub_block[-1]["target_op_index"]],
                Bytes(block5[last_offset:]),
            )

        # 加长测试==================================================
        # inst_i = 0
        # while inst_i < len(insts):
        #     inst = insts[inst_i]
        #     # 选项
        #     if inst["op"] == "64 00":
        #         select_count = int(de(inst["args"][0]))
        #         inst_i += 1
        #         for i in range(inst_i, inst_i + select_count):
        #             text_inst = insts[i]
        #             assert text_inst["op"] == "05 00"
        #             sub = sub_block[text_inst["meta"]["target"]]
        #             sub["value"][0] = "中文" + sub["value"][0]

        #         inst_i += select_count
        #         continue

        #     # "01 00"为无名字对话，"3C 00"为有名字对话
        #     if inst["op"] in ("01 00", "3C 00"):
        #         totol_count = int(de(inst["args"][0]))
        #         text_count = int(de(inst["args"][1]))
        #         extra_03_count = int(de(inst["args"][2]))
        #         assert totol_count - text_count - extra_03_count == 1

        #         # 随后的第一个OP一定是 "03 00"
        #         inst_i += 1
        #         assert insts[inst_i]["op"] == "03 00"

        #         # 然后是一系列 "05 00"
        #         inst_i += 1
        #         for i in range(inst_i, inst_i + text_count):
        #             text_inst = insts[i]
        #             assert text_inst["op"] == "05 00"
        #             sub = sub_block[text_inst["meta"]["target"]]
        #             if (
        #                 inst["op"] == "3C 00"
        #                 and i == inst_i
        #                 and len(sub["value"][0]) >= 8
        #             ):
        #                 # 最大名字长度只能是8，否则游戏会报错
        #                 pass
        #             else:
        #                 sub["value"][0] = "中文" + sub["value"][0]
        #         inst_i += text_count + extra_03_count
        #         continue

        #     inst_i += 1
        # 加长测试===================================================

        # 注意，block123的偏移均指向指令，但是预期我们并不会修改指令的个数和长度
        # 所以block123的偏移不需要修复。
        smk_file = {
            "magic": se(magic),
            "version": version,
            "block1_count": block1_count,
            "block1_size": block1_size,
            "block2_count": block2_count,
            "block2_size": block2_size,
            "block3_count": block3_count,
            "block3_size": block3_size,
            "block4_size": block4_size,
            "block5_size": block5_size,
            "header_crc": se(header_crc),
            "block1": block1,
            "block1_crc": se(block1_crc),
            "block2": block2,
            "block2_crc": se(block2_crc),
            "block3": block3,
            "block3_crc": se(block3_crc),
            "insts": insts,
            "sub_block": sub_block,
        }

        # 保存为JSON
        rel_path = file.relative_to(input_root)
        out_file = output_root / f"{rel_path.as_posix()}.json"
        out_file.parent.mkdir(parents=True, exist_ok=True)

        write_json(out_file, smk_file)

    print(f"[OK] decompile 完成: {input_root} -> {output_root}")


def compile(input_root: Path, output_root: Path) -> None:
    """编译：将JSON转换回二进制文件"""
    files = collect_files(input_root, "json")

    for file in files:
        smk_data = read_json(file)

        insts: list[Instruction] = smk_data["insts"]
        sub_blocks: list[dict] = smk_data["sub_block"]

        # ========= 第一步：编译 sub_block 并同步更新 insts 中的偏移和长度参数 =========
        writer5 = BinaryWriter()
        for sb in sub_blocks:
            target_idx = sb["target_op_index"]
            target_inst = insts[target_idx]

            # 记录当前在 block5 里面的偏移，用于填回指令参数
            new_offset = writer5.tell()

            # 编译当前 sub_block 数据
            sb_bytes = compile_sub_block(target_inst, sb["value"])
            writer5.write_bytes(sb_bytes)

            # 更新 target_inst 中指向 sub_block 的 offset 参数
            op = target_inst["op"]
            offset_arg_idx = OFFSET_OPS[op][0]
            old_offset_val = de(target_inst["args"][offset_arg_idx])
            target_inst["args"][offset_arg_idx] = se(
                retype_like(old_offset_val, new_offset)
            )

        block5_bytes = writer5.to_bytes()
        block5_crc = calc_zero_crc16([block5_bytes])

        # ========= 第二步：编译更新过参数的指令 (block4) =========
        block4_bytes = b"".join([assemble_one_inst(inst) for inst in insts])
        block4_crc = calc_zero_crc16([block4_bytes])

        # ========= 第三步：重组 block1, block2, block3 =========
        def compile_block_list(block_list: list[dict]) -> tuple[bytes, bytes]:
            w = BinaryWriter()
            crc = 0
            for item in block_list:
                offset_bytes = item["offset"].to_bytes(4, "little")
                w.write_bytes(offset_bytes)

                payload_bytes = item["payload"].encode("cp932")
                raw_size_bytes = len(payload_bytes).to_bytes(4, "little")

                # 写入加密后的数据
                w.write_bytes(xor_0x53(raw_size_bytes))
                w.write_bytes(xor_0x53(payload_bytes))

                # CRC计算严格基于原文数据（解密状态），与 decompile 保持一致
                crc = crc16_xmodem(offset_bytes, crc)
                crc = crc16_xmodem(raw_size_bytes, crc)
                crc = crc16_xmodem(payload_bytes, crc)

            b = w.to_bytes()
            return b, crc.to_bytes(2, byteorder="big")

        block1_bytes, block1_crc = compile_block_list(smk_data["block1"])
        block2_bytes, block2_crc = compile_block_list(smk_data["block2"])
        block3_bytes, block3_crc = compile_block_list(smk_data["block3"])

        # ========= 第四步：构建 Header (重新计算大小和 CRC) =========
        header_writer = BinaryWriter()
        header_writer.write_bytes(cast(Bytes, de(smk_data["magic"])))
        header_writer.write_u32(smk_data["version"])
        header_writer.write_u32(smk_data["block1_count"])
        header_writer.write_u32(len(block1_bytes))
        header_writer.write_u32(smk_data["block2_count"])
        header_writer.write_u32(len(block2_bytes))
        header_writer.write_u32(smk_data["block3_count"])
        header_writer.write_u32(len(block3_bytes))
        header_writer.write_u32(len(block4_bytes))
        header_writer.write_u32(len(block5_bytes))

        header_bytes = header_writer.to_bytes()
        header_crc = calc_zero_crc16([header_bytes])

        # ========= 第五步：合并整个二进制文件并输出 =========
        final_writer = BinaryWriter()
        final_writer.write_bytes(header_bytes)
        final_writer.write_bytes(header_crc)

        final_writer.write_bytes(block1_bytes)
        final_writer.write_bytes(block1_crc)

        final_writer.write_bytes(block2_bytes)
        final_writer.write_bytes(block2_crc)

        final_writer.write_bytes(block3_bytes)
        final_writer.write_bytes(block3_crc)

        final_writer.write_bytes(block4_bytes)
        final_writer.write_bytes(block4_crc)

        final_writer.write_bytes(block5_bytes)
        final_writer.write_bytes(block5_crc)

        # 保存二进制文件
        rel_path = file.relative_to(input_root)
        out_file = output_root / rel_path.with_suffix("")  # 脱掉 .json 的后缀
        out_file.parent.mkdir(parents=True, exist_ok=True)

        out_file.write_bytes(final_writer.to_bytes())

    print(f"[OK] compile 完成: {input_root} -> {output_root}")


if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("用法: python xuse_scrpiler.py <decompile|compile> <in> <out>")
        sys.exit(1)

    mode = sys.argv[1]
    in_path = Path(sys.argv[2])
    out_path = Path(sys.argv[3])

    if mode == "decompile":
        decompile(in_path, out_path)
    elif mode == "compile":
        compile(in_path, out_path)
    else:
        print(f"错误: 未知的模式 '{mode}'，只能是 'decompile' 或 'compile'")
        sys.exit(1)
