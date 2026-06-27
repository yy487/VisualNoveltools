; Full conservative disassembly for PS2A052A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      3  F300FC                                               IMM16_F3                  u16_be=252, u16_le=64512
0x0000000E      1  FF                                                   TERMINATOR_FF             
0x0000000F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000011      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000013      1  FF                                                   TERMINATOR_FF             
0x00000014      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000016      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000018      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x0000001A      1  FF                                                   TERMINATOR_FF             
0x0000001B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000001D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000001F      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00000021      1  FF                                                   TERMINATOR_FF             
0x00000022      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000024      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000026      1  FF                                                   TERMINATOR_FF             
0x00000027      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000029      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000002B      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000002D      1  FF                                                   TERMINATOR_FF             
0x0000002E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000030      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000032      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000034      1  FF                                                   TERMINATOR_FF             
0x00000035      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000037      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000039      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000003B      1  FF                                                   TERMINATOR_FF             
0x0000003C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000003E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000040      1  FF                                                   TERMINATOR_FF             
0x00000041      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000043      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000045      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000047      1  FF                                                   TERMINATOR_FF             
0x00000048      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000004A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000004C      3  F30E82                                               IMM16_F3                  u16_be=3714, u16_le=33294
0x0000004F      1  FF                                                   TERMINATOR_FF             
0x00000050      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000052      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000054     48  802E82DA82AD82BD82BF82CD82E082CC82E082A282ED82B8...  LEN8_STRING_CP932         length=46, text="ぼくたちはものもいわず、ゴミの山を駆けおりた。"
0x00000084      1  FF                                                   TERMINATOR_FF             
0x00000085      1  FF                                                   TERMINATOR_FF             
0x00000086      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000088      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000008A      1  FF                                                   TERMINATOR_FF             
0x0000008B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000008D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000008F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000091      1  FF                                                   TERMINATOR_FF             
0x00000092      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000094      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000096      3  F30E83                                               IMM16_F3                  u16_be=3715, u16_le=33550
0x00000099      1  FF                                                   TERMINATOR_FF             
0x0000009A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000009C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000009E    108  806A8FEA8F8A82C982E682C182C482CD8141935D82AA82E8...  LEN8_STRING_CP932         length=106, text="場所によっては、転がり落ちたでもいい。\nとにかくぼくたちは、この場所からできるだけ\n遠ざかろうとしていた。"
0x0000010A      1  FF                                                   TERMINATOR_FF             
0x0000010B      1  FF                                                   TERMINATOR_FF             
0x0000010C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000010E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000110      1  FF                                                   TERMINATOR_FF             
0x00000111      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000113      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000115      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000117      1  FF                                                   TERMINATOR_FF             
0x00000118      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000011A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000011C      3  F30E84                                               IMM16_F3                  u16_be=3716, u16_le=33806
0x0000011F      1  FF                                                   TERMINATOR_FF             
0x00000120      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000122      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000124    122  807882BB82EA82CD82E082A482B182C682CE82C582CD90E0...  LEN8_STRING_CP932         length=120, text="それはもうことばでは説明できない感情だった。自分の心の奥深くから噴き出した、死への恐怖。それが、ぼくたちを走らせていた。"
0x0000019E      1  FF                                                   TERMINATOR_FF             
0x0000019F      1  FF                                                   TERMINATOR_FF             
0x000001A0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001A2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001A4      1  FF                                                   TERMINATOR_FF             
0x000001A5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001A7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000001A9      7  800573652D3133                                       LEN8_STRING_CP932         length=5, text="se-13"
0x000001B0      1  FF                                                   TERMINATOR_FF             
0x000001B1      1  FF                                                   TERMINATOR_FF             
0x000001B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001B4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001B6      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x000001B8      1  FF                                                   TERMINATOR_FF             
0x000001B9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001BB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001BD      1  FF                                                   TERMINATOR_FF             
0x000001BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001C2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001C4      1  FF                                                   TERMINATOR_FF             
0x000001C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001C7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001C9      3  F30E85                                               IMM16_F3                  u16_be=3717, u16_le=34062
0x000001CC      1  FF                                                   TERMINATOR_FF             
0x000001CD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001D1    102  806482E082A482509378814190BA82AA8EFC88CD82C9969E...  LEN8_STRING_CP932         length=100, text="もう１度、声が周囲に満ちあふれる。\nぼくたちは、ふり返った。\nそうすべきではないとわかっていたのに。"
0x00000237      1  FF                                                   TERMINATOR_FF             
0x00000238      1  FF                                                   TERMINATOR_FF             
0x00000239      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000023B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000023D      1  FF                                                   TERMINATOR_FF             
0x0000023E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000240      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000242      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x00000244      1  FF                                                   TERMINATOR_FF             
0x00000245      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000247      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000249      1  FF                                                   TERMINATOR_FF             
0x0000024A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000024C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000024E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000250      1  FF                                                   TERMINATOR_FF             
0x00000251      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000253      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000255      3  F30E86                                               IMM16_F3                  u16_be=3718, u16_le=34318
0x00000258      1  FF                                                   TERMINATOR_FF             
0x00000259      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000025B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000025D    118  8074907D8F918AD982C592B282D782BD967B82C982CD8141...  LEN8_STRING_CP932         length=116, text="図書館で調べた本には、帰還の途中でふり返る\nことはすべての時代・地域・文化における共通\nしたタブーだと記されていた。"
0x000002D3      1  FF                                                   TERMINATOR_FF             
0x000002D4      1  FF                                                   TERMINATOR_FF             
0x000002D5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002D7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002D9      1  FF                                                   TERMINATOR_FF             
0x000002DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002DE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000002E0      1  FF                                                   TERMINATOR_FF             
0x000002E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002E3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000002E5      3  F30E87                                               IMM16_F3                  u16_be=3719, u16_le=34574
0x000002E8      1  FF                                                   TERMINATOR_FF             
0x000002E9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002EB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002ED     34  802082C582E0814182DA82AD82BD82BF82CD82D382E895D4...  LEN8_STRING_CP932         length=32, text="でも、ぼくたちはふり返っていた。"
0x0000030F      1  FF                                                   TERMINATOR_FF             
0x00000310      1  FF                                                   TERMINATOR_FF             
0x00000311      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000313      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000315      1  FF                                                   TERMINATOR_FF             
0x00000316      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000318      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000031A      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x0000031C      1  FF                                                   TERMINATOR_FF             
0x0000031D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000031F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000321      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000323      1  FF                                                   TERMINATOR_FF             
0x00000324      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000326      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000328      1  FF                                                   TERMINATOR_FF             
0x00000329      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000032B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000032D      2  F209                                                 IMM8_F2                   u8=9, s8=9
0x0000032F      1  FF                                                   TERMINATOR_FF             
0x00000330      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000332      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000334      2  F267                                                 IMM8_F2                   u8=103, s8=103
0x00000336      1  FF                                                   TERMINATOR_FF             
0x00000337      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000339      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000033B      1  FF                                                   TERMINATOR_FF             
0x0000033C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000033E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000340      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000342      1  FF                                                   TERMINATOR_FF             
0x00000343      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000345      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000347      3  F30102                                               IMM16_F3                  u16_be=258, u16_le=513
0x0000034A      1  FF                                                   TERMINATOR_FF             
0x0000034B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000034D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000034F      1  FF                                                   TERMINATOR_FF             
0x00000350      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000352      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000354      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000356      1  FF                                                   TERMINATOR_FF             
0x00000357      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000359      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000035B      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000035D      1  FF                                                   TERMINATOR_FF             
0x0000035E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000360      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000362      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000364      1  FF                                                   TERMINATOR_FF             
0x00000365      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000367      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000369      1  FF                                                   TERMINATOR_FF             
0x0000036A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000036C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000036E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000370      1  FF                                                   TERMINATOR_FF             
0x00000371      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000373      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000375      3  F30E88                                               IMM16_F3                  u16_be=3720, u16_le=34830
0x00000378      1  FF                                                   TERMINATOR_FF             
0x00000379      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000037B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000037D     82  8050313090948381815B8367838B82CC8B9797A382AA82A0...  LEN8_STRING_CP932         length=80, text="10数メートルの距離があるだろうか。\n動物の姿が月明かりの中に浮かび上がっていた。"
0x000003CF      1  FF                                                   TERMINATOR_FF             
0x000003D0      1  FF                                                   TERMINATOR_FF             
0x000003D1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003D3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003D5      1  FF                                                   TERMINATOR_FF             
0x000003D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003DA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000003DC      1  FF                                                   TERMINATOR_FF             
0x000003DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003DF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000003E1      3  F30E89                                               IMM16_F3                  u16_be=3721, u16_le=35086
0x000003E4      1  FF                                                   TERMINATOR_FF             
0x000003E5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003E9     62  803C83498349834A837E82A996EC8CA282F08E7682ED82B9...  LEN8_STRING_CP932         length=60, text="オオカミか野犬を思わせるシルエットがゴミの\n山を覆っている。"
0x00000427      1  FF                                                   TERMINATOR_FF             
0x00000428      1  FF                                                   TERMINATOR_FF             
0x00000429      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000042B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000042D      1  FF                                                   TERMINATOR_FF             
0x0000042E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000430      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000432      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000434      1  FF                                                   TERMINATOR_FF             
0x00000435      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000437      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000439      3  F30E8A                                               IMM16_F3                  u16_be=3722, u16_le=35342
0x0000043C      1  FF                                                   TERMINATOR_FF             
0x0000043D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000043F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000441     90  805890B89C9B82B382C582CD82C882AD81418BA5965C82C8...  LEN8_STRING_CP932         length=88, text="精悍さではなく、凶暴な狂気を全身から発散\nさせるそれは、明らかに生身の存在ではなかった。"
0x0000049B      1  FF                                                   TERMINATOR_FF             
0x0000049C      1  FF                                                   TERMINATOR_FF             
0x0000049D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000049F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004A1      1  FF                                                   TERMINATOR_FF             
0x000004A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004A6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000004A8      1  FF                                                   TERMINATOR_FF             
0x000004A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004AB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000004AD      3  F30E8B                                               IMM16_F3                  u16_be=3723, u16_le=35598
0x000004B0      1  FF                                                   TERMINATOR_FF             
0x000004B1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004B3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004B5     60  803A82BB82CC8E7082C982CD97A791CC8AB482AA82C882AD...  LEN8_STRING_CP932         length=58, text="その姿には立体感がなく、影が起き上がったかのように見える。"
0x000004F1      1  FF                                                   TERMINATOR_FF             
0x000004F2      1  FF                                                   TERMINATOR_FF             
0x000004F3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004F5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004F7      1  FF                                                   TERMINATOR_FF             
0x000004F8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004FC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000004FE      1  FF                                                   TERMINATOR_FF             
0x000004FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000501      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000503      3  F30E8C                                               IMM16_F3                  u16_be=3724, u16_le=35854
0x00000506      1  FF                                                   TERMINATOR_FF             
0x00000507      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000509      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000050B    126  807C82BB82B582C496DA82CC82A082E982D782AB8FEA8F8A...  LEN8_STRING_CP932         length=124, text="そして目のあるべき場所で冷たい炎が燃えていた。肉体的な痛みを伴う脅威と、自分自身を\nおびやかすなにかがその炎の中にはあった。"
0x00000589      1  FF                                                   TERMINATOR_FF             
0x0000058A      1  FF                                                   TERMINATOR_FF             
0x0000058B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000058D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000058F      1  FF                                                   TERMINATOR_FF             
0x00000590      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000592      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000594      7  800573652D3133                                       LEN8_STRING_CP932         length=5, text="se-13"
0x0000059B      1  FF                                                   TERMINATOR_FF             
0x0000059C      1  FF                                                   TERMINATOR_FF             
0x0000059D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000059F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005A1      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x000005A3      1  FF                                                   TERMINATOR_FF             
0x000005A4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005A6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005A8      1  FF                                                   TERMINATOR_FF             
0x000005A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005AD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000005AF      1  FF                                                   TERMINATOR_FF             
0x000005B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005B2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000005B4      3  F30E8D                                               IMM16_F3                  u16_be=3725, u16_le=36110
0x000005B7      1  FF                                                   TERMINATOR_FF             
0x000005B8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005BC     90  8058896582CC8CA282CD8F9F82BF8CD682C182BD82E682A4...  LEN8_STRING_CP932         length=88, text="影の犬は勝ち誇ったように遠吠えを響かせる。\nぼくたちは、ぼくたちの『死』を目にしていた。"
0x00000616      1  FF                                                   TERMINATOR_FF             
0x00000617      1  FF                                                   TERMINATOR_FF             
0x00000618      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000061A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000061C      1  FF                                                   TERMINATOR_FF             
0x0000061D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000061F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000621      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000623      1  FF                                                   TERMINATOR_FF             
0x00000624      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000626      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000628      3  F340A0                                               IMM16_F3                  u16_be=16544, u16_le=41024
0x0000062B      1  FF                                                   TERMINATOR_FF             
0x0000062C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000062E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000630     88  805682BB82EA82CD814193F791CC934982C982E090B8905F...  LEN8_STRING_CP932         length=86, text="それは、肉体的にも精神的にも……。\n存在に関する究極としての『死』なのだ、と\n思った。"
0x00000688      1  FF                                                   TERMINATOR_FF             
0x00000689      1  FF                                                   TERMINATOR_FF             
0x0000068A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000068C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000068E      1  FF                                                   TERMINATOR_FF             
0x0000068F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000691      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000693      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000695      1  FF                                                   TERMINATOR_FF             
0x00000696      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000698      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000069A      3  F30E8E                                               IMM16_F3                  u16_be=3726, u16_le=36366
0x0000069D      1  FF                                                   TERMINATOR_FF             
0x0000069E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006A0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006A2     12  800A93A682B082E682A48142                             LEN8_STRING_CP932         length=10, text="逃げよう。"
0x000006AE      1  FF                                                   TERMINATOR_FF             
0x000006AF      1  FF                                                   TERMINATOR_FF             
0x000006B0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006B2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006B4      1  FF                                                   TERMINATOR_FF             
0x000006B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006B9      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x000006BB      1  FF                                                   TERMINATOR_FF             
0x000006BC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006BE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006C0      1  FF                                                   TERMINATOR_FF             
0x000006C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006C5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000006C7      1  FF                                                   TERMINATOR_FF             
0x000006C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006CA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006CC      3  F30E8F                                               IMM16_F3                  u16_be=3727, u16_le=36622
0x000006CF      1  FF                                                   TERMINATOR_FF             
0x000006D0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006D2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006D4     44  802A82C782B182D681425C6E82A882D182A682BD88BB82CC...  LEN8_STRING_CP932         length=42, text="どこへ。\nおびえた綾の目がそう問いかける。"
0x00000700      1  FF                                                   TERMINATOR_FF             
0x00000701      1  FF                                                   TERMINATOR_FF             
0x00000702      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000704      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000706      1  FF                                                   TERMINATOR_FF             
0x00000707      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000709      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000070B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000070D      1  FF                                                   TERMINATOR_FF             
0x0000070E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000710      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000712      3  F30E90                                               IMM16_F3                  u16_be=3728, u16_le=36878
0x00000715      1  FF                                                   TERMINATOR_FF             
0x00000716      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000718      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000071A     36  802282DA82AD82CD94DE8F9782CC8EE882F082C682C182C4...  LEN8_STRING_CP932         length=34, text="ぼくは彼女の手をとって走り出した。"
0x0000073E      1  FF                                                   TERMINATOR_FF             
0x0000073F      1  FF                                                   TERMINATOR_FF             
0x00000740      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000742      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000744      1  FF                                                   TERMINATOR_FF             
0x00000745      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000747      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000749      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000074B      1  FF                                                   TERMINATOR_FF             
0x0000074C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000074E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000750      3  F30E91                                               IMM16_F3                  u16_be=3729, u16_le=37134
0x00000753      1  FF                                                   TERMINATOR_FF             
0x00000754      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000756      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000758     82  80508D6C82A682E982CC82CD8CE382C582A282A281425C6E...  LEN8_STRING_CP932         length=80, text="考えるのは後でいい。\n今はあの燃える目からできるだけ遠ざかること\nだけが必要だ。"
0x000007AA      1  FF                                                   TERMINATOR_FF             
0x000007AB      1  FF                                                   TERMINATOR_FF             
0x000007AC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007AE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007B0      1  FF                                                   TERMINATOR_FF             
0x000007B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007B3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007B5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000007B7      1  FF                                                   TERMINATOR_FF             
0x000007B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007BA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000007BC      3  F340A2                                               IMM16_F3                  u16_be=16546, u16_le=41536
0x000007BF      1  FF                                                   TERMINATOR_FF             
0x000007C0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007C2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007C4     38  802482DA82AD82BD82BF82CD8141834B838C834C82CC8E52...  LEN8_STRING_CP932         length=36, text="ぼくたちは、ガレキの山を転げ落ちる。"
0x000007EA      1  FF                                                   TERMINATOR_FF             
0x000007EB      1  FF                                                   TERMINATOR_FF             
0x000007EC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007EE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007F0      1  FF                                                   TERMINATOR_FF             
0x000007F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007F3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000007F5      7  800573652D3132                                       LEN8_STRING_CP932         length=5, text="se-12"
0x000007FC      1  FF                                                   TERMINATOR_FF             
0x000007FD      1  FF                                                   TERMINATOR_FF             
0x000007FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000800      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000802      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00000804      1  FF                                                   TERMINATOR_FF             
0x00000805      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000807      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000809      1  FF                                                   TERMINATOR_FF             
0x0000080A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000080C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000080E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000810      1  FF                                                   TERMINATOR_FF             
0x00000811      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000813      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000815      3  F30E92                                               IMM16_F3                  u16_be=3730, u16_le=37390
0x00000818      1  FF                                                   TERMINATOR_FF             
0x00000819      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000081B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000081D     94  805C92B782A28993966982A682CD82DC82BE91B182A282C4...  LEN8_STRING_CP932         length=92, text="長い遠吠えはまだ続いている。\n狩りはまだ始まっていない。\n影の犬はそれを楽しもうとしている。"
0x0000087B      1  FF                                                   TERMINATOR_FF             
0x0000087C      1  FF                                                   TERMINATOR_FF             
0x0000087D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000087F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000881      1  FF                                                   TERMINATOR_FF             
0x00000882      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000884      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000886      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000888      1  FF                                                   TERMINATOR_FF             
0x00000889      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000088B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000088D      3  F30E93                                               IMM16_F3                  u16_be=3731, u16_le=37646
0x00000890      1  FF                                                   TERMINATOR_FF             
0x00000891      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000893      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000895     52  80328993966982A682F094778CE382C995B782AB82C882AA...  LEN8_STRING_CP932         length=50, text="遠吠えを背後に聞きながら、ぼくはそう感じて\nいた。"
0x000008C9      1  FF                                                   TERMINATOR_FF             
0x000008CA      1  FF                                                   TERMINATOR_FF             
0x000008CB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008CD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008CF      1  FF                                                   TERMINATOR_FF             
0x000008D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008D4      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x000008D6      1  FF                                                   TERMINATOR_FF             
0x000008D7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008D9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008DB      1  FF                                                   TERMINATOR_FF             
0x000008DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008E0      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x000008E2      1  FF                                                   TERMINATOR_FF             
0x000008E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008E5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008E7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008E9      1  FF                                                   TERMINATOR_FF             
0x000008EA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008EC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008EE      1  FF                                                   TERMINATOR_FF             
0x000008EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008F3      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000008F5      1  FF                                                   TERMINATOR_FF             
0x000008F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008F8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008FA      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x000008FC      1  FF                                                   TERMINATOR_FF             
0x000008FD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008FF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000901      1  FF                                                   TERMINATOR_FF             
0x00000902      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000904      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000906      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000908      1  FF                                                   TERMINATOR_FF             
0x00000909      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000090B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000090D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000090F      1  FF                                                   TERMINATOR_FF             
0x00000910      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000912      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000914      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000916      1  FF                                                   TERMINATOR_FF             
0x00000917      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000919      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000091B      1  FF                                                   TERMINATOR_FF             
0x0000091C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000091E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000920      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000922      1  FF                                                   TERMINATOR_FF             
0x00000923      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000925      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000927      3  F30E94                                               IMM16_F3                  u16_be=3732, u16_le=37902
0x0000092A      1  FF                                                   TERMINATOR_FF             
0x0000092B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000092D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000092F      9  800741593033303530                                   LEN8_STRING_CP932         length=7, text="AY03050"
0x00000938      1  FF                                                   TERMINATOR_FF             
0x00000939      1  FF                                                   TERMINATOR_FF             
0x0000093A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000093C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000093E     36  8022817582A88AE882A28163816382CD829F82CD829F8163...  LEN8_STRING_CP932         length=34, text="「お願い……はぁはぁ……ちょっと」"
0x00000962      1  FF                                                   TERMINATOR_FF             
0x00000963      1  FF                                                   TERMINATOR_FF             
0x00000964      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000966      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000968      1  FF                                                   TERMINATOR_FF             
0x00000969      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000096B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000096D      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000096F      1  FF                                                   TERMINATOR_FF             
0x00000970      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000972      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000974      3  F30103                                               IMM16_F3                  u16_be=259, u16_le=769
0x00000977      1  FF                                                   TERMINATOR_FF             
0x00000978      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000097A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000097C      1  FF                                                   TERMINATOR_FF             
0x0000097D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000097F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000981      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000983      1  FF                                                   TERMINATOR_FF             
0x00000984      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000986      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000988      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000098A      1  FF                                                   TERMINATOR_FF             
0x0000098B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000098D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000098F      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000991      1  FF                                                   TERMINATOR_FF             
0x00000992      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000994      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000996      1  FF                                                   TERMINATOR_FF             
0x00000997      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000999      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000099B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000099D      1  FF                                                   TERMINATOR_FF             
0x0000099E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009A0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009A2      3  F30E95                                               IMM16_F3                  u16_be=3733, u16_le=38158
0x000009A5      1  FF                                                   TERMINATOR_FF             
0x000009A6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009A8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009AA     38  802482C282E782BB82A482C888BB82CC90BA82C982DA82AD...  LEN8_STRING_CP932         length=36, text="つらそうな綾の声にぼくは足を止める。"
0x000009D0      1  FF                                                   TERMINATOR_FF             
0x000009D1      1  FF                                                   TERMINATOR_FF             
0x000009D2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009D4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009D6      1  FF                                                   TERMINATOR_FF             
0x000009D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009DB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000009DD      1  FF                                                   TERMINATOR_FF             
0x000009DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009E2      3  F30E96                                               IMM16_F3                  u16_be=3734, u16_le=38414
0x000009E5      1  FF                                                   TERMINATOR_FF             
0x000009E6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009E8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009EA     94  805C82C782EA82BE82AF919682C182BD82CC82A982ED82A9...  LEN8_STRING_CP932         length=92, text="どれだけ走ったのかわからなかった。\n呼吸をするだけで苦しく、心臓は破裂しそうに\n鳴っている。"
0x00000A48      1  FF                                                   TERMINATOR_FF             
0x00000A49      1  FF                                                   TERMINATOR_FF             
0x00000A4A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A4C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A4E      1  FF                                                   TERMINATOR_FF             
0x00000A4F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A51      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A53      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000A55      1  FF                                                   TERMINATOR_FF             
0x00000A56      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A58      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A5A      3  F30E97                                               IMM16_F3                  u16_be=3735, u16_le=38670
0x00000A5D      1  FF                                                   TERMINATOR_FF             
0x00000A5E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A60      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A62    100  806282AF82EA82C7968482DF97A782C4926E82CD8F4982ED...  LEN8_STRING_CP932         length=98, text="けれど埋め立て地は終わらない。\n肩ごしにふり返ると月明かりの中になにもない\n空き地が浮かび上がる。"
0x00000AC6      1  FF                                                   TERMINATOR_FF             
0x00000AC7      1  FF                                                   TERMINATOR_FF             
0x00000AC8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000ACA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000ACC      1  FF                                                   TERMINATOR_FF             
0x00000ACD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000ACF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AD1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000AD3      1  FF                                                   TERMINATOR_FF             
0x00000AD4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AD6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000AD8      3  F30E98                                               IMM16_F3                  u16_be=3736, u16_le=38926
0x00000ADB      1  FF                                                   TERMINATOR_FF             
0x00000ADC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000ADE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AE0    118  807482E082A482A982A982B582CC8BCA8DC082CD82C782B1...  LEN8_STRING_CP932         length=116, text="もうかかしの玉座はどこにも見当たらず、人面の鳥は１羽も飛んでいなかった。\n空っぽの埋め立て地に月の光があふれている。"
0x00000B56      1  FF                                                   TERMINATOR_FF             
0x00000B57      1  FF                                                   TERMINATOR_FF             
0x00000B58      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B5A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B5C      1  FF                                                   TERMINATOR_FF             
0x00000B5D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B5F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B61      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B63      1  FF                                                   TERMINATOR_FF             
0x00000B64      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B66      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B68      3  F30E99                                               IMM16_F3                  u16_be=3737, u16_le=39182
0x00000B6B      1  FF                                                   TERMINATOR_FF             
0x00000B6C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B6E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B70     58  803882AF82EA82C7814182BB82EA82AA8B5582E882CC90C3...  LEN8_STRING_CP932         length=56, text="けれど、それが偽りの静けさだということは\nわかっていた。"
0x00000BAA      1  FF                                                   TERMINATOR_FF             
0x00000BAB      1  FF                                                   TERMINATOR_FF             
0x00000BAC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000BAE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000BB0      1  FF                                                   TERMINATOR_FF             
0x00000BB1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BB3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BB5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000BB7      1  FF                                                   TERMINATOR_FF             
0x00000BB8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BBA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000BBC      3  F30E9A                                               IMM16_F3                  u16_be=3738, u16_le=39438
0x00000BBF      1  FF                                                   TERMINATOR_FF             
0x00000BC0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BC2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BC4     92  805A8AD488E182A282C882AD896582CC8CA282CD82DA82AD...  LEN8_STRING_CP932         length=90, text="間違いなく影の犬はぼくたちに近づいていた。\n迫り来る脅威に、空気は痛いほど張りつめている。"
0x00000C20      1  FF                                                   TERMINATOR_FF             
0x00000C21      1  FF                                                   TERMINATOR_FF             
0x00000C22      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C24      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C26      1  FF                                                   TERMINATOR_FF             
0x00000C27      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C29      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C2B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000C2D      1  FF                                                   TERMINATOR_FF             
0x00000C2E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C30      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C32      3  F30E9B                                               IMM16_F3                  u16_be=3739, u16_le=39694
0x00000C35      1  FF                                                   TERMINATOR_FF             
0x00000C36      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C38      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C3A     48  802E91CC82F090DC82C182C491A782F090AE82A682C482A2...  LEN8_STRING_CP932         length=46, text="体を折って息を整えていた綾がぼくに目を向ける。"
0x00000C6A      1  FF                                                   TERMINATOR_FF             
0x00000C6B      1  FF                                                   TERMINATOR_FF             
0x00000C6C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C6E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C70      1  FF                                                   TERMINATOR_FF             
0x00000C71      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C73      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C75      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000C77      1  FF                                                   TERMINATOR_FF             
0x00000C78      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C7A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C7C      3  F30E9C                                               IMM16_F3                  u16_be=3740, u16_le=39950
0x00000C7F      1  FF                                                   TERMINATOR_FF             
0x00000C80      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C82      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000C84      9  800741593033303630                                   LEN8_STRING_CP932         length=7, text="AY03060"
0x00000C8D      1  FF                                                   TERMINATOR_FF             
0x00000C8E      1  FF                                                   TERMINATOR_FF             
0x00000C8F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C91      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C93     18  80108175816381638BDF82A282CC81488176                 LEN8_STRING_CP932         length=16, text="「……近いの？」"
0x00000CA5      1  FF                                                   TERMINATOR_FF             
0x00000CA6      1  FF                                                   TERMINATOR_FF             
0x00000CA7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CA9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CAB      1  FF                                                   TERMINATOR_FF             
0x00000CAC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CAE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CB0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000CB2      1  FF                                                   TERMINATOR_FF             
0x00000CB3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CB5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000CB7      3  F30E9D                                               IMM16_F3                  u16_be=3741, u16_le=40206
0x00000CBA      1  FF                                                   TERMINATOR_FF             
0x00000CBB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CBD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CBF     38  802482C68D7282A291A782CC8D878AD482C9814182E682A4...  LEN8_STRING_CP932         length=36, text="と荒い息の合間に、ようやく口にする。"
0x00000CE5      1  FF                                                   TERMINATOR_FF             
0x00000CE6      1  FF                                                   TERMINATOR_FF             
0x00000CE7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CE9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CEB      1  FF                                                   TERMINATOR_FF             
0x00000CEC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CEE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CF0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000CF2      1  FF                                                   TERMINATOR_FF             
0x00000CF3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CF5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000CF7      3  F30E9E                                               IMM16_F3                  u16_be=3742, u16_le=40462
0x00000CFA      1  FF                                                   TERMINATOR_FF             
0x00000CFB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CFD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CFF     26  8018817582BD82D482F1814282A982C882E881418BDF82A2...  LEN8_STRING_CP932         length=24, text="「たぶん。かなり、近い」"
0x00000D19      1  FF                                                   TERMINATOR_FF             
0x00000D1A      1  FF                                                   TERMINATOR_FF             
0x00000D1B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D1D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D1F      1  FF                                                   TERMINATOR_FF             
0x00000D20      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D22      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D24      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000D26      1  FF                                                   TERMINATOR_FF             
0x00000D27      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D29      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D2B      3  F30E9F                                               IMM16_F3                  u16_be=3743, u16_le=40718
0x00000D2E      1  FF                                                   TERMINATOR_FF             
0x00000D2F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D31      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000D33      9  800741593033303730                                   LEN8_STRING_CP932         length=7, text="AY03070"
0x00000D3C      1  FF                                                   TERMINATOR_FF             
0x00000D3D      1  FF                                                   TERMINATOR_FF             
0x00000D3E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D40      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D42     14  800C817582A4814182A482F18176                         LEN8_STRING_CP932         length=12, text="「う、うん」"
0x00000D50      1  FF                                                   TERMINATOR_FF             
0x00000D51      1  FF                                                   TERMINATOR_FF             
0x00000D52      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D54      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D56      1  FF                                                   TERMINATOR_FF             
0x00000D57      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D59      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D5B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000D5D      1  FF                                                   TERMINATOR_FF             
0x00000D5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D60      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D62      3  F30EA0                                               IMM16_F3                  u16_be=3744, u16_le=40974
0x00000D65      1  FF                                                   TERMINATOR_FF             
0x00000D66      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D68      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D6A     68  804288BB82CD8BEA82B582BB82A482C989BD937882A982A4...  LEN8_STRING_CP932         length=66, text="綾は苦しそうに何度かうなずく。\n激しい肩の動きに弓袋がずり落ちる。"
0x00000DAE      1  FF                                                   TERMINATOR_FF             
0x00000DAF      1  FF                                                   TERMINATOR_FF             
0x00000DB0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000DB2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000DB4      1  FF                                                   TERMINATOR_FF             
0x00000DB5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DB7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DB9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000DBB      1  FF                                                   TERMINATOR_FF             
0x00000DBC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DBE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000DC0      3  F30EA1                                               IMM16_F3                  u16_be=3745, u16_le=41230
0x00000DC3      1  FF                                                   TERMINATOR_FF             
0x00000DC4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DC6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DC8     84  805282BB82CC976C8E7182F08CA982C4814194DE8F9782CD...  LEN8_STRING_CP932         length=82, text="その様子を見て、彼女はもう走れないと思った。ならば、ぼくにできることはひとつだけ。"
0x00000E1C      1  FF                                                   TERMINATOR_FF             
0x00000E1D      1  FF                                                   TERMINATOR_FF             
0x00000E1E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E20      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E22      1  FF                                                   TERMINATOR_FF             
0x00000E23      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E25      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E27      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000E29      1  FF                                                   TERMINATOR_FF             
0x00000E2A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E2C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E2E      3  F30EA2                                               IMM16_F3                  u16_be=3746, u16_le=41486
0x00000E31      1  FF                                                   TERMINATOR_FF             
0x00000E32      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E34      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E36     56  803682DA82AD82CD94DE8F9782CC904D82B682E993B582C9...  LEN8_STRING_CP932         length=54, text="ぼくは彼女の信じる瞳にひかれ、彼女の失望を\n目にした。"
0x00000E6E      1  FF                                                   TERMINATOR_FF             
0x00000E6F      1  FF                                                   TERMINATOR_FF             
0x00000E70      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E72      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E74      1  FF                                                   TERMINATOR_FF             
0x00000E75      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E79      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000E7B      1  FF                                                   TERMINATOR_FF             
0x00000E7C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E7E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E80      3  F30EA3                                               IMM16_F3                  u16_be=3747, u16_le=41742
0x00000E83      1  FF                                                   TERMINATOR_FF             
0x00000E84      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E86      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E88     86  805494DE8F9782AA8EE389B982F082CD82A982B8814182C2...  LEN8_STRING_CP932         length=84, text="彼女が弱音をはかず、つらさを呑みこむのを見てきた。\nでも、そんな彼女にも限界がある。"
0x00000EDE      1  FF                                                   TERMINATOR_FF             
0x00000EDF      1  FF                                                   TERMINATOR_FF             
0x00000EE0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000EE2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000EE4      1  FF                                                   TERMINATOR_FF             
0x00000EE5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EE7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EE9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000EEB      1  FF                                                   TERMINATOR_FF             
0x00000EEC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EEE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000EF0      3  F30EA4                                               IMM16_F3                  u16_be=3748, u16_le=41998
0x00000EF3      1  FF                                                   TERMINATOR_FF             
0x00000EF4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000EF6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000EF8     54  80348DA182CC94DE8F9782CD8163816390DC82EA82BB82A4...  LEN8_STRING_CP932         length=52, text="今の彼女は……折れそうになっている。\n気持ちも体も。"
0x00000F2E      1  FF                                                   TERMINATOR_FF             
0x00000F2F      1  FF                                                   TERMINATOR_FF             
0x00000F30      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F32      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F34      1  FF                                                   TERMINATOR_FF             
0x00000F35      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F37      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F39      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000F3B      1  FF                                                   TERMINATOR_FF             
0x00000F3C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F3E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F40      3  F30EA5                                               IMM16_F3                  u16_be=3749, u16_le=42254
0x00000F43      1  FF                                                   TERMINATOR_FF             
0x00000F44      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F46      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F48     20  801282DA82AD82AA94DE8F9782F0816381638142             LEN8_STRING_CP932         length=18, text="ぼくが彼女を……。"
0x00000F5C      1  FF                                                   TERMINATOR_FF             
0x00000F5D      1  FF                                                   TERMINATOR_FF             
0x00000F5E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F60      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F62      1  FF                                                   TERMINATOR_FF             
0x00000F63      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F65      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F67      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000F69      1  FF                                                   TERMINATOR_FF             
0x00000F6A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F6C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F6E      3  F340A3                                               IMM16_F3                  u16_be=16547, u16_le=41792
0x00000F71      1  FF                                                   TERMINATOR_FF             
0x00000F72      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F74      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F76     38  802494DE8F9782CC8AF3965D82F08EE782E982BD82DF82C9...  LEN8_STRING_CP932         length=36, text="彼女の希望を守るために、ぼくが……。"
0x00000F9C      1  FF                                                   TERMINATOR_FF             
0x00000F9D      1  FF                                                   TERMINATOR_FF             
0x00000F9E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FA0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FA2      1  FF                                                   TERMINATOR_FF             
0x00000FA3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FA5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000FA7      7  800573652D3134                                       LEN8_STRING_CP932         length=5, text="se-14"
0x00000FAE      1  FF                                                   TERMINATOR_FF             
0x00000FAF      1  FF                                                   TERMINATOR_FF             
0x00000FB0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FB4      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00000FB6      1  FF                                                   TERMINATOR_FF             
0x00000FB7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FB9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FBB      1  FF                                                   TERMINATOR_FF             
0x00000FBC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FC0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000FC2      1  FF                                                   TERMINATOR_FF             
0x00000FC3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FC5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000FC7      3  F30EA6                                               IMM16_F3                  u16_be=3750, u16_le=42510
0x00000FCA      1  FF                                                   TERMINATOR_FF             
0x00000FCB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FCD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FCF    110  806C82BB82CC82C682AB814196DA82CC914F82C582D382BD...  LEN8_STRING_CP932         length=108, text="そのとき、目の前でふたつの炎が燃えあがる。\n驚きの声をあげる間もなく、影の犬はまっしぐらに綾へと飛びかかる。"
0x0000103D      1  FF                                                   TERMINATOR_FF             
0x0000103E      1  FF                                                   TERMINATOR_FF             
0x0000103F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001041      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001043      1  FF                                                   TERMINATOR_FF             
0x00001044      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001046      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001048      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x0000104A      1  FF                                                   TERMINATOR_FF             
0x0000104B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000104D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000104F      1  FF                                                   TERMINATOR_FF             
0x00001050      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001052      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001054      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001056      1  FF                                                   TERMINATOR_FF             
0x00001057      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001059      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000105B      3  F30104                                               IMM16_F3                  u16_be=260, u16_le=1025
0x0000105E      1  FF                                                   TERMINATOR_FF             
0x0000105F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001061      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001063      1  FF                                                   TERMINATOR_FF             
0x00001064      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001066      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001068      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000106A      1  FF                                                   TERMINATOR_FF             
0x0000106B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000106D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000106F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001071      1  FF                                                   TERMINATOR_FF             
0x00001072      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001074      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001076      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001078      1  FF                                                   TERMINATOR_FF             
0x00001079      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000107B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000107D      1  FF                                                   TERMINATOR_FF             
0x0000107E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001080      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001082      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001084      1  FF                                                   TERMINATOR_FF             
0x00001085      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001087      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001089      3  F30EA7                                               IMM16_F3                  u16_be=3751, u16_le=42766
0x0000108C      1  FF                                                   TERMINATOR_FF             
0x0000108D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000108F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001091     90  805891CC82CD8D6C82A682E982E682E882E082CD82E282AD...  LEN8_STRING_CP932         length=88, text="体は考えるよりもはやく動いていた。\n綾をつき飛ばしたぼくに向かって、影の犬が飛びかかる。"
0x000010EB      1  FF                                                   TERMINATOR_FF             
0x000010EC      1  FF                                                   TERMINATOR_FF             
0x000010ED      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010EF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010F1      1  FF                                                   TERMINATOR_FF             
0x000010F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010F6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000010F8      1  FF                                                   TERMINATOR_FF             
0x000010F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010FB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000010FD      3  F30EA8                                               IMM16_F3                  u16_be=3752, u16_le=43022
0x00001100      1  FF                                                   TERMINATOR_FF             
0x00001101      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001103      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001105     74  8048949282A289E582AA96DA82CC914F82C9949782C182C4...  LEN8_STRING_CP932         length=72, text="白い牙が目の前に迫っていた。\n立体感のない獣のシルエットと現実の鋭い牙。"
0x0000114F      1  FF                                                   TERMINATOR_FF             
0x00001150      1  FF                                                   TERMINATOR_FF             
0x00001151      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001153      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001155      1  FF                                                   TERMINATOR_FF             
0x00001156      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001158      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000115A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000115C      1  FF                                                   TERMINATOR_FF             
0x0000115D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000115F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001161      3  F340A4                                               IMM16_F3                  u16_be=16548, u16_le=42048
0x00001164      1  FF                                                   TERMINATOR_FF             
0x00001165      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001167      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001169     50  80308EE882B482ED82E882CD82C882A982C182BD81425C6E...  LEN8_STRING_CP932         length=48, text="手ざわりはなかった。\nあるのは、圧力だけだった。"
0x0000119B      1  FF                                                   TERMINATOR_FF             
0x0000119C      1  FF                                                   TERMINATOR_FF             
0x0000119D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000119F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000011A1      1  FF                                                   TERMINATOR_FF             
0x000011A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011A6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000011A8      1  FF                                                   TERMINATOR_FF             
0x000011A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011AB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000011AD      3  F340A5                                               IMM16_F3                  u16_be=16549, u16_le=42304
0x000011B0      1  FF                                                   TERMINATOR_FF             
0x000011B1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011B3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011B5     80  804E937B82E88141919E82B582DD81418C9988AB81425C6E...  LEN8_STRING_CP932         length=78, text="怒り、憎しみ、嫌悪。\nそういった負の感情で、ふれた場所が焼ける\nように痛んだ。"
0x00001205      1  FF                                                   TERMINATOR_FF             
0x00001206      1  FF                                                   TERMINATOR_FF             
0x00001207      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001209      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000120B      1  FF                                                   TERMINATOR_FF             
0x0000120C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000120E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001210      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001212      1  FF                                                   TERMINATOR_FF             
0x00001213      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001215      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001217      3  F30EA9                                               IMM16_F3                  u16_be=3753, u16_le=43278
0x0000121A      1  FF                                                   TERMINATOR_FF             
0x0000121B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000121D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000121F     94  805C82BB82EA82CD95738E768B6382C88CF58C6982BE82C1...  LEN8_STRING_CP932         length=92, text="それは不思議な光景だった。\n実際に自分の身にふりかかっているとは思えない幻想的な光景だった。"
0x0000127D      1  FF                                                   TERMINATOR_FF             
0x0000127E      1  FF                                                   TERMINATOR_FF             
0x0000127F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001281      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001283      1  FF                                                   TERMINATOR_FF             
0x00001284      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001286      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001288      8  800673652D653234                                     LEN8_STRING_CP932         length=6, text="se-e24"
0x00001290      1  FF                                                   TERMINATOR_FF             
0x00001291      1  FF                                                   TERMINATOR_FF             
0x00001292      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001294      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001296      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00001298      1  FF                                                   TERMINATOR_FF             
0x00001299      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000129B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000129D      1  FF                                                   TERMINATOR_FF             
0x0000129E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012A2      2  F221                                                 IMM8_F2                   u8=33, s8=33
0x000012A4      1  FF                                                   TERMINATOR_FF             
0x000012A5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012A7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012A9      2  F214                                                 IMM8_F2                   u8=20, s8=20
0x000012AB      1  FF                                                   TERMINATOR_FF             
0x000012AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012AE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000012B0      3  F301FF                                               IMM16_F3                  u16_be=511, u16_le=65281
0x000012B3      1  FF                                                   TERMINATOR_FF             
0x000012B4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012B6      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x000012B8      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000012BA      1  FF                                                   TERMINATOR_FF             
0x000012BB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012BD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012BF      1  FF                                                   TERMINATOR_FF             
0x000012C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012C2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012C4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000012C6      1  FF                                                   TERMINATOR_FF             
0x000012C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012C9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000012CB      3  F30EAA                                               IMM16_F3                  u16_be=3754, u16_le=43534
0x000012CE      1  FF                                                   TERMINATOR_FF             
0x000012CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012D1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012D3     44  802A896582CC8CA282CD82DA82AD82F0899F82B5937C82B5...  LEN8_STRING_CP932         length=42, text="影の犬はぼくを押し倒し、綾が悲鳴をあげる。"
0x000012FF      1  FF                                                   TERMINATOR_FF             
0x00001300      1  FF                                                   TERMINATOR_FF             
0x00001301      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001303      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001305      1  FF                                                   TERMINATOR_FF             
0x00001306      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001308      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000130A      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x0000130C      1  FF                                                   TERMINATOR_FF             
0x0000130D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000130F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001311      1  FF                                                   TERMINATOR_FF             
0x00001312      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001314      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001316      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001318      1  FF                                                   TERMINATOR_FF             
0x00001319      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000131B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000131D      3  F30EAB                                               IMM16_F3                  u16_be=3755, u16_le=43790
0x00001320      1  FF                                                   TERMINATOR_FF             
0x00001321      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001323      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001325      9  800741593030303130                                   LEN8_STRING_CP932         length=7, text="AY00010"
0x0000132E      1  FF                                                   TERMINATOR_FF             
0x0000132F      1  FF                                                   TERMINATOR_FF             
0x00001330      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001332      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001334     24  8016817582AB82E182A082A082A082A082A082A03F218176     LEN8_STRING_CP932         length=22, text="「きゃああああああ?!」"
0x0000134C      1  FF                                                   TERMINATOR_FF             
0x0000134D      1  FF                                                   TERMINATOR_FF             
0x0000134E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001350      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001352      1  FF                                                   TERMINATOR_FF             
0x00001353      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001355      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001357      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001359      1  FF                                                   TERMINATOR_FF             
0x0000135A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000135C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000135E      3  F30EAC                                               IMM16_F3                  u16_be=3756, u16_le=44046
0x00001361      1  FF                                                   TERMINATOR_FF             
0x00001362      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001364      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001366    106  806882DA82AD82CD82C682C182B382C98EF18BD882F0915F...  LEN8_STRING_CP932         length=104, text="ぼくはとっさに首筋を狙う白い牙をつかんでいた。冷たかった。\n背筋を震わす底知れない冷気が手の中にあった。"
0x000013D0      1  FF                                                   TERMINATOR_FF             
0x000013D1      1  FF                                                   TERMINATOR_FF             
0x000013D2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000013D4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000013D6      1  FF                                                   TERMINATOR_FF             
0x000013D7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013D9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000013DB      7  800573652D3238                                       LEN8_STRING_CP932         length=5, text="se-28"
0x000013E2      1  FF                                                   TERMINATOR_FF             
0x000013E3      1  FF                                                   TERMINATOR_FF             
0x000013E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013E8      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x000013EA      1  FF                                                   TERMINATOR_FF             
0x000013EB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000013ED      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000013EF      1  FF                                                   TERMINATOR_FF             
0x000013F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013F4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000013F6      1  FF                                                   TERMINATOR_FF             
0x000013F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013F9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000013FB      3  F30EAD                                               IMM16_F3                  u16_be=3757, u16_le=44302
0x000013FE      1  FF                                                   TERMINATOR_FF             
0x000013FF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001401      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001403     64  803E82BB82CC97E282BD82B382C988EA8F7582BD82B682EB...  LEN8_STRING_CP932         length=62, text="その冷たさに一瞬たじろいだせいで、牙が\nのどもとに向けられる。"
0x00001443      1  FF                                                   TERMINATOR_FF             
0x00001444      1  FF                                                   TERMINATOR_FF             
0x00001445      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001447      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001449      1  FF                                                   TERMINATOR_FF             
0x0000144A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000144C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000144E      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x00001450      1  FF                                                   TERMINATOR_FF             
0x00001451      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001453      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001455      1  FF                                                   TERMINATOR_FF             
0x00001456      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001458      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000145A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000145C      1  FF                                                   TERMINATOR_FF             
0x0000145D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000145F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001461      3  F30EAE                                               IMM16_F3                  u16_be=3758, u16_le=44558
0x00001464      1  FF                                                   TERMINATOR_FF             
0x00001465      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001467      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001469     17  800F817588BB2121208B7C82F021218176                   LEN8_STRING_CP932         length=15, text="「綾!! 弓を!!」"
0x0000147A      1  FF                                                   TERMINATOR_FF             
0x0000147B      1  FF                                                   TERMINATOR_FF             
0x0000147C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000147E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001480      1  FF                                                   TERMINATOR_FF             
0x00001481      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001483      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001485      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001487      1  FF                                                   TERMINATOR_FF             
0x00001488      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000148A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000148C      3  F30EAF                                               IMM16_F3                  u16_be=3759, u16_le=44814
0x0000148F      1  FF                                                   TERMINATOR_FF             
0x00001490      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001492      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001494     34  802082DA82AD82CD814188BB82C9937182AF82E982B582A9...  LEN8_STRING_CP932         length=32, text="ぼくは、綾に賭けるしかなかった。"
0x000014B6      1  FF                                                   TERMINATOR_FF             
0x000014B7      1  FF                                                   TERMINATOR_FF             
0x000014B8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000014BA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000014BC      1  FF                                                   TERMINATOR_FF             
0x000014BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014C1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000014C3      1  FF                                                   TERMINATOR_FF             
0x000014C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014C6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000014C8      3  F30EB0                                               IMM16_F3                  u16_be=3760, u16_le=45070
0x000014CB      1  FF                                                   TERMINATOR_FF             
0x000014CC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014CE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014D0     96  805E93648ED482CD94DE8F9782CC82BD82DF82C993AE82AD...  LEN8_STRING_CP932         length=94, text="電車は彼女のために動く。\nコンビニの食品は補充されている。\n彼女の家は、永遠の昼間の中にある。"
0x00001530      1  FF                                                   TERMINATOR_FF             
0x00001531      1  FF                                                   TERMINATOR_FF             
0x00001532      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001534      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001536      1  FF                                                   TERMINATOR_FF             
0x00001537      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001539      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000153B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000153D      1  FF                                                   TERMINATOR_FF             
0x0000153E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001540      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001542      3  F30EB1                                               IMM16_F3                  u16_be=3761, u16_le=45326
0x00001545      1  FF                                                   TERMINATOR_FF             
0x00001546      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001548      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000154A     60  803A82C882E782CE8163816394DE8F9782CC88D38E7582AA...  LEN8_STRING_CP932         length=58, text="ならば……彼女の意志がなんらかの力を持つの\nかもしれない。"
0x00001586      1  FF                                                   TERMINATOR_FF             
0x00001587      1  FF                                                   TERMINATOR_FF             
0x00001588      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000158A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000158C      1  FF                                                   TERMINATOR_FF             
0x0000158D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000158F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001591      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001593      1  FF                                                   TERMINATOR_FF             
0x00001594      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001596      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001598      3  F340A6                                               IMM16_F3                  u16_be=16550, u16_le=42560
0x0000159B      1  FF                                                   TERMINATOR_FF             
0x0000159C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000159E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015A0     28  801A82DA82AD82CD82BB82EA82C9965D82DD82F082A982AF...  LEN8_STRING_CP932         length=26, text="ぼくはそれに望みをかけた。"
0x000015BC      1  FF                                                   TERMINATOR_FF             
0x000015BD      1  FF                                                   TERMINATOR_FF             
0x000015BE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000015C0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000015C2      1  FF                                                   TERMINATOR_FF             
0x000015C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015C7      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000015C9      1  FF                                                   TERMINATOR_FF             
0x000015CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015CC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015CE      3  F30105                                               IMM16_F3                  u16_be=261, u16_le=1281
0x000015D1      1  FF                                                   TERMINATOR_FF             
0x000015D2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000015D4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000015D6      1  FF                                                   TERMINATOR_FF             
0x000015D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015DB      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000015DD      1  FF                                                   TERMINATOR_FF             
0x000015DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015E0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015E2      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000015E4      1  FF                                                   TERMINATOR_FF             
0x000015E5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015E7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000015E9      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000015EB      1  FF                                                   TERMINATOR_FF             
0x000015EC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000015EE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000015F0      1  FF                                                   TERMINATOR_FF             
0x000015F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015F5      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000015F7      1  FF                                                   TERMINATOR_FF             
0x000015F8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015FA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000015FC      3  F30EB2                                               IMM16_F3                  u16_be=3762, u16_le=45582
0x000015FF      1  FF                                                   TERMINATOR_FF             
0x00001600      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001602      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001604      9  800741593033303830                                   LEN8_STRING_CP932         length=7, text="AY03080"
0x0000160D      1  FF                                                   TERMINATOR_FF             
0x0000160E      1  FF                                                   TERMINATOR_FF             
0x0000160F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001611      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001613     26  8018817582C5814182C582E08163816382A4814182A482F1...  LEN8_STRING_CP932         length=24, text="「で、でも……う、うん」"
0x0000162D      1  FF                                                   TERMINATOR_FF             
0x0000162E      1  FF                                                   TERMINATOR_FF             
0x0000162F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001631      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001633      1  FF                                                   TERMINATOR_FF             
0x00001634      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001636      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001638      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000163A      1  FF                                                   TERMINATOR_FF             
0x0000163B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000163D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000163F      3  F30EB3                                               IMM16_F3                  u16_be=3763, u16_le=45838
0x00001642      1  FF                                                   TERMINATOR_FF             
0x00001643      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001645      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001647     40  802688BB82CD82A882DA82C282A982C882A28EE882C282AB...  LEN8_STRING_CP932         length=38, text="綾はおぼつかない手つきで、弓袋を開く。"
0x0000166F      1  FF                                                   TERMINATOR_FF             
0x00001670      1  FF                                                   TERMINATOR_FF             
0x00001671      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001673      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001675      1  FF                                                   TERMINATOR_FF             
0x00001676      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001678      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000167A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000167C      1  FF                                                   TERMINATOR_FF             
0x0000167D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000167F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001681      3  F30EB4                                               IMM16_F3                  u16_be=3764, u16_le=46094
0x00001684      1  FF                                                   TERMINATOR_FF             
0x00001685      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001687      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001689    102  80648BD992A382C882CC82A98BB0957C82C882CC82A98141...  LEN8_STRING_CP932         length=100, text="緊張なのか恐怖なのか、手がふるえている。\n矢筒からカーボン製の矢を取り出そうとして、\n地面に落とす。"
0x000016EF      1  FF                                                   TERMINATOR_FF             
0x000016F0      1  FF                                                   TERMINATOR_FF             
0x000016F1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016F3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016F5      1  FF                                                   TERMINATOR_FF             
0x000016F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016F8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016FA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000016FC      1  FF                                                   TERMINATOR_FF             
0x000016FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016FF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001701      3  F30EB5                                               IMM16_F3                  u16_be=3765, u16_le=46350
0x00001704      1  FF                                                   TERMINATOR_FF             
0x00001705      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001707      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001709     46  802C82BB82B582C48141905E82C1949282A289E582AA82DA...  LEN8_STRING_CP932         length=44, text="そして、真っ白い牙がぼくの体温を奪っていく。"
0x00001737      1  FF                                                   TERMINATOR_FF             
0x00001738      1  FF                                                   TERMINATOR_FF             
0x00001739      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000173B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000173D      1  FF                                                   TERMINATOR_FF             
0x0000173E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001740      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001742      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001744      1  FF                                                   TERMINATOR_FF             
0x00001745      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001747      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001749      3  F30EB6                                               IMM16_F3                  u16_be=3766, u16_le=46606
0x0000174C      1  FF                                                   TERMINATOR_FF             
0x0000174D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000174F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001751     14  800C817582CD82E282AD21218176                         LEN8_STRING_CP932         length=12, text="「はやく!!」"
0x0000175F      1  FF                                                   TERMINATOR_FF             
0x00001760      1  FF                                                   TERMINATOR_FF             
0x00001761      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001763      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001765      1  FF                                                   TERMINATOR_FF             
0x00001766      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001768      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000176A      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000176C      1  FF                                                   TERMINATOR_FF             
0x0000176D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000176F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001771      3  F30EB7                                               IMM16_F3                  u16_be=3767, u16_le=46862
0x00001774      1  FF                                                   TERMINATOR_FF             
0x00001775      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001777      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001779      9  800741593033303930                                   LEN8_STRING_CP932         length=7, text="AY03090"
0x00001782      1  FF                                                   TERMINATOR_FF             
0x00001783      1  FF                                                   TERMINATOR_FF             
0x00001784      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001786      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001788     14  800C817582A082C68FAD82B58176                         LEN8_STRING_CP932         length=12, text="「あと少し」"
0x00001796      1  FF                                                   TERMINATOR_FF             
0x00001797      1  FF                                                   TERMINATOR_FF             
0x00001798      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000179A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000179C      1  FF                                                   TERMINATOR_FF             
0x0000179D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000179F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017A1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000017A3      1  FF                                                   TERMINATOR_FF             
0x000017A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017A6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000017A8      3  F30EB8                                               IMM16_F3                  u16_be=3768, u16_le=47118
0x000017AB      1  FF                                                   TERMINATOR_FF             
0x000017AC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017AE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017B0     32  801E82BB82CC82C682AB814195D382E882AA896582C995EF...  LEN8_STRING_CP932         length=30, text="そのとき、辺りが影に包まれた。"
0x000017D0      1  FF                                                   TERMINATOR_FF             
0x000017D1      1  FF                                                   TERMINATOR_FF             
0x000017D2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000017D4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000017D6      1  FF                                                   TERMINATOR_FF             
0x000017D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017DB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000017DD      1  FF                                                   TERMINATOR_FF             
0x000017DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017E0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000017E2      3  F30EB9                                               IMM16_F3                  u16_be=3769, u16_le=47374
0x000017E5      1  FF                                                   TERMINATOR_FF             
0x000017E6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017E8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017EA     82  80508C8E82C9895F82AA82A982A982C182BD81425C6E8BF3...  LEN8_STRING_CP932         length=80, text="月に雲がかかった。\n空の色が変わる。\nなにもない埋め立て地に高層ビルが立ち並ぶ。"
0x0000183C      1  FF                                                   TERMINATOR_FF             
0x0000183D      1  FF                                                   TERMINATOR_FF             
0x0000183E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001840      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001842      1  FF                                                   TERMINATOR_FF             
0x00001843      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001845      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001847      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001849      1  FF                                                   TERMINATOR_FF             
0x0000184A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000184C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000184E      3  F30EBA                                               IMM16_F3                  u16_be=3770, u16_le=47630
0x00001851      1  FF                                                   TERMINATOR_FF             
0x00001852      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001854      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001856      9  800741593033313030                                   LEN8_STRING_CP932         length=7, text="AY03100"
0x0000185F      1  FF                                                   TERMINATOR_FF             
0x00001860      1  FF                                                   TERMINATOR_FF             
0x00001861      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001863      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001865     14  800C817582A0829F82C13F218176                         LEN8_STRING_CP932         length=12, text="「あぁっ?!」"
0x00001873      1  FF                                                   TERMINATOR_FF             
0x00001874      1  FF                                                   TERMINATOR_FF             
0x00001875      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001877      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001879      1  FF                                                   TERMINATOR_FF             
0x0000187A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000187C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000187E      8  800653452D533637                                     LEN8_STRING_CP932         length=6, text="SE-S67"
0x00001886      1  FF                                                   TERMINATOR_FF             
0x00001887      1  FF                                                   TERMINATOR_FF             
0x00001888      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000188A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000188C      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x0000188E      1  FF                                                   TERMINATOR_FF             
0x0000188F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001891      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001893      1  FF                                                   TERMINATOR_FF             
0x00001894      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001896      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001898      2  F221                                                 IMM8_F2                   u8=33, s8=33
0x0000189A      1  FF                                                   TERMINATOR_FF             
0x0000189B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000189D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000189F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000018A1      1  FF                                                   TERMINATOR_FF             
0x000018A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018A4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000018A6      3  F30199                                               IMM16_F3                  u16_be=409, u16_le=39169
0x000018A9      1  FF                                                   TERMINATOR_FF             
0x000018AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018AC      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x000018AE      2  F20F                                                 IMM8_F2                   u8=15, s8=15
0x000018B0      1  FF                                                   TERMINATOR_FF             
0x000018B1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000018B3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000018B5      1  FF                                                   TERMINATOR_FF             
0x000018B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018BA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000018BC      1  FF                                                   TERMINATOR_FF             
0x000018BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018BF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000018C1      3  F30EBB                                               IMM16_F3                  u16_be=3771, u16_le=47886
0x000018C4      1  FF                                                   TERMINATOR_FF             
0x000018C5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018C9    108  806A896582CC928682A982E782D382E882A882EB82B382EA...  LEN8_STRING_CP932         length=106, text="影の中からふりおろされた棒が綾の手から弓を\nたたき落とす。\nはじけ飛んだカーボン製の矢が足もとに転がった。"
0x00001935      1  FF                                                   TERMINATOR_FF             
0x00001936      1  FF                                                   TERMINATOR_FF             
0x00001937      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001939      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000193B      1  FF                                                   TERMINATOR_FF             
0x0000193C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000193E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001940      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001942      1  FF                                                   TERMINATOR_FF             
0x00001943      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001945      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001947      3  F30EBC                                               IMM16_F3                  u16_be=3772, u16_le=48142
0x0000194A      1  FF                                                   TERMINATOR_FF             
0x0000194B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000194D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000194F      9  800741593033313130                                   LEN8_STRING_CP932         length=7, text="AY03110"
0x00001958      1  FF                                                   TERMINATOR_FF             
0x00001959      1  FF                                                   TERMINATOR_FF             
0x0000195A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000195C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000195E     12  800A817582A682C13F218176                             LEN8_STRING_CP932         length=10, text="「えっ?!」"
0x0000196A      1  FF                                                   TERMINATOR_FF             
0x0000196B      1  FF                                                   TERMINATOR_FF             
0x0000196C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000196E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001970      1  FF                                                   TERMINATOR_FF             
0x00001971      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001973      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001975      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001977      1  FF                                                   TERMINATOR_FF             
0x00001978      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000197A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000197C      3  F30106                                               IMM16_F3                  u16_be=262, u16_le=1537
0x0000197F      1  FF                                                   TERMINATOR_FF             
0x00001980      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001982      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001984      1  FF                                                   TERMINATOR_FF             
0x00001985      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001987      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001989      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x0000198B      1  FF                                                   TERMINATOR_FF             
0x0000198C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000198E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001990      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001992      1  FF                                                   TERMINATOR_FF             
0x00001993      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001995      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001997      1  FF                                                   TERMINATOR_FF             
0x00001998      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000199A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000199C      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000199E      1  FF                                                   TERMINATOR_FF             
0x0000199F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019A1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019A3      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000019A5      1  FF                                                   TERMINATOR_FF             
0x000019A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019A8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000019AA      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000019AC      1  FF                                                   TERMINATOR_FF             
0x000019AD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000019AF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000019B1      1  FF                                                   TERMINATOR_FF             
0x000019B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019B4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000019B6      8  800653452D533638                                     LEN8_STRING_CP932         length=6, text="SE-S68"
0x000019BE      1  FF                                                   TERMINATOR_FF             
0x000019BF      1  FF                                                   TERMINATOR_FF             
0x000019C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019C2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019C4      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x000019C6      1  FF                                                   TERMINATOR_FF             
0x000019C7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000019C9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000019CB      1  FF                                                   TERMINATOR_FF             
0x000019CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019D0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000019D2      1  FF                                                   TERMINATOR_FF             
0x000019D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019D5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000019D7      3  F30EBD                                               IMM16_F3                  u16_be=3773, u16_le=48398
0x000019DA      1  FF                                                   TERMINATOR_FF             
0x000019DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019DD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019DF     78  804C88BB82CC91CC82C9814182A082CC926A82AA82B582AA...  LEN8_STRING_CP932         length=76, text="綾の体に、あの男がしがみついていた。\n妖精狩りのかたわれ。\n妖精を食べた男。"
0x00001A2D      1  FF                                                   TERMINATOR_FF             
0x00001A2E      1  FF                                                   TERMINATOR_FF             
0x00001A2F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A31      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A33      1  FF                                                   TERMINATOR_FF             
0x00001A34      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A38      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001A3A      1  FF                                                   TERMINATOR_FF             
0x00001A3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A3D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001A3F      3  F340A7                                               IMM16_F3                  u16_be=16551, u16_le=42816
0x00001A42      1  FF                                                   TERMINATOR_FF             
0x00001A43      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A45      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A47     46  802C926A82CD88BB82C982B582AA82DD82C282AB814182D3...  LEN8_STRING_CP932         length=44, text="男は綾にしがみつき、ふたりは地面に転がった。"
0x00001A75      1  FF                                                   TERMINATOR_FF             
0x00001A76      1  FF                                                   TERMINATOR_FF             
0x00001A77      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A79      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A7B      1  FF                                                   TERMINATOR_FF             
0x00001A7C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A80      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x00001A82      1  FF                                                   TERMINATOR_FF             
0x00001A83      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A85      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A87      1  FF                                                   TERMINATOR_FF             
0x00001A88      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A8A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A8C      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001A8E      1  FF                                                   TERMINATOR_FF             
0x00001A8F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A91      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001A93      3  F30EBE                                               IMM16_F3                  u16_be=3774, u16_le=48654
0x00001A96      1  FF                                                   TERMINATOR_FF             
0x00001A97      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A99      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001A9B      9  800742553030333430                                   LEN8_STRING_CP932         length=7, text="BU00340"
0x00001AA4      1  FF                                                   TERMINATOR_FF             
0x00001AA5      1  FF                                                   TERMINATOR_FF             
0x00001AA6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001AA8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001AAA     41  8027817582E282C182BD82C182B782E6814191E58FAB2121...  LEN8_STRING_CP932         length=39, text="「やったっすよ、大将!! 生きてるっす!!」"
0x00001AD3      1  FF                                                   TERMINATOR_FF             
0x00001AD4      1  FF                                                   TERMINATOR_FF             
0x00001AD5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001AD7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001AD9      1  FF                                                   TERMINATOR_FF             
0x00001ADA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001ADC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001ADE      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001AE0      1  FF                                                   TERMINATOR_FF             
0x00001AE1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001AE3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001AE5      3  F30EBF                                               IMM16_F3                  u16_be=3775, u16_le=48910
0x00001AE8      1  FF                                                   TERMINATOR_FF             
0x00001AE9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001AEB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001AED      9  800754533030323930                                   LEN8_STRING_CP932         length=7, text="TS00290"
0x00001AF6      1  FF                                                   TERMINATOR_FF             
0x00001AF7      1  FF                                                   TERMINATOR_FF             
0x00001AF8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001AFA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001AFC     44  802A817582ED82A982C182C482E982E681428B7282BF82E1...  LEN8_STRING_CP932         length=42, text="「わかってるよ。脚ちゃんとおさえてろって」"
0x00001B28      1  FF                                                   TERMINATOR_FF             
0x00001B29      1  FF                                                   TERMINATOR_FF             
0x00001B2A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B2C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B2E      1  FF                                                   TERMINATOR_FF             
0x00001B2F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B31      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B33      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001B35      1  FF                                                   TERMINATOR_FF             
0x00001B36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B38      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001B3A      3  F30EC0                                               IMM16_F3                  u16_be=3776, u16_le=49166
0x00001B3D      1  FF                                                   TERMINATOR_FF             
0x00001B3E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B40      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001B42      9  800741593033313230                                   LEN8_STRING_CP932         length=7, text="AY03120"
0x00001B4B      1  FF                                                   TERMINATOR_FF             
0x00001B4C      1  FF                                                   TERMINATOR_FF             
0x00001B4D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B4F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B51     35  8021817582A282E282A082A082A082A021212082E282BE82...  LEN8_STRING_CP932         length=33, text="「いやああああ!! やだああああ!!」"
0x00001B74      1  FF                                                   TERMINATOR_FF             
0x00001B75      1  FF                                                   TERMINATOR_FF             
0x00001B76      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B78      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B7A      1  FF                                                   TERMINATOR_FF             
0x00001B7B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B7D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B7F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001B81      1  FF                                                   TERMINATOR_FF             
0x00001B82      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B84      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001B86      3  F340A8                                               IMM16_F3                  u16_be=16552, u16_le=43072
0x00001B89      1  FF                                                   TERMINATOR_FF             
0x00001B8A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B8C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B8E     24  801688BB82AA8BE090D882E890BA82F082A082B082E98142     LEN8_STRING_CP932         length=22, text="綾が金切り声をあげる。"
0x00001BA6      1  FF                                                   TERMINATOR_FF             
0x00001BA7      1  FF                                                   TERMINATOR_FF             
0x00001BA8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001BAA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001BAC      1  FF                                                   TERMINATOR_FF             
0x00001BAD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001BAF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001BB1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001BB3      1  FF                                                   TERMINATOR_FF             
0x00001BB4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001BB6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001BB8      3  F340A9                                               IMM16_F3                  u16_be=16553, u16_le=43328
0x00001BBB      1  FF                                                   TERMINATOR_FF             
0x00001BBC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001BBE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001BC0     48  802E82D382BD82E882CD954B8E8082C5965C82EA82E988BB...  LEN8_STRING_CP932         length=46, text="ふたりは必死で暴れる綾を力ずくでおさえつける。"
0x00001BF0      1  FF                                                   TERMINATOR_FF             
0x00001BF1      1  FF                                                   TERMINATOR_FF             
0x00001BF2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001BF4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001BF6      1  FF                                                   TERMINATOR_FF             
0x00001BF7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001BF9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001BFB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001BFD      1  FF                                                   TERMINATOR_FF             
0x00001BFE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C00      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001C02      3  F30EC1                                               IMM16_F3                  u16_be=3777, u16_le=49422
0x00001C05      1  FF                                                   TERMINATOR_FF             
0x00001C06      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C08      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C0A     54  80348D4C82B082BD968390BB82CC95AA8CFA82A291DC82C9...  LEN8_STRING_CP932         length=52, text="広げた麻製の分厚い袋に綾を押し込め、肩に抱えあげる。"
0x00001C40      1  FF                                                   TERMINATOR_FF             
0x00001C41      1  FF                                                   TERMINATOR_FF             
0x00001C42      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001C44      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001C46      1  FF                                                   TERMINATOR_FF             
0x00001C47      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C49      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C4B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001C4D      1  FF                                                   TERMINATOR_FF             
0x00001C4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C50      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001C52      3  F30EC2                                               IMM16_F3                  u16_be=3778, u16_le=49678
0x00001C55      1  FF                                                   TERMINATOR_FF             
0x00001C56      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C58      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001C5A      9  800742553030333530                                   LEN8_STRING_CP932         length=7, text="BU00350"
0x00001C63      1  FF                                                   TERMINATOR_FF             
0x00001C64      1  FF                                                   TERMINATOR_FF             
0x00001C65      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C67      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C69     16  800E81758F6482A282B782A981488176                     LEN8_STRING_CP932         length=14, text="「重いすか？」"
0x00001C79      1  FF                                                   TERMINATOR_FF             
0x00001C7A      1  FF                                                   TERMINATOR_FF             
0x00001C7B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001C7D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001C7F      1  FF                                                   TERMINATOR_FF             
0x00001C80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C82      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C84      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001C86      1  FF                                                   TERMINATOR_FF             
0x00001C87      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C89      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001C8B      3  F30EC3                                               IMM16_F3                  u16_be=3779, u16_le=49934
0x00001C8E      1  FF                                                   TERMINATOR_FF             
0x00001C8F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C91      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001C93      9  800754533030333030                                   LEN8_STRING_CP932         length=7, text="TS00300"
0x00001C9C      1  FF                                                   TERMINATOR_FF             
0x00001C9D      1  FF                                                   TERMINATOR_FF             
0x00001C9E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CA0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CA2     24  8016817582B782C182B0815B8F6482A281428345835C8176     LEN8_STRING_CP932         length=22, text="「すっげー重い。ウソ」"
0x00001CBA      1  FF                                                   TERMINATOR_FF             
0x00001CBB      1  FF                                                   TERMINATOR_FF             
0x00001CBC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001CBE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001CC0      1  FF                                                   TERMINATOR_FF             
0x00001CC1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CC3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CC5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001CC7      1  FF                                                   TERMINATOR_FF             
0x00001CC8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CCA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001CCC      3  F30EC4                                               IMM16_F3                  u16_be=3780, u16_le=50190
0x00001CCF      1  FF                                                   TERMINATOR_FF             
0x00001CD0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CD2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CD4    116  807282D382BD82E882CD939682BD82E8914F82CC82E682A4...  LEN8_STRING_CP932         length=114, text="ふたりは当たり前のように影の中へ消えていく。かつぎあげられ、じたばたしていた綾の足も、\n闇の中へと消えてしまった。"
0x00001D48      1  FF                                                   TERMINATOR_FF             
0x00001D49      1  FF                                                   TERMINATOR_FF             
0x00001D4A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001D4C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001D4E      1  FF                                                   TERMINATOR_FF             
0x00001D4F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001D51      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001D53      7  800573652D3133                                       LEN8_STRING_CP932         length=5, text="se-13"
0x00001D5A      1  FF                                                   TERMINATOR_FF             
0x00001D5B      1  FF                                                   TERMINATOR_FF             
0x00001D5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D60      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00001D62      1  FF                                                   TERMINATOR_FF             
0x00001D63      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001D65      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001D67      1  FF                                                   TERMINATOR_FF             
0x00001D68      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D6A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D6C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001D6E      1  FF                                                   TERMINATOR_FF             
0x00001D6F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D71      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001D73      3  F30EC5                                               IMM16_F3                  u16_be=3781, u16_le=50446
0x00001D76      1  FF                                                   TERMINATOR_FF             
0x00001D77      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001D79      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001D7B     92  805A82DA82AD82F082A882B382A682C282AF82C482A282BD...  LEN8_STRING_CP932         length=90, text="ぼくをおさえつけていた圧力がふっと消失する。ぼくに目もくれず、影の犬はふたりを追っていく。"
0x00001DD7      1  FF                                                   TERMINATOR_FF             
0x00001DD8      1  FF                                                   TERMINATOR_FF             
0x00001DD9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001DDB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001DDD      1  FF                                                   TERMINATOR_FF             
0x00001DDE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001DE0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001DE2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001DE4      1  FF                                                   TERMINATOR_FF             
0x00001DE5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001DE7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001DE9      3  F340AA                                               IMM16_F3                  u16_be=16554, u16_le=43584
0x00001DEC      1  FF                                                   TERMINATOR_FF             
0x00001DED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001DEF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001DF1     42  80288CA282CC8E7082CD896582C693AF89BB82B5814188EA...  LEN8_STRING_CP932         length=40, text="犬の姿は影と同化し、一瞬闇が深くなった。"
0x00001E1B      1  FF                                                   TERMINATOR_FF             
0x00001E1C      1  FF                                                   TERMINATOR_FF             
0x00001E1D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001E1F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001E21      1  FF                                                   TERMINATOR_FF             
0x00001E22      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E24      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E26      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x00001E28      1  FF                                                   TERMINATOR_FF             
0x00001E29      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001E2B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001E2D      1  FF                                                   TERMINATOR_FF             
0x00001E2E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E30      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E32      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00001E34      1  FF                                                   TERMINATOR_FF             
0x00001E35      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E37      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001E39      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001E3B      1  FF                                                   TERMINATOR_FF             
0x00001E3C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001E3E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001E40      1  FF                                                   TERMINATOR_FF             
0x00001E41      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E43      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E45      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001E47      1  FF                                                   TERMINATOR_FF             
0x00001E48      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E4A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001E4C      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00001E4E      1  FF                                                   TERMINATOR_FF             
0x00001E4F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001E51      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001E53      1  FF                                                   TERMINATOR_FF             
0x00001E54      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E56      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E58      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001E5A      1  FF                                                   TERMINATOR_FF             
0x00001E5B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E5D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001E5F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001E61      1  FF                                                   TERMINATOR_FF             
0x00001E62      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E64      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001E66      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001E68      1  FF                                                   TERMINATOR_FF             
0x00001E69      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001E6B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001E6D      1  FF                                                   TERMINATOR_FF             
0x00001E6E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E70      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E72      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001E74      1  FF                                                   TERMINATOR_FF             
0x00001E75      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E77      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001E79      3  F30EC6                                               IMM16_F3                  u16_be=3782, u16_le=50702
0x00001E7C      1  FF                                                   TERMINATOR_FF             
0x00001E7D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001E7F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001E81     68  80428A5882C882DD82AA96DA82CC914F82C5949682EA82C4...  LEN8_STRING_CP932         length=66, text="街なみが目の前で薄れていく。\n月にかかっていた雲が晴れ始めていた。"
0x00001EC5      1  FF                                                   TERMINATOR_FF             
0x00001EC6      1  FF                                                   TERMINATOR_FF             
0x00001EC7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001EC9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001ECB      1  FF                                                   TERMINATOR_FF             
0x00001ECC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001ECE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001ED0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001ED2      1  FF                                                   TERMINATOR_FF             
0x00001ED3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001ED5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001ED7      3  F340AB                                               IMM16_F3                  u16_be=16555, u16_le=43840
0x00001EDA      1  FF                                                   TERMINATOR_FF             
0x00001EDB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001EDD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001EDF    118  8074895F82CC8358834E838A815B839382C98A5882AA8966...  LEN8_STRING_CP932         length=116, text="雲のスクリーンに街が映し出されていたように。次第に実体感を失っていく街。\n綾たちを呑みこんで……街は消えてしまった。"
0x00001F55      1  FF                                                   TERMINATOR_FF             
0x00001F56      1  FF                                                   TERMINATOR_FF             
0x00001F57      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001F59      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F5B      1  FF                                                   TERMINATOR_FF             
0x00001F5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F60      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00001F62      1  FF                                                   TERMINATOR_FF             
0x00001F63      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F65      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F67      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F69      1  FF                                                   TERMINATOR_FF             
0x00001F6A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001F6C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F6E      1  FF                                                   TERMINATOR_FF             
0x00001F6F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F71      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F73      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001F75      1  FF                                                   TERMINATOR_FF             
0x00001F76      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F78      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F7A      3  F300FC                                               IMM16_F3                  u16_be=252, u16_le=64512
0x00001F7D      1  FF                                                   TERMINATOR_FF             
0x00001F7E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001F80      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F82      1  FF                                                   TERMINATOR_FF             
0x00001F83      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F85      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F87      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00001F89      1  FF                                                   TERMINATOR_FF             
0x00001F8A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F8C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F8E      2  F20E                                                 IMM8_F2                   u8=14, s8=14
0x00001F90      1  FF                                                   TERMINATOR_FF             
0x00001F91      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001F93      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F95      1  FF                                                   TERMINATOR_FF             
0x00001F96      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F98      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F9A      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001F9C      1  FF                                                   TERMINATOR_FF             
0x00001F9D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F9F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001FA1      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00001FA3      1  FF                                                   TERMINATOR_FF             
0x00001FA4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001FA6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001FA8      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001FAA      1  FF                                                   TERMINATOR_FF             
0x00001FAB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001FAD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001FAF      1  FF                                                   TERMINATOR_FF             
0x00001FB0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001FB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001FB4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001FB6      1  FF                                                   TERMINATOR_FF             
0x00001FB7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001FB9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001FBB      3  F30EC7                                               IMM16_F3                  u16_be=3783, u16_le=50958
0x00001FBE      1  FF                                                   TERMINATOR_FF             
0x00001FBF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001FC1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001FC3     82  80508C8E82CC8B5082AD968482DF97A782C4926E82C982DA...  LEN8_STRING_CP932         length=80, text="月の輝く埋め立て地にぼくはひとりで取り残されていた。\nぼくは、まにあわなかった。"
0x00002015      1  FF                                                   TERMINATOR_FF             
0x00002016      1  FF                                                   TERMINATOR_FF             
0x00002017      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002019      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000201B      1  FF                                                   TERMINATOR_FF             
0x0000201C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000201E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002020      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002022      1  FF                                                   TERMINATOR_FF             
0x00002023      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002025      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002027      3  F30EC8                                               IMM16_F3                  u16_be=3784, u16_le=51214
0x0000202A      1  FF                                                   TERMINATOR_FF             
0x0000202B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000202D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000202F     64  803E88BB82CD896582F094F082AF82C482A282BD81425C6E...  LEN8_STRING_CP932         length=62, text="綾は影を避けていた。\n影の中に足を踏み入れることを恐れていた。"
0x0000206F      1  FF                                                   TERMINATOR_FF             
0x00002070      1  FF                                                   TERMINATOR_FF             
0x00002071      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002073      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002075      1  FF                                                   TERMINATOR_FF             
0x00002076      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002078      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000207A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000207C      1  FF                                                   TERMINATOR_FF             
0x0000207D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000207F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002081      3  F30EC9                                               IMM16_F3                  u16_be=3785, u16_le=51470
0x00002084      1  FF                                                   TERMINATOR_FF             
0x00002085      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002087      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002089    106  806882BB82F182C894DE8F9782AA8C8E82C982A982A982E9...  LEN8_STRING_CP932         length=104, text="そんな彼女が月にかかる雲を見落としていた。\n影の犬からぼくを救おうとして、自分を守る\nことを忘れていた。"
0x000020F3      1  FF                                                   TERMINATOR_FF             
0x000020F4      1  FF                                                   TERMINATOR_FF             
0x000020F5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000020F7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000020F9      1  FF                                                   TERMINATOR_FF             
0x000020FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000020FC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000020FE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002100      1  FF                                                   TERMINATOR_FF             
0x00002101      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002103      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002105      3  F340AC                                               IMM16_F3                  u16_be=16556, u16_le=44096
0x00002108      1  FF                                                   TERMINATOR_FF             
0x00002109      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000210B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000210D     42  802882A082F182C882C9928D88D3905B82AD814197709053...  LEN8_STRING_CP932         length=40, text="あんなに注意深く、用心していた綾が……。"
0x00002137      1  FF                                                   TERMINATOR_FF             
0x00002138      1  FF                                                   TERMINATOR_FF             
0x00002139      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000213B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000213D      1  FF                                                   TERMINATOR_FF             
0x0000213E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002140      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002142      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002144      1  FF                                                   TERMINATOR_FF             
0x00002145      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002147      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002149      3  F30ECA                                               IMM16_F3                  u16_be=3786, u16_le=51726
0x0000214C      1  FF                                                   TERMINATOR_FF             
0x0000214D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000214F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002151     46  802C94DE8F9782CC8B7C82AA935D82AA82C182C482A282E9...  LEN8_STRING_CP932         length=44, text="彼女の弓が転がっている。\n矢も散乱していた。"
0x0000217F      1  FF                                                   TERMINATOR_FF             
0x00002180      1  FF                                                   TERMINATOR_FF             
0x00002181      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002183      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002185      1  FF                                                   TERMINATOR_FF             
0x00002186      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002188      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000218A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000218C      1  FF                                                   TERMINATOR_FF             
0x0000218D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000218F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002191      3  F30ECB                                               IMM16_F3                  u16_be=3787, u16_le=51982
0x00002194      1  FF                                                   TERMINATOR_FF             
0x00002195      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002197      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002199     74  8048935D82AA82C182BD96EE82F082D082EB82A481425C6E...  LEN8_STRING_CP932         length=72, text="転がった矢をひろう。\n弓袋で長い弓を包む。\n矢筒をくくりつけ肩にかける。"
0x000021E3      1  FF                                                   TERMINATOR_FF             
0x000021E4      1  FF                                                   TERMINATOR_FF             
0x000021E5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000021E7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000021E9      1  FF                                                   TERMINATOR_FF             
0x000021EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000021EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000021EE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000021F0      1  FF                                                   TERMINATOR_FF             
0x000021F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000021F3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000021F5      3  F30ECC                                               IMM16_F3                  u16_be=3788, u16_le=52238
0x000021F8      1  FF                                                   TERMINATOR_FF             
0x000021F9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000021FB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000021FD     28  801A82DA82AD82CD814182DC82C982A082ED82C882A982C1...  LEN8_STRING_CP932         length=26, text="ぼくは、まにあわなかった。"
0x00002219      1  FF                                                   TERMINATOR_FF             
0x0000221A      1  FF                                                   TERMINATOR_FF             
0x0000221B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000221D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000221F      1  FF                                                   TERMINATOR_FF             
0x00002220      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002222      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002224      8  800673652D653234                                     LEN8_STRING_CP932         length=6, text="se-e24"
0x0000222C      1  FF                                                   TERMINATOR_FF             
0x0000222D      1  FF                                                   TERMINATOR_FF             
0x0000222E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002230      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002232      2  F214                                                 IMM8_F2                   u8=20, s8=20
0x00002234      1  FF                                                   TERMINATOR_FF             
0x00002235      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002237      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002239      1  FF                                                   TERMINATOR_FF             
0x0000223A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000223C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000223E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002240      1  FF                                                   TERMINATOR_FF             
0x00002241      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002243      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002245      3  F30ECD                                               IMM16_F3                  u16_be=3789, u16_le=52494
0x00002248      1  FF                                                   TERMINATOR_FF             
0x00002249      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000224B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000224D     48  802E82DA82AD82CD8F4982ED82E882CC8CA982A682C882A2...  LEN8_STRING_CP932         length=46, text="ぼくは終わりの見えない埋め立て地を歩き始めた。"
0x0000227D      1  FF                                                   TERMINATOR_FF             
0x0000227E      1  FF                                                   TERMINATOR_FF             
0x0000227F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002281      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002283      1  FF                                                   TERMINATOR_FF             
0x00002284      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002286      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002288      8  800673652D653234                                     LEN8_STRING_CP932         length=6, text="se-e24"
0x00002290      1  FF                                                   TERMINATOR_FF             
0x00002291      1  FF                                                   TERMINATOR_FF             
0x00002292      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002294      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002296      2  F214                                                 IMM8_F2                   u8=20, s8=20
0x00002298      1  FF                                                   TERMINATOR_FF             
0x00002299      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000229B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000229D      1  FF                                                   TERMINATOR_FF             
0x0000229E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022A2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000022A4      1  FF                                                   TERMINATOR_FF             
0x000022A5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022A7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000022A9      3  F30ECE                                               IMM16_F3                  u16_be=3790, u16_le=52750
0x000022AC      1  FF                                                   TERMINATOR_FF             
0x000022AD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000022AF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000022B1     90  805894DE8F9782CD82C882C982A982F0904D82B682C482A2...  LEN8_STRING_CP932         length=88, text="彼女はなにかを信じていた。\n悲しみやつらさを自分の中に閉じ込めて信じ続けようとして来た。"
0x0000230B      1  FF                                                   TERMINATOR_FF             
0x0000230C      1  FF                                                   TERMINATOR_FF             
0x0000230D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000230F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002311      1  FF                                                   TERMINATOR_FF             
0x00002312      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002314      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002316      8  800673652D653234                                     LEN8_STRING_CP932         length=6, text="se-e24"
0x0000231E      1  FF                                                   TERMINATOR_FF             
0x0000231F      1  FF                                                   TERMINATOR_FF             
0x00002320      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002322      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002324      2  F214                                                 IMM8_F2                   u8=20, s8=20
0x00002326      1  FF                                                   TERMINATOR_FF             
0x00002327      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002329      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000232B      1  FF                                                   TERMINATOR_FF             
0x0000232C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000232E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002330      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002332      1  FF                                                   TERMINATOR_FF             
0x00002333      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002335      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002337      3  F30ECF                                               IMM16_F3                  u16_be=3791, u16_le=53006
0x0000233A      1  FF                                                   TERMINATOR_FF             
0x0000233B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000233D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000233F     66  804082DA82AD82CD94DE8F9782F08F9582AF82BD82A982C1...  LEN8_STRING_CP932         length=64, text="ぼくは彼女を助けたかった。\n彼女がくじける瞬間を見たくなかった。"
0x00002381      1  FF                                                   TERMINATOR_FF             
0x00002382      1  FF                                                   TERMINATOR_FF             
0x00002383      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002385      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002387      1  FF                                                   TERMINATOR_FF             
0x00002388      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000238A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000238C      8  800673652D653234                                     LEN8_STRING_CP932         length=6, text="se-e24"
0x00002394      1  FF                                                   TERMINATOR_FF             
0x00002395      1  FF                                                   TERMINATOR_FF             
0x00002396      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002398      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000239A      2  F214                                                 IMM8_F2                   u8=20, s8=20
0x0000239C      1  FF                                                   TERMINATOR_FF             
0x0000239D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000239F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000023A1      1  FF                                                   TERMINATOR_FF             
0x000023A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023A6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000023A8      1  FF                                                   TERMINATOR_FF             
0x000023A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023AB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000023AD      3  F30ED0                                               IMM16_F3                  u16_be=3792, u16_le=53262
0x000023B0      1  FF                                                   TERMINATOR_FF             
0x000023B1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000023B3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000023B5     26  801882C582E0814194DE8F9782CD82E082A482A282C882A2...  LEN8_STRING_CP932         length=24, text="でも、彼女はもういない。"
0x000023CF      1  FF                                                   TERMINATOR_FF             
0x000023D0      1  FF                                                   TERMINATOR_FF             
0x000023D1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000023D3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000023D5      1  FF                                                   TERMINATOR_FF             
0x000023D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023DA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000023DC      1  FF                                                   TERMINATOR_FF             
0x000023DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023DF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000023E1      3  F30ED1                                               IMM16_F3                  u16_be=3793, u16_le=53518
0x000023E4      1  FF                                                   TERMINATOR_FF             
0x000023E5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000023E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000023E9    104  806682DA82AD82CD82BB82F182C882B182C682CC82BD82DF...  LEN8_STRING_CP932         length=102, text="ぼくはそんなことのために、この場所をさがしたわけではない。\nぼくも彼女と同じに…信じてみたかったのだ。"
0x00002451      1  FF                                                   TERMINATOR_FF             
0x00002452      1  FF                                                   TERMINATOR_FF             
0x00002453      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002455      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002457      1  FF                                                   TERMINATOR_FF             
0x00002458      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000245A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000245C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000245E      1  FF                                                   TERMINATOR_FF             
0x0000245F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002461      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002463      3  F340AD                                               IMM16_F3                  u16_be=16557, u16_le=44352
0x00002466      1  FF                                                   TERMINATOR_FF             
0x00002467      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002469      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000246B     76  804A8BAD82AD904D82B682E982B182C682C58163816382C8...  LEN8_STRING_CP932         length=74, text="強く信じることで……なにかができると思って\nいた。\nなのに……ぼくは……。"
0x000024B7      1  FF                                                   TERMINATOR_FF             
0x000024B8      1  FF                                                   TERMINATOR_FF             
0x000024B9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000024BB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000024BD      1  FF                                                   TERMINATOR_FF             
0x000024BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024C2      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x000024C4      1  FF                                                   TERMINATOR_FF             
0x000024C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000024C9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000024CB      1  FF                                                   TERMINATOR_FF             
0x000024CC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000024CE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000024D0      1  FF                                                   TERMINATOR_FF             
0x000024D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024D5      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000024D7      1  FF                                                   TERMINATOR_FF             
0x000024D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024DA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000024DC      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x000024DE      1  FF                                                   TERMINATOR_FF             
0x000024DF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000024E1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000024E3      1  FF                                                   TERMINATOR_FF             
0x000024E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024E8      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000024EA      1  FF                                                   TERMINATOR_FF             
0x000024EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024ED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000024EF      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000024F1      1  FF                                                   TERMINATOR_FF             
0x000024F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024F4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000024F6      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000024F8      1  FF                                                   TERMINATOR_FF             
0x000024F9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000024FB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000024FD      1  FF                                                   TERMINATOR_FF             
0x000024FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002500      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002502      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00002504      1  FF                                                   TERMINATOR_FF             
0x00002505      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002507      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002509      2  F20C                                                 IMM8_F2                   u8=12, s8=12
0x0000250B      1  FF                                                   TERMINATOR_FF             
0x0000250C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000250E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002510      1  FF                                                   TERMINATOR_FF             
0x00002511      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002513      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002515      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00002517      1  FF                                                   TERMINATOR_FF             
0x00002518      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000251A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000251C      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000251E      1  FF                                                   TERMINATOR_FF             
0x0000251F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002521      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002523      1  FF                                                   TERMINATOR_FF             
0x00002524      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002526      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002528      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000252A      1  FF                                                   TERMINATOR_FF             
0x0000252B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000252D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000252F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002531      1  FF                                                   TERMINATOR_FF             
0x00002532      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002534      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002536      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002538      1  FF                                                   TERMINATOR_FF             
0x00002539      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000253B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000253D      1  FF                                                   TERMINATOR_FF             
0x0000253E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002540      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002542      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002544      1  FF                                                   TERMINATOR_FF             
0x00002545      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002547      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002549      3  F30ED2                                               IMM16_F3                  u16_be=3794, u16_le=53774
0x0000254C      1  FF                                                   TERMINATOR_FF             
0x0000254D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000254F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002551     94  805C96B3906C82CC8A5882F082DA82AD82CD95E082A282C4...  LEN8_STRING_CP932         length=92, text="無人の街をぼくは歩いていた。\n影の中では妖精たちがぼくなどいないかのようにじゃれあっている。"
0x000025AF      1  FF                                                   TERMINATOR_FF             
0x000025B0      1  FF                                                   TERMINATOR_FF             
0x000025B1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000025B3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000025B5      1  FF                                                   TERMINATOR_FF             
0x000025B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025BA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000025BC      1  FF                                                   TERMINATOR_FF             
0x000025BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025BF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000025C1      3  F30ED3                                               IMM16_F3                  u16_be=3795, u16_le=54030
0x000025C4      1  FF                                                   TERMINATOR_FF             
0x000025C5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000025C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000025C9     98  80608C8E82CD95CF82ED82E782B896E98BF382F095A282A2...  LEN8_STRING_CP932         length=96, text="月は変わらず夜空を覆い尽くし、かわりばえの\nしない街並みが黒いシルエットを浮かび上がらせている。"
0x0000262B      1  FF                                                   TERMINATOR_FF             
0x0000262C      1  FF                                                   TERMINATOR_FF             
0x0000262D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000262F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002631      1  FF                                                   TERMINATOR_FF             
0x00002632      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002634      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002636      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002638      1  FF                                                   TERMINATOR_FF             
0x00002639      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000263B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000263D      3  F30ED4                                               IMM16_F3                  u16_be=3796, u16_le=54286
0x00002640      1  FF                                                   TERMINATOR_FF             
0x00002641      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002643      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002645     74  804882DA82AD82CD82A082C482E082C882AD8A5882F095E0...  LEN8_STRING_CP932         length=72, text="ぼくはあてもなく街を歩き回っていた。\nどうすればいいのかわからなかった。"
0x0000268F      1  FF                                                   TERMINATOR_FF             
0x00002690      1  FF                                                   TERMINATOR_FF             
0x00002691      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002693      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002695      1  FF                                                   TERMINATOR_FF             
0x00002696      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002698      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000269A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000269C      1  FF                                                   TERMINATOR_FF             
0x0000269D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000269F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000026A1      3  F30ED5                                               IMM16_F3                  u16_be=3797, u16_le=54542
0x000026A4      1  FF                                                   TERMINATOR_FF             
0x000026A5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000026A7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000026A9     52  803282A982C282C488BB82F0917B82B589F182C182BD82C6...  LEN8_STRING_CP932         length=50, text="かつて綾を捜し回ったときと同じ……では\nなかった。"
0x000026DD      1  FF                                                   TERMINATOR_FF             
0x000026DE      1  FF                                                   TERMINATOR_FF             
0x000026DF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000026E1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000026E3      1  FF                                                   TERMINATOR_FF             
0x000026E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000026E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000026E8      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000026EA      1  FF                                                   TERMINATOR_FF             
0x000026EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000026ED      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000026EF      3  F30ED6                                               IMM16_F3                  u16_be=3798, u16_le=54798
0x000026F2      1  FF                                                   TERMINATOR_FF             
0x000026F3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000026F5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000026F7     98  806082A082CC82C682AB82DA82AD82CD939A82A682F082B3...  LEN8_STRING_CP932         length=96, text="あのときぼくは答えをさがしていた。\n彼女に会うことさえできれば、なにかがわかるのだと思っていた。"
0x00002759      1  FF                                                   TERMINATOR_FF             
0x0000275A      1  FF                                                   TERMINATOR_FF             
0x0000275B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000275D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000275F      1  FF                                                   TERMINATOR_FF             
0x00002760      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002762      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002764      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002766      1  FF                                                   TERMINATOR_FF             
0x00002767      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002769      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000276B      3  F30ED7                                               IMM16_F3                  u16_be=3799, u16_le=55054
0x0000276E      1  FF                                                   TERMINATOR_FF             
0x0000276F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002771      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002773     94  805C82AF82EA82C78EC08DDB82C982CD82BB82A482C582CD...  LEN8_STRING_CP932         length=92, text="けれど実際にはそうではなかった。\n彼女と出会ってしまったことで疑問は増えていくばかりだった。"
0x000027D1      1  FF                                                   TERMINATOR_FF             
0x000027D2      1  FF                                                   TERMINATOR_FF             
0x000027D3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000027D5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000027D7      1  FF                                                   TERMINATOR_FF             
0x000027D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027DC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000027DE      1  FF                                                   TERMINATOR_FF             
0x000027DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027E1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000027E3      3  F30ED8                                               IMM16_F3                  u16_be=3800, u16_le=55310
0x000027E6      1  FF                                                   TERMINATOR_FF             
0x000027E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000027E9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000027EB     74  804882B182CC88D98FED82C88A5882CC88D396A181425C6E...  LEN8_STRING_CP932         length=72, text="この異常な街の意味。\n影の中で起きるできごと。\n小さな声で鳴くものたち。"
0x00002835      1  FF                                                   TERMINATOR_FF             
0x00002836      1  FF                                                   TERMINATOR_FF             
0x00002837      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002839      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000283B      1  FF                                                   TERMINATOR_FF             
0x0000283C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000283E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002840      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002842      1  FF                                                   TERMINATOR_FF             
0x00002843      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002845      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002847      3  F30ED9                                               IMM16_F3                  u16_be=3801, u16_le=55566
0x0000284A      1  FF                                                   TERMINATOR_FF             
0x0000284B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000284D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000284F     44  802A82BB82B582C488BB82CD82B182CC8A5882A982E782CC...  LEN8_STRING_CP932         length=42, text="そして綾はこの街からの出口をさがしている。"
0x0000287B      1  FF                                                   TERMINATOR_FF             
0x0000287C      1  FF                                                   TERMINATOR_FF             
0x0000287D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000287F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002881      1  FF                                                   TERMINATOR_FF             
0x00002882      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002884      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002886      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002888      1  FF                                                   TERMINATOR_FF             
0x00002889      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000288B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000288D      3  F30EDA                                               IMM16_F3                  u16_be=3802, u16_le=55822
0x00002890      1  FF                                                   TERMINATOR_FF             
0x00002891      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002893      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002895     90  805881778F6F8CFB81788163816382C182C482A282C182BD...  LEN8_STRING_CP932         length=88, text="『出口』……っていったいなんだろう。\n彼女はここから出て、どこへ行こうとして\nいたんだ？"
0x000028EF      1  FF                                                   TERMINATOR_FF             
0x000028F0      1  FF                                                   TERMINATOR_FF             
0x000028F1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000028F3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000028F5      1  FF                                                   TERMINATOR_FF             
0x000028F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028F8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028FA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000028FC      1  FF                                                   TERMINATOR_FF             
0x000028FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028FF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002901      3  F30EDB                                               IMM16_F3                  u16_be=3803, u16_le=56078
0x00002904      1  FF                                                   TERMINATOR_FF             
0x00002905      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002907      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002909     80  804E82BB82CC82C682AB814182DA82AD82CD8163816382C7...  LEN8_STRING_CP932         length=78, text="そのとき、ぼくは……どうするんだろう。\n考えることを避けて来た問いかけだった。"
0x00002959      1  FF                                                   TERMINATOR_FF             
0x0000295A      1  FF                                                   TERMINATOR_FF             
0x0000295B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000295D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000295F      1  FF                                                   TERMINATOR_FF             
0x00002960      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002962      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002964      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002966      1  FF                                                   TERMINATOR_FF             
0x00002967      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002969      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000296B      3  F30EDC                                               IMM16_F3                  u16_be=3804, u16_le=56334
0x0000296E      1  FF                                                   TERMINATOR_FF             
0x0000296F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002971      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002973     98  806082DA82AD82E088BB82C693AF82B682C981418F6F8CFB...  LEN8_STRING_CP932         length=96, text="ぼくも綾と同じに、出口をさがさなければ\nいけないはずなのに、それを考えようとは\nしてこなかった。"
0x000029D5      1  FF                                                   TERMINATOR_FF             
0x000029D6      1  FF                                                   TERMINATOR_FF             
0x000029D7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000029D9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000029DB      1  FF                                                   TERMINATOR_FF             
0x000029DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000029DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000029E0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000029E2      1  FF                                                   TERMINATOR_FF             
0x000029E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000029E5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000029E7      3  F30EDD                                               IMM16_F3                  u16_be=3805, u16_le=56590
0x000029EA      1  FF                                                   TERMINATOR_FF             
0x000029EB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000029ED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000029EF     32  801E82C782A482B582C48D6C82A682C882A982C182BD82CC...  LEN8_STRING_CP932         length=30, text="どうして考えなかったのだろう。"
0x00002A0F      1  FF                                                   TERMINATOR_FF             
0x00002A10      1  FF                                                   TERMINATOR_FF             
0x00002A11      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002A13      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002A15      1  FF                                                   TERMINATOR_FF             
0x00002A16      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A18      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A1A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002A1C      1  FF                                                   TERMINATOR_FF             
0x00002A1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A1F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002A21      3  F30EDE                                               IMM16_F3                  u16_be=3806, u16_le=56846
0x00002A24      1  FF                                                   TERMINATOR_FF             
0x00002A25      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A27      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A29     56  803682A282C282DC82C582E082B182CC8A5882C995C282B6...  LEN8_STRING_CP932         length=54, text="いつまでもこの街に閉じ込められているわけにはいかない。"
0x00002A61      1  FF                                                   TERMINATOR_FF             
0x00002A62      1  FF                                                   TERMINATOR_FF             
0x00002A63      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002A65      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002A67      1  FF                                                   TERMINATOR_FF             
0x00002A68      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A6A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A6C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002A6E      1  FF                                                   TERMINATOR_FF             
0x00002A6F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A71      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002A73      3  F30EDF                                               IMM16_F3                  u16_be=3807, u16_le=57102
0x00002A76      1  FF                                                   TERMINATOR_FF             
0x00002A77      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A79      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A7B     54  803482DA82AD82C982CD96DF82E782C882AF82EA82CE82C8...  LEN8_STRING_CP932         length=52, text="ぼくには戻らなければならない場所があるはず\nなのに。"
0x00002AB1      1  FF                                                   TERMINATOR_FF             
0x00002AB2      1  FF                                                   TERMINATOR_FF             
0x00002AB3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002AB5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002AB7      1  FF                                                   TERMINATOR_FF             
0x00002AB8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002ABA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002ABC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002ABE      1  FF                                                   TERMINATOR_FF             
0x00002ABF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002AC1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002AC3      3  F30EE0                                               IMM16_F3                  u16_be=3808, u16_le=57358
0x00002AC6      1  FF                                                   TERMINATOR_FF             
0x00002AC7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002AC9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002ACB     62  803C8CA882C982A982AF82BD8B7C91DC82CD8C7982AD8141...  LEN8_STRING_CP932         length=60, text="肩にかけた弓袋は軽く、ほとんど存在を感じられなくなっていた。"
0x00002B09      1  FF                                                   TERMINATOR_FF             
0x00002B0A      1  FF                                                   TERMINATOR_FF             
0x00002B0B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002B0D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002B0F      1  FF                                                   TERMINATOR_FF             
0x00002B10      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B12      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B14      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002B16      1  FF                                                   TERMINATOR_FF             
0x00002B17      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B19      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002B1B      3  F30EE1                                               IMM16_F3                  u16_be=3809, u16_le=57614
0x00002B1E      1  FF                                                   TERMINATOR_FF             
0x00002B1F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B21      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B23     52  803282DA82AD82CD8EA995AA82AA82C882C982F082B782D7...  LEN8_STRING_CP932         length=50, text="ぼくは自分がなにをすべきなのか見失いかけて\nいた。"
0x00002B57      1  FF                                                   TERMINATOR_FF             
0x00002B58      1  FF                                                   TERMINATOR_FF             
0x00002B59      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002B5B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002B5D      1  FF                                                   TERMINATOR_FF             
0x00002B5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B60      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B62      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002B64      1  FF                                                   TERMINATOR_FF             
0x00002B65      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B67      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002B69      3  F340AE                                               IMM16_F3                  u16_be=16558, u16_le=44608
0x00002B6C      1  FF                                                   TERMINATOR_FF             
0x00002B6D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B6F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B71     56  803688BB82F08CA98EB882A482C682A282A482B182C682CD...  LEN8_STRING_CP932         length=54, text="綾を見失うということは……。\n目的を見失うことだった。"
0x00002BA9      1  FF                                                   TERMINATOR_FF             
0x00002BAA      1  FF                                                   TERMINATOR_FF             
0x00002BAB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002BAD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002BAF      1  FF                                                   TERMINATOR_FF             
0x00002BB0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BB4      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00002BB6      1  FF                                                   TERMINATOR_FF             
0x00002BB7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BB9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002BBB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002BBD      1  FF                                                   TERMINATOR_FF             
0x00002BBE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002BC0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002BC2      1  FF                                                   TERMINATOR_FF             
0x00002BC3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BC5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BC7      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00002BC9      1  FF                                                   TERMINATOR_FF             
0x00002BCA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BCC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002BCE      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00002BD0      1  FF                                                   TERMINATOR_FF             
0x00002BD1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002BD3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002BD5      1  FF                                                   TERMINATOR_FF             
0x00002BD6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BD8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BDA      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002BDC      1  FF                                                   TERMINATOR_FF             
0x00002BDD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BDF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002BE1      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002BE3      1  FF                                                   TERMINATOR_FF             
0x00002BE4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BE6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002BE8      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002BEA      1  FF                                                   TERMINATOR_FF             
0x00002BEB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002BED      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002BEF      1  FF                                                   TERMINATOR_FF             
0x00002BF0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BF2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BF4      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00002BF6      1  FF                                                   TERMINATOR_FF             
0x00002BF7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002BF9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002BFB      1  FF                                                   TERMINATOR_FF             
0x00002BFC      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00002BFE      3  F10058                                               IMM16_F1                  u16_be=88, u16_le=22528
0x00002C01      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002C03      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00002C04      1  FF                                                   TERMINATOR_FF             
0x00002C05      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x00002C07      1  2D                                                   OPAQUE_RAW_BYTES          bytes=2D
0x00002C08      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C0A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C0C      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00002C0E      1  FF                                                   TERMINATOR_FF             
0x00002C0F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002C11      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00002C13     14  800C50533241303534612E62696E                         LEN8_STRING_CP932         length=12, text="PS2A054a.bin"
0x00002C21      1  FF                                                   TERMINATOR_FF             
0x00002C22      1  FF                                                   TERMINATOR_FF             
0x00002C23      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002C25      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002C27      1  FF                                                   TERMINATOR_FF             
0x00002C28      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00002C2A      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x00002C2C      1  59                                                   OPAQUE_RAW_BYTES          bytes=59
0x00002C2D      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00002C2F      3  F10058                                               IMM16_F1                  u16_be=88, u16_le=22528
0x00002C32      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002C34      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00002C35      1  FF                                                   TERMINATOR_FF             
0x00002C36      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x00002C38      1  59                                                   OPAQUE_RAW_BYTES          bytes=59
0x00002C39      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C3D      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00002C3F      1  FF                                                   TERMINATOR_FF             
0x00002C40      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002C42      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00002C44     14  800C50533241303533612E62696E                         LEN8_STRING_CP932         length=12, text="PS2A053a.bin"
0x00002C52      1  FF                                                   TERMINATOR_FF             
0x00002C53      1  FF                                                   TERMINATOR_FF             
0x00002C54      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002C56      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002C58      1  FF                                                   TERMINATOR_FF             
0x00002C59      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00002C5B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002C5D      1  FF                                                   TERMINATOR_FF             
0x00002C5E      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x00002C60      1  68                                                   OPAQUE_RAW_BYTES          bytes=68
0x00002C61      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x00002C63      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00002C65      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x00002C67      1  59                                                   OPAQUE_RAW_BYTES          bytes=59
0x00002C68      1  FF                                                   TERMINATOR_FF             
0x00002C69      1  FF                                                   TERMINATOR_FF             
