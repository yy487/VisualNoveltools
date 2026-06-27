import json
from pathlib import Path
import sys
from typing import cast
from instructions import BinaryReader, BinaryWriter, de, se


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


def unpack(source: Path, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)

    reader = BinaryReader(source.read_bytes())

    header_magic = reader.read_bytes(4)
    assert header_magic == b"XARC"

    unknown1 = reader.read_bytes(4)
    header_crc1 = reader.read_bytes(2)
    assert crc16_check([header_magic, unknown1, header_crc1])

    header_magic2 = reader.read_bytes(2)
    assert header_magic2 == bytes.fromhex("01 10")

    flag = reader.read_u32()
    assert flag & 0xF == 0  # 如果不为0会走到另一个分支，比较复杂
    entry_count = reader.read_u32()
    header_crc2 = reader.read_bytes(2)
    assert entry_count != 0
    assert crc16_check([
        header_magic2,
        flag.to_bytes(4, byteorder="little"),
        entry_count.to_bytes(4, byteorder="little"),
        header_crc2,
    ])

    header_magic3 = reader.read_bytes(4)
    assert header_magic3 == b"DFNM"
    entry_offset = reader.read_u64()
    header_crc3 = reader.read_bytes(2)
    assert crc16_check([
        header_magic3,
        entry_offset.to_bytes(8, byteorder="little"),
        header_crc3,
    ])

    ndix_table_magic = reader.read_bytes(4)
    assert ndix_table_magic == b"NDIX"

    ndix_table = []
    for _ in range(entry_count):
        magic = reader.read_bytes(2)
        offset = reader.read_u32()
        crc = reader.read_bytes(2)
        assert crc16_check([magic, offset.to_bytes(4, byteorder="little"), crc])
        ndix_table.append({"magic": se(magic), "offset": offset, "crc": se(crc)})

    edix_table_magic = reader.read_bytes(4)
    assert edix_table_magic == b"EDIX"

    edix_table = []
    for _ in range(entry_count):
        magic = reader.read_bytes(2)
        offset = reader.read_u32()
        crc = reader.read_bytes(2)
        assert crc16_check([magic, offset.to_bytes(4, byteorder="little"), crc])
        edix_table.append({"magic": se(magic), "offset": offset, "crc": se(crc)})

    ctif_table_magic = reader.read_bytes(4)
    assert ctif_table_magic == b"CTIF"

    ctif_table = []
    for _ in range(entry_count):
        magic = reader.read_bytes(2)
        unk = reader.read_u32()
        str_len = reader.read_u16()
        crc1 = reader.read_bytes(2)
        assert crc16_check([
            magic,
            unk.to_bytes(4, byteorder="little"),
            str_len.to_bytes(2, byteorder="little"),
            crc1,
        ])
        raw_filename = reader.read_bytes(str_len)
        filename = bytes([b ^ 0x56 for b in raw_filename]).decode("cp932")
        crc2 = reader.read_bytes(2)
        assert crc16_check([raw_filename, crc2])
        ctif_table.append({
            "magic": se(magic),
            "unk": unk,
            "str_len": str_len,
            "crc1": se(crc1),
            "filename": filename,
            "crc2": se(crc2),
        })

    cadr_table_magic = reader.read_bytes(4)
    assert cadr_table_magic == b"CADR"

    cadr_table = []
    for _ in range(entry_count):
        magic = reader.read_bytes(2)
        offset = reader.read_u32()
        unk = reader.read_u32()
        crc = reader.read_bytes(2)
        assert crc16_check([
            magic,
            offset.to_bytes(4, byteorder="little"),
            unk.to_bytes(4, byteorder="little"),
            crc,
        ])
        cadr_table.append({
            "magic": se(magic),
            "offset": offset,
            "unk": unk,
            "crc": se(crc),
        })

    data_meta = []
    for i in range(entry_count):
        filename = ctif_table[i]["filename"]
        offset = cadr_table[i]["offset"]

        assert reader.tell() == offset

        magic = reader.read_bytes(4)
        assert magic == b"DATA"

        magic2 = reader.read_bytes(16)
        assert magic2 == bytes.fromhex(
            "20 00 00 00 D9 07 04 00 06 00 12 00 0A 00 08 00"
        )

        unk = reader.read_bytes(4)
        size = reader.read_u32()
        crc1 = reader.read_bytes(2)

        assert crc16_check([
            magic,
            magic2,
            unk,
            size.to_bytes(4, byteorder="little"),
            crc1,
        ])

        file_content = reader.read_bytes(size)
        crc2 = reader.read_bytes(2)

        assert crc16_check([file_content, crc2])

        out_file_path = output_dir / filename
        out_file_path.parent.mkdir(parents=True, exist_ok=True)
        out_file_path.write_bytes(file_content)

        data_meta.append({
            "magic": se(magic),
            "magic2": se(magic2),
            "unk": se(unk),
            "size": size,
            "crc1": se(crc1),
            "crc2": se(crc2),
        })

    assert reader.is_eof()

    meta_data = {
        "header_magic": se(header_magic),
        "unknown1": se(unknown1),
        "header_crc1": se(header_crc1),
        "header_magic2": se(header_magic2),
        "flag": flag,
        "entry_count": entry_count,
        "header_crc2": se(header_crc2),
        "header_magic3": se(header_magic3),
        "entry_offset": entry_offset,
        "header_crc3": se(header_crc3),
        "ndix_table_magic": se(ndix_table_magic),
        "ndix_table": ndix_table,
        "edix_table_magic": se(edix_table_magic),
        "edix_table": edix_table,
        "ctif_table_magic": se(ctif_table_magic),
        "ctif_table": ctif_table,
        "cadr_table_magic": se(cadr_table_magic),
        # 注意，我们预期只修改解包出来的文件的内容
        # 所以应该只有下面两个字段会变动
        "cadr_table": cadr_table,
        "data_meta": data_meta,
    }

    (output_dir / "__META__.json").write_text(
        json.dumps(meta_data, indent=2, ensure_ascii=False), encoding="utf-8"
    )

    print(f"[OK] unpack 完成: {source} -> {output_dir}")


def pack(input_root: Path, output_path: Path) -> None:
    if not input_root.is_dir():
        raise ValueError(f"输入目录不存在: {input_root}")

    meta_path = input_root / "__META__.json"
    if not meta_path.is_file():
        raise ValueError(f"未找到必要的打包元数据文件: {meta_path}")

    meta_data = json.loads(meta_path.read_text(encoding="utf-8"))
    entry_count = meta_data["entry_count"]

    # --- 1. 读取修改后的文件数据，准备更新 CADR 表的偏移量 ---
    files_data = []
    for i in range(entry_count):
        filename = meta_data["ctif_table"][i]["filename"]
        file_path = input_root / filename
        if not file_path.is_file():
            raise ValueError(f"打包失败，未找到对应的修改文件: {file_path}")
        files_data.append(file_path.read_bytes())

    # --- 2. 精确计算全表头体积，计算初始 DATA 的偏移 ---
    header_size = 10 + 12 + 14  # 基础头三个部分的长度
    header_size += 4 + 8 * entry_count  # NDIX 头 + 表项
    header_size += 4 + 8 * entry_count  # EDIX 头 + 表项

    ctif_size = 4
    for entry in meta_data["ctif_table"]:
        ctif_size += 12 + entry["str_len"]  # CTIF 头 + 变长表项
    header_size += ctif_size

    header_size += 4 + 12 * entry_count  # CADR 头 + 表项

    # 更新 CADR 表偏移
    current_data_offset = header_size
    for i in range(entry_count):
        meta_data["cadr_table"][i]["offset"] = current_data_offset
        # DATA 块结构: magic(4) + magic2(16) + unk(4) + size(4) + crc1(2) + content_size + crc2(2)
        content_size = len(files_data[i])
        data_block_size = 32 + content_size
        current_data_offset += data_block_size

    # --- 3. 严格按顺序利用 BinaryWriter 执行打包 ---
    writer = BinaryWriter()

    # 封入 Header 1
    magic = cast(bytes, de(meta_data["header_magic"]))
    unk1 = cast(bytes, de(meta_data["unknown1"]))
    writer.write_bytes(magic)
    writer.write_bytes(unk1)
    writer.write_bytes(calc_zero_crc16([magic, unk1]))

    # 封入 Header 2
    magic2 = cast(bytes, de(meta_data["header_magic2"]))
    writer.write_bytes(magic2)
    flag = meta_data["flag"]
    writer.write_u32(flag)
    writer.write_u32(entry_count)
    writer.write_bytes(
        calc_zero_crc16([
            magic2,
            flag.to_bytes(4, byteorder="little"),
            entry_count.to_bytes(4, byteorder="little"),
        ])
    )

    # 封入 Header 3
    magic3 = cast(bytes, de(meta_data["header_magic3"]))
    entry_offset = meta_data["entry_offset"]
    writer.write_bytes(magic3)
    writer.write_u64(entry_offset)
    writer.write_bytes(
        calc_zero_crc16([magic3, entry_offset.to_bytes(8, byteorder="little")])
    )

    # 封入 NDIX 表
    writer.write_bytes(cast(bytes, de(meta_data["ndix_table_magic"])))
    for entry in meta_data["ndix_table"]:
        e_magic = cast(bytes, de(entry["magic"]))
        e_offset = entry["offset"]
        writer.write_bytes(e_magic)
        writer.write_u32(e_offset)
        writer.write_bytes(
            calc_zero_crc16([e_magic, e_offset.to_bytes(4, byteorder="little")])
        )

    # 封入 EDIX 表
    writer.write_bytes(cast(bytes, de(meta_data["edix_table_magic"])))
    for entry in meta_data["edix_table"]:
        e_magic = cast(bytes, de(entry["magic"]))
        e_offset = entry["offset"]
        writer.write_bytes(e_magic)
        writer.write_u32(e_offset)
        writer.write_bytes(
            calc_zero_crc16([e_magic, e_offset.to_bytes(4, byteorder="little")])
        )

    # 封入 CTIF 表
    writer.write_bytes(cast(bytes, de(meta_data["ctif_table_magic"])))
    for entry in meta_data["ctif_table"]:
        e_magic = cast(bytes, de(entry["magic"]))
        e_unk = entry["unk"]
        str_len = entry["str_len"]

        writer.write_bytes(e_magic)
        writer.write_u32(e_unk)
        writer.write_u16(str_len)
        writer.write_bytes(
            calc_zero_crc16([
                e_magic,
                e_unk.to_bytes(4, byteorder="little"),
                str_len.to_bytes(2, byteorder="little"),
            ])
        )

        # 将文件名重新用 0x56 进行异或混淆
        encoded_name = entry["filename"].encode("cp932")
        raw_filename = bytes([b ^ 0x56 for b in encoded_name])
        writer.write_bytes(raw_filename)
        writer.write_bytes(calc_zero_crc16([raw_filename]))

    # 封入 CADR 表 (此时的 offset 已更新为 DATA 实际偏移)
    writer.write_bytes(cast(bytes, de(meta_data["cadr_table_magic"])))
    for entry in meta_data["cadr_table"]:
        e_magic = cast(bytes, de(entry["magic"]))
        e_offset = entry["offset"]
        e_unk = entry["unk"]

        writer.write_bytes(e_magic)
        writer.write_u32(e_offset)
        writer.write_u32(e_unk)
        writer.write_bytes(
            calc_zero_crc16([
                e_magic,
                e_offset.to_bytes(4, byteorder="little"),
                e_unk.to_bytes(4, byteorder="little"),
            ])
        )

    # 确保当前偏移准确
    assert writer.tell() == header_size

    # --- 4. 封入所有被修改大小或内容的 DATA 块 ---
    for i, file_content in enumerate(files_data):
        meta = meta_data["data_meta"][i]

        d_magic = cast(bytes, de(meta["magic"]))
        d_magic2 = cast(bytes, de(meta["magic2"]))
        d_unk = cast(bytes, de(meta["unk"]))
        size = len(file_content)

        writer.write_bytes(d_magic)
        writer.write_bytes(d_magic2)
        writer.write_bytes(d_unk)
        writer.write_u32(size)

        # 重算元数据 CRC1
        writer.write_bytes(
            calc_zero_crc16([
                d_magic,
                d_magic2,
                d_unk,
                size.to_bytes(4, byteorder="little"),
            ])
        )

        # 写入文件体及重算文件 CRC2
        writer.write_bytes(file_content)
        writer.write_bytes(calc_zero_crc16([file_content]))

    # 写出二进制
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_bytes(writer.to_bytes())

    print(
        f"[OK] pack 完成: {input_root} -> {output_path}",
    )


if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("用法: python xuse_packer.py <unpack|pack> <in> <out>")
        sys.exit(1)

    mode = sys.argv[1]
    in_path = Path(sys.argv[2])
    out_path = Path(sys.argv[3])

    if mode == "unpack":
        unpack(in_path, out_path)
    elif mode == "pack":
        pack(in_path, out_path)
    else:
        print(f"错误: 未知的模式 '{mode}'，只能是 'unpack' 或 'pack'")
        sys.exit(1)
