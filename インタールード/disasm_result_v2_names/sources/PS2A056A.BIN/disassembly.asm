; Full conservative disassembly for PS2A056A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      2  F227                                                 IMM8_F2                   u8=39, s8=39
0x0000000D      1  FF                                                   TERMINATOR_FF             
0x0000000E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000010      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000012      1  FF                                                   TERMINATOR_FF             
0x00000013      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000015      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000017      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00000019      1  FF                                                   TERMINATOR_FF             
0x0000001A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000001C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000001E      2  F22A                                                 IMM8_F2                   u8=42, s8=42
0x00000020      1  FF                                                   TERMINATOR_FF             
0x00000021      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000023      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000025      1  FF                                                   TERMINATOR_FF             
0x00000026      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000028      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000002A      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000002C      1  FF                                                   TERMINATOR_FF             
0x0000002D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000002F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000031      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000033      1  FF                                                   TERMINATOR_FF             
0x00000034      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000036      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000038      2  F23C                                                 IMM8_F2                   u8=60, s8=60
0x0000003A      1  FF                                                   TERMINATOR_FF             
0x0000003B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000003D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000003F      1  FF                                                   TERMINATOR_FF             
0x00000040      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000042      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000044      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000046      1  FF                                                   TERMINATOR_FF             
0x00000047      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000049      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000004B      3  F31128                                               IMM16_F3                  u16_be=4392, u16_le=10257
0x0000004E      1  FF                                                   TERMINATOR_FF             
0x0000004F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000051      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000053     48  802E82DA82AD82CD96BE82E982A28A5882C993CB82C197A7...  LEN8_STRING_CP932         length=46, text="ぼくは明るい街に突っ立っていた。\n戻って来た。"
0x00000083      1  FF                                                   TERMINATOR_FF             
0x00000084      1  FF                                                   TERMINATOR_FF             
0x00000085      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000087      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000089      1  FF                                                   TERMINATOR_FF             
0x0000008A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000008C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000008E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000090      1  FF                                                   TERMINATOR_FF             
0x00000091      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000093      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000095      3  F31129                                               IMM16_F3                  u16_be=4393, u16_le=10513
0x00000098      1  FF                                                   TERMINATOR_FF             
0x00000099      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000009B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000009D     64  803E82B182B182CD8163816382A082CC8A5882C582CD82C8...  LEN8_STRING_CP932         length=62, text="ここは……あの街ではない。\nここは……ぼくの暮らしていた街だ。"
0x000000DD      1  FF                                                   TERMINATOR_FF             
0x000000DE      1  FF                                                   TERMINATOR_FF             
0x000000DF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000E1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000E3      1  FF                                                   TERMINATOR_FF             
0x000000E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E8      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000000EA      1  FF                                                   TERMINATOR_FF             
0x000000EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000ED      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000000EF      3  F340F7                                               IMM16_F3                  u16_be=16631, u16_le=63296
0x000000F2      1  FF                                                   TERMINATOR_FF             
0x000000F3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000F5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000F7     66  804082AF82EA82C78141906C82CC8E7082CD82C782B182C9...  LEN8_STRING_CP932         length=64, text="けれど、人の姿はどこにもなかった。\n通りにも、建てものの中にも。"
0x00000139      1  FF                                                   TERMINATOR_FF             
0x0000013A      1  FF                                                   TERMINATOR_FF             
0x0000013B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000013D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000013F      1  FF                                                   TERMINATOR_FF             
0x00000140      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000142      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000144      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000146      1  FF                                                   TERMINATOR_FF             
0x00000147      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000149      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000014B      3  F340F8                                               IMM16_F3                  u16_be=16632, u16_le=63552
0x0000014E      1  FF                                                   TERMINATOR_FF             
0x0000014F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000151      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000153     98  80608B699283935882C582CD8141834A8362837682CC9286...  LEN8_STRING_CP932         length=96, text="喫茶店では、カップの中でコーヒーが湯気を\nたてていた。\n焼きたてのトーストがまだ熱を持っていた。"
0x000001B5      1  FF                                                   TERMINATOR_FF             
0x000001B6      1  FF                                                   TERMINATOR_FF             
0x000001B7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001B9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001BB      1  FF                                                   TERMINATOR_FF             
0x000001BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001C0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001C2      1  FF                                                   TERMINATOR_FF             
0x000001C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001C5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001C7      3  F340F9                                               IMM16_F3                  u16_be=16633, u16_le=63808
0x000001CA      1  FF                                                   TERMINATOR_FF             
0x000001CB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001CD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001CF     38  802482BD82C182BD8DA18141906C82AA8FC182A682C482B5...  LEN8_STRING_CP932         length=36, text="たった今、人が消えてしまったように。"
0x000001F5      1  FF                                                   TERMINATOR_FF             
0x000001F6      1  FF                                                   TERMINATOR_FF             
0x000001F7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001F9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001FB      1  FF                                                   TERMINATOR_FF             
0x000001FC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000200      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000202      1  FF                                                   TERMINATOR_FF             
0x00000203      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000205      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000207      3  F340FA                                               IMM16_F3                  u16_be=16634, u16_le=64064
0x0000020A      1  FF                                                   TERMINATOR_FF             
0x0000020B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000020D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000020F     82  805082DA82AD82CD81418365838C837282F082C282AF82E9...  LEN8_STRING_CP932         length=80, text="ぼくは、テレビをつける。\nなにも映らなかった。\nただ、ノイズで画面がゆれていた。"
0x00000261      1  FF                                                   TERMINATOR_FF             
0x00000262      1  FF                                                   TERMINATOR_FF             
0x00000263      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000265      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000267      1  FF                                                   TERMINATOR_FF             
0x00000268      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000026A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000026C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000026E      1  FF                                                   TERMINATOR_FF             
0x0000026F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000271      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000273      3  F3112A                                               IMM16_F3                  u16_be=4394, u16_le=10769
0x00000276      1  FF                                                   TERMINATOR_FF             
0x00000277      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000279      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000027B     44  802A924E82E082A282C882A281425C6E82DA82AD82CD8163...  LEN8_STRING_CP932         length=42, text="誰もいない。\nぼくは……ひとりきりだった。"
0x000002A7      1  FF                                                   TERMINATOR_FF             
0x000002A8      1  FF                                                   TERMINATOR_FF             
0x000002A9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002AB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002AD      1  FF                                                   TERMINATOR_FF             
0x000002AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000002B4      1  FF                                                   TERMINATOR_FF             
0x000002B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000002B9      3  F3112B                                               IMM16_F3                  u16_be=4395, u16_le=11025
0x000002BC      1  FF                                                   TERMINATOR_FF             
0x000002BD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002C1     28  801A8163816388BB82CD81485C6E94DE8F9782CD82C782B1...  LEN8_STRING_CP932         length=26, text="……綾は？\n彼女はどこに？"
0x000002DD      1  FF                                                   TERMINATOR_FF             
0x000002DE      1  FF                                                   TERMINATOR_FF             
0x000002DF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002E1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002E3      1  FF                                                   TERMINATOR_FF             
0x000002E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002E8      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x000002EA      1  FF                                                   TERMINATOR_FF             
0x000002EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002ED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002EF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002F1      1  FF                                                   TERMINATOR_FF             
0x000002F2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002F4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002F6      1  FF                                                   TERMINATOR_FF             
0x000002F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002FB      2  F209                                                 IMM8_F2                   u8=9, s8=9
0x000002FD      1  FF                                                   TERMINATOR_FF             
0x000002FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000300      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000302      2  F267                                                 IMM8_F2                   u8=103, s8=103
0x00000304      1  FF                                                   TERMINATOR_FF             
0x00000305      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000307      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000309      1  FF                                                   TERMINATOR_FF             
0x0000030A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000030C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000030E      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000310      1  FF                                                   TERMINATOR_FF             
0x00000311      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000313      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000315      3  F30123                                               IMM16_F3                  u16_be=291, u16_le=8961
0x00000318      1  FF                                                   TERMINATOR_FF             
0x00000319      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000031B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000031D      1  FF                                                   TERMINATOR_FF             
0x0000031E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000320      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000322      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000324      1  FF                                                   TERMINATOR_FF             
0x00000325      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000327      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000329      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000032B      1  FF                                                   TERMINATOR_FF             
0x0000032C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000032E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000330      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000332      1  FF                                                   TERMINATOR_FF             
0x00000333      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000335      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000337      1  FF                                                   TERMINATOR_FF             
0x00000338      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000033A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000033C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000033E      1  FF                                                   TERMINATOR_FF             
0x0000033F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000341      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000343      3  F3112C                                               IMM16_F3                  u16_be=4396, u16_le=11281
0x00000346      1  FF                                                   TERMINATOR_FF             
0x00000347      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000349      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000034B     80  804E82A082CC8F758AD4814182DA82AD82CD8A6D82A982C9...  LEN8_STRING_CP932         length=78, text="あの瞬間、ぼくは確かに彼女の手を握った。\nその感触は今でも手の中に残っている。"
0x0000039B      1  FF                                                   TERMINATOR_FF             
0x0000039C      1  FF                                                   TERMINATOR_FF             
0x0000039D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000039F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003A1      1  FF                                                   TERMINATOR_FF             
0x000003A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003A6      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x000003A8      1  FF                                                   TERMINATOR_FF             
0x000003A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003AB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003AD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003AF      1  FF                                                   TERMINATOR_FF             
0x000003B0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003B2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003B4      1  FF                                                   TERMINATOR_FF             
0x000003B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003B9      2  F209                                                 IMM8_F2                   u8=9, s8=9
0x000003BB      1  FF                                                   TERMINATOR_FF             
0x000003BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003BE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003C0      2  F267                                                 IMM8_F2                   u8=103, s8=103
0x000003C2      1  FF                                                   TERMINATOR_FF             
0x000003C3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003C5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003C7      1  FF                                                   TERMINATOR_FF             
0x000003C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003CC      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000003CE      1  FF                                                   TERMINATOR_FF             
0x000003CF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003D1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003D3      2  F227                                                 IMM8_F2                   u8=39, s8=39
0x000003D5      1  FF                                                   TERMINATOR_FF             
0x000003D6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003D8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003DA      1  FF                                                   TERMINATOR_FF             
0x000003DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003DF      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000003E1      1  FF                                                   TERMINATOR_FF             
0x000003E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003E4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003E6      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000003E8      1  FF                                                   TERMINATOR_FF             
0x000003E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003EB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000003ED      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000003EF      1  FF                                                   TERMINATOR_FF             
0x000003F0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003F2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003F4      1  FF                                                   TERMINATOR_FF             
0x000003F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003F9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000003FB      1  FF                                                   TERMINATOR_FF             
0x000003FC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003FE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000400      3  F3112D                                               IMM16_F3                  u16_be=4397, u16_le=11537
0x00000403      1  FF                                                   TERMINATOR_FF             
0x00000404      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000406      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000408     72  804682C582E08163816382B682E182A0814194DE8F9782CD...  LEN8_STRING_CP932         length=70, text="でも……じゃあ、彼女はどこに？\nぼくはまた、彼女を失ってしまったのか？"
0x00000450      1  FF                                                   TERMINATOR_FF             
0x00000451      1  FF                                                   TERMINATOR_FF             
0x00000452      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000454      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000456      1  FF                                                   TERMINATOR_FF             
0x00000457      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000459      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000045B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000045D      1  FF                                                   TERMINATOR_FF             
0x0000045E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000460      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000462      3  F340FB                                               IMM16_F3                  u16_be=16635, u16_le=64320
0x00000465      1  FF                                                   TERMINATOR_FF             
0x00000466      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000468      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000046A     36  80228EE882F082CD82C882B582BD82A882DA82A682CD82C8...  LEN8_STRING_CP932         length=34, text="手をはなしたおぼえはないのに……。"
0x0000048E      1  FF                                                   TERMINATOR_FF             
0x0000048F      1  FF                                                   TERMINATOR_FF             
0x00000490      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000492      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000494      1  FF                                                   TERMINATOR_FF             
0x00000495      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000497      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000499      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000049B      1  FF                                                   TERMINATOR_FF             
0x0000049C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000049E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000004A0      3  F3112E                                               IMM16_F3                  u16_be=4398, u16_le=11793
0x000004A3      1  FF                                                   TERMINATOR_FF             
0x000004A4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004A6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004A8     62  803C88BB82CD82C782B182C982E082A282C882A982C182BD...  LEN8_STRING_CP932         length=60, text="綾はどこにもいなかった。そして、街からも人の姿が消えていた。"
0x000004E6      1  FF                                                   TERMINATOR_FF             
0x000004E7      1  FF                                                   TERMINATOR_FF             
0x000004E8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004EA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004EC      1  FF                                                   TERMINATOR_FF             
0x000004ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004F1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000004F3      1  FF                                                   TERMINATOR_FF             
0x000004F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004F6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000004F8      3  F3112F                                               IMM16_F3                  u16_be=4399, u16_le=12049
0x000004FB      1  FF                                                   TERMINATOR_FF             
0x000004FC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004FE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000500    110  806C82B182B182CD8163816382DA82AD82CC8A5882CC82CD...  LEN8_STRING_CP932         length=108, text="ここは……ぼくの街のはずなのに。\nここも、綾の街と同様に無人の街、動くものの\nない凍りついた街になっていた。"
0x0000056E      1  FF                                                   TERMINATOR_FF             
0x0000056F      1  FF                                                   TERMINATOR_FF             
0x00000570      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000572      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000574      1  FF                                                   TERMINATOR_FF             
0x00000575      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000577      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00000579      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000057B      1  FF                                                   TERMINATOR_FF             
0x0000057C      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000057E      3  F1007D                                               IMM16_F1                  u16_be=125, u16_le=32000
0x00000581      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000583      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000584      1  FF                                                   TERMINATOR_FF             
0x00000585      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00000587      1  96                                                   OPAQUE_RAW_BYTES          bytes=96
0x00000588      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000058A      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x0000058C      3  F10009                                               IMM16_F1                  u16_be=9, u16_le=2304
0x0000058F      1  F0                                                   OPAQUE_RAW_BYTES          bytes=F0
0x00000590      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000592      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000594      1  43                                                   OPAQUE_RAW_BYTES          bytes=43
0x00000595      1  FF                                                   TERMINATOR_FF             
0x00000596      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000598      3  F1007E                                               IMM16_F1                  u16_be=126, u16_le=32256
0x0000059B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000059D      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x0000059E      1  FF                                                   TERMINATOR_FF             
0x0000059F      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x000005A1      1  B0                                                   OPAQUE_RAW_BYTES          bytes=B0
0x000005A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005A4      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x000005A6      3  F10009                                               IMM16_F1                  u16_be=9, u16_le=2304
0x000005A9      1  F0                                                   OPAQUE_RAW_BYTES          bytes=F0
0x000005AA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005AC      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000005AE      1  43                                                   OPAQUE_RAW_BYTES          bytes=43
0x000005AF      1  FF                                                   TERMINATOR_FF             
0x000005B0      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000005B2      3  F1007D                                               IMM16_F1                  u16_be=125, u16_le=32000
0x000005B5      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000005B7      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000005B8      3  F1007E                                               IMM16_F1                  u16_be=126, u16_le=32256
0x000005BB      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000005BD      2  3551                                                 OPAQUE_RAW_BYTES          bytes=3551
0x000005BF      1  FF                                                   TERMINATOR_FF             
0x000005C0      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x000005C2      1  C8                                                   OPAQUE_RAW_BYTES          bytes=C8
0x000005C3      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x000005C5      2  000F                                                 WORD_00XX                 u16_be=15, low_byte=15
0x000005C7      1  78                                                   OPAQUE_RAW_BYTES          bytes=78
0x000005C8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005CA      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x000005CC     10  80088BCA82CC89C682D6                                 LEN8_STRING_CP932         length=8, text="玉の家へ"
0x000005D6      1  FF                                                   TERMINATOR_FF             
0x000005D7      1  FF                                                   TERMINATOR_FF             
0x000005D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005DA      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x000005DC      3  F31130                                               IMM16_F3                  u16_be=4400, u16_le=12305
0x000005DF      1  FF                                                   TERMINATOR_FF             
0x000005E0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005E2      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x000005E4      6  80048A5882D6                                         LEN8_STRING_CP932         length=4, text="街へ"
0x000005EA      1  FF                                                   TERMINATOR_FF             
0x000005EB      1  FF                                                   TERMINATOR_FF             
0x000005EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005EE      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x000005F0      3  F31131                                               IMM16_F3                  u16_be=4401, u16_le=12561
0x000005F3      1  FF                                                   TERMINATOR_FF             
0x000005F4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005F6      2  0006                                                 WORD_00XX                 u16_be=6, low_byte=6
0x000005F8      8  80068A778D5A82D6                                     LEN8_STRING_CP932         length=6, text="学校へ"
0x00000600      1  FF                                                   TERMINATOR_FF             
0x00000601      1  FF                                                   TERMINATOR_FF             
0x00000602      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000604      2  000C                                                 WORD_00XX                 u16_be=12, low_byte=12
0x00000606      3  F31132                                               IMM16_F3                  u16_be=4402, u16_le=12817
0x00000609      1  FF                                                   TERMINATOR_FF             
0x0000060A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000060C      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x0000060E      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000610      1  FF                                                   TERMINATOR_FF             
0x00000611      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000613      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000615      2  F211                                                 IMM8_F2                   u8=17, s8=17
0x00000617      1  FF                                                   TERMINATOR_FF             
0x00000618      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000061A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000061C      1  FF                                                   TERMINATOR_FF             
0x0000061D      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000061F      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00000622      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000624      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000625      1  FF                                                   TERMINATOR_FF             
0x00000626      2  000C                                                 WORD_00XX                 u16_be=12, low_byte=12
0x00000628      1  79                                                   OPAQUE_RAW_BYTES          bytes=79
0x00000629      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000062B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000062D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000062F      1  FF                                                   TERMINATOR_FF             
0x00000630      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000632      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000634      3  F31133                                               IMM16_F3                  u16_be=4403, u16_le=13073
0x00000637      1  FF                                                   TERMINATOR_FF             
0x00000638      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000063A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000063C     36  80228BCA82CD81485C6E82A082A282C2814182C782A482B5...  LEN8_STRING_CP932         length=34, text="玉は？\nあいつ、どうしたんだろう？"
0x00000660      1  FF                                                   TERMINATOR_FF             
0x00000661      1  FF                                                   TERMINATOR_FF             
0x00000662      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000664      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000666      1  FF                                                   TERMINATOR_FF             
0x00000667      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000669      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000066B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000066D      1  FF                                                   TERMINATOR_FF             
0x0000066E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000670      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000672      3  F31134                                               IMM16_F3                  u16_be=4404, u16_le=13329
0x00000675      1  FF                                                   TERMINATOR_FF             
0x00000676      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000678      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000067A    102  806482C882C982E082ED82A982C182C482A282C882A282A0...  LEN8_STRING_CP932         length=100, text="なにもわかっていないあいつも、このできごとにまきこまれたのか？\nそれは……あまりにかわいそうすぎる。"
0x000006E0      1  FF                                                   TERMINATOR_FF             
0x000006E1      1  FF                                                   TERMINATOR_FF             
0x000006E2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006E4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006E6      1  FF                                                   TERMINATOR_FF             
0x000006E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006EB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000006ED      1  FF                                                   TERMINATOR_FF             
0x000006EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006F0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006F2      3  F31135                                               IMM16_F3                  u16_be=4405, u16_le=13585
0x000006F5      1  FF                                                   TERMINATOR_FF             
0x000006F6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006F8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006FA     40  802682DA82AD82CD8BCA82CC89C682C98CFC82A982C182C4...  LEN8_STRING_CP932         length=38, text="ぼくは玉の家に向かって駆けだしていた。"
0x00000722      1  FF                                                   TERMINATOR_FF             
0x00000723      1  FF                                                   TERMINATOR_FF             
0x00000724      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000726      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000728      1  FF                                                   TERMINATOR_FF             
0x00000729      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000072B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000072D      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x0000072F      1  FF                                                   TERMINATOR_FF             
0x00000730      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000732      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000734      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000736      1  FF                                                   TERMINATOR_FF             
0x00000737      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000739      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000073B      1  FF                                                   TERMINATOR_FF             
0x0000073C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000073E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000740      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000742      1  FF                                                   TERMINATOR_FF             
0x00000743      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000745      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000747      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00000749      1  FF                                                   TERMINATOR_FF             
0x0000074A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000074C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000074E      1  FF                                                   TERMINATOR_FF             
0x0000074F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000751      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000753      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000755      1  FF                                                   TERMINATOR_FF             
0x00000756      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000758      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000075A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000075C      1  FF                                                   TERMINATOR_FF             
0x0000075D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000075F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000761      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000763      1  FF                                                   TERMINATOR_FF             
0x00000764      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000766      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000768      1  FF                                                   TERMINATOR_FF             
0x00000769      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000076B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000076D      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000076F      1  FF                                                   TERMINATOR_FF             
0x00000770      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000772      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000774      2  F25B                                                 IMM8_F2                   u8=91, s8=91
0x00000776      1  FF                                                   TERMINATOR_FF             
0x00000777      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000779      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000077B      1  FF                                                   TERMINATOR_FF             
0x0000077C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000077E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000780      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000782      1  FF                                                   TERMINATOR_FF             
0x00000783      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000785      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000787      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000789      1  FF                                                   TERMINATOR_FF             
0x0000078A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000078C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000078E      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000790      1  FF                                                   TERMINATOR_FF             
0x00000791      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000793      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000795      1  FF                                                   TERMINATOR_FF             
0x00000796      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000798      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000079A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000079C      1  FF                                                   TERMINATOR_FF             
0x0000079D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000079F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000007A1      3  F31136                                               IMM16_F3                  u16_be=4406, u16_le=13841
0x000007A4      1  FF                                                   TERMINATOR_FF             
0x000007A5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007A7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007A9    106  80688BCA82CC89C682C982CD8141924E82E082A282C882A9...  LEN8_STRING_CP932         length=104, text="玉の家には、誰もいなかった。\nほんの少し前まで人がいたのに、みんなで\nどこかへ行ってしまったようだった。"
0x00000813      1  FF                                                   TERMINATOR_FF             
0x00000814      1  FF                                                   TERMINATOR_FF             
0x00000815      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000817      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000819      1  FF                                                   TERMINATOR_FF             
0x0000081A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000081C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000081E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000820      1  FF                                                   TERMINATOR_FF             
0x00000821      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000823      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000825      3  F31137                                               IMM16_F3                  u16_be=4407, u16_le=14097
0x00000828      1  FF                                                   TERMINATOR_FF             
0x00000829      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000082B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000082D     92  805A8BCA82E0816381638FC182A682C482B582DC82C182BD...  LEN8_STRING_CP932         length=90, text="玉も……消えてしまった。\n玉はもう、どこにもいない。\n玉も、玉の家族も……消えてしまった。"
0x00000889      1  FF                                                   TERMINATOR_FF             
0x0000088A      1  FF                                                   TERMINATOR_FF             
0x0000088B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000088D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000088F      1  FF                                                   TERMINATOR_FF             
0x00000890      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000892      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000894      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000896      1  FF                                                   TERMINATOR_FF             
0x00000897      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000899      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000089B      3  F31138                                               IMM16_F3                  u16_be=4408, u16_le=14353
0x0000089E      1  FF                                                   TERMINATOR_FF             
0x0000089F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008A1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008A3     74  80488BCA81425C6E82DA82AD82CD81418BCA82CC8FCE8AE7...  LEN8_STRING_CP932         length=72, text="玉。\nぼくは、玉の笑顔を思い出す。\n幼稚園のときからずっと一緒だった玉。"
0x000008ED      1  FF                                                   TERMINATOR_FF             
0x000008EE      1  FF                                                   TERMINATOR_FF             
0x000008EF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008F1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008F3      1  FF                                                   TERMINATOR_FF             
0x000008F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008F8      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000008FA      1  FF                                                   TERMINATOR_FF             
0x000008FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008FD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000008FF      3  F31139                                               IMM16_F3                  u16_be=4409, u16_le=14609
0x00000902      1  FF                                                   TERMINATOR_FF             
0x00000903      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000905      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000907     70  804482A282C282E08CB38B4382C5814182C882C982F082B5...  LEN8_STRING_CP932         length=68, text="いつも元気で、なにをしでかすかわからない玉。彼女は、消えてしまった。"
0x0000094D      1  FF                                                   TERMINATOR_FF             
0x0000094E      1  FF                                                   TERMINATOR_FF             
0x0000094F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000951      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000953      1  FF                                                   TERMINATOR_FF             
0x00000954      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000956      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000958      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000095A      1  FF                                                   TERMINATOR_FF             
0x0000095B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000095D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000095F      3  F3113A                                               IMM16_F3                  u16_be=4410, u16_le=14865
0x00000962      1  FF                                                   TERMINATOR_FF             
0x00000963      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000965      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000967     46  802C8EA995AA82AA8FC182A682C482B582DC82C182BD82B1...  LEN8_STRING_CP932         length=44, text="自分が消えてしまったことさえわからずに……。"
0x00000995      1  FF                                                   TERMINATOR_FF             
0x00000996      1  FF                                                   TERMINATOR_FF             
0x00000997      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000999      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000099B      1  FF                                                   TERMINATOR_FF             
0x0000099C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000099E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009A0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000009A2      1  FF                                                   TERMINATOR_FF             
0x000009A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009A5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009A7      3  F3113B                                               IMM16_F3                  u16_be=4411, u16_le=15121
0x000009AA      1  FF                                                   TERMINATOR_FF             
0x000009AB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009AD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009AF     46  802C82C782A482B582C48163816381425C6E82B182F182C8...  LEN8_STRING_CP932         length=44, text="どうして……。\nこんなことになったんだろう。"
0x000009DD      1  FF                                                   TERMINATOR_FF             
0x000009DE      1  FF                                                   TERMINATOR_FF             
0x000009DF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009E1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009E3      1  FF                                                   TERMINATOR_FF             
0x000009E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E8      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000009EA      1  FF                                                   TERMINATOR_FF             
0x000009EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009ED      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009EF      3  F3113C                                               IMM16_F3                  u16_be=4412, u16_le=15377
0x000009F2      1  FF                                                   TERMINATOR_FF             
0x000009F3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009F5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009F7     90  80588BCA82CD814182C882C982E088AB82A282B182C682C8...  LEN8_STRING_CP932         length=88, text="玉は、なにも悪いことなんかしていないのに。\nあいつはただ、思いっきり生きてただけなのに。"
0x00000A51      1  FF                                                   TERMINATOR_FF             
0x00000A52      1  FF                                                   TERMINATOR_FF             
0x00000A53      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A55      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A57      1  FF                                                   TERMINATOR_FF             
0x00000A58      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A5A      2  007D                                                 WORD_00XX                 u16_be=125, low_byte=125
0x00000A5C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000A5E      1  FF                                                   TERMINATOR_FF             
0x00000A5F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A61      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A63      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000A65      1  FF                                                   TERMINATOR_FF             
0x00000A66      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A68      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A6A      3  F3113D                                               IMM16_F3                  u16_be=4413, u16_le=15633
0x00000A6D      1  FF                                                   TERMINATOR_FF             
0x00000A6E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A70      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A72     48  802E82C782A482B582C4816381635C6E8BCA82AA8FC182A6...  LEN8_STRING_CP932         length=46, text="どうして……\n玉が消えなきゃいけないんだろう。"
0x00000AA2      1  FF                                                   TERMINATOR_FF             
0x00000AA3      1  FF                                                   TERMINATOR_FF             
0x00000AA4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000AA6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AA8      1  FF                                                   TERMINATOR_FF             
0x00000AA9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AAB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AAD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000AAF      1  FF                                                   TERMINATOR_FF             
0x00000AB0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AB2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000AB4      3  F3113E                                               IMM16_F3                  u16_be=4414, u16_le=15889
0x00000AB7      1  FF                                                   TERMINATOR_FF             
0x00000AB8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000ABA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000ABC     62  803C82DA82AD82CD814196B3906C82C682C882C182BD8BCA...  LEN8_STRING_CP932         length=60, text="ぼくは、無人となった玉の家の前で、ずっと\n立ちつくしていた。"
0x00000AFA      1  FF                                                   TERMINATOR_FF             
0x00000AFB      1  FF                                                   TERMINATOR_FF             
0x00000AFC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000AFE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B00      1  FF                                                   TERMINATOR_FF             
0x00000B01      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B03      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B05      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B07      1  FF                                                   TERMINATOR_FF             
0x00000B08      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B0A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B0C      3  F340FC                                               IMM16_F3                  u16_be=16636, u16_le=64576
0x00000B0F      1  FF                                                   TERMINATOR_FF             
0x00000B10      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B12      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B14     52  803290A28A4582AA96C582D182E98163816382C682A282A4...  LEN8_STRING_CP932         length=50, text="世界が滅びる……というのは、こういうこと\nなのか。"
0x00000B48      1  FF                                                   TERMINATOR_FF             
0x00000B49      1  FF                                                   TERMINATOR_FF             
0x00000B4A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B4C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B4E      1  FF                                                   TERMINATOR_FF             
0x00000B4F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B51      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B53      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B55      1  FF                                                   TERMINATOR_FF             
0x00000B56      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B58      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B5A      3  F340FD                                               IMM16_F3                  u16_be=16637, u16_le=64832
0x00000B5D      1  FF                                                   TERMINATOR_FF             
0x00000B5E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B60      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B62     90  805882C882F182CC8DDF82E082C882A2906C82BD82BF82AA...  LEN8_STRING_CP932         length=88, text="なんの罪もない人たちが、その存在を消されて\nしまう。\nこれが……破滅なのだとしたら……。"
0x00000BBC      1  FF                                                   TERMINATOR_FF             
0x00000BBD      1  FF                                                   TERMINATOR_FF             
0x00000BBE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000BC0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000BC2      1  FF                                                   TERMINATOR_FF             
0x00000BC3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BC5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BC7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000BC9      1  FF                                                   TERMINATOR_FF             
0x00000BCA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BCC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000BCE      3  F340FE                                               IMM16_F3                  u16_be=16638, u16_le=65088
0x00000BD1      1  FF                                                   TERMINATOR_FF             
0x00000BD2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BD4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BD6     88  805695A88EBF82AA8E6382C182C482E08141906C82AA82A2...  LEN8_STRING_CP932         length=86, text="物質が残っても、人がいなくなったなら……。\nそれは、ぼくたちにとっての……終焉なのか。"
0x00000C2E      1  FF                                                   TERMINATOR_FF             
0x00000C2F      1  FF                                                   TERMINATOR_FF             
0x00000C30      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C32      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C34      1  FF                                                   TERMINATOR_FF             
0x00000C35      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C37      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C39      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000C3B      1  FF                                                   TERMINATOR_FF             
0x00000C3C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C3E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C40      3  F340FF                                               IMM16_F3                  u16_be=16639, u16_le=65344
0x00000C43      1  FF                                                   TERMINATOR_FF             
0x00000C44      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C46      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C48     32  801E82B182EA82AA8F4982ED82E882BE82C182BD82C882F1...  LEN8_STRING_CP932         length=30, text="これが終わりだったなんて……。"
0x00000C68      1  FF                                                   TERMINATOR_FF             
0x00000C69      1  FF                                                   TERMINATOR_FF             
0x00000C6A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C6C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C6E      1  FF                                                   TERMINATOR_FF             
0x00000C6F      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000C71      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00000C73      1  75                                                   OPAQUE_RAW_BYTES          bytes=75
0x00000C74      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000C76      2  000F                                                 WORD_00XX                 u16_be=15, low_byte=15
0x00000C78      1  78                                                   OPAQUE_RAW_BYTES          bytes=78
0x00000C79      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000C7B      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00000C7E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000C80      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000C81      1  FF                                                   TERMINATOR_FF             
0x00000C82      2  000F                                                 WORD_00XX                 u16_be=15, low_byte=15
0x00000C84      1  65                                                   OPAQUE_RAW_BYTES          bytes=65
0x00000C85      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C87      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C89      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000C8B      1  FF                                                   TERMINATOR_FF             
0x00000C8C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C8E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C90      2  F25D                                                 IMM8_F2                   u8=93, s8=93
0x00000C92      1  FF                                                   TERMINATOR_FF             
0x00000C93      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C95      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C97      1  FF                                                   TERMINATOR_FF             
0x00000C98      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C9A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C9C      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000C9E      1  FF                                                   TERMINATOR_FF             
0x00000C9F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CA1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CA3      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000CA5      1  FF                                                   TERMINATOR_FF             
0x00000CA6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CA8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000CAA      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000CAC      1  FF                                                   TERMINATOR_FF             
0x00000CAD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CAF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CB1      1  FF                                                   TERMINATOR_FF             
0x00000CB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CB4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CB6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000CB8      1  FF                                                   TERMINATOR_FF             
0x00000CB9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CBB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000CBD      3  F3113F                                               IMM16_F3                  u16_be=4415, u16_le=16145
0x00000CC0      1  FF                                                   TERMINATOR_FF             
0x00000CC1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CC3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CC5    120  80768A5882C982CD8163816393AE82AD82E082CC82D082C6...  LEN8_STRING_CP932         length=118, text="街には……動くものひとつなかった。\n昼下がりの大通りは、がらんとしていた。\nたくさんの人たちがどこかへ行ってしまった。"
0x00000D3D      1  FF                                                   TERMINATOR_FF             
0x00000D3E      1  FF                                                   TERMINATOR_FF             
0x00000D3F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D41      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D43      1  FF                                                   TERMINATOR_FF             
0x00000D44      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D46      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D48      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000D4A      1  FF                                                   TERMINATOR_FF             
0x00000D4B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D4D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D4F      3  F31140                                               IMM16_F3                  u16_be=4416, u16_le=16401
0x00000D52      1  FF                                                   TERMINATOR_FF             
0x00000D53      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D55      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D57     74  804882DA82AD82CD82D082C682E882AB82E882BE82C182BD...  LEN8_STRING_CP932         length=72, text="ぼくはひとりきりだった。\nどこへ行っても、ぼくは空虚な街に包まれていた。"
0x00000DA1      1  FF                                                   TERMINATOR_FF             
0x00000DA2      1  FF                                                   TERMINATOR_FF             
0x00000DA3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000DA5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000DA7      1  FF                                                   TERMINATOR_FF             
0x00000DA8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DAA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DAC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000DAE      1  FF                                                   TERMINATOR_FF             
0x00000DAF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DB1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000DB3      3  F31141                                               IMM16_F3                  u16_be=4417, u16_le=16657
0x00000DB6      1  FF                                                   TERMINATOR_FF             
0x00000DB7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DB9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DBB     80  804E82A082CC8C8E96BE82A982E882CC8A5882C682E088E1...  LEN8_STRING_CP932         length=78, text="あの月明かりの街とも違っていた。\nあの街では、影は重く、空気ははりつめていた。"
0x00000E0B      1  FF                                                   TERMINATOR_FF             
0x00000E0C      1  FF                                                   TERMINATOR_FF             
0x00000E0D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E0F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E11      1  FF                                                   TERMINATOR_FF             
0x00000E12      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E14      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E16      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000E18      1  FF                                                   TERMINATOR_FF             
0x00000E19      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E1B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E1D      3  F31142                                               IMM16_F3                  u16_be=4418, u16_le=16913
0x00000E20      1  FF                                                   TERMINATOR_FF             
0x00000E21      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E23      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E25     52  803282C582E0814182B182B182C982A082E982CC82CD8163...  LEN8_STRING_CP932         length=50, text="でも、ここにあるのは……本当に空っぽな街\nだった。"
0x00000E59      1  FF                                                   TERMINATOR_FF             
0x00000E5A      1  FF                                                   TERMINATOR_FF             
0x00000E5B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E5D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E5F      1  FF                                                   TERMINATOR_FF             
0x00000E60      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E62      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E64      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000E66      1  FF                                                   TERMINATOR_FF             
0x00000E67      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E69      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E6B      3  F31143                                               IMM16_F3                  u16_be=4419, u16_le=17169
0x00000E6E      1  FF                                                   TERMINATOR_FF             
0x00000E6F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E71      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E73     40  802682BB82A482A98163816388BB82CD814182B182EA82C9...  LEN8_STRING_CP932         length=38, text="そうか……綾は、これに耐えていたのか。"
0x00000E9B      1  FF                                                   TERMINATOR_FF             
0x00000E9C      1  FF                                                   TERMINATOR_FF             
0x00000E9D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E9F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000EA1      1  FF                                                   TERMINATOR_FF             
0x00000EA2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EA4      2  007E                                                 WORD_00XX                 u16_be=126, low_byte=126
0x00000EA6      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000EA8      1  FF                                                   TERMINATOR_FF             
0x00000EA9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EAB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EAD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000EAF      1  FF                                                   TERMINATOR_FF             
0x00000EB0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EB2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000EB4      3  F31144                                               IMM16_F3                  u16_be=4420, u16_le=17425
0x00000EB7      1  FF                                                   TERMINATOR_FF             
0x00000EB8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000EBA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000EBC     72  80468EA995AA82B582A982C882A28FEA8F8A81425C6E90BA...  LEN8_STRING_CP932         length=70, text="自分しかない場所。\n声をあげても、走り出しても、なにも変わらない場所。"
0x00000F04      1  FF                                                   TERMINATOR_FF             
0x00000F05      1  FF                                                   TERMINATOR_FF             
0x00000F06      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F08      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F0A      1  FF                                                   TERMINATOR_FF             
0x00000F0B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F0D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F0F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000F11      1  FF                                                   TERMINATOR_FF             
0x00000F12      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F14      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F16      3  F31145                                               IMM16_F3                  u16_be=4421, u16_le=17681
0x00000F19      1  FF                                                   TERMINATOR_FF             
0x00000F1A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F1C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F1E     54  803482DA82AD82CD82CD82B682DF82C4814188BB82CC94DF...  LEN8_STRING_CP932         length=52, text="ぼくははじめて、綾の悲しさを理解することが\nできた。"
0x00000F54      1  FF                                                   TERMINATOR_FF             
0x00000F55      1  FF                                                   TERMINATOR_FF             
0x00000F56      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F58      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F5A      1  FF                                                   TERMINATOR_FF             
0x00000F5B      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000F5D      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00000F5F      1  75                                                   OPAQUE_RAW_BYTES          bytes=75
0x00000F60      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000F62      2  000F                                                 WORD_00XX                 u16_be=15, low_byte=15
0x00000F64      1  78                                                   OPAQUE_RAW_BYTES          bytes=78
0x00000F65      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000F67      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00000F6A      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000F6C      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000F6D      1  FF                                                   TERMINATOR_FF             
0x00000F6E      2  000F                                                 WORD_00XX                 u16_be=15, low_byte=15
0x00000F70      1  78                                                   OPAQUE_RAW_BYTES          bytes=78
0x00000F71      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F73      2  007F                                                 WORD_00XX                 u16_be=127, low_byte=127
0x00000F75      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000F77      1  FF                                                   TERMINATOR_FF             
0x00000F78      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F7A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F7C      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000F7E      1  FF                                                   TERMINATOR_FF             
0x00000F7F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F81      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F83      3  F30088                                               IMM16_F3                  u16_be=136, u16_le=34816
0x00000F86      1  FF                                                   TERMINATOR_FF             
0x00000F87      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F89      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F8B      1  FF                                                   TERMINATOR_FF             
0x00000F8C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F8E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F90      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000F92      1  FF                                                   TERMINATOR_FF             
0x00000F93      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F95      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F97      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000F99      1  FF                                                   TERMINATOR_FF             
0x00000F9A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F9C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000F9E      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000FA0      1  FF                                                   TERMINATOR_FF             
0x00000FA1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FA3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FA5      1  FF                                                   TERMINATOR_FF             
0x00000FA6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FA8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FAA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000FAC      1  FF                                                   TERMINATOR_FF             
0x00000FAD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FAF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000FB1      3  F31146                                               IMM16_F3                  u16_be=4422, u16_le=17937
0x00000FB4      1  FF                                                   TERMINATOR_FF             
0x00000FB5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FB7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FB9     70  804482DA82AD82CD8A778D5A82D68CFC82A982C182BD8142...  LEN8_STRING_CP932         length=68, text="ぼくは学校へ向かった。\nなにかを説明できるのは、カウンセラーだけだ。"
0x00000FFF      1  FF                                                   TERMINATOR_FF             
0x00001000      1  FF                                                   TERMINATOR_FF             
0x00001001      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001003      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001005      1  FF                                                   TERMINATOR_FF             
0x00001006      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001008      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000100A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000100C      1  FF                                                   TERMINATOR_FF             
0x0000100D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000100F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001011      3  F31147                                               IMM16_F3                  u16_be=4423, u16_le=18193
0x00001014      1  FF                                                   TERMINATOR_FF             
0x00001015      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001017      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001019     86  8054834A83458393835A8389815B82C882E781418DA182C8...  LEN8_STRING_CP932         length=84, text="カウンセラーなら、今なにがおきているのか。\nみんなはどこへ行ったのか。\nそして……。"
0x0000106F      1  FF                                                   TERMINATOR_FF             
0x00001070      1  FF                                                   TERMINATOR_FF             
0x00001071      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001073      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001075      1  FF                                                   TERMINATOR_FF             
0x00001076      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001078      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000107A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000107C      1  FF                                                   TERMINATOR_FF             
0x0000107D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000107F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001081      3  F31148                                               IMM16_F3                  u16_be=4424, u16_le=18449
0x00001084      1  FF                                                   TERMINATOR_FF             
0x00001085      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001087      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001089     26  80188163816388BB82AA82C782B182C982A282E982CC82A9...  LEN8_STRING_CP932         length=24, text="……綾がどこにいるのか。"
0x000010A3      1  FF                                                   TERMINATOR_FF             
0x000010A4      1  FF                                                   TERMINATOR_FF             
0x000010A5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010A7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010A9      1  FF                                                   TERMINATOR_FF             
0x000010AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010AE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000010B0      1  FF                                                   TERMINATOR_FF             
0x000010B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010B3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000010B5      3  F31149                                               IMM16_F3                  u16_be=4425, u16_le=18705
0x000010B8      1  FF                                                   TERMINATOR_FF             
0x000010B9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010BB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010BD     32  801E82C882C982A982F0926D82C182C482A282E982CD82B8...  LEN8_STRING_CP932         length=30, text="なにかを知っているはずだった。"
0x000010DD      1  FF                                                   TERMINATOR_FF             
0x000010DE      1  FF                                                   TERMINATOR_FF             
0x000010DF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010E1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010E3      1  FF                                                   TERMINATOR_FF             
0x000010E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010E8      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000010EA      1  FF                                                   TERMINATOR_FF             
0x000010EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010ED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010EF      2  F274                                                 IMM8_F2                   u8=116, s8=116
0x000010F1      1  FF                                                   TERMINATOR_FF             
0x000010F2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010F4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010F6      1  FF                                                   TERMINATOR_FF             
0x000010F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010FB      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000010FD      1  FF                                                   TERMINATOR_FF             
0x000010FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001100      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001102      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001104      1  FF                                                   TERMINATOR_FF             
0x00001105      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001107      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001109      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000110B      1  FF                                                   TERMINATOR_FF             
0x0000110C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000110E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001110      1  FF                                                   TERMINATOR_FF             
0x00001111      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001113      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001115      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001117      1  FF                                                   TERMINATOR_FF             
0x00001118      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000111A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000111C      3  F3114A                                               IMM16_F3                  u16_be=4426, u16_le=18961
0x0000111F      1  FF                                                   TERMINATOR_FF             
0x00001120      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001122      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001124     82  80508A778D5A82C982E0906C82CC8B43947A82CD82C882A9...  LEN8_STRING_CP932         length=80, text="学校にも人の気配はなかった。\nもちろん、ここへ来るまでに、誰の姿も\n見なかった。"
0x00001176      1  FF                                                   TERMINATOR_FF             
0x00001177      1  FF                                                   TERMINATOR_FF             
0x00001178      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000117A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000117C      1  FF                                                   TERMINATOR_FF             
0x0000117D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000117F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001181      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001183      1  FF                                                   TERMINATOR_FF             
0x00001184      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001186      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001188      3  F30284                                               IMM16_F3                  u16_be=644, u16_le=33794
0x0000118B      1  FF                                                   TERMINATOR_FF             
0x0000118C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000118E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001190      1  FF                                                   TERMINATOR_FF             
0x00001191      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001193      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001195      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001197      1  FF                                                   TERMINATOR_FF             
0x00001198      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000119A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000119C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000119E      1  FF                                                   TERMINATOR_FF             
0x0000119F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011A1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000011A3      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000011A5      1  FF                                                   TERMINATOR_FF             
0x000011A6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000011A8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000011AA      1  FF                                                   TERMINATOR_FF             
0x000011AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011AF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000011B1      1  FF                                                   TERMINATOR_FF             
0x000011B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011B4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000011B6      3  F3114B                                               IMM16_F3                  u16_be=4427, u16_le=19217
0x000011B9      1  FF                                                   TERMINATOR_FF             
0x000011BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011BC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011BE    116  8072834A83458393835A838A8393834F8EBA82CD96B3906C...  LEN8_STRING_CP932         length=114, text="カウンセリング室は無人だった。\n机のまわりに、雑多な本が読み捨てられている。ジャンルや内容に関係性のない、本の山。"
0x00001232      1  FF                                                   TERMINATOR_FF             
0x00001233      1  FF                                                   TERMINATOR_FF             
0x00001234      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001236      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001238      1  FF                                                   TERMINATOR_FF             
0x00001239      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000123B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000123D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000123F      1  FF                                                   TERMINATOR_FF             
0x00001240      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001242      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001244      3  F3114C                                               IMM16_F3                  u16_be=4428, u16_le=19473
0x00001247      1  FF                                                   TERMINATOR_FF             
0x00001248      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000124A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000124C     88  8056975C917A82C688E182C182C481638163834A83458393...  LEN8_STRING_CP932         length=86, text="予想と違って……カウンセラーはいなかった。\nじゃあ、ぼくは……どうすればいいのだろう。"
0x000012A4      1  FF                                                   TERMINATOR_FF             
0x000012A5      1  FF                                                   TERMINATOR_FF             
0x000012A6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012A8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012AA      1  FF                                                   TERMINATOR_FF             
0x000012AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012AF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000012B1      1  FF                                                   TERMINATOR_FF             
0x000012B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012B4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000012B6      3  F34100                                               IMM16_F3                  u16_be=16640, u16_le=65
0x000012B9      1  FF                                                   TERMINATOR_FF             
0x000012BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012BC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012BE     72  804682DA82AD82CD88BB82F08EB882A281418EE882AA82A9...  LEN8_STRING_CP932         length=70, text="ぼくは綾を失い、手がかりをなくした。\n……できることは……なにがある？"
0x00001306      1  FF                                                   TERMINATOR_FF             
0x00001307      1  FF                                                   TERMINATOR_FF             
0x00001308      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000130A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000130C      1  FF                                                   TERMINATOR_FF             
0x0000130D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000130F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001311      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001313      1  FF                                                   TERMINATOR_FF             
0x00001314      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001316      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001318      2  F24D                                                 IMM8_F2                   u8=77, s8=77
0x0000131A      1  FF                                                   TERMINATOR_FF             
0x0000131B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000131D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000131F      1  FF                                                   TERMINATOR_FF             
0x00001320      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001322      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001324      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001326      1  FF                                                   TERMINATOR_FF             
0x00001327      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001329      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000132B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000132D      1  FF                                                   TERMINATOR_FF             
0x0000132E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001330      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001332      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001334      1  FF                                                   TERMINATOR_FF             
0x00001335      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001337      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001339      1  FF                                                   TERMINATOR_FF             
0x0000133A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000133C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000133E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001340      1  FF                                                   TERMINATOR_FF             
0x00001341      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001343      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001345      3  F3114D                                               IMM16_F3                  u16_be=4429, u16_le=19729
0x00001348      1  FF                                                   TERMINATOR_FF             
0x00001349      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000134B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000134D     94  805C82DA82AD82CD81638163926D82E782B882C98BB38EBA...  LEN8_STRING_CP932         length=92, text="ぼくは……知らずに教室へ来ていた。\nいつもなら、みんなが思い思いの時間を\nすごしている教室。"
0x000013AB      1  FF                                                   TERMINATOR_FF             
0x000013AC      1  FF                                                   TERMINATOR_FF             
0x000013AD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000013AF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000013B1      1  FF                                                   TERMINATOR_FF             
0x000013B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013B4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013B6      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000013B8      1  FF                                                   TERMINATOR_FF             
0x000013B9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013BB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000013BD      3  F3114E                                               IMM16_F3                  u16_be=4430, u16_le=19985
0x000013C0      1  FF                                                   TERMINATOR_FF             
0x000013C1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013C3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000013C5     10  80084D54313436393554                                 LEN8_STRING_CP932         length=8, text="MT14695T"
0x000013CF      1  FF                                                   TERMINATOR_FF             
0x000013D0      1  FF                                                   TERMINATOR_FF             
0x000013D1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013D3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013D5     24  8016817582C8816082C982B5815B82C482E9815B81488176     LEN8_STRING_CP932         length=22, text="「な～にしーてるー？」"
0x000013ED      1  FF                                                   TERMINATOR_FF             
0x000013EE      1  FF                                                   TERMINATOR_FF             
0x000013EF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000013F1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000013F3      1  FF                                                   TERMINATOR_FF             
0x000013F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013F8      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000013FA      1  FF                                                   TERMINATOR_FF             
0x000013FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013FD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000013FF      3  F3114F                                               IMM16_F3                  u16_be=4431, u16_le=20241
0x00001402      1  FF                                                   TERMINATOR_FF             
0x00001403      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001405      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001407     34  802082C582E0816381638BCA82CC90BA82E082E082A495B7...  LEN8_STRING_CP932         length=32, text="でも……玉の声ももう聞こえない。"
0x00001429      1  FF                                                   TERMINATOR_FF             
0x0000142A      1  FF                                                   TERMINATOR_FF             
0x0000142B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000142D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000142F      1  FF                                                   TERMINATOR_FF             
0x00001430      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001432      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001434      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001436      1  FF                                                   TERMINATOR_FF             
0x00001437      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001439      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000143B      3  F31150                                               IMM16_F3                  u16_be=4432, u16_le=20497
0x0000143E      1  FF                                                   TERMINATOR_FF             
0x0000143F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001441      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001443     58  803882DD82F182C882CD8FC182A682C482B582DC82C182BD...  LEN8_STRING_CP932         length=56, text="みんなは消えてしまった。\nぼくは……ひとりっきりだった。"
0x0000147D      1  FF                                                   TERMINATOR_FF             
0x0000147E      1  FF                                                   TERMINATOR_FF             
0x0000147F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001481      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00001483      6  80048A5882D6                                         LEN8_STRING_CP932         length=4, text="街へ"
0x00001489      1  FF                                                   TERMINATOR_FF             
0x0000148A      1  FF                                                   TERMINATOR_FF             
0x0000148B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000148D      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x0000148F      3  F31151                                               IMM16_F3                  u16_be=4433, u16_le=20753
0x00001492      1  FF                                                   TERMINATOR_FF             
0x00001493      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001495      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00001497     20  8012834A83458393835A838A8393834F8EBA82D6             LEN8_STRING_CP932         length=18, text="カウンセリング室へ"
0x000014AB      1  FF                                                   TERMINATOR_FF             
0x000014AC      1  FF                                                   TERMINATOR_FF             
0x000014AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014AF      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x000014B1      3  F31152                                               IMM16_F3                  u16_be=4434, u16_le=21009
0x000014B4      1  FF                                                   TERMINATOR_FF             
0x000014B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014B7      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x000014B9      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000014BB      1  FF                                                   TERMINATOR_FF             
0x000014BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014BE      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x000014C0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000014C2      1  FF                                                   TERMINATOR_FF             
0x000014C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014C7      2  F212                                                 IMM8_F2                   u8=18, s8=18
0x000014C9      1  FF                                                   TERMINATOR_FF             
0x000014CA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000014CC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000014CE      1  FF                                                   TERMINATOR_FF             
0x000014CF      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000014D1      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x000014D4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000014D6      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000014D7      1  FF                                                   TERMINATOR_FF             
0x000014D8      2  0018                                                 WORD_00XX                 u16_be=24, low_byte=24
0x000014DA      1  88                                                   OPAQUE_RAW_BYTES          bytes=88
0x000014DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014DF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000014E1      1  FF                                                   TERMINATOR_FF             
0x000014E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014E4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000014E6      3  F31153                                               IMM16_F3                  u16_be=4435, u16_le=21265
0x000014E9      1  FF                                                   TERMINATOR_FF             
0x000014EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014EC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014EE     78  804C82A282E2814182C582E08163816381425C6E82C782B1...  LEN8_STRING_CP932         length=76, text="いや、でも……。\nどこかに綾がいるはずだ。\nぼくと彼女は、一緒だったはずだ。"
0x0000153C      1  FF                                                   TERMINATOR_FF             
0x0000153D      1  FF                                                   TERMINATOR_FF             
0x0000153E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001540      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001542      1  FF                                                   TERMINATOR_FF             
0x00001543      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001545      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001547      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001549      1  FF                                                   TERMINATOR_FF             
0x0000154A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000154C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000154E      3  F31154                                               IMM16_F3                  u16_be=4436, u16_le=21521
0x00001551      1  FF                                                   TERMINATOR_FF             
0x00001552      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001554      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001556     52  803282C882E78163816382B182CC8A5882CC82C782B182A9...  LEN8_STRING_CP932         length=50, text="なら……この街のどこかに、彼女はいるはず\nだった。"
0x0000158A      1  FF                                                   TERMINATOR_FF             
0x0000158B      1  FF                                                   TERMINATOR_FF             
0x0000158C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000158E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001590      1  FF                                                   TERMINATOR_FF             
0x00001591      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001593      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001595      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001597      1  FF                                                   TERMINATOR_FF             
0x00001598      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000159A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000159C      3  F31155                                               IMM16_F3                  u16_be=4437, u16_le=21777
0x0000159F      1  FF                                                   TERMINATOR_FF             
0x000015A0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015A2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015A4     56  803682A082AB82E782DF82C482CD82A282AF82C882A28142...  LEN8_STRING_CP932         length=54, text="あきらめてはいけない。\n綾は、決してあきらめなかった。"
0x000015DC      1  FF                                                   TERMINATOR_FF             
0x000015DD      1  FF                                                   TERMINATOR_FF             
0x000015DE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000015E0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000015E2      1  FF                                                   TERMINATOR_FF             
0x000015E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015E5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015E7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000015E9      1  FF                                                   TERMINATOR_FF             
0x000015EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015EC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000015EE      3  F31156                                               IMM16_F3                  u16_be=4438, u16_le=22033
0x000015F1      1  FF                                                   TERMINATOR_FF             
0x000015F2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015F4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015F6     52  803282BE82A982E7814182DA82AD82E08163816381425C6E...  LEN8_STRING_CP932         length=50, text="だから、ぼくも……。\nあきらめるわけにはいかない。"
0x0000162A      1  FF                                                   TERMINATOR_FF             
0x0000162B      1  FF                                                   TERMINATOR_FF             
0x0000162C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000162E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001630      1  FF                                                   TERMINATOR_FF             
0x00001631      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001633      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001635      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001637      1  FF                                                   TERMINATOR_FF             
0x00001638      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000163A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000163C      3  F31157                                               IMM16_F3                  u16_be=4439, u16_le=22289
0x0000163F      1  FF                                                   TERMINATOR_FF             
0x00001640      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001642      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001644     60  803A82DA82AD82CD8BB38EBA82F08CE382C982B782E98142...  LEN8_STRING_CP932         length=58, text="ぼくは教室を後にする。\n綾は……必ず、どこかにいるはずだ。"
0x00001680      1  FF                                                   TERMINATOR_FF             
0x00001681      1  FF                                                   TERMINATOR_FF             
0x00001682      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001684      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001686      1  FF                                                   TERMINATOR_FF             
0x00001687      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001689      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000168B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000168D      1  FF                                                   TERMINATOR_FF             
0x0000168E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001690      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001692      3  F31158                                               IMM16_F3                  u16_be=4440, u16_le=22545
0x00001695      1  FF                                                   TERMINATOR_FF             
0x00001696      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001698      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000169A     18  80108163816382C68141904D82B682C48142                 LEN8_STRING_CP932         length=16, text="……と、信じて。"
0x000016AC      1  FF                                                   TERMINATOR_FF             
0x000016AD      1  FF                                                   TERMINATOR_FF             
0x000016AE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016B0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016B2      1  FF                                                   TERMINATOR_FF             
0x000016B3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016B7      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000016B9      1  FF                                                   TERMINATOR_FF             
0x000016BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016BC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016BE      2  F274                                                 IMM8_F2                   u8=116, s8=116
0x000016C0      1  FF                                                   TERMINATOR_FF             
0x000016C1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016C3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016C5      1  FF                                                   TERMINATOR_FF             
0x000016C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016CA      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000016CC      1  FF                                                   TERMINATOR_FF             
0x000016CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016D1      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000016D3      1  FF                                                   TERMINATOR_FF             
0x000016D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016D6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000016D8      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000016DA      1  FF                                                   TERMINATOR_FF             
0x000016DB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016DD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016DF      1  FF                                                   TERMINATOR_FF             
0x000016E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016E4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000016E6      1  FF                                                   TERMINATOR_FF             
0x000016E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016E9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000016EB      3  F31159                                               IMM16_F3                  u16_be=4441, u16_le=22801
0x000016EE      1  FF                                                   TERMINATOR_FF             
0x000016EF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016F3     46  802C984C89BA82F095E082A282C482A282C481418B4382C3...  LEN8_STRING_CP932         length=44, text="廊下を歩いていて、気づいた。\n音が聞こえる。"
0x00001721      1  FF                                                   TERMINATOR_FF             
0x00001722      1  FF                                                   TERMINATOR_FF             
0x00001723      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001725      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001727      1  FF                                                   TERMINATOR_FF             
0x00001728      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000172A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000172C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000172E      1  FF                                                   TERMINATOR_FF             
0x0000172F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001731      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001733      3  F3115A                                               IMM16_F3                  u16_be=4442, u16_le=23057
0x00001736      1  FF                                                   TERMINATOR_FF             
0x00001737      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001739      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000173B     14  800C82BB82EA82CD816381638142                         LEN8_STRING_CP932         length=12, text="それは……。"
0x00001749      1  FF                                                   TERMINATOR_FF             
0x0000174A      1  FF                                                   TERMINATOR_FF             
0x0000174B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000174D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000174F      1  FF                                                   TERMINATOR_FF             
0x00001750      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001752      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001754      7  800573652D3431                                       LEN8_STRING_CP932         length=5, text="se-41"
0x0000175B      1  FF                                                   TERMINATOR_FF             
0x0000175C      1  FF                                                   TERMINATOR_FF             
0x0000175D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000175F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001761      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00001763      1  FF                                                   TERMINATOR_FF             
0x00001764      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001766      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001768      1  FF                                                   TERMINATOR_FF             
0x00001769      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000176B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000176D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000176F      1  FF                                                   TERMINATOR_FF             
0x00001770      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001772      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001774      3  F3115B                                               IMM16_F3                  u16_be=4443, u16_le=23313
0x00001777      1  FF                                                   TERMINATOR_FF             
0x00001778      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000177A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000177C     30  801C9364986282AA96C282C182C482A282BD81425C6E82C7...  LEN8_STRING_CP932         length=28, text="電話が鳴っていた。\nどこで?!"
0x0000179A      1  FF                                                   TERMINATOR_FF             
0x0000179B      1  FF                                                   TERMINATOR_FF             
0x0000179C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000179E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000017A0      1  FF                                                   TERMINATOR_FF             
0x000017A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017A5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000017A7      1  FF                                                   TERMINATOR_FF             
0x000017A8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017AA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000017AC      3  F3115C                                               IMM16_F3                  u16_be=4444, u16_le=23569
0x000017AF      1  FF                                                   TERMINATOR_FF             
0x000017B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017B4     52  803282BB82EA82CD834A83458393835A838A8393834F8EBA...  LEN8_STRING_CP932         length=50, text="それはカウンセリング室のある方向から聞こえていた。"
0x000017E8      1  FF                                                   TERMINATOR_FF             
0x000017E9      1  FF                                                   TERMINATOR_FF             
0x000017EA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000017EC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000017EE      1  FF                                                   TERMINATOR_FF             
0x000017EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017F3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000017F5      1  FF                                                   TERMINATOR_FF             
0x000017F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017F8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000017FA      3  F34101                                               IMM16_F3                  u16_be=16641, u16_le=321
0x000017FD      1  FF                                                   TERMINATOR_FF             
0x000017FE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001800      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001802     12  800A81638163936498628142                             LEN8_STRING_CP932         length=10, text="……電話。"
0x0000180E      1  FF                                                   TERMINATOR_FF             
0x0000180F      1  FF                                                   TERMINATOR_FF             
0x00001810      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001812      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001814      1  FF                                                   TERMINATOR_FF             
0x00001815      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001817      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001819      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000181B      1  FF                                                   TERMINATOR_FF             
0x0000181C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000181E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001820      3  F3115D                                               IMM16_F3                  u16_be=4445, u16_le=23825
0x00001823      1  FF                                                   TERMINATOR_FF             
0x00001824      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001826      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001828     38  802482DA82AD82CD8141834A83458393835A838A8393834F...  LEN8_STRING_CP932         length=36, text="ぼくは、カウンセリング室へ向かった。"
0x0000184E      1  FF                                                   TERMINATOR_FF             
0x0000184F      1  FF                                                   TERMINATOR_FF             
0x00001850      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001852      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001854      1  FF                                                   TERMINATOR_FF             
0x00001855      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001857      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001859      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000185B      1  FF                                                   TERMINATOR_FF             
0x0000185C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000185E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001860      3  F30284                                               IMM16_F3                  u16_be=644, u16_le=33794
0x00001863      1  FF                                                   TERMINATOR_FF             
0x00001864      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001866      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001868      1  FF                                                   TERMINATOR_FF             
0x00001869      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000186B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000186D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000186F      1  FF                                                   TERMINATOR_FF             
0x00001870      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001872      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001874      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001876      1  FF                                                   TERMINATOR_FF             
0x00001877      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001879      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000187B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000187D      1  FF                                                   TERMINATOR_FF             
0x0000187E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001880      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001882      1  FF                                                   TERMINATOR_FF             
0x00001883      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00001885      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00001887      1  21                                                   OPAQUE_RAW_BYTES          bytes=21
0x00001888      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000188A      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x0000188D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000188F      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00001890      1  FF                                                   TERMINATOR_FF             
0x00001891      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00001893      1  21                                                   OPAQUE_RAW_BYTES          bytes=21
0x00001894      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001896      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001898      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000189A      1  FF                                                   TERMINATOR_FF             
0x0000189B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000189D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000189F      2  F274                                                 IMM8_F2                   u8=116, s8=116
0x000018A1      1  FF                                                   TERMINATOR_FF             
0x000018A2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000018A4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000018A6      1  FF                                                   TERMINATOR_FF             
0x000018A7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018AB      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000018AD      1  FF                                                   TERMINATOR_FF             
0x000018AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018B2      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000018B4      1  FF                                                   TERMINATOR_FF             
0x000018B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018B7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000018B9      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000018BB      1  FF                                                   TERMINATOR_FF             
0x000018BC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000018BE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000018C0      1  FF                                                   TERMINATOR_FF             
0x000018C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018C5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000018C7      1  FF                                                   TERMINATOR_FF             
0x000018C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018CA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000018CC      3  F3115E                                               IMM16_F3                  u16_be=4446, u16_le=24081
0x000018CF      1  FF                                                   TERMINATOR_FF             
0x000018D0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018D2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018D4     94  805C82DA82AD82CD814182E082A4825093788141834A8345...  LEN8_STRING_CP932         length=92, text="ぼくは、もう１度、カウンセリング室へ向かった。もう、あそこにしか、手がかりはないはずだった。"
0x00001932      1  FF                                                   TERMINATOR_FF             
0x00001933      1  FF                                                   TERMINATOR_FF             
0x00001934      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001936      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001938      1  FF                                                   TERMINATOR_FF             
0x00001939      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000193B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000193D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000193F      1  FF                                                   TERMINATOR_FF             
0x00001940      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001942      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001944      3  F3115F                                               IMM16_F3                  u16_be=4447, u16_le=24337
0x00001947      1  FF                                                   TERMINATOR_FF             
0x00001948      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000194A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000194C    116  807282C882C982A982F08CA9978E82C682B582C482A282E9...  LEN8_STRING_CP932         length=114, text="なにかを見落としているのかもしれない。\nたとえ、なにもなかったとしても……\nぼくはなにかを見つけなければいけない。"
0x000019C0      1  FF                                                   TERMINATOR_FF             
0x000019C1      1  FF                                                   TERMINATOR_FF             
0x000019C2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000019C4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000019C6      1  FF                                                   TERMINATOR_FF             
0x000019C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019C9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019CB      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x000019CD      1  FF                                                   TERMINATOR_FF             
0x000019CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019D0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019D2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000019D4      1  FF                                                   TERMINATOR_FF             
0x000019D5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000019D7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000019D9      1  FF                                                   TERMINATOR_FF             
0x000019DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019DE      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000019E0      1  FF                                                   TERMINATOR_FF             
0x000019E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019E3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019E5      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x000019E7      1  FF                                                   TERMINATOR_FF             
0x000019E8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000019EA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000019EC      1  FF                                                   TERMINATOR_FF             
0x000019ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019F1      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000019F3      1  FF                                                   TERMINATOR_FF             
0x000019F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019F6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019F8      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000019FA      1  FF                                                   TERMINATOR_FF             
0x000019FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019FD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000019FF      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001A01      1  FF                                                   TERMINATOR_FF             
0x00001A02      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A04      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A06      1  FF                                                   TERMINATOR_FF             
0x00001A07      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A09      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A0B      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001A0D      1  FF                                                   TERMINATOR_FF             
0x00001A0E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A10      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A12      3  F30284                                               IMM16_F3                  u16_be=644, u16_le=33794
0x00001A15      1  FF                                                   TERMINATOR_FF             
0x00001A16      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A18      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A1A      1  FF                                                   TERMINATOR_FF             
0x00001A1B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A1F      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001A21      1  FF                                                   TERMINATOR_FF             
0x00001A22      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A24      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A26      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001A28      1  FF                                                   TERMINATOR_FF             
0x00001A29      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A2B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001A2D      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001A2F      1  FF                                                   TERMINATOR_FF             
0x00001A30      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A32      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A34      1  FF                                                   TERMINATOR_FF             
0x00001A35      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A37      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A39      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001A3B      1  FF                                                   TERMINATOR_FF             
0x00001A3C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A3E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001A40      3  F31160                                               IMM16_F3                  u16_be=4448, u16_le=24593
0x00001A43      1  FF                                                   TERMINATOR_FF             
0x00001A44      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A46      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A48    132  808282DA82AD82CD96B3906C82CC834A83458393835A838A...  LEN8_STRING_CP932         length=130, text="ぼくは無人のカウンセリング室で考えていた。\nここで、ぼくは世界が滅びたとおしえられた。\nカウンセラーが、妖精を握りつぶすのを見た。"
0x00001ACC      1  FF                                                   TERMINATOR_FF             
0x00001ACD      1  FF                                                   TERMINATOR_FF             
0x00001ACE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001AD0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001AD2      1  FF                                                   TERMINATOR_FF             
0x00001AD3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001AD5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001AD7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001AD9      1  FF                                                   TERMINATOR_FF             
0x00001ADA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001ADC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001ADE      3  F31161                                               IMM16_F3                  u16_be=4449, u16_le=24849
0x00001AE1      1  FF                                                   TERMINATOR_FF             
0x00001AE2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001AE4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001AE6     46  802C8C8E96BE82A982E882CC928682C5814182DA82AD82CD...  LEN8_STRING_CP932         length=44, text="月明かりの中で、ぼくはあの街へと踏みこんだ。"
0x00001B14      1  FF                                                   TERMINATOR_FF             
0x00001B15      1  FF                                                   TERMINATOR_FF             
0x00001B16      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B18      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B1A      1  FF                                                   TERMINATOR_FF             
0x00001B1B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B1F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001B21      1  FF                                                   TERMINATOR_FF             
0x00001B22      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B24      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001B26      3  F31162                                               IMM16_F3                  u16_be=4450, u16_le=25105
0x00001B29      1  FF                                                   TERMINATOR_FF             
0x00001B2A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B2C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B2E     36  802282BD82AD82B382F182CC82B182C682AA814182B182B1...  LEN8_STRING_CP932         length=34, text="たくさんのことが、ここでおこった。"
0x00001B52      1  FF                                                   TERMINATOR_FF             
0x00001B53      1  FF                                                   TERMINATOR_FF             
0x00001B54      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B56      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B58      1  FF                                                   TERMINATOR_FF             
0x00001B59      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B5B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B5D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001B5F      1  FF                                                   TERMINATOR_FF             
0x00001B60      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B62      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001B64      3  F31163                                               IMM16_F3                  u16_be=4451, u16_le=25361
0x00001B67      1  FF                                                   TERMINATOR_FF             
0x00001B68      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B6A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B6C     58  803882AF82EA82C781418DA182B182B182C982CD924E82E0...  LEN8_STRING_CP932         length=56, text="けれど、今ここには誰もいない。\nなにもおきはしなかった。"
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
0x00001BB8      3  F31164                                               IMM16_F3                  u16_be=4452, u16_le=25617
0x00001BBB      1  FF                                                   TERMINATOR_FF             
0x00001BBC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001BBE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001BC0     90  805882DA82AD82CD82B882C182C691D282C182C482A282BD...  LEN8_STRING_CP932         length=88, text="ぼくはずっと待っていた。\nこれまでもそうだった。\nだから、もう１度、ここで待っていれば。"
0x00001C1A      1  FF                                                   TERMINATOR_FF             
0x00001C1B      1  FF                                                   TERMINATOR_FF             
0x00001C1C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001C1E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001C20      1  FF                                                   TERMINATOR_FF             
0x00001C21      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C23      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C25      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x00001C27      1  FF                                                   TERMINATOR_FF             
0x00001C28      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001C2A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001C2C      1  FF                                                   TERMINATOR_FF             
0x00001C2D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C2F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001C31      7  800573652D3431                                       LEN8_STRING_CP932         length=5, text="se-41"
0x00001C38      1  FF                                                   TERMINATOR_FF             
0x00001C39      1  FF                                                   TERMINATOR_FF             
0x00001C3A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C3C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C3E      2  F214                                                 IMM8_F2                   u8=20, s8=20
0x00001C40      1  FF                                                   TERMINATOR_FF             
0x00001C41      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001C43      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001C45      1  FF                                                   TERMINATOR_FF             
0x00001C46      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C48      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C4A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001C4C      1  FF                                                   TERMINATOR_FF             
0x00001C4D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C4F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001C51      3  F31165                                               IMM16_F3                  u16_be=4453, u16_le=25873
0x00001C54      1  FF                                                   TERMINATOR_FF             
0x00001C55      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C57      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C59     48  802E9364986282AA96C282C182C482A282BD81425C6E82DA...  LEN8_STRING_CP932         length=46, text="電話が鳴っていた。\nぼくは、受話器を手にする。"
0x00001C89      1  FF                                                   TERMINATOR_FF             
0x00001C8A      1  FF                                                   TERMINATOR_FF             
0x00001C8B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001C8D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001C8F      1  FF                                                   TERMINATOR_FF             
0x00001C90      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C92      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001C94      8  800653452D533237                                     LEN8_STRING_CP932         length=6, text="SE-S27"
0x00001C9C      1  FF                                                   TERMINATOR_FF             
0x00001C9D      1  FF                                                   TERMINATOR_FF             
0x00001C9E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CA0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CA2      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00001CA4      1  FF                                                   TERMINATOR_FF             
0x00001CA5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001CA7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001CA9      1  FF                                                   TERMINATOR_FF             
0x00001CAA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CAC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CAE      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001CB0      1  FF                                                   TERMINATOR_FF             
0x00001CB1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CB3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CB5      3  F30124                                               IMM16_F3                  u16_be=292, u16_le=9217
0x00001CB8      1  FF                                                   TERMINATOR_FF             
0x00001CB9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001CBB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001CBD      1  FF                                                   TERMINATOR_FF             
0x00001CBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CC0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CC2      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001CC4      1  FF                                                   TERMINATOR_FF             
0x00001CC5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CC7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CC9      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001CCB      1  FF                                                   TERMINATOR_FF             
0x00001CCC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CCE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001CD0      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001CD2      1  FF                                                   TERMINATOR_FF             
0x00001CD3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001CD5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001CD7      1  FF                                                   TERMINATOR_FF             
0x00001CD8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CDA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CDC      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001CDE      1  FF                                                   TERMINATOR_FF             
0x00001CDF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CE1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001CE3      3  F31166                                               IMM16_F3                  u16_be=4454, u16_le=26129
0x00001CE6      1  FF                                                   TERMINATOR_FF             
0x00001CE7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CE9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001CEB      9  800754413030343035                                   LEN8_STRING_CP932         length=7, text="TA00405"
0x00001CF4      1  FF                                                   TERMINATOR_FF             
0x00001CF5      1  FF                                                   TERMINATOR_FF             
0x00001CF6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CF8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CFA     20  8012817582A882A982A682E882C882B382A28176             LEN8_STRING_CP932         length=18, text="「おかえりなさい」"
0x00001D0E      1  FF                                                   TERMINATOR_FF             
0x00001D0F      1  FF                                                   TERMINATOR_FF             
0x00001D10      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001D12      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001D14      1  FF                                                   TERMINATOR_FF             
0x00001D15      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D17      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D19      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001D1B      1  FF                                                   TERMINATOR_FF             
0x00001D1C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D1E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001D20      3  F31167                                               IMM16_F3                  u16_be=4455, u16_le=26385
0x00001D23      1  FF                                                   TERMINATOR_FF             
0x00001D24      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001D26      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001D28     52  8032834A83458393835A8389815B82CC90BA82AA82B782E9...  LEN8_STRING_CP932         length=50, text="カウンセラーの声がする。\n彼女の声は、沈んでいた。"
0x00001D5C      1  FF                                                   TERMINATOR_FF             
0x00001D5D      1  FF                                                   TERMINATOR_FF             
0x00001D5E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001D60      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001D62      1  FF                                                   TERMINATOR_FF             
0x00001D63      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D65      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D67      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001D69      1  FF                                                   TERMINATOR_FF             
0x00001D6A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D6C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001D6E      3  F31168                                               IMM16_F3                  u16_be=4456, u16_le=26641
0x00001D71      1  FF                                                   TERMINATOR_FF             
0x00001D72      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001D74      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001D76      9  800754413030343135                                   LEN8_STRING_CP932         length=7, text="TA00415"
0x00001D7F      1  FF                                                   TERMINATOR_FF             
0x00001D80      1  FF                                                   TERMINATOR_FF             
0x00001D81      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001D83      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001D85     36  802281758E7392A18EC982C5937E96D882B382F182AA91D2...  LEN8_STRING_CP932         length=34, text="「市庁舎で冬木さんが待っているわ」"
0x00001DA9      1  FF                                                   TERMINATOR_FF             
0x00001DAA      1  FF                                                   TERMINATOR_FF             
0x00001DAB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001DAD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001DAF      1  FF                                                   TERMINATOR_FF             
0x00001DB0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001DB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001DB4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001DB6      1  FF                                                   TERMINATOR_FF             
0x00001DB7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001DB9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001DBB      3  F31169                                               IMM16_F3                  u16_be=4457, u16_le=26897
0x00001DBE      1  FF                                                   TERMINATOR_FF             
0x00001DBF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001DC1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001DC3     66  804082BB82EA82BE82AF82F08D9082B082C4814193649862...  LEN8_STRING_CP932         length=64, text="それだけを告げて、電話はきれた。\n受話器は、完全に沈黙していた。"
0x00001E05      1  FF                                                   TERMINATOR_FF             
0x00001E06      1  FF                                                   TERMINATOR_FF             
0x00001E07      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001E09      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001E0B      1  FF                                                   TERMINATOR_FF             
0x00001E0C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E0E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E10      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001E12      1  FF                                                   TERMINATOR_FF             
0x00001E13      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E15      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001E17      3  F34102                                               IMM16_F3                  u16_be=16642, u16_le=577
0x00001E1A      1  FF                                                   TERMINATOR_FF             
0x00001E1B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001E1D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001E1F    112  806E92CA986289B982E08CC482D18F6F82B589B982E082C8...  LEN8_STRING_CP932         length=110, text="通話音も呼び出し音もない、完全な無音の状態。ぼくは受話器を置いた。\n世界はまだかすかに生きのびようとしていた。"
0x00001E8F      1  FF                                                   TERMINATOR_FF             
0x00001E90      1  FF                                                   TERMINATOR_FF             
0x00001E91      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001E93      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001E95      1  FF                                                   TERMINATOR_FF             
0x00001E96      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E98      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E9A      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00001E9C      1  FF                                                   TERMINATOR_FF             
0x00001E9D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E9F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001EA1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001EA3      1  FF                                                   TERMINATOR_FF             
0x00001EA4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001EA6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001EA8      1  FF                                                   TERMINATOR_FF             
0x00001EA9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EAB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EAD      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001EAF      1  FF                                                   TERMINATOR_FF             
0x00001EB0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EB2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001EB4      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00001EB6      1  FF                                                   TERMINATOR_FF             
0x00001EB7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001EB9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001EBB      1  FF                                                   TERMINATOR_FF             
0x00001EBC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EC0      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001EC2      1  FF                                                   TERMINATOR_FF             
0x00001EC3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EC5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001EC7      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001EC9      1  FF                                                   TERMINATOR_FF             
0x00001ECA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001ECC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001ECE      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001ED0      1  FF                                                   TERMINATOR_FF             
0x00001ED1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001ED3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001ED5      1  FF                                                   TERMINATOR_FF             
0x00001ED6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001ED8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EDA      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001EDC      1  FF                                                   TERMINATOR_FF             
0x00001EDD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EDF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001EE1      3  F30088                                               IMM16_F3                  u16_be=136, u16_le=34816
0x00001EE4      1  FF                                                   TERMINATOR_FF             
0x00001EE5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001EE7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001EE9      1  FF                                                   TERMINATOR_FF             
0x00001EEA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EEC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EEE      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001EF0      1  FF                                                   TERMINATOR_FF             
0x00001EF1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EF3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001EF5      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001EF7      1  FF                                                   TERMINATOR_FF             
0x00001EF8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001EFA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001EFC      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001EFE      1  FF                                                   TERMINATOR_FF             
0x00001EFF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001F01      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F03      1  FF                                                   TERMINATOR_FF             
0x00001F04      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F06      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F08      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001F0A      1  FF                                                   TERMINATOR_FF             
0x00001F0B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F0D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001F0F      3  F3116A                                               IMM16_F3                  u16_be=4458, u16_le=27153
0x00001F12      1  FF                                                   TERMINATOR_FF             
0x00001F13      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F15      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F17     56  803682DA82AD82CD81418E7392A18EC982D68CFC82A982C1...  LEN8_STRING_CP932         length=54, text="ぼくは、市庁舎へ向かった。\nカウンセラーの指示通りに。"
0x00001F4F      1  FF                                                   TERMINATOR_FF             
0x00001F50      1  FF                                                   TERMINATOR_FF             
0x00001F51      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001F53      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F55      1  FF                                                   TERMINATOR_FF             
0x00001F56      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F58      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F5A      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001F5C      1  FF                                                   TERMINATOR_FF             
0x00001F5D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F5F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F61      2  F25F                                                 IMM8_F2                   u8=95, s8=95
0x00001F63      1  FF                                                   TERMINATOR_FF             
0x00001F64      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001F66      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F68      1  FF                                                   TERMINATOR_FF             
0x00001F69      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F6B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F6D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001F6F      1  FF                                                   TERMINATOR_FF             
0x00001F70      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F72      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F74      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001F76      1  FF                                                   TERMINATOR_FF             
0x00001F77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F79      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001F7B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001F7D      1  FF                                                   TERMINATOR_FF             
0x00001F7E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001F80      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F82      1  FF                                                   TERMINATOR_FF             
0x00001F83      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F85      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F87      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001F89      1  FF                                                   TERMINATOR_FF             
0x00001F8A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F8C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001F8E      3  F3116B                                               IMM16_F3                  u16_be=4459, u16_le=27409
0x00001F91      1  FF                                                   TERMINATOR_FF             
0x00001F92      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F94      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F96     84  8052924E82E082A282C882A28A5881425C6E906C82CC8E70...  LEN8_STRING_CP932         length=82, text="誰もいない街。\n人の姿は見当たらず、道路には１台の車もない。なのに、廃墟ではない。"
0x00001FEA      1  FF                                                   TERMINATOR_FF             
0x00001FEB      1  FF                                                   TERMINATOR_FF             
0x00001FEC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001FEE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001FF0      1  FF                                                   TERMINATOR_FF             
0x00001FF1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001FF3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001FF5      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001FF7      1  FF                                                   TERMINATOR_FF             
0x00001FF8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001FFA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001FFC      2  F227                                                 IMM8_F2                   u8=39, s8=39
0x00001FFE      1  FF                                                   TERMINATOR_FF             
0x00001FFF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002001      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002003      1  FF                                                   TERMINATOR_FF             
0x00002004      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002006      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002008      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000200A      1  FF                                                   TERMINATOR_FF             
0x0000200B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000200D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000200F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002011      1  FF                                                   TERMINATOR_FF             
0x00002012      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002014      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002016      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002018      1  FF                                                   TERMINATOR_FF             
0x00002019      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000201B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000201D      1  FF                                                   TERMINATOR_FF             
0x0000201E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002020      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002022      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002024      1  FF                                                   TERMINATOR_FF             
0x00002025      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002027      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002029      3  F3116C                                               IMM16_F3                  u16_be=4460, u16_le=27665
0x0000202C      1  FF                                                   TERMINATOR_FF             
0x0000202D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000202F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002031     82  805082B382DC82B482DC82C88FA4935882CD82A8935882F0...  LEN8_STRING_CP932         length=80, text="さまざまな商店はお店を開いていたし、パン屋の前には焼きたてのかおりが漂っている。"
0x00002083      1  FF                                                   TERMINATOR_FF             
0x00002084      1  FF                                                   TERMINATOR_FF             
0x00002085      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002087      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002089      1  FF                                                   TERMINATOR_FF             
0x0000208A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000208C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000208E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002090      1  FF                                                   TERMINATOR_FF             
0x00002091      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002093      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002095      3  F3116D                                               IMM16_F3                  u16_be=4461, u16_le=27921
0x00002098      1  FF                                                   TERMINATOR_FF             
0x00002099      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000209B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000209D     90  80588A58928682CC906C82AA814182D982F182CC82BF82E5...  LEN8_STRING_CP932         length=88, text="街中の人が、ほんのちょっとだけ、小さな用事をかたづけるため、どこかへ行ってしまったよう。"
0x000020F7      1  FF                                                   TERMINATOR_FF             
0x000020F8      1  FF                                                   TERMINATOR_FF             
0x000020F9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000020FB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000020FD      1  FF                                                   TERMINATOR_FF             
0x000020FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002100      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002102      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002104      1  FF                                                   TERMINATOR_FF             
0x00002105      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002107      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002109      3  F3116E                                               IMM16_F3                  u16_be=4462, u16_le=28177
0x0000210C      1  FF                                                   TERMINATOR_FF             
0x0000210D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000210F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002111     42  802888BB82CC816381635C6E82A082CC8A5882C682CD8141...  LEN8_STRING_CP932         length=40, text="綾の……\nあの街とは、そこが違っていた。"
0x0000213B      1  FF                                                   TERMINATOR_FF             
0x0000213C      1  FF                                                   TERMINATOR_FF             
0x0000213D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000213F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002141      1  FF                                                   TERMINATOR_FF             
0x00002142      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002144      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002146      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002148      1  FF                                                   TERMINATOR_FF             
0x00002149      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000214B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000214D      3  F3116F                                               IMM16_F3                  u16_be=4463, u16_le=28433
0x00002150      1  FF                                                   TERMINATOR_FF             
0x00002151      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002153      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002155     52  803282A082CC8A5882A982E782CD8141906C82CC8B43947A...  LEN8_STRING_CP932         length=50, text="あの街からは、人の気配がほとんど消えかけて\nいた。"
0x00002189      1  FF                                                   TERMINATOR_FF             
0x0000218A      1  FF                                                   TERMINATOR_FF             
0x0000218B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000218D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000218F      1  FF                                                   TERMINATOR_FF             
0x00002190      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002192      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002194      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002196      1  FF                                                   TERMINATOR_FF             
0x00002197      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002199      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000219B      3  F31170                                               IMM16_F3                  u16_be=4464, u16_le=28689
0x0000219E      1  FF                                                   TERMINATOR_FF             
0x0000219F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000021A1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000021A3     96  805E82AB82C182C6814182B182CC8A5882E0816381635C6E...  LEN8_STRING_CP932         length=94, text="きっと、この街も……\nぼくの通いなれたここも、遠くないうちに\nすべてをなくしてしまうのだろう。"
0x00002203      1  FF                                                   TERMINATOR_FF             
0x00002204      1  FF                                                   TERMINATOR_FF             
0x00002205      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002207      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002209      1  FF                                                   TERMINATOR_FF             
0x0000220A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000220C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000220E      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00002210      1  FF                                                   TERMINATOR_FF             
0x00002211      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002213      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002215      2  F25D                                                 IMM8_F2                   u8=93, s8=93
0x00002217      1  FF                                                   TERMINATOR_FF             
0x00002218      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000221A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000221C      1  FF                                                   TERMINATOR_FF             
0x0000221D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000221F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002221      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002223      1  FF                                                   TERMINATOR_FF             
0x00002224      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002226      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002228      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000222A      1  FF                                                   TERMINATOR_FF             
0x0000222B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000222D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000222F      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002231      1  FF                                                   TERMINATOR_FF             
0x00002232      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002234      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002236      1  FF                                                   TERMINATOR_FF             
0x00002237      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002239      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000223B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000223D      1  FF                                                   TERMINATOR_FF             
0x0000223E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002240      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002242      3  F31171                                               IMM16_F3                  u16_be=4465, u16_le=28945
0x00002245      1  FF                                                   TERMINATOR_FF             
0x00002246      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002248      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000224A     90  805882DA82AD82CD96C082A282E082C882AD814196B3906C...  LEN8_STRING_CP932         length=88, text="ぼくは迷いもなく、無人の駅に向かう。\n運転手のいない電車が、ホームで待っている\nはずだ。"
0x000022A4      1  FF                                                   TERMINATOR_FF             
0x000022A5      1  FF                                                   TERMINATOR_FF             
0x000022A6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000022A8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000022AA      1  FF                                                   TERMINATOR_FF             
0x000022AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022AF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000022B1      1  FF                                                   TERMINATOR_FF             
0x000022B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022B4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000022B6      3  F34103                                               IMM16_F3                  u16_be=16643, u16_le=833
0x000022B9      1  FF                                                   TERMINATOR_FF             
0x000022BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000022BC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000022BE     78  804C96DA9349926E82DC82C582CC90D8958482F0948382A2...  LEN8_STRING_CP932         length=76, text="目的地までの切符を買い（まだ文字の変質は\n始まっていなかった）、電車に乗る。"
0x0000230C      1  FF                                                   TERMINATOR_FF             
0x0000230D      1  FF                                                   TERMINATOR_FF             
0x0000230E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002310      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002312      1  FF                                                   TERMINATOR_FF             
0x00002313      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002315      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002317      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002319      1  FF                                                   TERMINATOR_FF             
0x0000231A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000231C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000231E      3  F34104                                               IMM16_F3                  u16_be=16644, u16_le=1089
0x00002321      1  FF                                                   TERMINATOR_FF             
0x00002322      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002324      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002326     52  803293648ED482CD82DA82AD82AA8FE682E982C693AF8E9E...  LEN8_STRING_CP932         length=50, text="電車はぼくが乗ると同時に、ゆっくりと\n動き始めた。"
0x0000235A      1  FF                                                   TERMINATOR_FF             
0x0000235B      1  FF                                                   TERMINATOR_FF             
0x0000235C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000235E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002360      1  FF                                                   TERMINATOR_FF             
0x00002361      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002363      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002365      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002367      1  FF                                                   TERMINATOR_FF             
0x00002368      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000236A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000236C      3  F31172                                               IMM16_F3                  u16_be=4466, u16_le=29201
0x0000236F      1  FF                                                   TERMINATOR_FF             
0x00002370      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002372      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002374     60  803A82DA82AD82CD81418E7392A18EC982CC82A082E98977...  LEN8_STRING_CP932         length=58, text="ぼくは、市庁舎のある駅で電車をおりた。\nアナウンスはなし。"
0x000023B0      1  FF                                                   TERMINATOR_FF             
0x000023B1      1  FF                                                   TERMINATOR_FF             
0x000023B2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000023B4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000023B6      1  FF                                                   TERMINATOR_FF             
0x000023B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023B9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023BB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000023BD      1  FF                                                   TERMINATOR_FF             
0x000023BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023C0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000023C2      3  F31173                                               IMM16_F3                  u16_be=4467, u16_le=29457
0x000023C5      1  FF                                                   TERMINATOR_FF             
0x000023C6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000023C8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000023CA     60  803A967B97888141825093FA92868141906C82C582A082D3...  LEN8_STRING_CP932         length=58, text="本来、１日中、人であふれ返っているはずの駅に人の姿はない。"
0x00002406      1  FF                                                   TERMINATOR_FF             
0x00002407      1  FF                                                   TERMINATOR_FF             
0x00002408      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000240A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000240C      1  FF                                                   TERMINATOR_FF             
0x0000240D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000240F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002411      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002413      1  FF                                                   TERMINATOR_FF             
0x00002414      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002416      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002418      3  F31174                                               IMM16_F3                  u16_be=4468, u16_le=29713
0x0000241B      1  FF                                                   TERMINATOR_FF             
0x0000241C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000241E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002420     72  804682DA82AD82CD8EA995AA82CC91AB89B982BE82AF82F0...  LEN8_STRING_CP932         length=70, text="ぼくは自分の足音だけを聞きながら、指示板に\nしたがって市庁舎をめざす。"
0x00002468      1  FF                                                   TERMINATOR_FF             
0x00002469      1  FF                                                   TERMINATOR_FF             
0x0000246A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000246C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000246E      1  FF                                                   TERMINATOR_FF             
0x0000246F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002471      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002473      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00002475      1  FF                                                   TERMINATOR_FF             
0x00002476      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002478      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000247A      2  F23A                                                 IMM8_F2                   u8=58, s8=58
0x0000247C      1  FF                                                   TERMINATOR_FF             
0x0000247D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000247F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002481      1  FF                                                   TERMINATOR_FF             
0x00002482      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002484      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002486      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002488      1  FF                                                   TERMINATOR_FF             
0x00002489      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000248B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000248D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000248F      1  FF                                                   TERMINATOR_FF             
0x00002490      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002492      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002494      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002496      1  FF                                                   TERMINATOR_FF             
0x00002497      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002499      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000249B      1  FF                                                   TERMINATOR_FF             
0x0000249C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000249E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024A0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000024A2      1  FF                                                   TERMINATOR_FF             
0x000024A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024A5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000024A7      3  F31175                                               IMM16_F3                  u16_be=4469, u16_le=29969
0x000024AA      1  FF                                                   TERMINATOR_FF             
0x000024AB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000024AD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000024AF     32  801E82DA82AD82CD81418E7392A18EC982C982BD82C782E8...  LEN8_STRING_CP932         length=30, text="ぼくは、市庁舎にたどりついた。"
0x000024CF      1  FF                                                   TERMINATOR_FF             
0x000024D0      1  FF                                                   TERMINATOR_FF             
0x000024D1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000024D3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000024D5      1  FF                                                   TERMINATOR_FF             
0x000024D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024DA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000024DC      1  FF                                                   TERMINATOR_FF             
0x000024DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024DF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000024E1      3  F31176                                               IMM16_F3                  u16_be=4470, u16_le=30225
0x000024E4      1  FF                                                   TERMINATOR_FF             
0x000024E5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000024E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000024E9    100  8062906C82CC82AB82A682BD8A5882CC82C882A982C58141...  LEN8_STRING_CP932         length=98, text="人のきえた街のなかで、ここだけは違うと直感\nする。\n空気にこめられた、あの冷たい気配が希薄だった。"
0x0000254D      1  FF                                                   TERMINATOR_FF             
0x0000254E      1  FF                                                   TERMINATOR_FF             
0x0000254F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002551      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002553      1  FF                                                   TERMINATOR_FF             
0x00002554      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002556      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002558      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000255A      1  FF                                                   TERMINATOR_FF             
0x0000255B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000255D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000255F      3  F31177                                               IMM16_F3                  u16_be=4471, u16_le=30481
0x00002562      1  FF                                                   TERMINATOR_FF             
0x00002563      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002565      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002567     60  803A82BD82D482F1814182B182B182BE82AF82CD82C882F1...  LEN8_STRING_CP932         length=58, text="たぶん、ここだけはなんらかの理由で生き続けているのだろう。"
0x000025A3      1  FF                                                   TERMINATOR_FF             
0x000025A4      1  FF                                                   TERMINATOR_FF             
0x000025A5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000025A7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000025A9      1  FF                                                   TERMINATOR_FF             
0x000025AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025AE      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000025B0      1  FF                                                   TERMINATOR_FF             
0x000025B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025B3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000025B5      2  F24A                                                 IMM8_F2                   u8=74, s8=74
0x000025B7      1  FF                                                   TERMINATOR_FF             
0x000025B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025BA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000025BC      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000025BE      1  FF                                                   TERMINATOR_FF             
0x000025BF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000025C1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000025C3      1  FF                                                   TERMINATOR_FF             
0x000025C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025C8      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000025CA      1  FF                                                   TERMINATOR_FF             
0x000025CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025CD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000025CF      2  F249                                                 IMM8_F2                   u8=73, s8=73
0x000025D1      1  FF                                                   TERMINATOR_FF             
0x000025D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025D4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000025D6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000025D8      1  FF                                                   TERMINATOR_FF             
0x000025D9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000025DB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000025DD      1  FF                                                   TERMINATOR_FF             
0x000025DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025E2      2  F206                                                 IMM8_F2                   u8=6, s8=6
0x000025E4      1  FF                                                   TERMINATOR_FF             
0x000025E5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000025E9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000025EB      1  FF                                                   TERMINATOR_FF             
0x000025EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025EE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000025F0      3  F300E6                                               IMM16_F3                  u16_be=230, u16_le=58880
0x000025F3      1  FF                                                   TERMINATOR_FF             
0x000025F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025F6      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x000025F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000025FA      1  FF                                                   TERMINATOR_FF             
0x000025FB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000025FD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000025FF      1  FF                                                   TERMINATOR_FF             
0x00002600      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002602      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002604      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002606      1  FF                                                   TERMINATOR_FF             
0x00002607      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002609      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000260B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000260D      1  FF                                                   TERMINATOR_FF             
0x0000260E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002610      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002612      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002614      1  FF                                                   TERMINATOR_FF             
0x00002615      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002617      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002619      1  FF                                                   TERMINATOR_FF             
0x0000261A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000261C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000261E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002620      1  FF                                                   TERMINATOR_FF             
0x00002621      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002623      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002625      3  F31178                                               IMM16_F3                  u16_be=4472, u16_le=30737
0x00002628      1  FF                                                   TERMINATOR_FF             
0x00002629      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000262B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000262D    114  80708E7392A18EC982CC914F82C582CD814196688CEC959E...  LEN8_STRING_CP932         length=112, text="市庁舎の前では、防護服を着た技師が会話をしており、そのわきには、いたたまれない表情の\nカウンセラーが立っている。"
0x0000269F      1  FF                                                   TERMINATOR_FF             
0x000026A0      1  FF                                                   TERMINATOR_FF             
0x000026A1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000026A3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000026A5      1  FF                                                   TERMINATOR_FF             
0x000026A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000026A8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000026AA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000026AC      1  FF                                                   TERMINATOR_FF             
0x000026AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000026AF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000026B1      3  F31179                                               IMM16_F3                  u16_be=4473, u16_le=30993
0x000026B4      1  FF                                                   TERMINATOR_FF             
0x000026B5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000026B7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000026B9      9  800745583135303135                                   LEN8_STRING_CP932         length=7, text="EX15015"
0x000026C2      1  FF                                                   TERMINATOR_FF             
0x000026C3      1  FF                                                   TERMINATOR_FF             
0x000026C4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000026C6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000026C8     66  8040817582E282CD82E88E6E82DC82C182C482A282DC82B7...  LEN8_STRING_CP932         length=64, text="「やはり始まっていますね。0.06パーセント\nの空間が観測不能です」"
0x0000270A      1  FF                                                   TERMINATOR_FF             
0x0000270B      1  FF                                                   TERMINATOR_FF             
0x0000270C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000270E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002710      1  FF                                                   TERMINATOR_FF             
0x00002711      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002713      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002715      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002717      1  FF                                                   TERMINATOR_FF             
0x00002718      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000271A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000271C      3  F3117A                                               IMM16_F3                  u16_be=4474, u16_le=31249
0x0000271F      1  FF                                                   TERMINATOR_FF             
0x00002720      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002722      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002724      9  800745583136303135                                   LEN8_STRING_CP932         length=7, text="EX16015"
0x0000272D      1  FF                                                   TERMINATOR_FF             
0x0000272E      1  FF                                                   TERMINATOR_FF             
0x0000272F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002731      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002733     30  801C817582BB82A482A9814296E982DC82C582E082C282A9...  LEN8_STRING_CP932         length=28, text="「そうか。夜までもつかな？」"
0x00002751      1  FF                                                   TERMINATOR_FF             
0x00002752      1  FF                                                   TERMINATOR_FF             
0x00002753      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002755      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002757      1  FF                                                   TERMINATOR_FF             
0x00002758      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000275A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000275C      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000275E      1  FF                                                   TERMINATOR_FF             
0x0000275F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002761      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002763      3  F3117B                                               IMM16_F3                  u16_be=4475, u16_le=31505
0x00002766      1  FF                                                   TERMINATOR_FF             
0x00002767      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002769      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000276B      9  800745583135303235                                   LEN8_STRING_CP932         length=7, text="EX15025"
0x00002774      1  FF                                                   TERMINATOR_FF             
0x00002775      1  FF                                                   TERMINATOR_FF             
0x00002776      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002778      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000277A     78  804C817582C782A482C582B582E582A481428AB18FC282CC...  LEN8_STRING_CP932         length=76, text="「どうでしょう。干渉の度合いによっては、この計測の確度さえ疑問が生じますし」"
0x000027C8      1  FF                                                   TERMINATOR_FF             
0x000027C9      1  FF                                                   TERMINATOR_FF             
0x000027CA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000027CC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000027CE      1  FF                                                   TERMINATOR_FF             
0x000027CF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027D3      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000027D5      1  FF                                                   TERMINATOR_FF             
0x000027D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027D8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000027DA      3  F3117C                                               IMM16_F3                  u16_be=4476, u16_le=31761
0x000027DD      1  FF                                                   TERMINATOR_FF             
0x000027DE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000027E0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000027E2      9  800745583136303235                                   LEN8_STRING_CP932         length=7, text="EX16025"
0x000027EB      1  FF                                                   TERMINATOR_FF             
0x000027EC      1  FF                                                   TERMINATOR_FF             
0x000027ED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000027EF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000027F1     54  8034817582BB82A482BE82C88142835A8393835E815B82CC...  LEN8_STRING_CP932         length=52, text="「そうだな。センターの連中が判断すればいい\nことか」"
0x00002827      1  FF                                                   TERMINATOR_FF             
0x00002828      1  FF                                                   TERMINATOR_FF             
0x00002829      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000282B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000282D      1  FF                                                   TERMINATOR_FF             
0x0000282E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002830      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002832      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002834      1  FF                                                   TERMINATOR_FF             
0x00002835      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002837      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002839      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000283B      1  FF                                                   TERMINATOR_FF             
0x0000283C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000283E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002840      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002842      1  FF                                                   TERMINATOR_FF             
0x00002843      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002845      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002847      1  FF                                                   TERMINATOR_FF             
0x00002848      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000284A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000284C      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000284E      1  FF                                                   TERMINATOR_FF             
0x0000284F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002851      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002853      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002855      1  FF                                                   TERMINATOR_FF             
0x00002856      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002858      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000285A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000285C      1  FF                                                   TERMINATOR_FF             
0x0000285D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000285F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002861      1  FF                                                   TERMINATOR_FF             
0x00002862      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002864      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002866      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002868      1  FF                                                   TERMINATOR_FF             
0x00002869      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000286B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000286D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000286F      1  FF                                                   TERMINATOR_FF             
0x00002870      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002872      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002874      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002876      1  FF                                                   TERMINATOR_FF             
0x00002877      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002879      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000287B      1  FF                                                   TERMINATOR_FF             
0x0000287C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000287E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002880      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002882      1  FF                                                   TERMINATOR_FF             
0x00002883      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002885      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002887      3  F3117D                                               IMM16_F3                  u16_be=4477, u16_le=32017
0x0000288A      1  FF                                                   TERMINATOR_FF             
0x0000288B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000288D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000288F    102  806482DC82E982C58CA982A682C882A2978E82C682B58C8A...  LEN8_STRING_CP932         length=100, text="まるで見えない落とし穴でもあるかのように、\n防護服のふたりは慎重な足取りで庁舎の中へと\n姿を消した。"
0x000028F5      1  FF                                                   TERMINATOR_FF             
0x000028F6      1  FF                                                   TERMINATOR_FF             
0x000028F7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000028F9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000028FB      1  FF                                                   TERMINATOR_FF             
0x000028FC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002900      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002902      1  FF                                                   TERMINATOR_FF             
0x00002903      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002905      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002907      3  F30261                                               IMM16_F3                  u16_be=609, u16_le=24834
0x0000290A      1  FF                                                   TERMINATOR_FF             
0x0000290B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000290D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000290F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002911      1  FF                                                   TERMINATOR_FF             
0x00002912      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002914      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002916      1  FF                                                   TERMINATOR_FF             
0x00002917      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002919      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000291B      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000291D      1  FF                                                   TERMINATOR_FF             
0x0000291E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002920      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002922      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002924      1  FF                                                   TERMINATOR_FF             
0x00002925      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002927      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002929      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000292B      1  FF                                                   TERMINATOR_FF             
0x0000292C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000292E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002930      1  FF                                                   TERMINATOR_FF             
0x00002931      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002933      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002935      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002937      1  FF                                                   TERMINATOR_FF             
0x00002938      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000293A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000293C      3  F3117E                                               IMM16_F3                  u16_be=4478, u16_le=32273
0x0000293F      1  FF                                                   TERMINATOR_FF             
0x00002940      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002942      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002944     94  805C82BB82B582C48141834A83458393835A8389815B82AA...  LEN8_STRING_CP932         length=92, text="そして、カウンセラーがたったひとりで残された。近づいてみて、彼女の雰囲気が違うことに気づく。"
0x000029A2      1  FF                                                   TERMINATOR_FF             
0x000029A3      1  FF                                                   TERMINATOR_FF             
0x000029A4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000029A6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000029A8      1  FF                                                   TERMINATOR_FF             
0x000029A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000029AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000029AD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000029AF      1  FF                                                   TERMINATOR_FF             
0x000029B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000029B2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000029B4      3  F3117F                                               IMM16_F3                  u16_be=4479, u16_le=32529
0x000029B7      1  FF                                                   TERMINATOR_FF             
0x000029B8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000029BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000029BC     78  804C8EA9904D82C9969E82BF82BD955C8FEE82C582CD82C8...  LEN8_STRING_CP932         length=76, text="自信に満ちた表情ではなく、なにかをあきらめたような目をして、うつむいていた。"
0x00002A0A      1  FF                                                   TERMINATOR_FF             
0x00002A0B      1  FF                                                   TERMINATOR_FF             
0x00002A0C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002A0E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002A10      1  FF                                                   TERMINATOR_FF             
0x00002A11      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A13      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A15      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002A17      1  FF                                                   TERMINATOR_FF             
0x00002A18      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A1A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002A1C      3  F31180                                               IMM16_F3                  u16_be=4480, u16_le=32785
0x00002A1F      1  FF                                                   TERMINATOR_FF             
0x00002A20      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A22      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A24     74  804882DA82AD82CD814192A18EC982CC93FC82E88CFB82D6...  LEN8_STRING_CP932         length=72, text="ぼくは、庁舎の入り口へ向かう。\nカウンセラーはぼくに気づいて顔をあげた。"
0x00002A6E      1  FF                                                   TERMINATOR_FF             
0x00002A6F      1  FF                                                   TERMINATOR_FF             
0x00002A70      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002A72      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002A74      1  FF                                                   TERMINATOR_FF             
0x00002A75      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A79      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002A7B      1  FF                                                   TERMINATOR_FF             
0x00002A7C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A7E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A80      3  F30262                                               IMM16_F3                  u16_be=610, u16_le=25090
0x00002A83      1  FF                                                   TERMINATOR_FF             
0x00002A84      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A86      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002A88      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002A8A      1  FF                                                   TERMINATOR_FF             
0x00002A8B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002A8D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002A8F      1  FF                                                   TERMINATOR_FF             
0x00002A90      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A92      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A94      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002A96      1  FF                                                   TERMINATOR_FF             
0x00002A97      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A99      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A9B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002A9D      1  FF                                                   TERMINATOR_FF             
0x00002A9E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002AA0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002AA2      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002AA4      1  FF                                                   TERMINATOR_FF             
0x00002AA5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002AA7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002AA9      1  FF                                                   TERMINATOR_FF             
0x00002AAA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002AAC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002AAE      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002AB0      1  FF                                                   TERMINATOR_FF             
0x00002AB1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002AB3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002AB5      3  F31181                                               IMM16_F3                  u16_be=4481, u16_le=33041
0x00002AB8      1  FF                                                   TERMINATOR_FF             
0x00002AB9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002ABB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002ABD      9  800754413030343630                                   LEN8_STRING_CP932         length=7, text="TA00460"
0x00002AC6      1  FF                                                   TERMINATOR_FF             
0x00002AC7      1  FF                                                   TERMINATOR_FF             
0x00002AC8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002ACA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002ACC     14  800C8175978882BD82CC82CB8176                         LEN8_STRING_CP932         length=12, text="「来たのね」"
0x00002ADA      1  FF                                                   TERMINATOR_FF             
0x00002ADB      1  FF                                                   TERMINATOR_FF             
0x00002ADC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002ADE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002AE0      1  FF                                                   TERMINATOR_FF             
0x00002AE1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002AE3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002AE5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002AE7      1  FF                                                   TERMINATOR_FF             
0x00002AE8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002AEA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002AEC      3  F31182                                               IMM16_F3                  u16_be=4482, u16_le=33297
0x00002AEF      1  FF                                                   TERMINATOR_FF             
0x00002AF0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002AF2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002AF4    108  806A94DE8F9782CD82BB82A482A282C182C4814196B3979D...  LEN8_STRING_CP932         length=106, text="彼女はそういって、無理に微笑もうとした。\nけれど、悲しみに打ち砕かれているせいか、\nうまくはいかなかった。"
0x00002B60      1  FF                                                   TERMINATOR_FF             
0x00002B61      1  FF                                                   TERMINATOR_FF             
0x00002B62      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002B64      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002B66      1  FF                                                   TERMINATOR_FF             
0x00002B67      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B69      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B6B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002B6D      1  FF                                                   TERMINATOR_FF             
0x00002B6E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B70      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002B72      3  F31183                                               IMM16_F3                  u16_be=4483, u16_le=33553
0x00002B75      1  FF                                                   TERMINATOR_FF             
0x00002B76      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B78      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002B7A      9  800754413030343730                                   LEN8_STRING_CP932         length=7, text="TA00470"
0x00002B83      1  FF                                                   TERMINATOR_FF             
0x00002B84      1  FF                                                   TERMINATOR_FF             
0x00002B85      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B87      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B89     30  801C8175937E96D882B382F182AA814191D282C182C482A2...  LEN8_STRING_CP932         length=28, text="「冬木さんが、待っているわ」"
0x00002BA7      1  FF                                                   TERMINATOR_FF             
0x00002BA8      1  FF                                                   TERMINATOR_FF             
0x00002BA9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002BAB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002BAD      1  FF                                                   TERMINATOR_FF             
0x00002BAE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BB0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BB2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002BB4      1  FF                                                   TERMINATOR_FF             
0x00002BB5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BB7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002BB9      3  F34105                                               IMM16_F3                  u16_be=16645, u16_le=1345
0x00002BBC      1  FF                                                   TERMINATOR_FF             
0x00002BBD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002BBF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002BC1     46  802C834A83458393835A8389815B82CD82E082A488EA9378...  LEN8_STRING_CP932         length=44, text="カウンセラーはもう一度、その名前を口にした。"
0x00002BEF      1  FF                                                   TERMINATOR_FF             
0x00002BF0      1  FF                                                   TERMINATOR_FF             
0x00002BF1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002BF3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002BF5      1  FF                                                   TERMINATOR_FF             
0x00002BF6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BF8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BFA      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00002BFC      1  FF                                                   TERMINATOR_FF             
0x00002BFD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BFF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002C01      2  F241                                                 IMM8_F2                   u8=65, s8=65
0x00002C03      1  FF                                                   TERMINATOR_FF             
0x00002C04      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002C06      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002C08      1  FF                                                   TERMINATOR_FF             
0x00002C09      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C0B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C0D      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002C0F      1  FF                                                   TERMINATOR_FF             
0x00002C10      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C12      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002C14      3  F30261                                               IMM16_F3                  u16_be=609, u16_le=24834
0x00002C17      1  FF                                                   TERMINATOR_FF             
0x00002C18      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C1A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002C1C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002C1E      1  FF                                                   TERMINATOR_FF             
0x00002C1F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002C21      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002C23      1  FF                                                   TERMINATOR_FF             
0x00002C24      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C26      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C28      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002C2A      1  FF                                                   TERMINATOR_FF             
0x00002C2B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C2D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002C2F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002C31      1  FF                                                   TERMINATOR_FF             
0x00002C32      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C34      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002C36      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002C38      1  FF                                                   TERMINATOR_FF             
0x00002C39      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002C3B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002C3D      1  FF                                                   TERMINATOR_FF             
0x00002C3E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002C40      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002C42     98  806082DA82AD82CC914F82F08141834A83458393835A8389...  LEN8_STRING_CP932         length=96, text="ぼくの前を、カウンセラーが歩いていた。\n暗い庁舎のフロアを、案内もなく、迷いもせずに歩いていた。"
0x00002CA4      1  FF                                                   TERMINATOR_FF             
0x00002CA5      1  FF                                                   TERMINATOR_FF             
0x00002CA6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002CA8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002CAA      1  FF                                                   TERMINATOR_FF             
0x00002CAB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002CAD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002CAF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002CB1      1  FF                                                   TERMINATOR_FF             
0x00002CB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002CB4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002CB6      3  F34106                                               IMM16_F3                  u16_be=16646, u16_le=1601
0x00002CB9      1  FF                                                   TERMINATOR_FF             
0x00002CBA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002CBC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002CBE     88  805688C382AD90C382A982C892CA984882AA8E7392A18EC9...  LEN8_STRING_CP932         length=86, text="暗く静かな通路が市庁舎の奥にのびていた。\nよく見るお役所の雰囲気とはまるで違った空間。"
0x00002D16      1  FF                                                   TERMINATOR_FF             
0x00002D17      1  FF                                                   TERMINATOR_FF             
0x00002D18      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002D1A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002D1C      1  FF                                                   TERMINATOR_FF             
0x00002D1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D1F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D21      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002D23      1  FF                                                   TERMINATOR_FF             
0x00002D24      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D26      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002D28      3  F31184                                               IMM16_F3                  u16_be=4484, u16_le=33809
0x00002D2B      1  FF                                                   TERMINATOR_FF             
0x00002D2C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002D2E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002D30    106  806896F08F8A82C582CD82C882AD8163816382C882C982A9...  LEN8_STRING_CP932         length=104, text="役所ではなく……なにかの研究施設のようだった。そうか……。本当の彼女は、ここに属しているんだ、と思った。"
0x00002D9A      1  FF                                                   TERMINATOR_FF             
0x00002D9B      1  FF                                                   TERMINATOR_FF             
0x00002D9C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002D9E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002DA0      1  FF                                                   TERMINATOR_FF             
0x00002DA1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DA3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DA5      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002DA7      1  FF                                                   TERMINATOR_FF             
0x00002DA8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DAA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002DAC      3  F31185                                               IMM16_F3                  u16_be=4485, u16_le=34065
0x00002DAF      1  FF                                                   TERMINATOR_FF             
0x00002DB0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002DB2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002DB4      9  800754413030343830                                   LEN8_STRING_CP932         length=7, text="TA00480"
0x00002DBD      1  FF                                                   TERMINATOR_FF             
0x00002DBE      1  FF                                                   TERMINATOR_FF             
0x00002DBF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002DC1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002DC3     16  800E817582AB82DD82CD816381638176                     LEN8_STRING_CP932         length=14, text="「きみは……」"
0x00002DD3      1  FF                                                   TERMINATOR_FF             
0x00002DD4      1  FF                                                   TERMINATOR_FF             
0x00002DD5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002DD7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002DD9      1  FF                                                   TERMINATOR_FF             
0x00002DDA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DDC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DDE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002DE0      1  FF                                                   TERMINATOR_FF             
0x00002DE1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DE3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002DE5      3  F31186                                               IMM16_F3                  u16_be=4486, u16_le=34321
0x00002DE8      1  FF                                                   TERMINATOR_FF             
0x00002DE9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002DEB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002DED     44  802A834A83458393835A8389815B82CD82D382E895D482E8...  LEN8_STRING_CP932         length=42, text="カウンセラーはふり返りもせずにそういった。"
0x00002E19      1  FF                                                   TERMINATOR_FF             
0x00002E1A      1  FF                                                   TERMINATOR_FF             
0x00002E1B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002E1D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002E1F      1  FF                                                   TERMINATOR_FF             
0x00002E20      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002E22      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002E24      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002E26      1  FF                                                   TERMINATOR_FF             
0x00002E27      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002E29      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002E2B      3  F31187                                               IMM16_F3                  u16_be=4487, u16_le=34577
0x00002E2E      1  FF                                                   TERMINATOR_FF             
0x00002E2F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002E31      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002E33      9  800754413030343930                                   LEN8_STRING_CP932         length=7, text="TA00490"
0x00002E3C      1  FF                                                   TERMINATOR_FF             
0x00002E3D      1  FF                                                   TERMINATOR_FF             
0x00002E3E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002E40      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002E42     32  801E81758EA995AA82AA924E82BE82A982ED82A982C182C4...  LEN8_STRING_CP932         length=30, text="「自分が誰だかわかっている？」"
0x00002E62      1  FF                                                   TERMINATOR_FF             
0x00002E63      1  FF                                                   TERMINATOR_FF             
0x00002E64      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002E66      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002E68      1  FF                                                   TERMINATOR_FF             
0x00002E69      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002E6B      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00002E6D     10  800882E082BF82EB82F1                                 LEN8_STRING_CP932         length=8, text="もちろん"
0x00002E77      1  FF                                                   TERMINATOR_FF             
0x00002E78      1  FF                                                   TERMINATOR_FF             
0x00002E79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002E7B      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x00002E7D      3  F31188                                               IMM16_F3                  u16_be=4488, u16_le=34833
0x00002E80      1  FF                                                   TERMINATOR_FF             
0x00002E81      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002E83      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00002E85     28  801A88D396A182AA82E682AD82ED82A982E782C882A282CC...  LEN8_STRING_CP932         length=26, text="意味がよくわからないのだが"
0x00002EA1      1  FF                                                   TERMINATOR_FF             
0x00002EA2      1  FF                                                   TERMINATOR_FF             
0x00002EA3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002EA5      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00002EA7      3  F31189                                               IMM16_F3                  u16_be=4489, u16_le=35089
0x00002EAA      1  FF                                                   TERMINATOR_FF             
0x00002EAB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002EAD      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x00002EAF      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00002EB1      1  FF                                                   TERMINATOR_FF             
0x00002EB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002EB4      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00002EB6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002EB8      1  FF                                                   TERMINATOR_FF             
0x00002EB9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002EBB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002EBD      2  F211                                                 IMM8_F2                   u8=17, s8=17
0x00002EBF      1  FF                                                   TERMINATOR_FF             
0x00002EC0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002EC2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002EC4      1  FF                                                   TERMINATOR_FF             
0x00002EC5      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00002EC7      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00002ECA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002ECC      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00002ECD      1  FF                                                   TERMINATOR_FF             
0x00002ECE      2  0031                                                 WORD_00XX                 u16_be=49, low_byte=49
0x00002ED0      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00002ED1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002ED3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002ED5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002ED7      1  FF                                                   TERMINATOR_FF             
0x00002ED8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002EDA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002EDC      3  F3118A                                               IMM16_F3                  u16_be=4490, u16_le=35345
0x00002EDF      1  FF                                                   TERMINATOR_FF             
0x00002EE0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002EE2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002EE4     32  801E817582ED82A982C182C482DC82B782E6814182BB82EA...  LEN8_STRING_CP932         length=30, text="「わかってますよ、それくらい」"
0x00002F04      1  FF                                                   TERMINATOR_FF             
0x00002F05      1  FF                                                   TERMINATOR_FF             
0x00002F06      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002F08      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002F0A      1  FF                                                   TERMINATOR_FF             
0x00002F0B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F0D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F0F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002F11      1  FF                                                   TERMINATOR_FF             
0x00002F12      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F14      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002F16      3  F3118B                                               IMM16_F3                  u16_be=4491, u16_le=35601
0x00002F19      1  FF                                                   TERMINATOR_FF             
0x00002F1A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002F1C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002F1E      9  800754413030353030                                   LEN8_STRING_CP932         length=7, text="TA00500"
0x00002F27      1  FF                                                   TERMINATOR_FF             
0x00002F28      1  FF                                                   TERMINATOR_FF             
0x00002F29      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002F2B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002F2D     32  801E817582C782A482B582C48163816392668CBE82C582AB...  LEN8_STRING_CP932         length=30, text="「どうして……断言できるの？」"
0x00002F4D      1  FF                                                   TERMINATOR_FF             
0x00002F4E      1  FF                                                   TERMINATOR_FF             
0x00002F4F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002F51      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002F53      1  FF                                                   TERMINATOR_FF             
0x00002F54      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F56      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F58      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002F5A      1  FF                                                   TERMINATOR_FF             
0x00002F5B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F5D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002F5F      3  F3118C                                               IMM16_F3                  u16_be=4492, u16_le=35857
0x00002F62      1  FF                                                   TERMINATOR_FF             
0x00002F63      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002F65      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002F67     82  805082DA82AD82CD82B182C682CE82C982C282DC82E98142...  LEN8_STRING_CP932         length=80, text="ぼくはことばにつまる。\nだって、それを前提にしていなければ、なにも\n始まらない。"
0x00002FB9      1  FF                                                   TERMINATOR_FF             
0x00002FBA      1  FF                                                   TERMINATOR_FF             
0x00002FBB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002FBD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002FBF      1  FF                                                   TERMINATOR_FF             
0x00002FC0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002FC2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002FC4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002FC6      1  FF                                                   TERMINATOR_FF             
0x00002FC7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002FC9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002FCB      3  F3118D                                               IMM16_F3                  u16_be=4493, u16_le=36113
0x00002FCE      1  FF                                                   TERMINATOR_FF             
0x00002FCF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002FD1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002FD3     52  803282BB82F182C882B182C682F096E291E882C982B382EA...  LEN8_STRING_CP932         length=50, text="そんなことを問題にされるなんて思いも\nしなかった。"
0x00003007      1  FF                                                   TERMINATOR_FF             
0x00003008      1  FF                                                   TERMINATOR_FF             
0x00003009      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000300B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000300D      1  FF                                                   TERMINATOR_FF             
0x0000300E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003010      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003012      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003014      1  FF                                                   TERMINATOR_FF             
0x00003015      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003017      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003019      3  F3118E                                               IMM16_F3                  u16_be=4494, u16_le=36369
0x0000301C      1  FF                                                   TERMINATOR_FF             
0x0000301D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000301F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003021      9  800754413030353131                                   LEN8_STRING_CP932         length=7, text="TA00511"
0x0000302A      1  FF                                                   TERMINATOR_FF             
0x0000302B      1  FF                                                   TERMINATOR_FF             
0x0000302C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000302E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003030    106  8068817582AB82DD82CD816381638B4C89AF82C982E082C6...  LEN8_STRING_CP932         length=104, text="「きみは……記憶にもとづいて、そういっているだけだわ。でも、きみは今この瞬間に創造されたのかもしれない」"
0x0000309A      1  FF                                                   TERMINATOR_FF             
0x0000309B      1  FF                                                   TERMINATOR_FF             
0x0000309C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000309E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000030A0      1  FF                                                   TERMINATOR_FF             
0x000030A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030A5      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000030A7      1  FF                                                   TERMINATOR_FF             
0x000030A8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030AA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000030AC      3  F3118F                                               IMM16_F3                  u16_be=4495, u16_le=36625
0x000030AF      1  FF                                                   TERMINATOR_FF             
0x000030B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000030B2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000030B4      9  800754413030353132                                   LEN8_STRING_CP932         length=7, text="TA00512"
0x000030BD      1  FF                                                   TERMINATOR_FF             
0x000030BE      1  FF                                                   TERMINATOR_FF             
0x000030BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000030C1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000030C3    102  8064817582AB82DD82AA90B682DC82EA82C482A982E78141...  LEN8_STRING_CP932         length=100, text="「きみが生まれてから、時間をさかのぼって\n記憶が造られたのかもしれない。そうは考えられないかしら？」"
0x00003129      1  FF                                                   TERMINATOR_FF             
0x0000312A      1  FF                                                   TERMINATOR_FF             
0x0000312B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000312D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000312F      1  FF                                                   TERMINATOR_FF             
0x00003130      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00003132      2  0032                                                 WORD_00XX                 u16_be=50, low_byte=50
0x00003134      1  4E                                                   OPAQUE_RAW_BYTES          bytes=4E
0x00003135      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003137      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x0000313A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000313C      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x0000313D      1  FF                                                   TERMINATOR_FF             
0x0000313E      2  0032                                                 WORD_00XX                 u16_be=50, low_byte=50
0x00003140      1  4E                                                   OPAQUE_RAW_BYTES          bytes=4E
0x00003141      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003143      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003145      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003147      1  FF                                                   TERMINATOR_FF             
0x00003148      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000314A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000314C      3  F31190                                               IMM16_F3                  u16_be=4496, u16_le=36881
0x0000314F      1  FF                                                   TERMINATOR_FF             
0x00003150      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003152      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003154     42  802881758EBF96E282CC88D396A182AA82E682AD82ED82A9...  LEN8_STRING_CP932         length=40, text="「質問の意味がよくわからないんですけど」"
0x0000317E      1  FF                                                   TERMINATOR_FF             
0x0000317F      1  FF                                                   TERMINATOR_FF             
0x00003180      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003182      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003184      1  FF                                                   TERMINATOR_FF             
0x00003185      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003187      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003189      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000318B      1  FF                                                   TERMINATOR_FF             
0x0000318C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000318E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003190      3  F31191                                               IMM16_F3                  u16_be=4497, u16_le=37137
0x00003193      1  FF                                                   TERMINATOR_FF             
0x00003194      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003196      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003198      9  800754413030353230                                   LEN8_STRING_CP932         length=7, text="TA00520"
0x000031A1      1  FF                                                   TERMINATOR_FF             
0x000031A2      1  FF                                                   TERMINATOR_FF             
0x000031A3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000031A5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000031A7     66  8040817582AB82DD82CD8141924E82A982AA8CA982C482A2...  LEN8_STRING_CP932         length=64, text="「きみは、誰かが見ている夢じゃないといえる\nかって、聞いてるの」"
0x000031E9      1  FF                                                   TERMINATOR_FF             
0x000031EA      1  FF                                                   TERMINATOR_FF             
0x000031EB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000031ED      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000031EF      1  FF                                                   TERMINATOR_FF             
0x000031F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000031F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000031F4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000031F6      1  FF                                                   TERMINATOR_FF             
0x000031F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000031F9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000031FB      3  F34107                                               IMM16_F3                  u16_be=16647, u16_le=1857
0x000031FE      1  FF                                                   TERMINATOR_FF             
0x000031FF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003201      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003203     68  8042834A83458393835A8389815B82CD90BA82F08D7282B0...  LEN8_STRING_CP932         length=66, text="カウンセラーは声を荒げる。\nそして、すぐ自嘲気味の笑いを浮かべた。"
0x00003247      1  FF                                                   TERMINATOR_FF             
0x00003248      1  FF                                                   TERMINATOR_FF             
0x00003249      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000324B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000324D      1  FF                                                   TERMINATOR_FF             
0x0000324E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003250      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003252      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003254      1  FF                                                   TERMINATOR_FF             
0x00003255      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003257      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003259      3  F31192                                               IMM16_F3                  u16_be=4498, u16_le=37393
0x0000325C      1  FF                                                   TERMINATOR_FF             
0x0000325D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000325F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003261      9  800754413030353331                                   LEN8_STRING_CP932         length=7, text="TA00531"
0x0000326A      1  FF                                                   TERMINATOR_FF             
0x0000326B      1  FF                                                   TERMINATOR_FF             
0x0000326C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000326E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003270    112  806E817582E082A48163816382A282A282ED814282AB82DD...  LEN8_STRING_CP932         length=110, text="「もう……いいわ。きみは、自分を信じているのだもの。自分がここにいて、常に正しい選択を\nしていると信じている」"
0x000032E0      1  FF                                                   TERMINATOR_FF             
0x000032E1      1  FF                                                   TERMINATOR_FF             
0x000032E2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000032E4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000032E6      1  FF                                                   TERMINATOR_FF             
0x000032E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032EB      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000032ED      1  FF                                                   TERMINATOR_FF             
0x000032EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032F0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000032F2      3  F31193                                               IMM16_F3                  u16_be=4499, u16_le=37649
0x000032F5      1  FF                                                   TERMINATOR_FF             
0x000032F6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000032F8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000032FA      9  800754413030353332                                   LEN8_STRING_CP932         length=7, text="TA00532"
0x00003303      1  FF                                                   TERMINATOR_FF             
0x00003304      1  FF                                                   TERMINATOR_FF             
0x00003305      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003307      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003309     54  8034817582BB82CC816381638BF082A982C882AD82E782A2...  LEN8_STRING_CP932         length=52, text="「その……愚かなくらいの思いこみが、うらやましいわ」"
0x0000333F      1  FF                                                   TERMINATOR_FF             
0x00003340      1  FF                                                   TERMINATOR_FF             
0x00003341      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003343      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003345      1  FF                                                   TERMINATOR_FF             
0x00003346      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003348      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000334A      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000334C      1  FF                                                   TERMINATOR_FF             
0x0000334D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000334F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003351      3  F31194                                               IMM16_F3                  u16_be=4500, u16_le=37905
0x00003354      1  FF                                                   TERMINATOR_FF             
0x00003355      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003357      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003359      9  800754413030353430                                   LEN8_STRING_CP932         length=7, text="TA00540"
0x00003362      1  FF                                                   TERMINATOR_FF             
0x00003363      1  FF                                                   TERMINATOR_FF             
0x00003364      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003366      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003368     72  80468175914F82C982E0986282B582BD82ED814290A28A45...  LEN8_STRING_CP932         length=70, text="「前にも話したわ。世界は滅びたって。理由を\nおしえてあげましょうか？」"
0x000033B0      1  FF                                                   TERMINATOR_FF             
0x000033B1      1  FF                                                   TERMINATOR_FF             
0x000033B2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000033B4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000033B6      1  FF                                                   TERMINATOR_FF             
0x000033B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000033B9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000033BB      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000033BD      1  FF                                                   TERMINATOR_FF             
0x000033BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000033C0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000033C2      3  F31195                                               IMM16_F3                  u16_be=4501, u16_le=38161
0x000033C5      1  FF                                                   TERMINATOR_FF             
0x000033C6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000033C8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000033CA      9  800754413030353530                                   LEN8_STRING_CP932         length=7, text="TA00550"
0x000033D3      1  FF                                                   TERMINATOR_FF             
0x000033D4      1  FF                                                   TERMINATOR_FF             
0x000033D5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000033D7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000033D9     74  804881758E8482BD82BF82C982CD979D89F082C582AB82C8...  LEN8_STRING_CP932         length=72, text="「私たちには理解できない高次のレベルで、\n因果律の法則が変更されたのよ」"
0x00003423      1  FF                                                   TERMINATOR_FF             
0x00003424      1  FF                                                   TERMINATOR_FF             
0x00003425      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003427      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003429      1  FF                                                   TERMINATOR_FF             
0x0000342A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000342C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000342E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003430      1  FF                                                   TERMINATOR_FF             
0x00003431      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003433      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003435      3  F31196                                               IMM16_F3                  u16_be=4502, u16_le=38417
0x00003438      1  FF                                                   TERMINATOR_FF             
0x00003439      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000343B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000343D      9  800754413030353630                                   LEN8_STRING_CP932         length=7, text="TA00560"
0x00003446      1  FF                                                   TERMINATOR_FF             
0x00003447      1  FF                                                   TERMINATOR_FF             
0x00003448      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000344A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000344C    102  80648175917A919C82E082C282A982C882A282ED82E682CB...  LEN8_STRING_CP932         length=100, text="「想像もつかないわよね。ふふっ。私にだって、わからないもの。でも、そうとしか考えられ\nないんだから」"
0x000034B2      1  FF                                                   TERMINATOR_FF             
0x000034B3      1  FF                                                   TERMINATOR_FF             
0x000034B4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000034B6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000034B8      1  FF                                                   TERMINATOR_FF             
0x000034B9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000034BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000034BD      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000034BF      1  FF                                                   TERMINATOR_FF             
0x000034C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000034C2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000034C4      3  F31197                                               IMM16_F3                  u16_be=4503, u16_le=38673
0x000034C7      1  FF                                                   TERMINATOR_FF             
0x000034C8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000034CA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000034CC      9  800754413030353730                                   LEN8_STRING_CP932         length=7, text="TA00570"
0x000034D5      1  FF                                                   TERMINATOR_FF             
0x000034D6      1  FF                                                   TERMINATOR_FF             
0x000034D7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000034D9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000034DB    120  807681758E8482CD82A882DA82A682C482A282E982CC8142...  LEN8_STRING_CP932         length=118, text="「私はおぼえているの。後からうえつけられた\n記憶だったとしてもよ。秩序を失い、混沌の渦へと飲み込まれていくあの世界を」"
0x00003553      1  FF                                                   TERMINATOR_FF             
0x00003554      1  FF                                                   TERMINATOR_FF             
0x00003555      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003557      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003559      1  FF                                                   TERMINATOR_FF             
0x0000355A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000355C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000355E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003560      1  FF                                                   TERMINATOR_FF             
0x00003561      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003563      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003565      3  F31198                                               IMM16_F3                  u16_be=4504, u16_le=38929
0x00003568      1  FF                                                   TERMINATOR_FF             
0x00003569      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000356B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000356D      9  800754413030353831                                   LEN8_STRING_CP932         length=7, text="TA00581"
0x00003576      1  FF                                                   TERMINATOR_FF             
0x00003577      1  FF                                                   TERMINATOR_FF             
0x00003578      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000357A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000357C    111  806D81758E63944F82BE82AF82C78141906C82CC905382CD...  LEN8_STRING_CP932         length=109, text="「残念だけど、人の心は……精神活動っていい\nなおしましょうか？ 混沌に耐えうるだけの強度を\n有していないのよ」"
0x000035EB      1  FF                                                   TERMINATOR_FF             
0x000035EC      1  FF                                                   TERMINATOR_FF             
0x000035ED      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000035EF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000035F1      1  FF                                                   TERMINATOR_FF             
0x000035F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000035F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000035F6      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000035F8      1  FF                                                   TERMINATOR_FF             
0x000035F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000035FB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000035FD      3  F31199                                               IMM16_F3                  u16_be=4505, u16_le=39185
0x00003600      1  FF                                                   TERMINATOR_FF             
0x00003601      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003603      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003605      9  800754413030353832                                   LEN8_STRING_CP932         length=7, text="TA00582"
0x0000360E      1  FF                                                   TERMINATOR_FF             
0x0000360F      1  FF                                                   TERMINATOR_FF             
0x00003610      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003612      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003614     80  804E817582C582E082CB814190A28A4582AA979D89F082C5...  LEN8_STRING_CP932         length=78, text="「でもね、世界が理解できなくなったとき、\n私たちには心しか残されていなかった」"
0x00003664      1  FF                                                   TERMINATOR_FF             
0x00003665      1  FF                                                   TERMINATOR_FF             
0x00003666      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003668      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000366A      1  FF                                                   TERMINATOR_FF             
0x0000366B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000366D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000366F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003671      1  FF                                                   TERMINATOR_FF             
0x00003672      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003674      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003676      3  F3119A                                               IMM16_F3                  u16_be=4506, u16_le=39441
0x00003679      1  FF                                                   TERMINATOR_FF             
0x0000367A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000367C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000367E     42  802881758E8482CC82A282C182C482E982B182C68141979D...  LEN8_STRING_CP932         length=40, text="「私のいってること、理解できるかしら？」"
0x000036A8      1  FF                                                   TERMINATOR_FF             
0x000036A9      1  FF                                                   TERMINATOR_FF             
0x000036AA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000036AC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000036AE      9  800754413030353930                                   LEN8_STRING_CP932         length=7, text="TA00590"
0x000036B7      1  FF                                                   TERMINATOR_FF             
0x000036B8      1  FF                                                   TERMINATOR_FF             
0x000036B9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000036BB      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x000036BD     28  801A979D89F082C582AB82E982A982E082B582EA82C882A2...  LEN8_STRING_CP932         length=26, text="理解できるかもしれない……"
0x000036D9      1  FF                                                   TERMINATOR_FF             
0x000036DA      1  FF                                                   TERMINATOR_FF             
0x000036DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000036DD      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x000036DF      3  F3119B                                               IMM16_F3                  u16_be=4507, u16_le=39697
0x000036E2      1  FF                                                   TERMINATOR_FF             
0x000036E3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000036E5      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x000036E7     22  801489BD82F082A282C182C482A282E982F182BE8148         LEN8_STRING_CP932         length=20, text="何をいっているんだ？"
0x000036FD      1  FF                                                   TERMINATOR_FF             
0x000036FE      1  FF                                                   TERMINATOR_FF             
0x000036FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003701      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00003703      3  F3119C                                               IMM16_F3                  u16_be=4508, u16_le=39953
0x00003706      1  FF                                                   TERMINATOR_FF             
0x00003707      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003709      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x0000370B      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000370D      1  FF                                                   TERMINATOR_FF             
0x0000370E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003710      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00003712      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003714      1  FF                                                   TERMINATOR_FF             
0x00003715      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003717      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003719      2  F213                                                 IMM8_F2                   u8=19, s8=19
0x0000371B      1  FF                                                   TERMINATOR_FF             
0x0000371C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000371E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003720      1  FF                                                   TERMINATOR_FF             
0x00003721      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003723      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00003726      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003728      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003729      1  FF                                                   TERMINATOR_FF             
0x0000372A      2  0037                                                 WORD_00XX                 u16_be=55, low_byte=55
0x0000372C      1  32                                                   OPAQUE_RAW_BYTES          bytes=32
0x0000372D      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x0000372F      2  0037                                                 WORD_00XX                 u16_be=55, low_byte=55
0x00003731      1  3E                                                   OPAQUE_RAW_BYTES          bytes=3E
0x00003732      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003734      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00003737      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003739      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x0000373A      1  FF                                                   TERMINATOR_FF             
0x0000373B      2  0037                                                 WORD_00XX                 u16_be=55, low_byte=55
0x0000373D      1  3E                                                   OPAQUE_RAW_BYTES          bytes=3E
0x0000373E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003740      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003742      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003744      1  FF                                                   TERMINATOR_FF             
0x00003745      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003747      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003749      3  F3119D                                               IMM16_F3                  u16_be=4509, u16_le=40209
0x0000374C      1  FF                                                   TERMINATOR_FF             
0x0000374D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000374F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003751      9  800754413030363030                                   LEN8_STRING_CP932         length=7, text="TA00600"
0x0000375A      1  FF                                                   TERMINATOR_FF             
0x0000375B      1  FF                                                   TERMINATOR_FF             
0x0000375C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000375E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003760    116  8072817582BB82A4814282C782C182BF82C582E082A282A2...  LEN8_STRING_CP932         length=114, text="「そう。どっちでもいいわ。私にはもう、関係のないことだから。そういうことなんだって、\nあきらめるしかないんだもの」"
0x000037D4      1  FF                                                   TERMINATOR_FF             
0x000037D5      1  FF                                                   TERMINATOR_FF             
0x000037D6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000037D8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000037DA      1  FF                                                   TERMINATOR_FF             
0x000037DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000037DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000037DF      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000037E1      1  FF                                                   TERMINATOR_FF             
0x000037E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000037E4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000037E6      3  F3119E                                               IMM16_F3                  u16_be=4510, u16_le=40465
0x000037E9      1  FF                                                   TERMINATOR_FF             
0x000037EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000037EC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000037EE      9  800754413030363130                                   LEN8_STRING_CP932         length=7, text="TA00610"
0x000037F7      1  FF                                                   TERMINATOR_FF             
0x000037F8      1  FF                                                   TERMINATOR_FF             
0x000037F9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000037FB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000037FD     38  8024817582B182B182E68142937E96D882B382F182AA8141...  LEN8_STRING_CP932         length=36, text="「ここよ。冬木さんが、待っているわ」"
0x00003823      1  FF                                                   TERMINATOR_FF             
0x00003824      1  FF                                                   TERMINATOR_FF             
0x00003825      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003827      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003829      1  FF                                                   TERMINATOR_FF             
0x0000382A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000382C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000382E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003830      1  FF                                                   TERMINATOR_FF             
0x00003831      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003833      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003835      3  F34108                                               IMM16_F3                  u16_be=16648, u16_le=2113
0x00003838      1  FF                                                   TERMINATOR_FF             
0x00003839      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000383B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000383D     58  8038834A83458393835A8389815B82CD82BB82A482A282C1...  LEN8_STRING_CP932         length=56, text="カウンセラーはそういって、ひとつの扉の前で\n立ちどまる。"
0x00003877      1  FF                                                   TERMINATOR_FF             
0x00003878      1  FF                                                   TERMINATOR_FF             
0x00003879      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000387B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000387D      1  FF                                                   TERMINATOR_FF             
0x0000387E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003880      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003882      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003884      1  FF                                                   TERMINATOR_FF             
0x00003885      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003887      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003889      3  F34109                                               IMM16_F3                  u16_be=16649, u16_le=2369
0x0000388C      1  FF                                                   TERMINATOR_FF             
0x0000388D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000388F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003891     84  805291BC82CC94E082C682C782A488E182A482CC82A982CD...  LEN8_STRING_CP932         length=82, text="他の扉とどう違うのかはわからないが、\nカウンセラーには間違いようのないことだった。"
0x000038E5      1  FF                                                   TERMINATOR_FF             
0x000038E6      1  FF                                                   TERMINATOR_FF             
0x000038E7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000038E9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000038EB      1  FF                                                   TERMINATOR_FF             
0x000038EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000038EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000038F0      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000038F2      1  FF                                                   TERMINATOR_FF             
0x000038F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000038F5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000038F7      3  F3119F                                               IMM16_F3                  u16_be=4511, u16_le=40721
0x000038FA      1  FF                                                   TERMINATOR_FF             
0x000038FB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000038FD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000038FF      9  800754413030363230                                   LEN8_STRING_CP932         length=7, text="TA00620"
0x00003908      1  FF                                                   TERMINATOR_FF             
0x00003909      1  FF                                                   TERMINATOR_FF             
0x0000390A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000390C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000390E     68  804281758A6D82A982DF82C482DD82E982C682A282A282ED...  LEN8_STRING_CP932         length=66, text="「確かめてみるといいわ。きみが信じてる、\n自分の正しさってやつを」"
0x00003952      1  FF                                                   TERMINATOR_FF             
0x00003953      1  FF                                                   TERMINATOR_FF             
0x00003954      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003956      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003958      1  FF                                                   TERMINATOR_FF             
0x00003959      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000395B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000395D      8  800653452D533131                                     LEN8_STRING_CP932         length=6, text="SE-S11"
0x00003965      1  FF                                                   TERMINATOR_FF             
0x00003966      1  FF                                                   TERMINATOR_FF             
0x00003967      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003969      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000396B      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x0000396D      1  FF                                                   TERMINATOR_FF             
0x0000396E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003970      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003972      1  FF                                                   TERMINATOR_FF             
0x00003973      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003975      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003977      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003979      1  FF                                                   TERMINATOR_FF             
0x0000397A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000397C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000397E      3  F3410A                                               IMM16_F3                  u16_be=16650, u16_le=2625
0x00003981      1  FF                                                   TERMINATOR_FF             
0x00003982      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003984      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003986     52  8032834A83458393835A8389815B82CD94E082F08A4A82AF...  LEN8_STRING_CP932         length=50, text="カウンセラーは扉を開けたぼくの背中にそう\nいった。"
0x000039BA      1  FF                                                   TERMINATOR_FF             
0x000039BB      1  FF                                                   TERMINATOR_FF             
0x000039BC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000039BE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000039C0      1  FF                                                   TERMINATOR_FF             
0x000039C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000039C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000039C5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000039C7      1  FF                                                   TERMINATOR_FF             
0x000039C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000039CA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000039CC      3  F3410B                                               IMM16_F3                  u16_be=16651, u16_le=2881
0x000039CF      1  FF                                                   TERMINATOR_FF             
0x000039D0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000039D2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000039D4     70  8044937B82E882C694DF82B582DD82C6814182A082AB82E7...  LEN8_STRING_CP932         length=68, text="怒りと悲しみと、あきらめと……。\nそんな感情がいりまじった声だった。"
0x00003A1A      1  FF                                                   TERMINATOR_FF             
0x00003A1B      1  FF                                                   TERMINATOR_FF             
0x00003A1C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003A1E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003A20      1  FF                                                   TERMINATOR_FF             
0x00003A21      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A23      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A25      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00003A27      1  FF                                                   TERMINATOR_FF             
0x00003A28      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A2A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003A2C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003A2E      1  FF                                                   TERMINATOR_FF             
0x00003A2F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003A31      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003A33      1  FF                                                   TERMINATOR_FF             
0x00003A34      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A38      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00003A3A      1  FF                                                   TERMINATOR_FF             
0x00003A3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A3D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003A3F      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00003A41      1  FF                                                   TERMINATOR_FF             
0x00003A42      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003A44      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003A46      1  FF                                                   TERMINATOR_FF             
0x00003A47      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A49      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A4B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00003A4D      1  FF                                                   TERMINATOR_FF             
0x00003A4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A50      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003A52      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003A54      1  FF                                                   TERMINATOR_FF             
0x00003A55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A57      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003A59      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003A5B      1  FF                                                   TERMINATOR_FF             
0x00003A5C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003A5E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003A60      1  FF                                                   TERMINATOR_FF             
0x00003A61      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A63      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A65      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00003A67      1  FF                                                   TERMINATOR_FF             
0x00003A68      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A6A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003A6C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003A6E      1  FF                                                   TERMINATOR_FF             
0x00003A6F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A71      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003A73      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00003A75      1  FF                                                   TERMINATOR_FF             
0x00003A76      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003A78      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003A7A      1  FF                                                   TERMINATOR_FF             
0x00003A7B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A7D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A7F      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00003A81      1  FF                                                   TERMINATOR_FF             
0x00003A82      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A84      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003A86      3  F30125                                               IMM16_F3                  u16_be=293, u16_le=9473
0x00003A89      1  FF                                                   TERMINATOR_FF             
0x00003A8A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003A8C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003A8E      1  FF                                                   TERMINATOR_FF             
0x00003A8F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A91      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A93      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00003A95      1  FF                                                   TERMINATOR_FF             
0x00003A96      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A98      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003A9A      2  F219                                                 IMM8_F2                   u8=25, s8=25
0x00003A9C      1  FF                                                   TERMINATOR_FF             
0x00003A9D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003A9F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003AA1      1  FF                                                   TERMINATOR_FF             
0x00003AA2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003AA4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003AA6      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00003AA8      1  FF                                                   TERMINATOR_FF             
0x00003AA9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003AAB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003AAD      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003AAF      1  FF                                                   TERMINATOR_FF             
0x00003AB0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003AB2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003AB4      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00003AB6      1  FF                                                   TERMINATOR_FF             
0x00003AB7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003AB9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003ABB      1  FF                                                   TERMINATOR_FF             
0x00003ABC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003ABE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003AC0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003AC2      1  FF                                                   TERMINATOR_FF             
0x00003AC3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003AC5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003AC7      3  F311A0                                               IMM16_F3                  u16_be=4512, u16_le=40977
0x00003ACA      1  FF                                                   TERMINATOR_FF             
0x00003ACB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003ACD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003ACF     66  804082BB82B182CD81418AF782AA82D082C682C2927582A2...  LEN8_STRING_CP932         length=64, text="そこは、机がひとつ置いてあるだけの、がらんとしたオフィスだった。"
0x00003B11      1  FF                                                   TERMINATOR_FF             
0x00003B12      1  FF                                                   TERMINATOR_FF             
0x00003B13      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003B15      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003B17      1  FF                                                   TERMINATOR_FF             
0x00003B18      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B1A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B1C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003B1E      1  FF                                                   TERMINATOR_FF             
0x00003B1F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B21      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003B23      3  F311A1                                               IMM16_F3                  u16_be=4513, u16_le=41233
0x00003B26      1  FF                                                   TERMINATOR_FF             
0x00003B27      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003B29      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003B2B     52  80328375838983438393836882CC82B782AB82DC82A982E7...  LEN8_STRING_CP932         length=50, text="ブラインドのすきまから差し込む光がまぶし\nかった。"
0x00003B5F      1  FF                                                   TERMINATOR_FF             
0x00003B60      1  FF                                                   TERMINATOR_FF             
0x00003B61      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003B63      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003B65      1  FF                                                   TERMINATOR_FF             
0x00003B66      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B68      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B6A      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003B6C      1  FF                                                   TERMINATOR_FF             
0x00003B6D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B6F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003B71      3  F311A2                                               IMM16_F3                  u16_be=4514, u16_le=41489
0x00003B74      1  FF                                                   TERMINATOR_FF             
0x00003B75      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003B77      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003B79     10  80084655303036393054                                 LEN8_STRING_CP932         length=8, text="FU00690T"
0x00003B83      1  FF                                                   TERMINATOR_FF             
0x00003B84      1  FF                                                   TERMINATOR_FF             
0x00003B85      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003B87      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003B89     80  804E817582AB82C182C68141978882C482AD82EA82E982C6...  LEN8_STRING_CP932         length=78, text="「きっと、来てくれると思ってました。\n『パンドラ計画』チーフの冬木イクオです」"
0x00003BD9      1  FF                                                   TERMINATOR_FF             
0x00003BDA      1  FF                                                   TERMINATOR_FF             
0x00003BDB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003BDD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003BDF      1  FF                                                   TERMINATOR_FF             
0x00003BE0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003BE2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003BE4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003BE6      1  FF                                                   TERMINATOR_FF             
0x00003BE7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003BE9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003BEB      3  F311A3                                               IMM16_F3                  u16_be=4515, u16_le=41745
0x00003BEE      1  FF                                                   TERMINATOR_FF             
0x00003BEF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003BF1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003BF3     84  8052896582CC928682A982E790BA82AA82B782E981425C6E...  LEN8_STRING_CP932         length=82, text="影の中から声がする。\nデスクに、人のシルエットがあった。\n逆光で表情はわからない。"
0x00003C47      1  FF                                                   TERMINATOR_FF             
0x00003C48      1  FF                                                   TERMINATOR_FF             
0x00003C49      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003C4B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003C4D      1  FF                                                   TERMINATOR_FF             
0x00003C4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C50      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C52      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003C54      1  FF                                                   TERMINATOR_FF             
0x00003C55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C57      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003C59      3  F311A4                                               IMM16_F3                  u16_be=4516, u16_le=42001
0x00003C5C      1  FF                                                   TERMINATOR_FF             
0x00003C5D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003C5F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003C61     40  802682AF82C78141934788D382AA82A082E982E682A482C9...  LEN8_STRING_CP932         length=38, text="けど、敵意があるようには思えなかった。"
0x00003C89      1  FF                                                   TERMINATOR_FF             
0x00003C8A      1  FF                                                   TERMINATOR_FF             
0x00003C8B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003C8D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003C8F      1  FF                                                   TERMINATOR_FF             
0x00003C90      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C92      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C94      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003C96      1  FF                                                   TERMINATOR_FF             
0x00003C97      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C99      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003C9B      3  F311A5                                               IMM16_F3                  u16_be=4517, u16_le=42257
0x00003C9E      1  FF                                                   TERMINATOR_FF             
0x00003C9F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003CA1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003CA3     10  80084655303037303054                                 LEN8_STRING_CP932         length=8, text="FU00700T"
0x00003CAD      1  FF                                                   TERMINATOR_FF             
0x00003CAE      1  FF                                                   TERMINATOR_FF             
0x00003CAF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003CB1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003CB3    108  806A81758E9E8AD482E082A082E882DC82B982F182CC82C5...  LEN8_STRING_CP932         length=106, text="「時間もありませんので、ぼくたちが直面して\nいる問題について説明しようと思うのですが、\n構わないですか？」"
0x00003D1F      1  FF                                                   TERMINATOR_FF             
0x00003D20      1  FF                                                   TERMINATOR_FF             
0x00003D21      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003D23      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003D25      1  FF                                                   TERMINATOR_FF             
0x00003D26      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003D28      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00003D2A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003D2C      1  FF                                                   TERMINATOR_FF             
0x00003D2D      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003D2F      3  F10080                                               IMM16_F1                  u16_be=128, u16_le=32768
0x00003D32      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003D34      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003D35      3  F10081                                               IMM16_F1                  u16_be=129, u16_le=33024
0x00003D38      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003D3A      2  3551                                                 OPAQUE_RAW_BYTES          bytes=3551
0x00003D3C      3  F10082                                               IMM16_F1                  u16_be=130, u16_le=33280
0x00003D3F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003D41      2  3551                                                 OPAQUE_RAW_BYTES          bytes=3551
0x00003D43      1  FF                                                   TERMINATOR_FF             
0x00003D44      2  003D                                                 WORD_00XX                 u16_be=61, low_byte=61
0x00003D46      1  4C                                                   OPAQUE_RAW_BYTES          bytes=4C
0x00003D47      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00003D49      2  0047                                                 WORD_00XX                 u16_be=71, low_byte=71
0x00003D4B      1  0A                                                   OPAQUE_RAW_BYTES          bytes=0A
0x00003D4C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003D4E      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00003D50     26  801895B782AB82BD82A282B182C682AA82A082E982F182C5...  LEN8_STRING_CP932         length=24, text="聞きたいことがあるんです"
0x00003D6A      1  FF                                                   TERMINATOR_FF             
0x00003D6B      1  FF                                                   TERMINATOR_FF             
0x00003D6C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003D6E      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x00003D70      3  F311A6                                               IMM16_F3                  u16_be=4518, u16_le=42513
0x00003D73      1  FF                                                   TERMINATOR_FF             
0x00003D74      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003D76      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00003D78     14  800C82ED82A982E882DC82B582BD                         LEN8_STRING_CP932         length=12, text="わかりました"
0x00003D86      1  FF                                                   TERMINATOR_FF             
0x00003D87      1  FF                                                   TERMINATOR_FF             
0x00003D88      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003D8A      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00003D8C      3  F311A7                                               IMM16_F3                  u16_be=4519, u16_le=42769
0x00003D8F      1  FF                                                   TERMINATOR_FF             
0x00003D90      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003D92      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x00003D94      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00003D96      1  FF                                                   TERMINATOR_FF             
0x00003D97      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003D99      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00003D9B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003D9D      1  FF                                                   TERMINATOR_FF             
0x00003D9E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003DA0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003DA2      2  F211                                                 IMM8_F2                   u8=17, s8=17
0x00003DA4      1  FF                                                   TERMINATOR_FF             
0x00003DA5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003DA7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003DA9      1  FF                                                   TERMINATOR_FF             
0x00003DAA      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003DAC      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00003DAF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003DB1      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003DB2      1  FF                                                   TERMINATOR_FF             
0x00003DB3      2  0046                                                 WORD_00XX                 u16_be=70, low_byte=70
0x00003DB5      1  9F                                                   OPAQUE_RAW_BYTES          bytes=9F
0x00003DB6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003DB8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003DBA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003DBC      1  FF                                                   TERMINATOR_FF             
0x00003DBD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003DBF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003DC1      3  F311A8                                               IMM16_F3                  u16_be=4520, u16_le=43025
0x00003DC4      1  FF                                                   TERMINATOR_FF             
0x00003DC5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003DC7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003DC9     10  80084655303037313054                                 LEN8_STRING_CP932         length=8, text="FU00710T"
0x00003DD3      1  FF                                                   TERMINATOR_FF             
0x00003DD4      1  FF                                                   TERMINATOR_FF             
0x00003DD5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003DD7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003DD9    116  80728175816381639396915282C582B582E582A482CB8142...  LEN8_STRING_CP932         length=114, text="「……当然でしょうね。少し急ぎすぎたのかも\nしれません。どうぞ。できる限りのことはお話\nしたいと思っていますから」"
0x00003E4D      1  FF                                                   TERMINATOR_FF             
0x00003E4E      1  FF                                                   TERMINATOR_FF             
0x00003E4F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003E51      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003E53      1  FF                                                   TERMINATOR_FF             
0x00003E54      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003E56      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00003E58      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003E5A      1  FF                                                   TERMINATOR_FF             
0x00003E5B      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003E5D      3  F10080                                               IMM16_F1                  u16_be=128, u16_le=32768
0x00003E60      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003E62      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003E63      1  FF                                                   TERMINATOR_FF             
0x00003E64      2  003E                                                 WORD_00XX                 u16_be=62, low_byte=62
0x00003E66      1  75                                                   OPAQUE_RAW_BYTES          bytes=75
0x00003E67      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003E69      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00003E6B      3  F10009                                               IMM16_F1                  u16_be=9, u16_le=2304
0x00003E6E      1  F0                                                   OPAQUE_RAW_BYTES          bytes=F0
0x00003E6F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003E71      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00003E73      1  43                                                   OPAQUE_RAW_BYTES          bytes=43
0x00003E74      1  FF                                                   TERMINATOR_FF             
0x00003E75      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003E77      3  F10081                                               IMM16_F1                  u16_be=129, u16_le=33024
0x00003E7A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003E7C      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003E7D      1  FF                                                   TERMINATOR_FF             
0x00003E7E      2  003E                                                 WORD_00XX                 u16_be=62, low_byte=62
0x00003E80      1  8F                                                   OPAQUE_RAW_BYTES          bytes=8F
0x00003E81      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003E83      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00003E85      3  F10009                                               IMM16_F1                  u16_be=9, u16_le=2304
0x00003E88      1  F0                                                   OPAQUE_RAW_BYTES          bytes=F0
0x00003E89      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003E8B      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00003E8D      1  43                                                   OPAQUE_RAW_BYTES          bytes=43
0x00003E8E      1  FF                                                   TERMINATOR_FF             
0x00003E8F      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003E91      3  F10082                                               IMM16_F1                  u16_be=130, u16_le=33280
0x00003E94      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003E96      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003E97      1  FF                                                   TERMINATOR_FF             
0x00003E98      2  003E                                                 WORD_00XX                 u16_be=62, low_byte=62
0x00003E9A      1  A9                                                   OPAQUE_RAW_BYTES          bytes=A9
0x00003E9B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003E9D      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00003E9F      3  F10009                                               IMM16_F1                  u16_be=9, u16_le=2304
0x00003EA2      1  F0                                                   OPAQUE_RAW_BYTES          bytes=F0
0x00003EA3      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00003EA5      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00003EA7      1  43                                                   OPAQUE_RAW_BYTES          bytes=43
0x00003EA8      1  FF                                                   TERMINATOR_FF             
0x00003EA9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003EAB      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00003EAD     14  800C88BB82CD82C782B182C98148                         LEN8_STRING_CP932         length=12, text="綾はどこに？"
0x00003EBB      1  FF                                                   TERMINATOR_FF             
0x00003EBC      1  FF                                                   TERMINATOR_FF             
0x00003EBD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003EBF      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x00003EC1      3  F311A9                                               IMM16_F3                  u16_be=4521, u16_le=43281
0x00003EC4      1  FF                                                   TERMINATOR_FF             
0x00003EC5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003EC7      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00003EC9     30  801C88BB82CD82A282C182BD82A289BD8ED282C882F182C5...  LEN8_STRING_CP932         length=28, text="綾はいったい何者なんですか？"
0x00003EE7      1  FF                                                   TERMINATOR_FF             
0x00003EE8      1  FF                                                   TERMINATOR_FF             
0x00003EE9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003EEB      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00003EED      3  F311AA                                               IMM16_F3                  u16_be=4522, u16_le=43537
0x00003EF0      1  FF                                                   TERMINATOR_FF             
0x00003EF1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003EF3      2  0006                                                 WORD_00XX                 u16_be=6, low_byte=6
0x00003EF5     34  802082B182CC8A5882CD82C782A482C882C182C482B582DC...  LEN8_STRING_CP932         length=32, text="この街はどうなってしまうんです？"
0x00003F17      1  FF                                                   TERMINATOR_FF             
0x00003F18      1  FF                                                   TERMINATOR_FF             
0x00003F19      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F1B      2  000C                                                 WORD_00XX                 u16_be=12, low_byte=12
0x00003F1D      3  F311AB                                               IMM16_F3                  u16_be=4523, u16_le=43793
0x00003F20      1  FF                                                   TERMINATOR_FF             
0x00003F21      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F23      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x00003F25      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00003F27      1  FF                                                   TERMINATOR_FF             
0x00003F28      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F2A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F2C      2  F211                                                 IMM8_F2                   u8=17, s8=17
0x00003F2E      1  FF                                                   TERMINATOR_FF             
0x00003F2F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003F31      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003F33      1  FF                                                   TERMINATOR_FF             
0x00003F34      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003F36      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00003F39      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003F3B      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003F3C      1  FF                                                   TERMINATOR_FF             
0x00003F3D      2  0041                                                 WORD_00XX                 u16_be=65, low_byte=65
0x00003F3F      1  94                                                   OPAQUE_RAW_BYTES          bytes=94
0x00003F40      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F42      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00003F44      3  F10009                                               IMM16_F1                  u16_be=9, u16_le=2304
0x00003F47      1  F0                                                   OPAQUE_RAW_BYTES          bytes=F0
0x00003F48      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F4A      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00003F4C      1  43                                                   OPAQUE_RAW_BYTES          bytes=43
0x00003F4D      1  FF                                                   TERMINATOR_FF             
0x00003F4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F50      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F52      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003F54      1  FF                                                   TERMINATOR_FF             
0x00003F55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F57      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003F59      3  F311AC                                               IMM16_F3                  u16_be=4524, u16_le=44049
0x00003F5C      1  FF                                                   TERMINATOR_FF             
0x00003F5D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003F5F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003F61     10  80084655303037323054                                 LEN8_STRING_CP932         length=8, text="FU00720T"
0x00003F6B      1  FF                                                   TERMINATOR_FF             
0x00003F6C      1  FF                                                   TERMINATOR_FF             
0x00003F6D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003F6F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003F71     68  8042817582ED82A982E882DC82B982F1814294DE8F9782AA...  LEN8_STRING_CP932         length=66, text="「わかりません。彼女が近くにいることは\nわかっているのですが……」"
0x00003FB5      1  FF                                                   TERMINATOR_FF             
0x00003FB6      1  FF                                                   TERMINATOR_FF             
0x00003FB7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003FB9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003FBB      1  FF                                                   TERMINATOR_FF             
0x00003FBC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003FBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003FC0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003FC2      1  FF                                                   TERMINATOR_FF             
0x00003FC3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003FC5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003FC7      3  F311AD                                               IMM16_F3                  u16_be=4525, u16_le=44305
0x00003FCA      1  FF                                                   TERMINATOR_FF             
0x00003FCB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003FCD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003FCF     46  802C817582BB82EA82CD81418A5882A982E7906C82AA8FC1...  LEN8_STRING_CP932         length=44, text="「それは、街から人が消えたことと関係して？」"
0x00003FFD      1  FF                                                   TERMINATOR_FF             
0x00003FFE      1  FF                                                   TERMINATOR_FF             
0x00003FFF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004001      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004003      1  FF                                                   TERMINATOR_FF             
0x00004004      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004006      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004008      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000400A      1  FF                                                   TERMINATOR_FF             
0x0000400B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000400D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000400F      3  F311AE                                               IMM16_F3                  u16_be=4526, u16_le=44561
0x00004012      1  FF                                                   TERMINATOR_FF             
0x00004013      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004015      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004017     10  80084655303037333154                                 LEN8_STRING_CP932         length=8, text="FU00731T"
0x00004021      1  FF                                                   TERMINATOR_FF             
0x00004022      1  FF                                                   TERMINATOR_FF             
0x00004023      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004025      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004027     60  803A817582BB82A482C582B7814294DE8F9782CC91B68DDD...  LEN8_STRING_CP932         length=58, text="「そうです。彼女の存在が、ぼくたちの街を変質させています」"
0x00004063      1  FF                                                   TERMINATOR_FF             
0x00004064      1  FF                                                   TERMINATOR_FF             
0x00004065      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004067      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004069      1  FF                                                   TERMINATOR_FF             
0x0000406A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000406C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000406E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004070      1  FF                                                   TERMINATOR_FF             
0x00004071      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004073      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004075      3  F311AF                                               IMM16_F3                  u16_be=4527, u16_le=44817
0x00004078      1  FF                                                   TERMINATOR_FF             
0x00004079      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000407B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000407D     10  80084655303037333254                                 LEN8_STRING_CP932         length=8, text="FU00732T"
0x00004087      1  FF                                                   TERMINATOR_FF             
0x00004088      1  FF                                                   TERMINATOR_FF             
0x00004089      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000408B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000408D    106  8068817582A082CC8A5882C991AE82B782E994DE8F9782CC...  LEN8_STRING_CP932         length=104, text="「あの街に属する彼女の存在にくらべれば、\nぼくたちがつくりあげた世界なんて、もろく、\nはかないものです」"
0x000040F7      1  FF                                                   TERMINATOR_FF             
0x000040F8      1  FF                                                   TERMINATOR_FF             
0x000040F9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000040FB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000040FD      1  FF                                                   TERMINATOR_FF             
0x000040FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004100      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004102      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004104      1  FF                                                   TERMINATOR_FF             
0x00004105      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004107      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004109      3  F311B0                                               IMM16_F3                  u16_be=4528, u16_le=45073
0x0000410C      1  FF                                                   TERMINATOR_FF             
0x0000410D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000410F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004111     28  801A817582A082CC8A5882C991AE82B782E982C182C48163...  LEN8_STRING_CP932         length=26, text="「あの街に属するって……」"
0x0000412D      1  FF                                                   TERMINATOR_FF             
0x0000412E      1  FF                                                   TERMINATOR_FF             
0x0000412F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004131      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004133      1  FF                                                   TERMINATOR_FF             
0x00004134      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004136      2  0080                                                 WORD_00XX                 u16_be=128, low_byte=128
0x00004138      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000413A      1  FF                                                   TERMINATOR_FF             
0x0000413B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000413D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000413F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004141      1  FF                                                   TERMINATOR_FF             
0x00004142      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004144      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004146      3  F311B1                                               IMM16_F3                  u16_be=4529, u16_le=45329
0x00004149      1  FF                                                   TERMINATOR_FF             
0x0000414A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000414C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000414E     10  80084655303037343054                                 LEN8_STRING_CP932         length=8, text="FU00740T"
0x00004158      1  FF                                                   TERMINATOR_FF             
0x00004159      1  FF                                                   TERMINATOR_FF             
0x0000415A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000415C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000415E     42  8028817582BB82A4814182A082CC8A5882AA94DE8F9782F0...  LEN8_STRING_CP932         length=40, text="「そう、あの街が彼女を生んだのですから」"
0x00004188      1  FF                                                   TERMINATOR_FF             
0x00004189      1  FF                                                   TERMINATOR_FF             
0x0000418A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000418C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000418E      1  FF                                                   TERMINATOR_FF             
0x0000418F      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00004191      2  0046                                                 WORD_00XX                 u16_be=70, low_byte=70
0x00004193      1  19                                                   OPAQUE_RAW_BYTES          bytes=19
0x00004194      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00004196      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00004199      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000419B      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x0000419C      1  FF                                                   TERMINATOR_FF             
0x0000419D      2  0043                                                 WORD_00XX                 u16_be=67, low_byte=67
0x0000419F      1  E4                                                   OPAQUE_RAW_BYTES          bytes=E4
0x000041A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041A2      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x000041A4      3  F10009                                               IMM16_F1                  u16_be=9, u16_le=2304
0x000041A7      1  F0                                                   OPAQUE_RAW_BYTES          bytes=F0
0x000041A8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000041AA      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000041AC      1  43                                                   OPAQUE_RAW_BYTES          bytes=43
0x000041AD      1  FF                                                   TERMINATOR_FF             
0x000041AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041B2      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000041B4      1  FF                                                   TERMINATOR_FF             
0x000041B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041B7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000041B9      3  F311B2                                               IMM16_F3                  u16_be=4530, u16_le=45585
0x000041BC      1  FF                                                   TERMINATOR_FF             
0x000041BD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000041BF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000041C1     10  80084655303037353054                                 LEN8_STRING_CP932         length=8, text="FU00750T"
0x000041CB      1  FF                                                   TERMINATOR_FF             
0x000041CC      1  FF                                                   TERMINATOR_FF             
0x000041CD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000041CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000041D1     96  805E817590B392BC82A282A682CE8163816382ED82A982C1...  LEN8_STRING_CP932         length=94, text="「正直いえば……わかっていません。あの街に\nよって生じた存在であろうと予測はできるの\nですが」"
0x00004231      1  FF                                                   TERMINATOR_FF             
0x00004232      1  FF                                                   TERMINATOR_FF             
0x00004233      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004235      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004237      1  FF                                                   TERMINATOR_FF             
0x00004238      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000423A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000423C      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000423E      1  FF                                                   TERMINATOR_FF             
0x0000423F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004241      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004243      3  F311B3                                               IMM16_F3                  u16_be=4531, u16_le=45841
0x00004246      1  FF                                                   TERMINATOR_FF             
0x00004247      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004249      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000424B     10  80084655303037363054                                 LEN8_STRING_CP932         length=8, text="FU00760T"
0x00004255      1  FF                                                   TERMINATOR_FF             
0x00004256      1  FF                                                   TERMINATOR_FF             
0x00004257      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004259      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000425B     94  805C817582A082CC8A5882C982CD8141979D985F8FE38177...  LEN8_STRING_CP932         length=92, text="「あの街には、理論上『世界』が存在しています。ぼくたちはそのすべてを把握しているはずでした」"
0x000042B9      1  FF                                                   TERMINATOR_FF             
0x000042BA      1  FF                                                   TERMINATOR_FF             
0x000042BB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000042BD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000042BF      1  FF                                                   TERMINATOR_FF             
0x000042C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000042C2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000042C4      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000042C6      1  FF                                                   TERMINATOR_FF             
0x000042C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000042C9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000042CB      3  F311B4                                               IMM16_F3                  u16_be=4532, u16_le=46097
0x000042CE      1  FF                                                   TERMINATOR_FF             
0x000042CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000042D1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000042D3     10  80084655303037373054                                 LEN8_STRING_CP932         length=8, text="FU00770T"
0x000042DD      1  FF                                                   TERMINATOR_FF             
0x000042DE      1  FF                                                   TERMINATOR_FF             
0x000042DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000042E1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000042E3    122  8078817582AF82EA82C78163816382BD82BE82CC83718367...  LEN8_STRING_CP932         length=120, text="「けれど……ただのヒトでしかないぼくたちが\n『世界のすべて』を制御できるなんて、思い\nあがりでしかないのかもしれません」"
0x0000435D      1  FF                                                   TERMINATOR_FF             
0x0000435E      1  FF                                                   TERMINATOR_FF             
0x0000435F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004361      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004363      1  FF                                                   TERMINATOR_FF             
0x00004364      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004366      2  0081                                                 WORD_00XX                 u16_be=129, low_byte=129
0x00004368      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000436A      1  FF                                                   TERMINATOR_FF             
0x0000436B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000436D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000436F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004371      1  FF                                                   TERMINATOR_FF             
0x00004372      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004374      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004376      3  F311B5                                               IMM16_F3                  u16_be=4533, u16_le=46353
0x00004379      1  FF                                                   TERMINATOR_FF             
0x0000437A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000437C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000437E     10  80084655303037383054                                 LEN8_STRING_CP932         length=8, text="FU00780T"
0x00004388      1  FF                                                   TERMINATOR_FF             
0x00004389      1  FF                                                   TERMINATOR_FF             
0x0000438A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000438C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000438E     74  8048817588BB82CD8163816382BB82F182C8814182DA82AD...  LEN8_STRING_CP932         length=72, text="「綾は……そんな、ぼくたちの理解の限界を\n越えた存在、なのだと思います」"
0x000043D8      1  FF                                                   TERMINATOR_FF             
0x000043D9      1  FF                                                   TERMINATOR_FF             
0x000043DA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000043DC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000043DE      1  FF                                                   TERMINATOR_FF             
0x000043DF      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x000043E1      2  0046                                                 WORD_00XX                 u16_be=70, low_byte=70
0x000043E3      1  19                                                   OPAQUE_RAW_BYTES          bytes=19
0x000043E4      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000043E6      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x000043E9      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000043EB      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000043EC      1  FF                                                   TERMINATOR_FF             
0x000043ED      2  0046                                                 WORD_00XX                 u16_be=70, low_byte=70
0x000043EF      1  19                                                   OPAQUE_RAW_BYTES          bytes=19
0x000043F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043F2      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x000043F4      3  F10009                                               IMM16_F1                  u16_be=9, u16_le=2304
0x000043F7      1  F0                                                   OPAQUE_RAW_BYTES          bytes=F0
0x000043F8      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x000043FA      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000043FC      1  43                                                   OPAQUE_RAW_BYTES          bytes=43
0x000043FD      1  FF                                                   TERMINATOR_FF             
0x000043FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004400      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004402      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004404      1  FF                                                   TERMINATOR_FF             
0x00004405      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004407      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004409      3  F311B6                                               IMM16_F3                  u16_be=4534, u16_le=46609
0x0000440C      1  FF                                                   TERMINATOR_FF             
0x0000440D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000440F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004411     10  80084655303037393054                                 LEN8_STRING_CP932         length=8, text="FU00790T"
0x0000441B      1  FF                                                   TERMINATOR_FF             
0x0000441C      1  FF                                                   TERMINATOR_FF             
0x0000441D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000441F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004421    126  807C817596EE96EC82AD82F182A982E795B782A282C482A2...  LEN8_STRING_CP932         length=124, text="「矢野くんから聞いている、と思います。世界の崩壊について。このままでは、あの……誰も\nいない街に飲み込まれてしまうでしょう」"
0x0000449F      1  FF                                                   TERMINATOR_FF             
0x000044A0      1  FF                                                   TERMINATOR_FF             
0x000044A1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000044A3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000044A5      1  FF                                                   TERMINATOR_FF             
0x000044A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044A8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044AA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000044AC      1  FF                                                   TERMINATOR_FF             
0x000044AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044AF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000044B1      3  F311B7                                               IMM16_F3                  u16_be=4535, u16_le=46865
0x000044B4      1  FF                                                   TERMINATOR_FF             
0x000044B5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000044B7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000044B9     10  80084655303038303054                                 LEN8_STRING_CP932         length=8, text="FU00800T"
0x000044C3      1  FF                                                   TERMINATOR_FF             
0x000044C4      1  FF                                                   TERMINATOR_FF             
0x000044C5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000044C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000044C9     26  8018817582BB82EA82E0814182A082C690948E9E8AD482C5...  LEN8_STRING_CP932         length=24, text="「それも、あと数時間で」"
0x000044E3      1  FF                                                   TERMINATOR_FF             
0x000044E4      1  FF                                                   TERMINATOR_FF             
0x000044E5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000044E7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000044E9      1  FF                                                   TERMINATOR_FF             
0x000044EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044EE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000044F0      1  FF                                                   TERMINATOR_FF             
0x000044F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044F3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000044F5      3  F311B8                                               IMM16_F3                  u16_be=4536, u16_le=47121
0x000044F8      1  FF                                                   TERMINATOR_FF             
0x000044F9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000044FB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000044FD     28  801A817582BB82EA82AA814188BB82CC82B982A282BE82C6...  LEN8_STRING_CP932         length=26, text="「それが、綾のせいだと？」"
0x00004519      1  FF                                                   TERMINATOR_FF             
0x0000451A      1  FF                                                   TERMINATOR_FF             
0x0000451B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000451D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000451F      1  FF                                                   TERMINATOR_FF             
0x00004520      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004522      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004524      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004526      1  FF                                                   TERMINATOR_FF             
0x00004527      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004529      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000452B      3  F311B9                                               IMM16_F3                  u16_be=4537, u16_le=47377
0x0000452E      1  FF                                                   TERMINATOR_FF             
0x0000452F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004531      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004533     10  80084655303038313154                                 LEN8_STRING_CP932         length=8, text="FU00811T"
0x0000453D      1  FF                                                   TERMINATOR_FF             
0x0000453E      1  FF                                                   TERMINATOR_FF             
0x0000453F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004541      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004543     62  803C817594DE8F9782AA814182BB82EA82F088D3907D82B5...  LEN8_STRING_CP932         length=60, text="「彼女が、それを意図しているといっているわけではありません」"
0x00004581      1  FF                                                   TERMINATOR_FF             
0x00004582      1  FF                                                   TERMINATOR_FF             
0x00004583      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004585      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004587      1  FF                                                   TERMINATOR_FF             
0x00004588      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000458A      2  0082                                                 WORD_00XX                 u16_be=130, low_byte=130
0x0000458C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000458E      1  FF                                                   TERMINATOR_FF             
0x0000458F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004591      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004593      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004595      1  FF                                                   TERMINATOR_FF             
0x00004596      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004598      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000459A      3  F311BA                                               IMM16_F3                  u16_be=4538, u16_le=47633
0x0000459D      1  FF                                                   TERMINATOR_FF             
0x0000459E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000045A0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000045A2     10  80084655303038313254                                 LEN8_STRING_CP932         length=8, text="FU00812T"
0x000045AC      1  FF                                                   TERMINATOR_FF             
0x000045AD      1  FF                                                   TERMINATOR_FF             
0x000045AE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000045B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000045B2     96  805E817582BD82BE814194DE8F9782CC91B68DDD82AA8141...  LEN8_STRING_CP932         length=94, text="「ただ、彼女の存在が、ぼくたちの世界を変質\nさせている、ということだけは忘れないでくだ\nさい」"
0x00004612      1  FF                                                   TERMINATOR_FF             
0x00004613      1  FF                                                   TERMINATOR_FF             
0x00004614      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004616      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004618      1  FF                                                   TERMINATOR_FF             
0x00004619      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000461B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000461D      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000461F      1  FF                                                   TERMINATOR_FF             
0x00004620      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004622      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004624      3  F311BB                                               IMM16_F3                  u16_be=4539, u16_le=47889
0x00004627      1  FF                                                   TERMINATOR_FF             
0x00004628      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000462A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000462C     10  80084655303038323054                                 LEN8_STRING_CP932         length=8, text="FU00820T"
0x00004636      1  FF                                                   TERMINATOR_FF             
0x00004637      1  FF                                                   TERMINATOR_FF             
0x00004638      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000463A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000463C     46  802C817582BB82EB82BB82EB81638163967B91E882C982C2...  LEN8_STRING_CP932         length=44, text="「そろそろ……本題について話させてください」"
0x0000466A      1  FF                                                   TERMINATOR_FF             
0x0000466B      1  FF                                                   TERMINATOR_FF             
0x0000466C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000466E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004670      1  FF                                                   TERMINATOR_FF             
0x00004671      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00004673      3  F10080                                               IMM16_F1                  u16_be=128, u16_le=32768
0x00004676      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004678      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00004679      3  F10081                                               IMM16_F1                  u16_be=129, u16_le=33024
0x0000467C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000467E      2  3551                                                 OPAQUE_RAW_BYTES          bytes=3551
0x00004680      3  F10082                                               IMM16_F1                  u16_be=130, u16_le=33280
0x00004683      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004685      2  3551                                                 OPAQUE_RAW_BYTES          bytes=3551
0x00004687      1  FF                                                   TERMINATOR_FF             
0x00004688      2  0046                                                 WORD_00XX                 u16_be=70, low_byte=70
0x0000468A      1  95                                                   OPAQUE_RAW_BYTES          bytes=95
0x0000468B      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x0000468D      2  0047                                                 WORD_00XX                 u16_be=71, low_byte=71
0x0000468F      1  0A                                                   OPAQUE_RAW_BYTES          bytes=0A
0x00004690      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00004692      2  0046                                                 WORD_00XX                 u16_be=70, low_byte=70
0x00004694      1  9A                                                   OPAQUE_RAW_BYTES          bytes=9A
0x00004695      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00004697      2  003D                                                 WORD_00XX                 u16_be=61, low_byte=61
0x00004699      1  2D                                                   OPAQUE_RAW_BYTES          bytes=2D
0x0000469A      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x0000469C      2  0047                                                 WORD_00XX                 u16_be=71, low_byte=71
0x0000469E      1  0A                                                   OPAQUE_RAW_BYTES          bytes=0A
0x0000469F      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000046A1      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x000046A4      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000046A6      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000046A7      1  FF                                                   TERMINATOR_FF             
0x000046A8      2  0047                                                 WORD_00XX                 u16_be=71, low_byte=71
0x000046AA      1  0A                                                   OPAQUE_RAW_BYTES          bytes=0A
0x000046AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000046AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000046AF      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000046B1      1  FF                                                   TERMINATOR_FF             
0x000046B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000046B4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000046B6      3  F311BC                                               IMM16_F3                  u16_be=4540, u16_le=48145
0x000046B9      1  FF                                                   TERMINATOR_FF             
0x000046BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000046BC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000046BE     10  80084655303038333054                                 LEN8_STRING_CP932         length=8, text="FU00830T"
0x000046C8      1  FF                                                   TERMINATOR_FF             
0x000046C9      1  FF                                                   TERMINATOR_FF             
0x000046CA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000046CC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000046CE     48  802E817582A082E882AA82C682A4814282ED82A982C182C4...  LEN8_STRING_CP932         length=46, text="「ありがとう。わかってくれると思っていました」"
0x000046FE      1  FF                                                   TERMINATOR_FF             
0x000046FF      1  FF                                                   TERMINATOR_FF             
0x00004700      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004702      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004704      1  FF                                                   TERMINATOR_FF             
0x00004705      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00004707      2  0047                                                 WORD_00XX                 u16_be=71, low_byte=71
0x00004709      1  0A                                                   OPAQUE_RAW_BYTES          bytes=0A
0x0000470A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000470C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000470E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004710      1  FF                                                   TERMINATOR_FF             
0x00004711      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004713      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004715      3  F311BD                                               IMM16_F3                  u16_be=4541, u16_le=48401
0x00004718      1  FF                                                   TERMINATOR_FF             
0x00004719      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000471B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000471D     10  80084655303038343054                                 LEN8_STRING_CP932         length=8, text="FU00840T"
0x00004727      1  FF                                                   TERMINATOR_FF             
0x00004728      1  FF                                                   TERMINATOR_FF             
0x00004729      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000472B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000472D    134  8084817582E082A496DA82C982B582BD82C582B582E582A4...  LEN8_STRING_CP932         length=132, text="「もう目にしたでしょうけれど、街から人の姿が消えました。わかっているでしょうけれど、次に来るのは……明けない夜と、空を覆う満月です」"
0x000047B3      1  FF                                                   TERMINATOR_FF             
0x000047B4      1  FF                                                   TERMINATOR_FF             
0x000047B5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000047B7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000047B9      1  FF                                                   TERMINATOR_FF             
0x000047BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000047BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000047BE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000047C0      1  FF                                                   TERMINATOR_FF             
0x000047C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000047C3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000047C5      3  F311BE                                               IMM16_F3                  u16_be=4542, u16_le=48657
0x000047C8      1  FF                                                   TERMINATOR_FF             
0x000047C9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000047CB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000047CD     68  804282BB82A482BE82EB82A482C682CD8E7682C182C482A2...  LEN8_STRING_CP932         length=66, text="そうだろうとは思っていた。でも、そうでなければいいとも思っていた。"
0x00004811      1  FF                                                   TERMINATOR_FF             
0x00004812      1  FF                                                   TERMINATOR_FF             
0x00004813      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004815      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004817      1  FF                                                   TERMINATOR_FF             
0x00004818      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000481A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000481C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000481E      1  FF                                                   TERMINATOR_FF             
0x0000481F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004821      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004823      3  F311BF                                               IMM16_F3                  u16_be=4543, u16_le=48913
0x00004826      1  FF                                                   TERMINATOR_FF             
0x00004827      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004829      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000482B     42  802882BB82F182C882DA82AD82CC8AFA91D282CD81418AC8...  LEN8_STRING_CP932         length=40, text="そんなぼくの期待は、簡単に打ち砕かれた。"
0x00004855      1  FF                                                   TERMINATOR_FF             
0x00004856      1  FF                                                   TERMINATOR_FF             
0x00004857      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004859      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000485B      1  FF                                                   TERMINATOR_FF             
0x0000485C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000485E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004860      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004862      1  FF                                                   TERMINATOR_FF             
0x00004863      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004865      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004867      3  F311C0                                               IMM16_F3                  u16_be=4544, u16_le=49169
0x0000486A      1  FF                                                   TERMINATOR_FF             
0x0000486B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000486D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000486F     10  80084655303038353154                                 LEN8_STRING_CP932         length=8, text="FU00851T"
0x00004879      1  FF                                                   TERMINATOR_FF             
0x0000487A      1  FF                                                   TERMINATOR_FF             
0x0000487B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000487D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000487F     80  804E817582DA82AD82BD82BF82CD814195F689F382F096DA...  LEN8_STRING_CP932         length=78, text="「ぼくたちは、崩壊を目前にしたとき、世界の\nすべてを維持しようと計画しました」"
0x000048CF      1  FF                                                   TERMINATOR_FF             
0x000048D0      1  FF                                                   TERMINATOR_FF             
0x000048D1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000048D3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000048D5      1  FF                                                   TERMINATOR_FF             
0x000048D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048DA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000048DC      1  FF                                                   TERMINATOR_FF             
0x000048DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048DF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000048E1      3  F311C1                                               IMM16_F3                  u16_be=4545, u16_le=49425
0x000048E4      1  FF                                                   TERMINATOR_FF             
0x000048E5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000048E7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000048E9     10  80084655303038353254                                 LEN8_STRING_CP932         length=8, text="FU00852T"
0x000048F3      1  FF                                                   TERMINATOR_FF             
0x000048F4      1  FF                                                   TERMINATOR_FF             
0x000048F5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000048F7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000048F9     86  80548175979D985F82CC8DAA8AB282C9814190A28A4582CD...  LEN8_STRING_CP932         length=84, text="「理論の根幹に、世界は意識によって認識される、という唯心論があるのは否めないですが」"
0x0000494F      1  FF                                                   TERMINATOR_FF             
0x00004950      1  FF                                                   TERMINATOR_FF             
0x00004951      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004953      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004955      1  FF                                                   TERMINATOR_FF             
0x00004956      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004958      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000495A      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000495C      1  FF                                                   TERMINATOR_FF             
0x0000495D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000495F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004961      3  F311C2                                               IMM16_F3                  u16_be=4546, u16_le=49681
0x00004964      1  FF                                                   TERMINATOR_FF             
0x00004965      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004967      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004969     10  80084655303038363154                                 LEN8_STRING_CP932         length=8, text="FU00861T"
0x00004973      1  FF                                                   TERMINATOR_FF             
0x00004974      1  FF                                                   TERMINATOR_FF             
0x00004975      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004977      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004979    119  8075817582DA82AD82BD82BF82CD81418E4F8E7D2096728C...  LEN8_STRING_CP932         length=117, text="「ぼくたちは、三枝 睦月という女の子にすべて\nを託しました。世界を認識する彼女の意識を保ち続けることができるなら……」"
0x000049F0      1  FF                                                   TERMINATOR_FF             
0x000049F1      1  FF                                                   TERMINATOR_FF             
0x000049F2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000049F4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000049F6      1  FF                                                   TERMINATOR_FF             
0x000049F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000049F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000049FB      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000049FD      1  FF                                                   TERMINATOR_FF             
0x000049FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004A00      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004A02      3  F311C3                                               IMM16_F3                  u16_be=4547, u16_le=49937
0x00004A05      1  FF                                                   TERMINATOR_FF             
0x00004A06      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A08      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004A0A     10  80084655303038363254                                 LEN8_STRING_CP932         length=8, text="FU00862T"
0x00004A14      1  FF                                                   TERMINATOR_FF             
0x00004A15      1  FF                                                   TERMINATOR_FF             
0x00004A16      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A18      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A1A     86  8054817582BB82A481418FAD82C882AD82C682E094DE8F97...  LEN8_STRING_CP932         length=84, text="「そう、少なくとも彼女という場にあっては、\n世界は永遠であり続けるかもしれない、と」"
0x00004A70      1  FF                                                   TERMINATOR_FF             
0x00004A71      1  FF                                                   TERMINATOR_FF             
0x00004A72      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004A74      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004A76      1  FF                                                   TERMINATOR_FF             
0x00004A77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004A79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004A7B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004A7D      1  FF                                                   TERMINATOR_FF             
0x00004A7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004A80      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004A82      3  F311C4                                               IMM16_F3                  u16_be=4548, u16_le=50193
0x00004A85      1  FF                                                   TERMINATOR_FF             
0x00004A86      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A88      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004A8A     10  80084655303038373054                                 LEN8_STRING_CP932         length=8, text="FU00870T"
0x00004A94      1  FF                                                   TERMINATOR_FF             
0x00004A95      1  FF                                                   TERMINATOR_FF             
0x00004A96      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A98      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A9A     96  805E817582ED82A982E882E282B782AD82A282A282A982A6...  LEN8_STRING_CP932         length=94, text="「わかりやすくいいかえるなら、睦月ちゃんの\n夢の中に、世界を隔離した、ということ\nでしょうか」"
0x00004AFA      1  FF                                                   TERMINATOR_FF             
0x00004AFB      1  FF                                                   TERMINATOR_FF             
0x00004AFC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004AFE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004B00      1  FF                                                   TERMINATOR_FF             
0x00004B01      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B03      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B05      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004B07      1  FF                                                   TERMINATOR_FF             
0x00004B08      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B0A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004B0C      3  F311C5                                               IMM16_F3                  u16_be=4549, u16_le=50449
0x00004B0F      1  FF                                                   TERMINATOR_FF             
0x00004B10      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004B12      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004B14     38  802482BB82EA82AA814182A082CC8A588163816382C682A2...  LEN8_STRING_CP932         length=36, text="それが、あの街……ということなのか。"
0x00004B3A      1  FF                                                   TERMINATOR_FF             
0x00004B3B      1  FF                                                   TERMINATOR_FF             
0x00004B3C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004B3E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004B40      1  FF                                                   TERMINATOR_FF             
0x00004B41      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00004B43      3  F10058                                               IMM16_F1                  u16_be=88, u16_le=22528
0x00004B46      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004B48      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00004B49      1  FF                                                   TERMINATOR_FF             
0x00004B4A      2  004C                                                 WORD_00XX                 u16_be=76, low_byte=76
0x00004B4C      1  25                                                   OPAQUE_RAW_BYTES          bytes=25
0x00004B4D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B4F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B51      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004B53      1  FF                                                   TERMINATOR_FF             
0x00004B54      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B56      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004B58      3  F3410C                                               IMM16_F3                  u16_be=16652, u16_le=3137
0x00004B5B      1  FF                                                   TERMINATOR_FF             
0x00004B5C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004B5E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004B60     66  804082BB82B582C4814182DA82AD82CD8E7682A28F6F82B5...  LEN8_STRING_CP932         length=64, text="そして、ぼくは思い出した。\n睦月の記憶の中の『冬木さん』を……。"
0x00004BA2      1  FF                                                   TERMINATOR_FF             
0x00004BA3      1  FF                                                   TERMINATOR_FF             
0x00004BA4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004BA6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004BA8      1  FF                                                   TERMINATOR_FF             
0x00004BA9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BAB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BAD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004BAF      1  FF                                                   TERMINATOR_FF             
0x00004BB0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BB2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004BB4      3  F3410D                                               IMM16_F3                  u16_be=16653, u16_le=3393
0x00004BB7      1  FF                                                   TERMINATOR_FF             
0x00004BB8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004BBA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004BBC     38  802490A28A4582F08EE782EB82A482C682B782E982E282B3...  LEN8_STRING_CP932         length=36, text="世界を守ろうとするやさしい人を……。"
0x00004BE2      1  FF                                                   TERMINATOR_FF             
0x00004BE3      1  FF                                                   TERMINATOR_FF             
0x00004BE4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004BE6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004BE8      1  FF                                                   TERMINATOR_FF             
0x00004BE9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BEB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BED      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004BEF      1  FF                                                   TERMINATOR_FF             
0x00004BF0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BF2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004BF4      3  F3410E                                               IMM16_F3                  u16_be=16654, u16_le=3649
0x00004BF7      1  FF                                                   TERMINATOR_FF             
0x00004BF8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004BFA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004BFC     34  802096728C8E82AA904D978A82B582AB82C182C482A282BD...  LEN8_STRING_CP932         length=32, text="睦月が信頼しきっていた人を……。"
0x00004C1E      1  FF                                                   TERMINATOR_FF             
0x00004C1F      1  FF                                                   TERMINATOR_FF             
0x00004C20      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004C22      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004C24      1  FF                                                   TERMINATOR_FF             
0x00004C25      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C27      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C29      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004C2B      1  FF                                                   TERMINATOR_FF             
0x00004C2C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C2E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004C30      3  F311C6                                               IMM16_F3                  u16_be=4550, u16_le=50705
0x00004C33      1  FF                                                   TERMINATOR_FF             
0x00004C34      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004C36      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004C38     10  80084655303038383154                                 LEN8_STRING_CP932         length=8, text="FU00881T"
0x00004C42      1  FF                                                   TERMINATOR_FF             
0x00004C43      1  FF                                                   TERMINATOR_FF             
0x00004C44      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004C46      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004C48     78  804C8175834D838A83568341905F986282C9814182B782D7...  LEN8_STRING_CP932         length=76, text="「ギリシア神話に、すべてを封じ込めた\n『パンドラの箱』という物語があります」"
0x00004C96      1  FF                                                   TERMINATOR_FF             
0x00004C97      1  FF                                                   TERMINATOR_FF             
0x00004C98      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004C9A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004C9C      1  FF                                                   TERMINATOR_FF             
0x00004C9D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C9F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004CA1      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004CA3      1  FF                                                   TERMINATOR_FF             
0x00004CA4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004CA6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004CA8      3  F311C7                                               IMM16_F3                  u16_be=4551, u16_le=50961
0x00004CAB      1  FF                                                   TERMINATOR_FF             
0x00004CAC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004CAE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004CB0     10  80084655303038383254                                 LEN8_STRING_CP932         length=8, text="FU00882T"
0x00004CBA      1  FF                                                   TERMINATOR_FF             
0x00004CBB      1  FF                                                   TERMINATOR_FF             
0x00004CBC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004CBE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004CC0     84  8052817582DA82AD82BD82BF82CC8C7689E682CD814182BB...  LEN8_STRING_CP932         length=82, text="「ぼくたちの計画は、その名にちなんで\n『パンドラ計画』と呼ばれることになりました」"
0x00004D14      1  FF                                                   TERMINATOR_FF             
0x00004D15      1  FF                                                   TERMINATOR_FF             
0x00004D16      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004D18      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004D1A      1  FF                                                   TERMINATOR_FF             
0x00004D1B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D1F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004D21      1  FF                                                   TERMINATOR_FF             
0x00004D22      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D24      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004D26      3  F311C8                                               IMM16_F3                  u16_be=4552, u16_le=51217
0x00004D29      1  FF                                                   TERMINATOR_FF             
0x00004D2A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004D2C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004D2E     10  80084655303038393054                                 LEN8_STRING_CP932         length=8, text="FU00890T"
0x00004D38      1  FF                                                   TERMINATOR_FF             
0x00004D39      1  FF                                                   TERMINATOR_FF             
0x00004D3A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004D3C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004D3E    128  807E8175837083938368838982CD8BF082A982C982E08141...  LEN8_STRING_CP932         length=126, text="「パンドラは愚かにも、その箱を開けてしまい、世界に災いを広めてしまったわけです。でも、\n箱の中には『希望』が残されていた……」"
0x00004DBE      1  FF                                                   TERMINATOR_FF             
0x00004DBF      1  FF                                                   TERMINATOR_FF             
0x00004DC0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004DC2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004DC4      1  FF                                                   TERMINATOR_FF             
0x00004DC5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004DC7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004DC9      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004DCB      1  FF                                                   TERMINATOR_FF             
0x00004DCC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004DCE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004DD0      3  F311C9                                               IMM16_F3                  u16_be=4553, u16_le=51473
0x00004DD3      1  FF                                                   TERMINATOR_FF             
0x00004DD4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004DD6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004DD8     10  80084655303039303054                                 LEN8_STRING_CP932         length=8, text="FU00900T"
0x00004DE2      1  FF                                                   TERMINATOR_FF             
0x00004DE3      1  FF                                                   TERMINATOR_FF             
0x00004DE4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004DE6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004DE8     86  8054817582DC82B382A98163816390A28A4582CC95F689F3...  LEN8_STRING_CP932         length=84, text="「まさか……世界の崩壊を前にして、神話が繰り返されるとは……誰が想像できたでしょう」"
0x00004E3E      1  FF                                                   TERMINATOR_FF             
0x00004E3F      1  FF                                                   TERMINATOR_FF             
0x00004E40      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004E42      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004E44      1  FF                                                   TERMINATOR_FF             
0x00004E45      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004E47      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004E49      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004E4B      1  FF                                                   TERMINATOR_FF             
0x00004E4C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004E4E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004E50      3  F311CA                                               IMM16_F3                  u16_be=4554, u16_le=51729
0x00004E53      1  FF                                                   TERMINATOR_FF             
0x00004E54      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004E56      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004E58     10  80084655303039313154                                 LEN8_STRING_CP932         length=8, text="FU00911T"
0x00004E62      1  FF                                                   TERMINATOR_FF             
0x00004E63      1  FF                                                   TERMINATOR_FF             
0x00004E64      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004E66      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004E68     90  8058817582B2979782CC92CA82E8814182DA82AD82BD82BF...  LEN8_STRING_CP932         length=88, text="「ご覧の通り、ぼくたちは『パンドラの箱』を\nしっかりと閉じておくことができませんでした」"
0x00004EC2      1  FF                                                   TERMINATOR_FF             
0x00004EC3      1  FF                                                   TERMINATOR_FF             
0x00004EC4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004EC6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004EC8      1  FF                                                   TERMINATOR_FF             
0x00004EC9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004ECB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004ECD      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004ECF      1  FF                                                   TERMINATOR_FF             
0x00004ED0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004ED2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004ED4      3  F311CB                                               IMM16_F3                  u16_be=4555, u16_le=51985
0x00004ED7      1  FF                                                   TERMINATOR_FF             
0x00004ED8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004EDA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004EDC     10  80084655303039313254                                 LEN8_STRING_CP932         length=8, text="FU00912T"
0x00004EE6      1  FF                                                   TERMINATOR_FF             
0x00004EE7      1  FF                                                   TERMINATOR_FF             
0x00004EE8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004EEA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004EEC     86  805481758C7689E682C98F6491E582C8837E835882AA82A0...  LEN8_STRING_CP932         length=84, text="「計画に重大なミスがあったのか、それとも\nとても些細な不注意のせいかはわかりません」"
0x00004F42      1  FF                                                   TERMINATOR_FF             
0x00004F43      1  FF                                                   TERMINATOR_FF             
0x00004F44      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004F46      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004F48      1  FF                                                   TERMINATOR_FF             
0x00004F49      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004F4B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004F4D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004F4F      1  FF                                                   TERMINATOR_FF             
0x00004F50      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004F52      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004F54      3  F311CC                                               IMM16_F3                  u16_be=4556, u16_le=52241
0x00004F57      1  FF                                                   TERMINATOR_FF             
0x00004F58      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004F5A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004F5C     34  8020937E96D882B382F182CD82BB82A482A282C182C48141...  LEN8_STRING_CP932         length=32, text="冬木さんはそういって、苦笑した。"
0x00004F7E      1  FF                                                   TERMINATOR_FF             
0x00004F7F      1  FF                                                   TERMINATOR_FF             
0x00004F80      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004F82      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004F84      1  FF                                                   TERMINATOR_FF             
0x00004F85      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00004F87      3  F10058                                               IMM16_F1                  u16_be=88, u16_le=22528
0x00004F8A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004F8C      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00004F8D      1  FF                                                   TERMINATOR_FF             
0x00004F8E      2  0050                                                 WORD_00XX                 u16_be=80, low_byte=80
0x00004F90      1  67                                                   OPAQUE_RAW_BYTES          bytes=67
0x00004F91      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004F93      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004F95      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004F97      1  FF                                                   TERMINATOR_FF             
0x00004F98      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004F9A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004F9C      3  F311CD                                               IMM16_F3                  u16_be=4557, u16_le=52497
0x00004F9F      1  FF                                                   TERMINATOR_FF             
0x00004FA0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004FA2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004FA4     66  804082DA82AD82CD82C882C982E082A282A682C882A982C1...  LEN8_STRING_CP932         length=64, text="ぼくはなにもいえなかった。\n睦月の、三枝先生の痛みを知っている。"
0x00004FE6      1  FF                                                   TERMINATOR_FF             
0x00004FE7      1  FF                                                   TERMINATOR_FF             
0x00004FE8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004FEA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004FEC      1  FF                                                   TERMINATOR_FF             
0x00004FED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FEF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FF1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004FF3      1  FF                                                   TERMINATOR_FF             
0x00004FF4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FF6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004FF8      3  F311CE                                               IMM16_F3                  u16_be=4558, u16_le=52753
0x00004FFB      1  FF                                                   TERMINATOR_FF             
0x00004FFC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004FFE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005000     96  805E825094D491E590D882C882E082CC82F082C882AD82B5...  LEN8_STRING_CP932         length=94, text="１番大切なものをなくしてでも、世界を守ろうとした人たちが、どれほど苦しんだのかを知って\nいる。"
0x00005060      1  FF                                                   TERMINATOR_FF             
0x00005061      1  FF                                                   TERMINATOR_FF             
0x00005062      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005064      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005066      1  FF                                                   TERMINATOR_FF             
0x00005067      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005069      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000506B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000506D      1  FF                                                   TERMINATOR_FF             
0x0000506E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005070      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005072      3  F311CF                                               IMM16_F3                  u16_be=4559, u16_le=53009
0x00005075      1  FF                                                   TERMINATOR_FF             
0x00005076      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005078      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000507A     10  80084655303039323154                                 LEN8_STRING_CP932         length=8, text="FU00921T"
0x00005084      1  FF                                                   TERMINATOR_FF             
0x00005085      1  FF                                                   TERMINATOR_FF             
0x00005086      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005088      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000508A     34  8020817582DA82AD82C982CD82E082A482ED82A982E782C8...  LEN8_STRING_CP932         length=32, text="「ぼくにはもうわからないんです」"
0x000050AC      1  FF                                                   TERMINATOR_FF             
0x000050AD      1  FF                                                   TERMINATOR_FF             
0x000050AE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000050B0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000050B2      1  FF                                                   TERMINATOR_FF             
0x000050B3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000050B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000050B7      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000050B9      1  FF                                                   TERMINATOR_FF             
0x000050BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000050BC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000050BE      3  F311D0                                               IMM16_F3                  u16_be=4560, u16_le=53265
0x000050C1      1  FF                                                   TERMINATOR_FF             
0x000050C2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000050C4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000050C6     10  80084655303039323254                                 LEN8_STRING_CP932         length=8, text="FU00922T"
0x000050D0      1  FF                                                   TERMINATOR_FF             
0x000050D1      1  FF                                                   TERMINATOR_FF             
0x000050D2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000050D4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000050D6    128  807E81758177837083938368838982CC94A0817882AA8A4A...  LEN8_STRING_CP932         length=126, text="「『パンドラの箱』が開こうとしているときに、こんなことをいうのは無責任かもしれませんが、本当にわからなくなってしまったんです」"
0x00005156      1  FF                                                   TERMINATOR_FF             
0x00005157      1  FF                                                   TERMINATOR_FF             
0x00005158      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000515A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000515C      1  FF                                                   TERMINATOR_FF             
0x0000515D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000515F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005161      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005163      1  FF                                                   TERMINATOR_FF             
0x00005164      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005166      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005168      3  F311D1                                               IMM16_F3                  u16_be=4561, u16_le=53521
0x0000516B      1  FF                                                   TERMINATOR_FF             
0x0000516C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000516E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005170     10  80084655303039333154                                 LEN8_STRING_CP932         length=8, text="FU00931T"
0x0000517A      1  FF                                                   TERMINATOR_FF             
0x0000517B      1  FF                                                   TERMINATOR_FF             
0x0000517C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000517E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005180    126  807C817582C582E0814182DA82AD82C982CD82ED82A982E7...  LEN8_STRING_CP932         length=124, text="「でも、ぼくにはわからないんです。彼女が\n『不幸の箱を開けてしまう愚かなパンドラ』なのか、それとも『箱に残った希望』なのか」"
0x000051FE      1  FF                                                   TERMINATOR_FF             
0x000051FF      1  FF                                                   TERMINATOR_FF             
0x00005200      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005202      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005204      1  FF                                                   TERMINATOR_FF             
0x00005205      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005207      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005209      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000520B      1  FF                                                   TERMINATOR_FF             
0x0000520C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000520E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005210      3  F311D2                                               IMM16_F3                  u16_be=4562, u16_le=53777
0x00005213      1  FF                                                   TERMINATOR_FF             
0x00005214      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005216      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005218     10  80084655303039333254                                 LEN8_STRING_CP932         length=8, text="FU00932T"
0x00005222      1  FF                                                   TERMINATOR_FF             
0x00005223      1  FF                                                   TERMINATOR_FF             
0x00005224      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005226      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005228     98  8060817590B394BD91CE82CC8C8B89CA82AA90B682B682E9...  LEN8_STRING_CP932         length=96, text="「正反対の結果が生じるのですから、軽々しく\n判断することはできませんでした。だから、ぼくたちは」"
0x0000528A      1  FF                                                   TERMINATOR_FF             
0x0000528B      1  FF                                                   TERMINATOR_FF             
0x0000528C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000528E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005290      1  FF                                                   TERMINATOR_FF             
0x00005291      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005293      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005295      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005297      1  FF                                                   TERMINATOR_FF             
0x00005298      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000529A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000529C      3  F311D3                                               IMM16_F3                  u16_be=4563, u16_le=54033
0x0000529F      1  FF                                                   TERMINATOR_FF             
0x000052A0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000052A2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000052A4     10  80084655303039343154                                 LEN8_STRING_CP932         length=8, text="FU00941T"
0x000052AE      1  FF                                                   TERMINATOR_FF             
0x000052AF      1  FF                                                   TERMINATOR_FF             
0x000052B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000052B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000052B4    118  807481758CBB8FF388DB8E9D82F0914991F082B582DC82B5...  LEN8_STRING_CP932         length=116, text="「現状維持を選択しました。箱を開けさえ\nしなければ、災いは生じません。でも、希望を\n手にすることもできないんですね」"
0x0000532A      1  FF                                                   TERMINATOR_FF             
0x0000532B      1  FF                                                   TERMINATOR_FF             
0x0000532C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000532E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005330      1  FF                                                   TERMINATOR_FF             
0x00005331      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005333      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005335      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005337      1  FF                                                   TERMINATOR_FF             
0x00005338      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000533A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000533C      3  F311D4                                               IMM16_F3                  u16_be=4564, u16_le=54289
0x0000533F      1  FF                                                   TERMINATOR_FF             
0x00005340      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005342      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005344     10  80084655303039343254                                 LEN8_STRING_CP932         length=8, text="FU00942T"
0x0000534E      1  FF                                                   TERMINATOR_FF             
0x0000534F      1  FF                                                   TERMINATOR_FF             
0x00005350      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005352      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005354    132  8082817589BD89AD82E082CC906C82BD82BF82CC96BE93FA...  LEN8_STRING_CP932         length=130, text="「何億もの人たちの明日を考えたとき、ぼく\nたちは判断することを回避したのです。それが\n最善ではないにしても、最良に思えたからです」"
0x000053D8      1  FF                                                   TERMINATOR_FF             
0x000053D9      1  FF                                                   TERMINATOR_FF             
0x000053DA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000053DC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000053DE      1  FF                                                   TERMINATOR_FF             
0x000053DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053E3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000053E5      1  FF                                                   TERMINATOR_FF             
0x000053E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053E8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000053EA      3  F311D5                                               IMM16_F3                  u16_be=4565, u16_le=54545
0x000053ED      1  FF                                                   TERMINATOR_FF             
0x000053EE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000053F0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000053F2     60  803A82DA82AD82CD816381635C6E82B182B182C582E093AF...  LEN8_STRING_CP932         length=58, text="ぼくは……\nここでも同じ善意にふれていることに気がついた。"
0x0000542E      1  FF                                                   TERMINATOR_FF             
0x0000542F      1  FF                                                   TERMINATOR_FF             
0x00005430      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005432      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005434      1  FF                                                   TERMINATOR_FF             
0x00005435      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005437      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005439      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000543B      1  FF                                                   TERMINATOR_FF             
0x0000543C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000543E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005440      3  F311D6                                               IMM16_F3                  u16_be=4566, u16_le=54801
0x00005443      1  FF                                                   TERMINATOR_FF             
0x00005444      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005446      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005448     60  803A937E96D882B382F182E0814182BD82AD82B382F182CC...  LEN8_STRING_CP932         length=58, text="冬木さんも、たくさんの人たちのために、\nなにかを判断した。"
0x00005484      1  FF                                                   TERMINATOR_FF             
0x00005485      1  FF                                                   TERMINATOR_FF             
0x00005486      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005488      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000548A      1  FF                                                   TERMINATOR_FF             
0x0000548B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000548D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000548F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005491      1  FF                                                   TERMINATOR_FF             
0x00005492      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005494      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005496      3  F311D7                                               IMM16_F3                  u16_be=4567, u16_le=55057
0x00005499      1  FF                                                   TERMINATOR_FF             
0x0000549A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000549C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000549E    112  806E82BB82CC8C8B89CA82AA975C917A82C582AB82C882A2...  LEN8_STRING_CP932         length=110, text="その結果が予想できないものだったとしても、\nこの人は自分たちのできることを全力でやりとげようとしただけだった。"
0x0000550E      1  FF                                                   TERMINATOR_FF             
0x0000550F      1  FF                                                   TERMINATOR_FF             
0x00005510      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005512      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005514      1  FF                                                   TERMINATOR_FF             
0x00005515      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005517      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005519      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000551B      1  FF                                                   TERMINATOR_FF             
0x0000551C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000551E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005520      3  F311D8                                               IMM16_F3                  u16_be=4568, u16_le=55313
0x00005523      1  FF                                                   TERMINATOR_FF             
0x00005524      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005526      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005528     58  803882B582A982E08141937E96D882B382F182CD81638163...  LEN8_STRING_CP932         length=56, text="しかも、冬木さんは……\n自分のあやまちに気づいてもいる。"
0x00005562      1  FF                                                   TERMINATOR_FF             
0x00005563      1  FF                                                   TERMINATOR_FF             
0x00005564      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005566      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005568      1  FF                                                   TERMINATOR_FF             
0x00005569      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000556B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000556D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000556F      1  FF                                                   TERMINATOR_FF             
0x00005570      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005572      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005574      3  F311D9                                               IMM16_F3                  u16_be=4569, u16_le=55569
0x00005577      1  FF                                                   TERMINATOR_FF             
0x00005578      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000557A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000557C    134  808482BD82AD82B382F182CC814196BE93FA82AA82AD82E9...  LEN8_STRING_CP932         length=132, text="たくさんの、明日がくることに疑問も持たずに\n暮らしている人たちのため、たとえ真実では\nなかったにしても、毎日を維持しようとしてきた。"
0x00005602      1  FF                                                   TERMINATOR_FF             
0x00005603      1  FF                                                   TERMINATOR_FF             
0x00005604      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005606      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005608      1  FF                                                   TERMINATOR_FF             
0x00005609      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000560B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000560D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000560F      1  FF                                                   TERMINATOR_FF             
0x00005610      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005612      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005614      3  F311DA                                               IMM16_F3                  u16_be=4570, u16_le=55825
0x00005617      1  FF                                                   TERMINATOR_FF             
0x00005618      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000561A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000561C     68  804282DC82AA82A282E082CC82C582A082C182BD82C682B5...  LEN8_STRING_CP932         length=66, text="まがいものであったとしても、明日を信じてさえいれば、希望を持てる。"
0x00005660      1  FF                                                   TERMINATOR_FF             
0x00005661      1  FF                                                   TERMINATOR_FF             
0x00005662      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005664      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005666      1  FF                                                   TERMINATOR_FF             
0x00005667      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005669      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000566B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000566D      1  FF                                                   TERMINATOR_FF             
0x0000566E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005670      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005672      3  F311DB                                               IMM16_F3                  u16_be=4571, u16_le=56081
0x00005675      1  FF                                                   TERMINATOR_FF             
0x00005676      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005678      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000567A     88  805682BB82EA82AA895282C582A082C182BD82C682B582C4...  LEN8_STRING_CP932         length=86, text="それが嘘であったとしても、圧倒的な善意から\n生じたものであることにまちがいはなかった。"
0x000056D2      1  FF                                                   TERMINATOR_FF             
0x000056D3      1  FF                                                   TERMINATOR_FF             
0x000056D4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000056D6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000056D8      1  FF                                                   TERMINATOR_FF             
0x000056D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000056DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000056DD      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000056DF      1  FF                                                   TERMINATOR_FF             
0x000056E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000056E2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000056E4      3  F311DC                                               IMM16_F3                  u16_be=4572, u16_le=56337
0x000056E7      1  FF                                                   TERMINATOR_FF             
0x000056E8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000056EA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000056EC     10  80084655303039353054                                 LEN8_STRING_CP932         length=8, text="FU00950T"
0x000056F6      1  FF                                                   TERMINATOR_FF             
0x000056F7      1  FF                                                   TERMINATOR_FF             
0x000056F8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000056FA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000056FC     58  8038817582DA82AD82BD82BF82CD814188BB82C990DA9047...  LEN8_STRING_CP932         length=56, text="「ぼくたちは、綾に接触することが……\nできませんでした」"
0x00005736      1  FF                                                   TERMINATOR_FF             
0x00005737      1  FF                                                   TERMINATOR_FF             
0x00005738      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000573A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000573C      1  FF                                                   TERMINATOR_FF             
0x0000573D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000573F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005741      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005743      1  FF                                                   TERMINATOR_FF             
0x00005744      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005746      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005748      3  F311DD                                               IMM16_F3                  u16_be=4573, u16_le=56593
0x0000574B      1  FF                                                   TERMINATOR_FF             
0x0000574C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000574E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005750     62  803C937E96D882B382F182CC90BA82C988A382B582DD82AA...  LEN8_STRING_CP932         length=60, text="冬木さんの声に哀しみが混じる。\n自嘲しているのかもしれない。"
0x0000578E      1  FF                                                   TERMINATOR_FF             
0x0000578F      1  FF                                                   TERMINATOR_FF             
0x00005790      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005792      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005794      1  FF                                                   TERMINATOR_FF             
0x00005795      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005797      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005799      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000579B      1  FF                                                   TERMINATOR_FF             
0x0000579C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000579E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000057A0      3  F311DE                                               IMM16_F3                  u16_be=4574, u16_le=56849
0x000057A3      1  FF                                                   TERMINATOR_FF             
0x000057A4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000057A6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000057A8     10  80084655303039363154                                 LEN8_STRING_CP932         length=8, text="FU00961T"
0x000057B2      1  FF                                                   TERMINATOR_FF             
0x000057B3      1  FF                                                   TERMINATOR_FF             
0x000057B4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000057B6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000057B8     90  8058817594DE8F9782AA81418BF082A982C8837083938368...  LEN8_STRING_CP932         length=88, text="「彼女が、愚かなパンドラか、希望なのか、ぼくたちにはきっと永遠に理解できないのでしょう」"
0x00005812      1  FF                                                   TERMINATOR_FF             
0x00005813      1  FF                                                   TERMINATOR_FF             
0x00005814      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005816      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005818      1  FF                                                   TERMINATOR_FF             
0x00005819      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000581B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000581D      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000581F      1  FF                                                   TERMINATOR_FF             
0x00005820      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005822      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005824      3  F311DF                                               IMM16_F3                  u16_be=4575, u16_le=57105
0x00005827      1  FF                                                   TERMINATOR_FF             
0x00005828      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000582A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000582C     10  80084655303039363254                                 LEN8_STRING_CP932         length=8, text="FU00962T"
0x00005836      1  FF                                                   TERMINATOR_FF             
0x00005837      1  FF                                                   TERMINATOR_FF             
0x00005838      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000583A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000583C     64  803E817582BB82EA82F08163816394BB926682C582AB82E9...  LEN8_STRING_CP932         length=62, text="「それを……判断できるのは、きみだけなんで\nしょうね、きっと」"
0x0000587C      1  FF                                                   TERMINATOR_FF             
0x0000587D      1  FF                                                   TERMINATOR_FF             
0x0000587E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005880      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005882      1  FF                                                   TERMINATOR_FF             
0x00005883      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005885      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005887      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005889      1  FF                                                   TERMINATOR_FF             
0x0000588A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000588C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000588E      3  F3410F                                               IMM16_F3                  u16_be=16655, u16_le=3905
0x00005891      1  FF                                                   TERMINATOR_FF             
0x00005892      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005894      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005896     80  804E937E96D882B382F182CD82BB82A482A282C182C48141...  LEN8_STRING_CP932         length=78, text="冬木さんはそういって、うつむいた。\n自分の失意を他人にみせまいとする気配りで。"
0x000058E6      1  FF                                                   TERMINATOR_FF             
0x000058E7      1  FF                                                   TERMINATOR_FF             
0x000058E8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000058EA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000058EC      1  FF                                                   TERMINATOR_FF             
0x000058ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000058EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000058F1      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000058F3      1  FF                                                   TERMINATOR_FF             
0x000058F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000058F6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000058F8      3  F311E0                                               IMM16_F3                  u16_be=4576, u16_le=57361
0x000058FB      1  FF                                                   TERMINATOR_FF             
0x000058FC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000058FE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005900     10  80084655303039373154                                 LEN8_STRING_CP932         length=8, text="FU00971T"
0x0000590A      1  FF                                                   TERMINATOR_FF             
0x0000590B      1  FF                                                   TERMINATOR_FF             
0x0000590C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000590E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005910    114  8070817582DA82AD82BD82BF82CD81418EA995AA82BD82BF...  LEN8_STRING_CP932         length=112, text="「ぼくたちは、自分たちなりに……傲慢で愚か\nだったのかもしれませんが、できうる限りの\nことをしてきたつもりです」"
0x00005982      1  FF                                                   TERMINATOR_FF             
0x00005983      1  FF                                                   TERMINATOR_FF             
0x00005984      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005986      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005988      1  FF                                                   TERMINATOR_FF             
0x00005989      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000598B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000598D      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000598F      1  FF                                                   TERMINATOR_FF             
0x00005990      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005992      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005994      3  F311E1                                               IMM16_F3                  u16_be=4577, u16_le=57617
0x00005997      1  FF                                                   TERMINATOR_FF             
0x00005998      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000599A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000599C     10  80084655303039373254                                 LEN8_STRING_CP932         length=8, text="FU00972T"
0x000059A6      1  FF                                                   TERMINATOR_FF             
0x000059A7      1  FF                                                   TERMINATOR_FF             
0x000059A8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000059AA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000059AC     52  8032817582C582E08163816382AB82DD82CC8C88926682C9...  LEN8_STRING_CP932         length=50, text="「でも……きみの決断に異を唱えようとは思いません」"
0x000059E0      1  FF                                                   TERMINATOR_FF             
0x000059E1      1  FF                                                   TERMINATOR_FF             
0x000059E2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000059E4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000059E6      1  FF                                                   TERMINATOR_FF             
0x000059E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000059E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000059EB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000059ED      1  FF                                                   TERMINATOR_FF             
0x000059EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000059F0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000059F2      3  F311E2                                               IMM16_F3                  u16_be=4578, u16_le=57873
0x000059F5      1  FF                                                   TERMINATOR_FF             
0x000059F6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000059F8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000059FA     34  8020937E96D882B382F182CD814182B382D182B582BB82A4...  LEN8_STRING_CP932         length=32, text="冬木さんは、さびしそうに笑った。"
0x00005A1C      1  FF                                                   TERMINATOR_FF             
0x00005A1D      1  FF                                                   TERMINATOR_FF             
0x00005A1E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005A20      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005A22      1  FF                                                   TERMINATOR_FF             
0x00005A23      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A25      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A27      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005A29      1  FF                                                   TERMINATOR_FF             
0x00005A2A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A2C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005A2E      3  F311E3                                               IMM16_F3                  u16_be=4579, u16_le=58129
0x00005A31      1  FF                                                   TERMINATOR_FF             
0x00005A32      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005A34      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005A36     10  80084655303039383154                                 LEN8_STRING_CP932         length=8, text="FU00981T"
0x00005A40      1  FF                                                   TERMINATOR_FF             
0x00005A41      1  FF                                                   TERMINATOR_FF             
0x00005A42      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005A44      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005A46     88  8056817582AB82DD82AA94BB926682B782E982D782AB82BE...  LEN8_STRING_CP932         length=86, text="「きみが判断するべきだと思います。彼女が\n世界を滅ぼす災厄なのか、残された希望なのか」"
0x00005A9E      1  FF                                                   TERMINATOR_FF             
0x00005A9F      1  FF                                                   TERMINATOR_FF             
0x00005AA0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005AA2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005AA4      1  FF                                                   TERMINATOR_FF             
0x00005AA5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005AA7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005AA9      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005AAB      1  FF                                                   TERMINATOR_FF             
0x00005AAC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005AAE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005AB0      3  F311E4                                               IMM16_F3                  u16_be=4580, u16_le=58385
0x00005AB3      1  FF                                                   TERMINATOR_FF             
0x00005AB4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005AB6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005AB8     10  80084655303039383254                                 LEN8_STRING_CP932         length=8, text="FU00982T"
0x00005AC2      1  FF                                                   TERMINATOR_FF             
0x00005AC3      1  FF                                                   TERMINATOR_FF             
0x00005AC4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005AC6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005AC8     88  80568175969C82AA88EA814194DE8F9782C982E682C182C4...  LEN8_STRING_CP932         length=86, text="「万が一、彼女によって世界が消え失せたと\nしても、誰もきみを責めたりはしないでしょう」"
0x00005B20      1  FF                                                   TERMINATOR_FF             
0x00005B21      1  FF                                                   TERMINATOR_FF             
0x00005B22      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005B24      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005B26      1  FF                                                   TERMINATOR_FF             
0x00005B27      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B29      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B2B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005B2D      1  FF                                                   TERMINATOR_FF             
0x00005B2E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B30      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005B32      3  F311E5                                               IMM16_F3                  u16_be=4581, u16_le=58641
0x00005B35      1  FF                                                   TERMINATOR_FF             
0x00005B36      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005B38      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005B3A     10  80084655303039383354                                 LEN8_STRING_CP932         length=8, text="FU00983T"
0x00005B44      1  FF                                                   TERMINATOR_FF             
0x00005B45      1  FF                                                   TERMINATOR_FF             
0x00005B46      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005B48      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005B4A     68  8042817582AB82DD82CC914991F082BE82AF82B582A98141...  LEN8_STRING_CP932         length=66, text="「きみの選択だけしか、ぼくたちにのこされた\n未来はないのですから」"
0x00005B8E      1  FF                                                   TERMINATOR_FF             
0x00005B8F      1  FF                                                   TERMINATOR_FF             
0x00005B90      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005B92      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005B94      1  FF                                                   TERMINATOR_FF             
0x00005B95      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B97      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B99      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005B9B      1  FF                                                   TERMINATOR_FF             
0x00005B9C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B9E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005BA0      3  F311E6                                               IMM16_F3                  u16_be=4582, u16_le=58897
0x00005BA3      1  FF                                                   TERMINATOR_FF             
0x00005BA4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005BA6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005BA8     32  801E937E96D882B382F182CD814191E582AB82AD91A782F0...  LEN8_STRING_CP932         length=30, text="冬木さんは、大きく息をはいた。"
0x00005BC8      1  FF                                                   TERMINATOR_FF             
0x00005BC9      1  FF                                                   TERMINATOR_FF             
0x00005BCA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005BCC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005BCE      1  FF                                                   TERMINATOR_FF             
0x00005BCF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BD1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BD3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005BD5      1  FF                                                   TERMINATOR_FF             
0x00005BD6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BD8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005BDA      3  F34110                                               IMM16_F3                  u16_be=16656, u16_le=4161
0x00005BDD      1  FF                                                   TERMINATOR_FF             
0x00005BDE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005BE0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005BE2     80  804E8EA995AA82AA9477958982A482D782AB82E082CC82CC...  LEN8_STRING_CP932         length=78, text="自分が背負うべきものの重さを知っている人が、その重荷を他人に渡そうとしている。"
0x00005C32      1  FF                                                   TERMINATOR_FF             
0x00005C33      1  FF                                                   TERMINATOR_FF             
0x00005C34      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005C36      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005C38      1  FF                                                   TERMINATOR_FF             
0x00005C39      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C3D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005C3F      1  FF                                                   TERMINATOR_FF             
0x00005C40      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C42      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005C44      3  F34111                                               IMM16_F3                  u16_be=16657, u16_le=4417
0x00005C47      1  FF                                                   TERMINATOR_FF             
0x00005C48      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005C4A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005C4C     78  804C967B978882C882E781418EA995AA82D082C682E882C5...  LEN8_STRING_CP932         length=76, text="本来なら、自分ひとりで背負い続けるつもり\nだったのに、それができなくて……。"
0x00005C9A      1  FF                                                   TERMINATOR_FF             
0x00005C9B      1  FF                                                   TERMINATOR_FF             
0x00005C9C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005C9E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005CA0      1  FF                                                   TERMINATOR_FF             
0x00005CA1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005CA3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005CA5      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005CA7      1  FF                                                   TERMINATOR_FF             
0x00005CA8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005CAA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005CAC      3  F311E7                                               IMM16_F3                  u16_be=4583, u16_le=59153
0x00005CAF      1  FF                                                   TERMINATOR_FF             
0x00005CB0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005CB2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005CB4     10  80084655303039393054                                 LEN8_STRING_CP932         length=8, text="FU00990T"
0x00005CBE      1  FF                                                   TERMINATOR_FF             
0x00005CBF      1  FF                                                   TERMINATOR_FF             
0x00005CC0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005CC2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005CC4    108  806A81758F9F8EE882C882A88AE882A282BE82C682ED82A9...  LEN8_STRING_CP932         length=106, text="「勝手なお願いだとわかっています。でも、綾はきみになら会ってくれるはずです。たぶん、この街のどこかで……」"
0x00005D30      1  FF                                                   TERMINATOR_FF             
0x00005D31      1  FF                                                   TERMINATOR_FF             
0x00005D32      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005D34      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005D36      1  FF                                                   TERMINATOR_FF             
0x00005D37      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D39      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D3B      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00005D3D      1  FF                                                   TERMINATOR_FF             
0x00005D3E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D40      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005D42      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005D44      1  FF                                                   TERMINATOR_FF             
0x00005D45      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005D47      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005D49      1  FF                                                   TERMINATOR_FF             
0x00005D4A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D4C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D4E      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00005D50      1  FF                                                   TERMINATOR_FF             
0x00005D51      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D53      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005D55      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00005D57      1  FF                                                   TERMINATOR_FF             
0x00005D58      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005D5A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005D5C      1  FF                                                   TERMINATOR_FF             
0x00005D5D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D5F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D61      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005D63      1  FF                                                   TERMINATOR_FF             
0x00005D64      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D66      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005D68      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005D6A      1  FF                                                   TERMINATOR_FF             
0x00005D6B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D6D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005D6F      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005D71      1  FF                                                   TERMINATOR_FF             
0x00005D72      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005D74      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005D76      1  FF                                                   TERMINATOR_FF             
0x00005D77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D7B      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00005D7D      1  FF                                                   TERMINATOR_FF             
0x00005D7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D80      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005D82      2  F227                                                 IMM8_F2                   u8=39, s8=39
0x00005D84      1  FF                                                   TERMINATOR_FF             
0x00005D85      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005D87      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005D89      1  FF                                                   TERMINATOR_FF             
0x00005D8A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D8C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D8E      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00005D90      1  FF                                                   TERMINATOR_FF             
0x00005D91      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D93      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005D95      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00005D97      1  FF                                                   TERMINATOR_FF             
0x00005D98      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005D9A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005D9C      1  FF                                                   TERMINATOR_FF             
0x00005D9D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D9F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005DA1      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005DA3      1  FF                                                   TERMINATOR_FF             
0x00005DA4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005DA6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005DA8      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005DAA      1  FF                                                   TERMINATOR_FF             
0x00005DAB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005DAD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005DAF      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005DB1      1  FF                                                   TERMINATOR_FF             
0x00005DB2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005DB4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005DB6      1  FF                                                   TERMINATOR_FF             
0x00005DB7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005DB9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005DBB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005DBD      1  FF                                                   TERMINATOR_FF             
0x00005DBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005DC0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005DC2      3  F34113                                               IMM16_F3                  u16_be=16659, u16_le=4929
0x00005DC5      1  FF                                                   TERMINATOR_FF             
0x00005DC6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005DC8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005DCA     90  80588E7392A18EC982F08F6F82C4814182DA82AD82CD8A58...  LEN8_STRING_CP932         length=88, text="市庁舎を出て、ぼくは街を歩いた。\n考えをまとめるためにも歩き続けたかった。\nそして……。"
0x00005E24      1  FF                                                   TERMINATOR_FF             
0x00005E25      1  FF                                                   TERMINATOR_FF             
0x00005E26      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005E28      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005E2A      1  FF                                                   TERMINATOR_FF             
0x00005E2B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E2D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E2F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005E31      1  FF                                                   TERMINATOR_FF             
0x00005E32      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E34      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005E36      3  F311E8                                               IMM16_F3                  u16_be=4584, u16_le=59409
0x00005E39      1  FF                                                   TERMINATOR_FF             
0x00005E3A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005E3C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005E3E     52  803282DA82AD82CD88BB82F082B382AA82B582BD81425C6E...  LEN8_STRING_CP932         length=50, text="ぼくは綾をさがした。\n綾の姿をさがして歩き続けた。"
0x00005E72      1  FF                                                   TERMINATOR_FF             
0x00005E73      1  FF                                                   TERMINATOR_FF             
0x00005E74      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005E76      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005E78      1  FF                                                   TERMINATOR_FF             
0x00005E79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E7B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E7D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005E7F      1  FF                                                   TERMINATOR_FF             
0x00005E80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E82      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005E84      3  F311E9                                               IMM16_F3                  u16_be=4585, u16_le=59665
0x00005E87      1  FF                                                   TERMINATOR_FF             
0x00005E88      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005E8A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005E8C     98  8060937E96D882C682A282A482A082CC906C82CC82A282C1...  LEN8_STRING_CP932         length=96, text="冬木というあの人のいったことばを思い返す。\n綾をとめなければ、ぼくたちの世界は失われて\nしまう。"
0x00005EEE      1  FF                                                   TERMINATOR_FF             
0x00005EEF      1  FF                                                   TERMINATOR_FF             
0x00005EF0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005EF2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005EF4      1  FF                                                   TERMINATOR_FF             
0x00005EF5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005EF7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005EF9      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00005EFB      1  FF                                                   TERMINATOR_FF             
0x00005EFC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005EFE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F00      3  F30151                                               IMM16_F3                  u16_be=337, u16_le=20737
0x00005F03      1  FF                                                   TERMINATOR_FF             
0x00005F04      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005F06      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005F08      1  FF                                                   TERMINATOR_FF             
0x00005F09      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F0B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F0D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005F0F      1  FF                                                   TERMINATOR_FF             
0x00005F10      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F12      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F14      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005F16      1  FF                                                   TERMINATOR_FF             
0x00005F17      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F19      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005F1B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005F1D      1  FF                                                   TERMINATOR_FF             
0x00005F1E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005F20      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005F22      1  FF                                                   TERMINATOR_FF             
0x00005F23      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F25      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F27      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005F29      1  FF                                                   TERMINATOR_FF             
0x00005F2A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F2C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005F2E      3  F311EA                                               IMM16_F3                  u16_be=4586, u16_le=59921
0x00005F31      1  FF                                                   TERMINATOR_FF             
0x00005F32      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F34      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F36      8  80068AF3965D8142                                     LEN8_STRING_CP932         length=6, text="希望。"
0x00005F3E      1  FF                                                   TERMINATOR_FF             
0x00005F3F      1  FF                                                   TERMINATOR_FF             
0x00005F40      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005F42      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005F44      1  FF                                                   TERMINATOR_FF             
0x00005F45      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F47      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F49      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005F4B      1  FF                                                   TERMINATOR_FF             
0x00005F4C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F4E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005F50      3  F311EB                                               IMM16_F3                  u16_be=4587, u16_le=60177
0x00005F53      1  FF                                                   TERMINATOR_FF             
0x00005F54      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F56      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F58     16  800E82BB82EA82C682E08DD082A28142                     LEN8_STRING_CP932         length=14, text="それとも災い。"
0x00005F68      1  FF                                                   TERMINATOR_FF             
0x00005F69      1  FF                                                   TERMINATOR_FF             
0x00005F6A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005F6C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005F6E      1  FF                                                   TERMINATOR_FF             
0x00005F6F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F71      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F73      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005F75      1  FF                                                   TERMINATOR_FF             
0x00005F76      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F78      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005F7A      3  F311EC                                               IMM16_F3                  u16_be=4588, u16_le=60433
0x00005F7D      1  FF                                                   TERMINATOR_FF             
0x00005F7E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F80      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F82     76  804A88BB82CD814182DA82AD82BD82BF82C982CD979D89F0...  LEN8_STRING_CP932         length=74, text="綾は、ぼくたちには理解できない存在だった。\nそれは、どこかでわかっていた。"
0x00005FCE      1  FF                                                   TERMINATOR_FF             
0x00005FCF      1  FF                                                   TERMINATOR_FF             
0x00005FD0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005FD2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005FD4      1  FF                                                   TERMINATOR_FF             
0x00005FD5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005FD7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005FD9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005FDB      1  FF                                                   TERMINATOR_FF             
0x00005FDC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005FDE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005FE0      3  F311ED                                               IMM16_F3                  u16_be=4589, u16_le=60689
0x00005FE3      1  FF                                                   TERMINATOR_FF             
0x00005FE4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005FE6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005FE8     36  802294DE8F9782CD814182C782B182A982C595CF82ED82C1...  LEN8_STRING_CP932         length=34, text="彼女は、どこかで変わってしまった。"
0x0000600C      1  FF                                                   TERMINATOR_FF             
0x0000600D      1  FF                                                   TERMINATOR_FF             
0x0000600E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006010      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006012      1  FF                                                   TERMINATOR_FF             
0x00006013      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006015      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006017      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006019      1  FF                                                   TERMINATOR_FF             
0x0000601A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000601C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000601E      3  F311EE                                               IMM16_F3                  u16_be=4590, u16_le=60945
0x00006021      1  FF                                                   TERMINATOR_FF             
0x00006022      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006024      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006026     74  8048926E92EA82C581418B9091E582C882E082CC82C69198...  LEN8_STRING_CP932         length=72, text="地底で、巨大なものと遭遇したとき……。\n彼女は、あの存在を理解していた。"
0x00006070      1  FF                                                   TERMINATOR_FF             
0x00006071      1  FF                                                   TERMINATOR_FF             
0x00006072      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006074      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006076      1  FF                                                   TERMINATOR_FF             
0x00006077      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006079      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000607B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000607D      1  FF                                                   TERMINATOR_FF             
0x0000607E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006080      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006082      3  F311EF                                               IMM16_F3                  u16_be=4591, u16_le=61201
0x00006085      1  FF                                                   TERMINATOR_FF             
0x00006086      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006088      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000608A     52  803294DE8F9782CD82C782B182A982C5814182A082CC8A58...  LEN8_STRING_CP932         length=50, text="彼女はどこかで、あの街に属する存在になって\nいた。"
0x000060BE      1  FF                                                   TERMINATOR_FF             
0x000060BF      1  FF                                                   TERMINATOR_FF             
0x000060C0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000060C2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000060C4      1  FF                                                   TERMINATOR_FF             
0x000060C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000060C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000060C9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000060CB      1  FF                                                   TERMINATOR_FF             
0x000060CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000060CE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000060D0      3  F34114                                               IMM16_F3                  u16_be=16660, u16_le=5185
0x000060D3      1  FF                                                   TERMINATOR_FF             
0x000060D4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000060D6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000060D8     20  801282BE82C682B782E982C882E7816381638142             LEN8_STRING_CP932         length=18, text="だとするなら……。"
0x000060EC      1  FF                                                   TERMINATOR_FF             
0x000060ED      1  FF                                                   TERMINATOR_FF             
0x000060EE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000060F0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000060F2      1  FF                                                   TERMINATOR_FF             
0x000060F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000060F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000060F7      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000060F9      1  FF                                                   TERMINATOR_FF             
0x000060FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000060FC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000060FE      2  F227                                                 IMM8_F2                   u8=39, s8=39
0x00006100      1  FF                                                   TERMINATOR_FF             
0x00006101      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006103      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006105      1  FF                                                   TERMINATOR_FF             
0x00006106      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006108      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000610A      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000610C      1  FF                                                   TERMINATOR_FF             
0x0000610D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000610F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006111      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00006113      1  FF                                                   TERMINATOR_FF             
0x00006114      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006116      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006118      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000611A      1  FF                                                   TERMINATOR_FF             
0x0000611B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000611D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000611F      1  FF                                                   TERMINATOR_FF             
0x00006120      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006122      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006124      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006126      1  FF                                                   TERMINATOR_FF             
0x00006127      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006129      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000612B      3  F311F0                                               IMM16_F3                  u16_be=4592, u16_le=61457
0x0000612E      1  FF                                                   TERMINATOR_FF             
0x0000612F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006131      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006133    116  807294DE8F9782AA82A282E98CC082E8814190A28A4582CD...  LEN8_STRING_CP932         length=114, text="彼女がいる限り、世界は失われてしまう。\nたとえつくられた世界だとしても、そこには\nたくさんの人たちが暮らしている。"
0x000061A7      1  FF                                                   TERMINATOR_FF             
0x000061A8      1  FF                                                   TERMINATOR_FF             
0x000061A9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000061AB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000061AD      1  FF                                                   TERMINATOR_FF             
0x000061AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000061B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000061B2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000061B4      1  FF                                                   TERMINATOR_FF             
0x000061B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000061B7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000061B9      3  F311F1                                               IMM16_F3                  u16_be=4593, u16_le=61713
0x000061BC      1  FF                                                   TERMINATOR_FF             
0x000061BD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000061BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000061C1     52  803282BB82B182C582CD814182DD82F182C882AA8FCE82A2...  LEN8_STRING_CP932         length=50, text="そこでは、みんなが笑い、泣き、明日を待って\nいる。"
0x000061F5      1  FF                                                   TERMINATOR_FF             
0x000061F6      1  FF                                                   TERMINATOR_FF             
0x000061F7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000061F9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000061FB      1  FF                                                   TERMINATOR_FF             
0x000061FC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000061FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006200      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006202      1  FF                                                   TERMINATOR_FF             
0x00006203      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006205      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006207      3  F311F2                                               IMM16_F3                  u16_be=4594, u16_le=61969
0x0000620A      1  FF                                                   TERMINATOR_FF             
0x0000620B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000620D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000620F     70  804482BB82EA82AA81418B5582E882CC8FEA8F8A82BE82C6...  LEN8_STRING_CP932         length=68, text="それが、偽りの場所だとしても、みんなが生きていることに変わりはない。"
0x00006255      1  FF                                                   TERMINATOR_FF             
0x00006256      1  FF                                                   TERMINATOR_FF             
0x00006257      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006259      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000625B      1  FF                                                   TERMINATOR_FF             
0x0000625C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000625E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006260      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006262      1  FF                                                   TERMINATOR_FF             
0x00006263      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006265      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006267      3  F311F3                                               IMM16_F3                  u16_be=4595, u16_le=62225
0x0000626A      1  FF                                                   TERMINATOR_FF             
0x0000626B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000626D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000626F     96  805E8BCA82CD8163816381425C6E82A082CC8FAC82B382C8...  LEN8_STRING_CP932         length=94, text="玉は……。\nあの小さな脳みそで考えて、一生懸命生きていた。今日がだめでも、明日があると信じて。"
0x000062CF      1  FF                                                   TERMINATOR_FF             
0x000062D0      1  FF                                                   TERMINATOR_FF             
0x000062D1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000062D3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000062D5      1  FF                                                   TERMINATOR_FF             
0x000062D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000062D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000062DA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000062DC      1  FF                                                   TERMINATOR_FF             
0x000062DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000062DF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000062E1      3  F311F4                                               IMM16_F3                  u16_be=4596, u16_le=62481
0x000062E4      1  FF                                                   TERMINATOR_FF             
0x000062E5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000062E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000062E9     56  80368B5582E882CC968893FA82BE82A982E782C682A282C1...  LEN8_STRING_CP932         length=54, text="偽りの毎日だからといって、それを奪われる\nなんて……。"
0x00006321      1  FF                                                   TERMINATOR_FF             
0x00006322      1  FF                                                   TERMINATOR_FF             
0x00006323      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006325      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006327      1  FF                                                   TERMINATOR_FF             
0x00006328      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000632A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000632C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000632E      1  FF                                                   TERMINATOR_FF             
0x0000632F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006331      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006333      3  F311F5                                               IMM16_F3                  u16_be=4597, u16_le=62737
0x00006336      1  FF                                                   TERMINATOR_FF             
0x00006337      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006339      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000633B     36  802282BB82F182C88CA0979882CD924E82C982E082C882A2...  LEN8_STRING_CP932         length=34, text="そんな権利は誰にもないはずだった。"
0x0000635F      1  FF                                                   TERMINATOR_FF             
0x00006360      1  FF                                                   TERMINATOR_FF             
0x00006361      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006363      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006365      1  FF                                                   TERMINATOR_FF             
0x00006366      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006368      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000636A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000636C      1  FF                                                   TERMINATOR_FF             
0x0000636D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000636F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006371      3  F311F6                                               IMM16_F3                  u16_be=4598, u16_le=62993
0x00006374      1  FF                                                   TERMINATOR_FF             
0x00006375      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006377      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006379     78  804C82C582E08163816388BB82CD82C782A482C882CC82BE...  LEN8_STRING_CP932         length=76, text="でも……綾はどうなのだろう。\n彼女は、自分の場所をさがそうとしているだけだ。"
0x000063C7      1  FF                                                   TERMINATOR_FF             
0x000063C8      1  FF                                                   TERMINATOR_FF             
0x000063C9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000063CB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000063CD      1  FF                                                   TERMINATOR_FF             
0x000063CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000063D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000063D2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000063D4      1  FF                                                   TERMINATOR_FF             
0x000063D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000063D7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000063D9      3  F311F7                                               IMM16_F3                  u16_be=4599, u16_le=63249
0x000063DC      1  FF                                                   TERMINATOR_FF             
0x000063DD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000063DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000063E1     70  80448B5582E882CC8FEA8F8A82C995DF82E782ED82EA82BD...  LEN8_STRING_CP932         length=68, text="偽りの場所に捕らわれた彼女は、そこから逃げ\n出そうとしているだけだ。"
0x00006427      1  FF                                                   TERMINATOR_FF             
0x00006428      1  FF                                                   TERMINATOR_FF             
0x00006429      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000642B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000642D      1  FF                                                   TERMINATOR_FF             
0x0000642E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006430      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006432      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006434      1  FF                                                   TERMINATOR_FF             
0x00006435      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006437      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006439      3  F311F8                                               IMM16_F3                  u16_be=4600, u16_le=63505
0x0000643C      1  FF                                                   TERMINATOR_FF             
0x0000643D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000643F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006441     36  802282DA82AD82CD8163816382C782A482B782EA82CE82A2...  LEN8_STRING_CP932         length=34, text="ぼくは……どうすればいいんだろう。"
0x00006465      1  FF                                                   TERMINATOR_FF             
0x00006466      1  FF                                                   TERMINATOR_FF             
0x00006467      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006469      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000646B      1  FF                                                   TERMINATOR_FF             
0x0000646C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000646E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006470      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00006472      1  FF                                                   TERMINATOR_FF             
0x00006473      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006475      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006477      2  F25D                                                 IMM8_F2                   u8=93, s8=93
0x00006479      1  FF                                                   TERMINATOR_FF             
0x0000647A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000647C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000647E      1  FF                                                   TERMINATOR_FF             
0x0000647F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006481      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006483      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00006485      1  FF                                                   TERMINATOR_FF             
0x00006486      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006488      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000648A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000648C      1  FF                                                   TERMINATOR_FF             
0x0000648D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000648F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006491      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00006493      1  FF                                                   TERMINATOR_FF             
0x00006494      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006496      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006498      1  FF                                                   TERMINATOR_FF             
0x00006499      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000649B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000649D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000649F      1  FF                                                   TERMINATOR_FF             
0x000064A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000064A2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000064A4      3  F311F9                                               IMM16_F3                  u16_be=4601, u16_le=63761
0x000064A7      1  FF                                                   TERMINATOR_FF             
0x000064A8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000064AA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000064AC     44  802A82DA82AD82CD814182C782A482B782EA82CE82A282A2...  LEN8_STRING_CP932         length=42, text="ぼくは、どうすればいいのかわからなかった。"
0x000064D8      1  FF                                                   TERMINATOR_FF             
0x000064D9      1  FF                                                   TERMINATOR_FF             
0x000064DA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000064DC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000064DE      1  FF                                                   TERMINATOR_FF             
0x000064DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000064E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000064E3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000064E5      1  FF                                                   TERMINATOR_FF             
0x000064E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000064E8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000064EA      3  F311FA                                               IMM16_F3                  u16_be=4602, u16_le=64017
0x000064ED      1  FF                                                   TERMINATOR_FF             
0x000064EE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000064F0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000064F2     74  804888BB82C98B4382C382A282BD82C682AB814182DA82AD...  LEN8_STRING_CP932         length=72, text="綾に気づいたとき、ぼくの旅は始まった。\nそして今、終わりが近づいている。"
0x0000653C      1  FF                                                   TERMINATOR_FF             
0x0000653D      1  FF                                                   TERMINATOR_FF             
0x0000653E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006540      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006542      1  FF                                                   TERMINATOR_FF             
0x00006543      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006545      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006547      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006549      1  FF                                                   TERMINATOR_FF             
0x0000654A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000654C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000654E      3  F311FB                                               IMM16_F3                  u16_be=4603, u16_le=64273
0x00006551      1  FF                                                   TERMINATOR_FF             
0x00006552      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006554      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006556     32  801E82DA82AD82CD82C782A482B782EA82CE82A282A282F1...  LEN8_STRING_CP932         length=30, text="ぼくはどうすればいいんだろう。"
0x00006576      1  FF                                                   TERMINATOR_FF             
0x00006577      1  FF                                                   TERMINATOR_FF             
0x00006578      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000657A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000657C      1  FF                                                   TERMINATOR_FF             
0x0000657D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000657F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006581      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006583      1  FF                                                   TERMINATOR_FF             
0x00006584      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006586      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006588      3  F311FC                                               IMM16_F3                  u16_be=4604, u16_le=64529
0x0000658B      1  FF                                                   TERMINATOR_FF             
0x0000658C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000658E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006590     66  804094DE8F9782F08D7382A982B982EA82CE814182DA82AD...  LEN8_STRING_CP932         length=64, text="彼女を行かせれば、ぼくたちはもとの生活を取りもどすことができる。"
0x000065D2      1  FF                                                   TERMINATOR_FF             
0x000065D3      1  FF                                                   TERMINATOR_FF             
0x000065D4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000065D6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000065D8      1  FF                                                   TERMINATOR_FF             
0x000065D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000065DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000065DD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000065DF      1  FF                                                   TERMINATOR_FF             
0x000065E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000065E2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000065E4      3  F311FD                                               IMM16_F3                  u16_be=4605, u16_le=64785
0x000065E7      1  FF                                                   TERMINATOR_FF             
0x000065E8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000065EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000065EC     94  805C82BB82EA82AA8141924E82A982C982C282AD82E782EA...  LEN8_STRING_CP932         length=92, text="それが、誰かにつくられた毎日だったとしても、ぼくたちは以前と同じように暮らすことが\nできる。"
0x0000664A      1  FF                                                   TERMINATOR_FF             
0x0000664B      1  FF                                                   TERMINATOR_FF             
0x0000664C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000664E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006650      1  FF                                                   TERMINATOR_FF             
0x00006651      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006653      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006655      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006657      1  FF                                                   TERMINATOR_FF             
0x00006658      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000665A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000665C      3  F311FE                                               IMM16_F3                  u16_be=4606, u16_le=65041
0x0000665F      1  FF                                                   TERMINATOR_FF             
0x00006660      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006662      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006664     68  804282DA82AD82CD968892A981419051965682C88BCA82F0...  LEN8_STRING_CP932         length=66, text="ぼくは毎朝、寝坊な玉を迎えに行き、お守りを\nしながら１日をすごす。"
0x000066A8      1  FF                                                   TERMINATOR_FF             
0x000066A9      1  FF                                                   TERMINATOR_FF             
0x000066AA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000066AC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000066AE      1  FF                                                   TERMINATOR_FF             
0x000066AF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000066B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000066B3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000066B5      1  FF                                                   TERMINATOR_FF             
0x000066B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000066B8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000066BA      3  F311FF                                               IMM16_F3                  u16_be=4607, u16_le=65297
0x000066BD      1  FF                                                   TERMINATOR_FF             
0x000066BE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000066C0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000066C2     82  805082D982C182C482A882AD82C682C882C982F082B582C5...  LEN8_STRING_CP932         length=80, text="ほっておくとなにをしでかすかわからない\n玉だから、ぼくがいてやらないといけない。"
0x00006714      1  FF                                                   TERMINATOR_FF             
0x00006715      1  FF                                                   TERMINATOR_FF             
0x00006716      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006718      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000671A      1  FF                                                   TERMINATOR_FF             
0x0000671B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000671D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000671F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006721      1  FF                                                   TERMINATOR_FF             
0x00006722      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006724      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006726      3  F31200                                               IMM16_F3                  u16_be=4608, u16_le=18
0x00006729      1  FF                                                   TERMINATOR_FF             
0x0000672A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000672C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000672E    106  80688BCA82CD816381638B5582E882BE82C682B582C482E0...  LEN8_STRING_CP932         length=104, text="玉は……偽りだとしても、そんなことに関係なく、必死で生きている。\nそれを、消すことなんて誰にできるんだ。"
0x00006798      1  FF                                                   TERMINATOR_FF             
0x00006799      1  FF                                                   TERMINATOR_FF             
0x0000679A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000679C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000679E      1  FF                                                   TERMINATOR_FF             
0x0000679F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000067A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000067A3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000067A5      1  FF                                                   TERMINATOR_FF             
0x000067A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000067A8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000067AA      3  F31201                                               IMM16_F3                  u16_be=4609, u16_le=274
0x000067AD      1  FF                                                   TERMINATOR_FF             
0x000067AE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000067B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000067B2    110  806C82C582E0814188BB82CD82D082C682E882C182AB82E8...  LEN8_STRING_CP932         length=108, text="でも、綾はひとりっきりで行こうとしている。\n彼女だって、ひとりっきりでいたいなんて\n思ってるはずがないのに。"
0x00006820      1  FF                                                   TERMINATOR_FF             
0x00006821      1  FF                                                   TERMINATOR_FF             
0x00006822      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006824      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006826      1  FF                                                   TERMINATOR_FF             
0x00006827      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006829      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000682B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000682D      1  FF                                                   TERMINATOR_FF             
0x0000682E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006830      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006832      3  F31202                                               IMM16_F3                  u16_be=4610, u16_le=530
0x00006835      1  FF                                                   TERMINATOR_FF             
0x00006836      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006838      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000683A     56  803694DE8F9782BE82C182C481418CC793C682C582A882B5...  LEN8_STRING_CP932         length=54, text="彼女だって、孤独でおしつぶされそうになって\nいるのに。"
0x00006872      1  FF                                                   TERMINATOR_FF             
0x00006873      1  FF                                                   TERMINATOR_FF             
0x00006874      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006876      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006878      1  FF                                                   TERMINATOR_FF             
0x00006879      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000687B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000687D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000687F      1  FF                                                   TERMINATOR_FF             
0x00006880      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006882      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006884      3  F31203                                               IMM16_F3                  u16_be=4611, u16_le=786
0x00006887      1  FF                                                   TERMINATOR_FF             
0x00006888      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000688A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000688C     74  804894DE8F9782CC91B68DDD82AA8163816382B782D782C4...  LEN8_STRING_CP932         length=72, text="彼女の存在が……すべてを失わせる。\nじゃあ、彼女にはなにが残されるんだ？"
0x000068D6      1  FF                                                   TERMINATOR_FF             
0x000068D7      1  FF                                                   TERMINATOR_FF             
0x000068D8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000068DA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000068DC      1  FF                                                   TERMINATOR_FF             
0x000068DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000068DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000068E1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000068E3      1  FF                                                   TERMINATOR_FF             
0x000068E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000068E6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000068E8      3  F31204                                               IMM16_F3                  u16_be=4612, u16_le=1042
0x000068EB      1  FF                                                   TERMINATOR_FF             
0x000068EC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000068EE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000068F0     34  802082DA82AD82CD814182C782A482B782EA82CE82A282A2...  LEN8_STRING_CP932         length=32, text="ぼくは、どうすればいいんだろう。"
0x00006912      1  FF                                                   TERMINATOR_FF             
0x00006913      1  FF                                                   TERMINATOR_FF             
0x00006914      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006916      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006918      1  FF                                                   TERMINATOR_FF             
0x00006919      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000691B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000691D      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000691F      1  FF                                                   TERMINATOR_FF             
0x00006920      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006922      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006924      3  F30126                                               IMM16_F3                  u16_be=294, u16_le=9729
0x00006927      1  FF                                                   TERMINATOR_FF             
0x00006928      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000692A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000692C      1  FF                                                   TERMINATOR_FF             
0x0000692D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000692F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006931      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00006933      1  FF                                                   TERMINATOR_FF             
0x00006934      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006936      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006938      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000693A      1  FF                                                   TERMINATOR_FF             
0x0000693B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000693D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000693F      1  FF                                                   TERMINATOR_FF             
0x00006940      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006942      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006944      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00006946      1  FF                                                   TERMINATOR_FF             
0x00006947      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006949      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000694B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000694D      1  FF                                                   TERMINATOR_FF             
0x0000694E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006950      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006952      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00006954      1  FF                                                   TERMINATOR_FF             
0x00006955      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006957      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006959      1  FF                                                   TERMINATOR_FF             
0x0000695A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000695C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000695E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006960      1  FF                                                   TERMINATOR_FF             
0x00006961      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006963      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006965      3  F31205                                               IMM16_F3                  u16_be=4613, u16_le=1298
0x00006968      1  FF                                                   TERMINATOR_FF             
0x00006969      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000696B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000696D     80  804E906C896582AA82A082C182BD81425C6E82B182CC924E...  LEN8_STRING_CP932         length=78, text="人影があった。\nこの誰もいない、動くものひとつない街で、\n初めて見かけた人影。"
0x000069BD      1  FF                                                   TERMINATOR_FF             
0x000069BE      1  FF                                                   TERMINATOR_FF             
0x000069BF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000069C1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000069C3      1  FF                                                   TERMINATOR_FF             
0x000069C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000069C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000069C8      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000069CA      1  FF                                                   TERMINATOR_FF             
0x000069CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000069CD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000069CF      3  F31206                                               IMM16_F3                  u16_be=4614, u16_le=1554
0x000069D2      1  FF                                                   TERMINATOR_FF             
0x000069D3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000069D5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000069D7     16  800E88BB8163816382BE82C182BD8142                     LEN8_STRING_CP932         length=14, text="綾……だった。"
0x000069E7      1  FF                                                   TERMINATOR_FF             
0x000069E8      1  FF                                                   TERMINATOR_FF             
0x000069E9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000069EB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000069ED      1  FF                                                   TERMINATOR_FF             
0x000069EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000069F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000069F2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000069F4      1  FF                                                   TERMINATOR_FF             
0x000069F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000069F7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000069F9      3  F31207                                               IMM16_F3                  u16_be=4615, u16_le=1810
0x000069FC      1  FF                                                   TERMINATOR_FF             
0x000069FD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000069FF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006A01     10  8008817588BB21218176                                 LEN8_STRING_CP932         length=8, text="「綾!!」"
0x00006A0B      1  FF                                                   TERMINATOR_FF             
0x00006A0C      1  FF                                                   TERMINATOR_FF             
0x00006A0D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006A0F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006A11      1  FF                                                   TERMINATOR_FF             
0x00006A12      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006A14      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006A16      8  800673652D653037                                     LEN8_STRING_CP932         length=6, text="se-e07"
0x00006A1E      1  FF                                                   TERMINATOR_FF             
0x00006A1F      1  FF                                                   TERMINATOR_FF             
0x00006A20      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006A22      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006A24      2  F214                                                 IMM8_F2                   u8=20, s8=20
0x00006A26      1  FF                                                   TERMINATOR_FF             
0x00006A27      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006A29      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006A2B      1  FF                                                   TERMINATOR_FF             
0x00006A2C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006A2E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006A30      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006A32      1  FF                                                   TERMINATOR_FF             
0x00006A33      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006A35      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006A37      3  F31208                                               IMM16_F3                  u16_be=4616, u16_le=2066
0x00006A3A      1  FF                                                   TERMINATOR_FF             
0x00006A3B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006A3D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006A3F     52  8032906C896582CD814182DA82AD82CC90BA82C98BEC82AF...  LEN8_STRING_CP932         length=50, text="人影は、ぼくの声に駆けだした。\nふり返りもせずに。"
0x00006A73      1  FF                                                   TERMINATOR_FF             
0x00006A74      1  FF                                                   TERMINATOR_FF             
0x00006A75      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006A77      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006A79      1  FF                                                   TERMINATOR_FF             
0x00006A7A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006A7C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006A7E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006A80      1  FF                                                   TERMINATOR_FF             
0x00006A81      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006A83      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006A85      3  F31209                                               IMM16_F3                  u16_be=4617, u16_le=2322
0x00006A88      1  FF                                                   TERMINATOR_FF             
0x00006A89      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006A8B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006A8D     46  802C82C782A482B582C493A682B082E981485C6E82DA82AD...  LEN8_STRING_CP932         length=44, text="どうして逃げる？\nぼくだと、わからないのか?!"
0x00006ABB      1  FF                                                   TERMINATOR_FF             
0x00006ABC      1  FF                                                   TERMINATOR_FF             
0x00006ABD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006ABF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006AC1      1  FF                                                   TERMINATOR_FF             
0x00006AC2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006AC4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006AC6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006AC8      1  FF                                                   TERMINATOR_FF             
0x00006AC9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006ACB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006ACD      3  F3120A                                               IMM16_F3                  u16_be=4618, u16_le=2578
0x00006AD0      1  FF                                                   TERMINATOR_FF             
0x00006AD1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006AD3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006AD5     90  805882DA82AD82CD94DE8F9782CC8CE382F092C782A48142...  LEN8_STRING_CP932         length=88, text="ぼくは彼女の後を追う。\n綾は、必死で駆けていた。\nまるで、タカに追われるウサギのように。"
0x00006B2F      1  FF                                                   TERMINATOR_FF             
0x00006B30      1  FF                                                   TERMINATOR_FF             
0x00006B31      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006B33      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006B35      1  FF                                                   TERMINATOR_FF             
0x00006B36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006B38      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006B3A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006B3C      1  FF                                                   TERMINATOR_FF             
0x00006B3D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006B3F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006B41      3  F3120B                                               IMM16_F3                  u16_be=4619, u16_le=2834
0x00006B44      1  FF                                                   TERMINATOR_FF             
0x00006B45      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006B47      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006B49     90  805882C782A482B582C493A682B082C882AB82E182A282AF...  LEN8_STRING_CP932         length=88, text="どうして逃げなきゃいけない？\nその理由がわからなかったし、ぼくは彼女と話す必要があった。"
0x00006BA3      1  FF                                                   TERMINATOR_FF             
0x00006BA4      1  FF                                                   TERMINATOR_FF             
0x00006BA5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006BA7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006BA9      1  FF                                                   TERMINATOR_FF             
0x00006BAA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006BAC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006BAE      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006BB0      1  FF                                                   TERMINATOR_FF             
0x00006BB1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006BB3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006BB5      3  F3120C                                               IMM16_F3                  u16_be=4620, u16_le=3090
0x00006BB8      1  FF                                                   TERMINATOR_FF             
0x00006BB9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006BBB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006BBD      9  800741593033343430                                   LEN8_STRING_CP932         length=7, text="AY03440"
0x00006BC6      1  FF                                                   TERMINATOR_FF             
0x00006BC7      1  FF                                                   TERMINATOR_FF             
0x00006BC8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006BCA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006BCC     34  8020817582CD82C1814182CD82C1814182CD82C1814182CD...  LEN8_STRING_CP932         length=32, text="「はっ、はっ、はっ、はっ、はっ」"
0x00006BEE      1  FF                                                   TERMINATOR_FF             
0x00006BEF      1  FF                                                   TERMINATOR_FF             
0x00006BF0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006BF2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006BF4      1  FF                                                   TERMINATOR_FF             
0x00006BF5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006BF7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006BF9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006BFB      1  FF                                                   TERMINATOR_FF             
0x00006BFC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006BFE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006C00      3  F3120D                                               IMM16_F3                  u16_be=4621, u16_le=3346
0x00006C03      1  FF                                                   TERMINATOR_FF             
0x00006C04      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006C06      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006C08    102  806494DE8F9782CC8BEA82B582A291A782C382A982A282AA...  LEN8_STRING_CP932         length=100, text="彼女の苦しい息づかいが聞こえる。\nスタミナのなさを気力でカバーするタイプの\n綾には、もう限界だった。"
0x00006C6E      1  FF                                                   TERMINATOR_FF             
0x00006C6F      1  FF                                                   TERMINATOR_FF             
0x00006C70      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006C72      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006C74      1  FF                                                   TERMINATOR_FF             
0x00006C75      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006C77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006C79      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006C7B      1  FF                                                   TERMINATOR_FF             
0x00006C7C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006C7E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006C80      3  F3120E                                               IMM16_F3                  u16_be=4622, u16_le=3602
0x00006C83      1  FF                                                   TERMINATOR_FF             
0x00006C84      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006C86      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006C88     35  8021817588BB814182DA82AD82BE21212093A682B082C882...  LEN8_STRING_CP932         length=33, text="「綾、ぼくだ!! 逃げなくていい!!」"
0x00006CAB      1  FF                                                   TERMINATOR_FF             
0x00006CAC      1  FF                                                   TERMINATOR_FF             
0x00006CAD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006CAF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006CB1      1  FF                                                   TERMINATOR_FF             
0x00006CB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006CB4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006CB6      2  F221                                                 IMM8_F2                   u8=33, s8=33
0x00006CB8      1  FF                                                   TERMINATOR_FF             
0x00006CB9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006CBB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006CBD      2  F214                                                 IMM8_F2                   u8=20, s8=20
0x00006CBF      1  FF                                                   TERMINATOR_FF             
0x00006CC0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006CC2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006CC4      2  F255                                                 IMM8_F2                   u8=85, s8=85
0x00006CC6      1  FF                                                   TERMINATOR_FF             
0x00006CC7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006CC9      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00006CCB      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00006CCD      1  FF                                                   TERMINATOR_FF             
0x00006CCE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006CD0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006CD2      1  FF                                                   TERMINATOR_FF             
0x00006CD3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006CD5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006CD7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006CD9      1  FF                                                   TERMINATOR_FF             
0x00006CDA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006CDC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006CDE      3  F3120F                                               IMM16_F3                  u16_be=4623, u16_le=3858
0x00006CE1      1  FF                                                   TERMINATOR_FF             
0x00006CE2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006CE4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006CE6     32  801E8CA882C98EE882F082A982AF814197A782BF8E7E82DC...  LEN8_STRING_CP932         length=30, text="肩に手をかけ、立ち止まらせる。"
0x00006D06      1  FF                                                   TERMINATOR_FF             
0x00006D07      1  FF                                                   TERMINATOR_FF             
0x00006D08      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006D0A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006D0C      1  FF                                                   TERMINATOR_FF             
0x00006D0D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D0F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D11      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00006D13      1  FF                                                   TERMINATOR_FF             
0x00006D14      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D16      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006D18      3  F30127                                               IMM16_F3                  u16_be=295, u16_le=9985
0x00006D1B      1  FF                                                   TERMINATOR_FF             
0x00006D1C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006D1E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006D20      1  FF                                                   TERMINATOR_FF             
0x00006D21      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D23      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D25      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00006D27      1  FF                                                   TERMINATOR_FF             
0x00006D28      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D2A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006D2C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00006D2E      1  FF                                                   TERMINATOR_FF             
0x00006D2F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D31      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006D33      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00006D35      1  FF                                                   TERMINATOR_FF             
0x00006D36      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006D38      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006D3A      1  FF                                                   TERMINATOR_FF             
0x00006D3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D3D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D3F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006D41      1  FF                                                   TERMINATOR_FF             
0x00006D42      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D44      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006D46      3  F31210                                               IMM16_F3                  u16_be=4624, u16_le=4114
0x00006D49      1  FF                                                   TERMINATOR_FF             
0x00006D4A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006D4C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006D4E      9  800741593033343530                                   LEN8_STRING_CP932         length=7, text="AY03450"
0x00006D57      1  FF                                                   TERMINATOR_FF             
0x00006D58      1  FF                                                   TERMINATOR_FF             
0x00006D59      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006D5B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006D5D     14  800C817582AD82C1816381638176                         LEN8_STRING_CP932         length=12, text="「くっ……」"
0x00006D6B      1  FF                                                   TERMINATOR_FF             
0x00006D6C      1  FF                                                   TERMINATOR_FF             
0x00006D6D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006D6F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006D71      1  FF                                                   TERMINATOR_FF             
0x00006D72      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D74      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D76      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006D78      1  FF                                                   TERMINATOR_FF             
0x00006D79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006D7B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006D7D      3  F31211                                               IMM16_F3                  u16_be=4625, u16_le=4370
0x00006D80      1  FF                                                   TERMINATOR_FF             
0x00006D81      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006D83      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006D85     70  80448CA882B282B582C982D382E895D482C182BD94DE8F97...  LEN8_STRING_CP932         length=68, text="肩ごしにふり返った彼女は、ぼくを敵意の\nこもった目でにらみつけ……。"
0x00006DCB      1  FF                                                   TERMINATOR_FF             
0x00006DCC      1  FF                                                   TERMINATOR_FF             
0x00006DCD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006DCF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006DD1      1  FF                                                   TERMINATOR_FF             
0x00006DD2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006DD4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006DD6      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006DD8      1  FF                                                   TERMINATOR_FF             
0x00006DD9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006DDB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006DDD      3  F31212                                               IMM16_F3                  u16_be=4626, u16_le=4626
0x00006DE0      1  FF                                                   TERMINATOR_FF             
0x00006DE1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006DE3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006DE5      9  800741593033343630                                   LEN8_STRING_CP932         length=7, text="AY03460"
0x00006DEE      1  FF                                                   TERMINATOR_FF             
0x00006DEF      1  FF                                                   TERMINATOR_FF             
0x00006DF0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006DF2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006DF4     14  800C81758163816382A03F218176                         LEN8_STRING_CP932         length=12, text="「……あ?!」"
0x00006E02      1  FF                                                   TERMINATOR_FF             
0x00006E03      1  FF                                                   TERMINATOR_FF             
0x00006E04      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006E06      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006E08      1  FF                                                   TERMINATOR_FF             
0x00006E09      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E0B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E0D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006E0F      1  FF                                                   TERMINATOR_FF             
0x00006E10      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E12      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006E14      3  F31213                                               IMM16_F3                  u16_be=4627, u16_le=4882
0x00006E17      1  FF                                                   TERMINATOR_FF             
0x00006E18      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006E1A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006E1C     72  804694DE8F9782CD92C782A282A982AF82C482A282E982CC...  LEN8_STRING_CP932         length=70, text="彼女は追いかけているのが、ぼくであることに\n気づき、驚いたようだった。"
0x00006E64      1  FF                                                   TERMINATOR_FF             
0x00006E65      1  FF                                                   TERMINATOR_FF             
0x00006E66      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006E68      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006E6A      1  FF                                                   TERMINATOR_FF             
0x00006E6B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E6D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E6F      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00006E71      1  FF                                                   TERMINATOR_FF             
0x00006E72      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E74      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006E76      3  F30128                                               IMM16_F3                  u16_be=296, u16_le=10241
0x00006E79      1  FF                                                   TERMINATOR_FF             
0x00006E7A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006E7C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006E7E      1  FF                                                   TERMINATOR_FF             
0x00006E7F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E81      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E83      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00006E85      1  FF                                                   TERMINATOR_FF             
0x00006E86      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E88      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006E8A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00006E8C      1  FF                                                   TERMINATOR_FF             
0x00006E8D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E8F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006E91      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00006E93      1  FF                                                   TERMINATOR_FF             
0x00006E94      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006E96      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006E98      1  FF                                                   TERMINATOR_FF             
0x00006E99      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E9B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006E9D      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006E9F      1  FF                                                   TERMINATOR_FF             
0x00006EA0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006EA2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006EA4      3  F31214                                               IMM16_F3                  u16_be=4628, u16_le=5138
0x00006EA7      1  FF                                                   TERMINATOR_FF             
0x00006EA8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006EAA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006EAC      9  800741593033343730                                   LEN8_STRING_CP932         length=7, text="AY03470"
0x00006EB5      1  FF                                                   TERMINATOR_FF             
0x00006EB6      1  FF                                                   TERMINATOR_FF             
0x00006EB7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006EB9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006EBB     70  8044817582CD829F82CD829F8163816382B282DF82F18163...  LEN8_STRING_CP932         length=68, text="「はぁはぁ……ごめん…なさい。はぁ………\n誰だかわからなかったから」"
0x00006F01      1  FF                                                   TERMINATOR_FF             
0x00006F02      1  FF                                                   TERMINATOR_FF             
0x00006F03      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006F05      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006F07      1  FF                                                   TERMINATOR_FF             
0x00006F08      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006F0A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006F0C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006F0E      1  FF                                                   TERMINATOR_FF             
0x00006F0F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006F11      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006F13      3  F31215                                               IMM16_F3                  u16_be=4629, u16_le=5394
0x00006F16      1  FF                                                   TERMINATOR_FF             
0x00006F17      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006F19      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006F1B     44  802A88BB82F095EF82F182C582A282BD8B9190E282CC8B43...  LEN8_STRING_CP932         length=42, text="綾を包んでいた拒絶の気配がかすかに和らぐ。"
0x00006F47      1  FF                                                   TERMINATOR_FF             
0x00006F48      1  FF                                                   TERMINATOR_FF             
0x00006F49      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006F4B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006F4D      1  FF                                                   TERMINATOR_FF             
0x00006F4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006F50      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006F52      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00006F54      1  FF                                                   TERMINATOR_FF             
0x00006F55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006F57      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006F59      3  F31216                                               IMM16_F3                  u16_be=4630, u16_le=5650
0x00006F5C      1  FF                                                   TERMINATOR_FF             
0x00006F5D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006F5F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006F61     96  805E94DE8F9782CD82A882D182A682C482A282BD81425C6E...  LEN8_STRING_CP932         length=94, text="彼女はおびえていた。\n見知らぬ街に放り込まれた彼女は、また\nひとりぼっちの心細さを感じていた。"
0x00006FC1      1  FF                                                   TERMINATOR_FF             
0x00006FC2      1  FF                                                   TERMINATOR_FF             
0x00006FC3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006FC5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006FC7      1  FF                                                   TERMINATOR_FF             
0x00006FC8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006FCA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006FCC      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006FCE      1  FF                                                   TERMINATOR_FF             
0x00006FCF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006FD1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006FD3      3  F31217                                               IMM16_F3                  u16_be=4631, u16_le=5906
0x00006FD6      1  FF                                                   TERMINATOR_FF             
0x00006FD7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006FD9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006FDB      9  800741593033343830                                   LEN8_STRING_CP932         length=7, text="AY03480"
0x00006FE4      1  FF                                                   TERMINATOR_FF             
0x00006FE5      1  FF                                                   TERMINATOR_FF             
0x00006FE6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006FE8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006FEA     38  8024817582CD82A08163816382CD82A08163816382CD82A0...  LEN8_STRING_CP932         length=36, text="「はあ……はあ……はあ……はあ……」"
0x00007010      1  FF                                                   TERMINATOR_FF             
0x00007011      1  FF                                                   TERMINATOR_FF             
0x00007012      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007014      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007016      1  FF                                                   TERMINATOR_FF             
0x00007017      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007019      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000701B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000701D      1  FF                                                   TERMINATOR_FF             
0x0000701E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007020      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007022      3  F31218                                               IMM16_F3                  u16_be=4632, u16_le=6162
0x00007025      1  FF                                                   TERMINATOR_FF             
0x00007026      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007028      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000702A     44  802A8D7282A291A782F082CD82AD94DE8F97814296DA82CC...  LEN8_STRING_CP932         length=42, text="荒い息をはく彼女。目の前には……綾がいた。"
0x00007056      1  FF                                                   TERMINATOR_FF             
0x00007057      1  FF                                                   TERMINATOR_FF             
0x00007058      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000705A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000705C      1  FF                                                   TERMINATOR_FF             
0x0000705D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000705F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007061      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007063      1  FF                                                   TERMINATOR_FF             
0x00007064      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007066      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007068      3  F31219                                               IMM16_F3                  u16_be=4633, u16_le=6418
0x0000706B      1  FF                                                   TERMINATOR_FF             
0x0000706C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000706E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007070     42  802882DA82AD82CD814182BD82AD82B382F182CC82B182C6...  LEN8_STRING_CP932         length=40, text="ぼくは、たくさんのことを思い出していた。"
0x0000709A      1  FF                                                   TERMINATOR_FF             
0x0000709B      1  FF                                                   TERMINATOR_FF             
0x0000709C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000709E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000070A0      1  FF                                                   TERMINATOR_FF             
0x000070A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000070A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000070A5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000070A7      1  FF                                                   TERMINATOR_FF             
0x000070A8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000070AA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000070AC      3  F3121A                                               IMM16_F3                  u16_be=4634, u16_le=6674
0x000070AF      1  FF                                                   TERMINATOR_FF             
0x000070B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000070B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000070B4     78  804C834A83458393835A8389815B82CC82B182C682CE8142...  LEN8_STRING_CP932         length=76, text="カウンセラーのことば。\n冬木さんのことば。\n綾と歩いた、あの街でのできごと。"
0x00007102      1  FF                                                   TERMINATOR_FF             
0x00007103      1  FF                                                   TERMINATOR_FF             
0x00007104      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007106      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007108      1  FF                                                   TERMINATOR_FF             
0x00007109      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000710B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000710D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000710F      1  FF                                                   TERMINATOR_FF             
0x00007110      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007112      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007114      3  F34115                                               IMM16_F3                  u16_be=16661, u16_le=5441
0x00007117      1  FF                                                   TERMINATOR_FF             
0x00007118      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000711A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000711C     26  801882BD82AD82B382F182CC82B182C682F08D6C82A682BD...  LEN8_STRING_CP932         length=24, text="たくさんのことを考えた。"
0x00007136      1  FF                                                   TERMINATOR_FF             
0x00007137      1  FF                                                   TERMINATOR_FF             
0x00007138      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000713A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000713C      1  FF                                                   TERMINATOR_FF             
0x0000713D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000713F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007141      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007143      1  FF                                                   TERMINATOR_FF             
0x00007144      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007146      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007148      3  F34116                                               IMM16_F3                  u16_be=16662, u16_le=5697
0x0000714B      1  FF                                                   TERMINATOR_FF             
0x0000714C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000714E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007150     46  802C90A28A4582CC96C5965381425C6E8BCA82CC968893FA...  LEN8_STRING_CP932         length=44, text="世界の滅亡。\n玉の毎日。\n綾がなくしたもの。"
0x0000717E      1  FF                                                   TERMINATOR_FF             
0x0000717F      1  FF                                                   TERMINATOR_FF             
0x00007180      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007182      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007184      1  FF                                                   TERMINATOR_FF             
0x00007185      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007187      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007189      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000718B      1  FF                                                   TERMINATOR_FF             
0x0000718C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000718E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007190      3  F34117                                               IMM16_F3                  u16_be=16663, u16_le=5953
0x00007193      1  FF                                                   TERMINATOR_FF             
0x00007194      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007196      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007198    102  806482A082DC82E882C982BD82AD82B382F182CC906C8158...  LEN8_STRING_CP932         length=100, text="あまりにたくさんの人々の毎日。\nそれが、ぼくの両肩にかかっていた。\nその重さに、ぼくは軽々しく……。"
0x000071FE      1  FF                                                   TERMINATOR_FF             
0x000071FF      1  FF                                                   TERMINATOR_FF             
0x00007200      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007202      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007204      1  FF                                                   TERMINATOR_FF             
0x00007205      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007207      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007209      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000720B      1  FF                                                   TERMINATOR_FF             
0x0000720C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000720E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007210      3  F3121B                                               IMM16_F3                  u16_be=4635, u16_le=6930
0x00007213      1  FF                                                   TERMINATOR_FF             
0x00007214      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007216      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007218     28  801A8C8B985F82F08F6F82B982BB82A482C982C882A982C1...  LEN8_STRING_CP932         length=26, text="結論を出せそうになかった。"
0x00007234      1  FF                                                   TERMINATOR_FF             
0x00007235      1  FF                                                   TERMINATOR_FF             
0x00007236      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007238      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000723A      1  FF                                                   TERMINATOR_FF             
0x0000723B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000723D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000723F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00007241      1  FF                                                   TERMINATOR_FF             
0x00007242      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007244      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007246      3  F3121C                                               IMM16_F3                  u16_be=4636, u16_le=7186
0x00007249      1  FF                                                   TERMINATOR_FF             
0x0000724A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000724C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000724E      9  800741593033343930                                   LEN8_STRING_CP932         length=7, text="AY03490"
0x00007257      1  FF                                                   TERMINATOR_FF             
0x00007258      1  FF                                                   TERMINATOR_FF             
0x00007259      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000725B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000725D     38  8024817582CD82A08163816382CD82A08163816382CD82A0...  LEN8_STRING_CP932         length=36, text="「はあ……はあ……はあ……はあ……」"
0x00007283      1  FF                                                   TERMINATOR_FF             
0x00007284      1  FF                                                   TERMINATOR_FF             
0x00007285      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007287      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007289      1  FF                                                   TERMINATOR_FF             
0x0000728A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000728C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000728E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007290      1  FF                                                   TERMINATOR_FF             
0x00007291      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007293      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007295      3  F3121D                                               IMM16_F3                  u16_be=4637, u16_le=7442
0x00007298      1  FF                                                   TERMINATOR_FF             
0x00007299      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000729B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000729D     60  803A88BB82E0814182DA82AD82CC82B182C682CE82F091D2...  LEN8_STRING_CP932         length=58, text="綾も、ぼくのことばを待っているのか、なにも\nいわなかった。"
0x000072D9      1  FF                                                   TERMINATOR_FF             
0x000072DA      1  FF                                                   TERMINATOR_FF             
0x000072DB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000072DD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000072DF      1  FF                                                   TERMINATOR_FF             
0x000072E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000072E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000072E4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000072E6      1  FF                                                   TERMINATOR_FF             
0x000072E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000072E9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000072EB      3  F3121E                                               IMM16_F3                  u16_be=4638, u16_le=7698
0x000072EE      1  FF                                                   TERMINATOR_FF             
0x000072EF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000072F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000072F3    118  807494DE8F9782AA8163816396DA82CC914F82C58BEA82B5...  LEN8_STRING_CP932         length=116, text="彼女が……目の前で苦しそうに息をはいている\n綾が、その姿以上の意味を持っているなんて、\n想像することもできなかった。"
0x00007369      1  FF                                                   TERMINATOR_FF             
0x0000736A      1  FF                                                   TERMINATOR_FF             
0x0000736B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000736D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000736F      1  FF                                                   TERMINATOR_FF             
0x00007370      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007372      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007374      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007376      1  FF                                                   TERMINATOR_FF             
0x00007377      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007379      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000737B      3  F3121F                                               IMM16_F3                  u16_be=4639, u16_le=7954
0x0000737E      1  FF                                                   TERMINATOR_FF             
0x0000737F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007381      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007383     84  805282DA82AD82C982C682C182C482CD814182BF82E582C1...  LEN8_STRING_CP932         length=82, text="ぼくにとっては、ちょっと頑固で、ちょっと\n場違いなことをいう女の子でしかなかった。"
0x000073D7      1  FF                                                   TERMINATOR_FF             
0x000073D8      1  FF                                                   TERMINATOR_FF             
0x000073D9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000073DB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000073DD      1  FF                                                   TERMINATOR_FF             
0x000073DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000073E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000073E2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000073E4      1  FF                                                   TERMINATOR_FF             
0x000073E5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000073E7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000073E9      3  F34118                                               IMM16_F3                  u16_be=16664, u16_le=6209
0x000073EC      1  FF                                                   TERMINATOR_FF             
0x000073ED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000073EF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000073F1    104  80668B4393EF82B582AD82C4814182B782AE82D382AD82EA...  LEN8_STRING_CP932         length=102, text="気難しくて、すぐふくれっ面をし、相手のことをいつでも気づかっているのに、それを全然\nおもてに出さない。"
0x00007459      1  FF                                                   TERMINATOR_FF             
0x0000745A      1  FF                                                   TERMINATOR_FF             
0x0000745B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000745D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000745F      1  FF                                                   TERMINATOR_FF             
0x00007460      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007462      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007464      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007466      1  FF                                                   TERMINATOR_FF             
0x00007467      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007469      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000746B      3  F34119                                               IMM16_F3                  u16_be=16665, u16_le=6465
0x0000746E      1  FF                                                   TERMINATOR_FF             
0x0000746F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007471      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007473     26  801882C582E0814188BB82C982CD96DA934982AA82A082E9...  LEN8_STRING_CP932         length=24, text="でも、綾には目的がある。"
0x0000748D      1  FF                                                   TERMINATOR_FF             
0x0000748E      1  FF                                                   TERMINATOR_FF             
0x0000748F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007491      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007493      1  FF                                                   TERMINATOR_FF             
0x00007494      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007496      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007498      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000749A      1  FF                                                   TERMINATOR_FF             
0x0000749B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000749D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000749F      3  F31220                                               IMM16_F3                  u16_be=4640, u16_le=8210
0x000074A2      1  FF                                                   TERMINATOR_FF             
0x000074A3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000074A5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000074A7     68  804282BB82A4814182B182F182C88FF38BB582C582C882AF...  LEN8_STRING_CP932         length=66, text="そう、こんな状況でなければ、彼女とぼくは……どうしていたんだろう。"
0x000074EB      1  FF                                                   TERMINATOR_FF             
0x000074EC      1  FF                                                   TERMINATOR_FF             
0x000074ED      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000074EF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000074F1      1  FF                                                   TERMINATOR_FF             
0x000074F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000074F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000074F6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000074F8      1  FF                                                   TERMINATOR_FF             
0x000074F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000074FB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000074FD      3  F31221                                               IMM16_F3                  u16_be=4641, u16_le=8466
0x00007500      1  FF                                                   TERMINATOR_FF             
0x00007501      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007503      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007505    112  806E82A082CC82C682AB814188BB82AA817793568E678178...  LEN8_STRING_CP932         length=110, text="あのとき、綾が『天使』と呼ぶ存在に見せられたように、ぼくたちは普通に話をして、友だちに\nなっていたのだろうか。"
0x00007575      1  FF                                                   TERMINATOR_FF             
0x00007576      1  FF                                                   TERMINATOR_FF             
0x00007577      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007579      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000757B      1  FF                                                   TERMINATOR_FF             
0x0000757C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000757E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007580      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007582      1  FF                                                   TERMINATOR_FF             
0x00007583      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007585      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007587      3  F31222                                               IMM16_F3                  u16_be=4642, u16_le=8722
0x0000758A      1  FF                                                   TERMINATOR_FF             
0x0000758B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000758D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000758F     52  803288BB82C982CD96DA934982AA82A082E981425C6E82B1...  LEN8_STRING_CP932         length=50, text="綾には目的がある。\nこの街はもう変わってしまった。"
0x000075C3      1  FF                                                   TERMINATOR_FF             
0x000075C4      1  FF                                                   TERMINATOR_FF             
0x000075C5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000075C7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000075C9      1  FF                                                   TERMINATOR_FF             
0x000075CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000075CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000075CE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000075D0      1  FF                                                   TERMINATOR_FF             
0x000075D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000075D3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000075D5      3  F31223                                               IMM16_F3                  u16_be=4643, u16_le=8978
0x000075D8      1  FF                                                   TERMINATOR_FF             
0x000075D9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000075DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000075DD     56  803682BD82D482F1816381635C6E96E982C982C882EA82CE...  LEN8_STRING_CP932         length=54, text="たぶん……\n夜になれば、巨大な満月が空を覆うのだろう。"
0x00007615      1  FF                                                   TERMINATOR_FF             
0x00007616      1  FF                                                   TERMINATOR_FF             
0x00007617      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007619      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000761B      1  FF                                                   TERMINATOR_FF             
0x0000761C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000761E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007620      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007622      1  FF                                                   TERMINATOR_FF             
0x00007623      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007625      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007627      3  F31224                                               IMM16_F3                  u16_be=4644, u16_le=9234
0x0000762A      1  FF                                                   TERMINATOR_FF             
0x0000762B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000762D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000762F     86  805488BB82AA82A282E98CC082E8814182DA82AD82BD82BF...  LEN8_STRING_CP932         length=84, text="綾がいる限り、ぼくたちは……冬木さんがいったように、なにかをなくしてしまうのだろう。"
0x00007685      1  FF                                                   TERMINATOR_FF             
0x00007686      1  FF                                                   TERMINATOR_FF             
0x00007687      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007689      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000768B      1  FF                                                   TERMINATOR_FF             
0x0000768C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000768E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007690      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00007692      1  FF                                                   TERMINATOR_FF             
0x00007693      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007695      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007697      3  F31225                                               IMM16_F3                  u16_be=4645, u16_le=9490
0x0000769A      1  FF                                                   TERMINATOR_FF             
0x0000769B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000769D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000769F      9  800741593033353030                                   LEN8_STRING_CP932         length=7, text="AY03500"
0x000076A8      1  FF                                                   TERMINATOR_FF             
0x000076A9      1  FF                                                   TERMINATOR_FF             
0x000076AA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000076AC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000076AE     30  801C81758163816382CD829F816381638163816382CD829F...  LEN8_STRING_CP932         length=28, text="「……はぁ…………はぁ……」"
0x000076CC      1  FF                                                   TERMINATOR_FF             
0x000076CD      1  FF                                                   TERMINATOR_FF             
0x000076CE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000076D0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000076D2      1  FF                                                   TERMINATOR_FF             
0x000076D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000076D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000076D7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000076D9      1  FF                                                   TERMINATOR_FF             
0x000076DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000076DC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000076DE      3  F31226                                               IMM16_F3                  u16_be=4646, u16_le=9746
0x000076E1      1  FF                                                   TERMINATOR_FF             
0x000076E2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000076E4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000076E6     42  802888BB82CD82C882C982E082A282ED82B882C9814182DA...  LEN8_STRING_CP932         length=40, text="綾はなにもいわずに、ぼくを見つめている。"
0x00007710      1  FF                                                   TERMINATOR_FF             
0x00007711      1  FF                                                   TERMINATOR_FF             
0x00007712      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007714      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007716      1  FF                                                   TERMINATOR_FF             
0x00007717      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007719      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000771B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000771D      1  FF                                                   TERMINATOR_FF             
0x0000771E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007720      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007722      3  F31227                                               IMM16_F3                  u16_be=4647, u16_le=10002
0x00007725      1  FF                                                   TERMINATOR_FF             
0x00007726      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007728      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000772A    110  806C835E8343837E8393834F82AA82D982F182CC8FAD82B5...  LEN8_STRING_CP932         length=108, text="タイミングがほんの少しズレてる彼女だから、\nもしかすると本当にただ息をととのえているだけなのかもしれないが。"
0x00007798      1  FF                                                   TERMINATOR_FF             
0x00007799      1  FF                                                   TERMINATOR_FF             
0x0000779A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000779C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000779E      1  FF                                                   TERMINATOR_FF             
0x0000779F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000077A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000077A3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000077A5      1  FF                                                   TERMINATOR_FF             
0x000077A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000077A8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000077AA      3  F31228                                               IMM16_F3                  u16_be=4648, u16_le=10258
0x000077AD      1  FF                                                   TERMINATOR_FF             
0x000077AE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000077B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000077B2     46  802C82BB82A48D6C82A682BD82C682AB814182BF82E582C1...  LEN8_STRING_CP932         length=44, text="そう考えたとき、ちょっとだけおかしくなった。"
0x000077E0      1  FF                                                   TERMINATOR_FF             
0x000077E1      1  FF                                                   TERMINATOR_FF             
0x000077E2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000077E4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000077E6      1  FF                                                   TERMINATOR_FF             
0x000077E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000077E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000077EB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000077ED      1  FF                                                   TERMINATOR_FF             
0x000077EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000077F0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000077F2      3  F31229                                               IMM16_F3                  u16_be=4649, u16_le=10514
0x000077F5      1  FF                                                   TERMINATOR_FF             
0x000077F6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000077F8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000077FA     22  801482BB82A481425C6E82BB82A482C882F182BE8142         LEN8_STRING_CP932         length=20, text="そう。\nそうなんだ。"
0x00007810      1  FF                                                   TERMINATOR_FF             
0x00007811      1  FF                                                   TERMINATOR_FF             
0x00007812      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007814      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007816      1  FF                                                   TERMINATOR_FF             
0x00007817      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007819      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000781B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000781D      1  FF                                                   TERMINATOR_FF             
0x0000781E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007820      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007822      3  F3122A                                               IMM16_F3                  u16_be=4650, u16_le=10770
0x00007825      1  FF                                                   TERMINATOR_FF             
0x00007826      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007828      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000782A     70  804494DE8F9782CD82C682AB82C782AB814195CF82C882B1...  LEN8_STRING_CP932         length=68, text="彼女はときどき、変なことで気分を害し、\nおかしなことで頬を赤らめる。"
0x00007870      1  FF                                                   TERMINATOR_FF             
0x00007871      1  FF                                                   TERMINATOR_FF             
0x00007872      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007874      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007876      1  FF                                                   TERMINATOR_FF             
0x00007877      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007879      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000787B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000787D      1  FF                                                   TERMINATOR_FF             
0x0000787E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007880      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007882      3  F3122B                                               IMM16_F3                  u16_be=4651, u16_le=11026
0x00007885      1  FF                                                   TERMINATOR_FF             
0x00007886      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007888      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000788A      9  800741593033353130                                   LEN8_STRING_CP932         length=7, text="AY03510"
0x00007893      1  FF                                                   TERMINATOR_FF             
0x00007894      1  FF                                                   TERMINATOR_FF             
0x00007895      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007897      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007899     30  801C81758163816382C882C982AA814182A882A982B582A2...  LEN8_STRING_CP932         length=28, text="「……なにが、おかしいの？」"
0x000078B7      1  FF                                                   TERMINATOR_FF             
0x000078B8      1  FF                                                   TERMINATOR_FF             
0x000078B9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000078BB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000078BD      1  FF                                                   TERMINATOR_FF             
0x000078BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000078C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000078C2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000078C4      1  FF                                                   TERMINATOR_FF             
0x000078C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000078C7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000078C9      3  F3122C                                               IMM16_F3                  u16_be=4652, u16_le=11282
0x000078CC      1  FF                                                   TERMINATOR_FF             
0x000078CD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000078CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000078D1     64  803E82C282A28FCE82DD82F0958282A982D782C482A282BD...  LEN8_STRING_CP932         length=62, text="つい笑みを浮かべていたらしく、彼女は不機嫌\nそうな声でいった。"
0x00007911      1  FF                                                   TERMINATOR_FF             
0x00007912      1  FF                                                   TERMINATOR_FF             
0x00007913      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007915      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007917      1  FF                                                   TERMINATOR_FF             
0x00007918      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000791A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000791C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000791E      1  FF                                                   TERMINATOR_FF             
0x0000791F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007921      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007923      3  F3122D                                               IMM16_F3                  u16_be=4653, u16_le=11538
0x00007926      1  FF                                                   TERMINATOR_FF             
0x00007927      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007929      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000792B    112  806E82BB82A4814188BB82C182C482B182A482C882CC82BE...  LEN8_STRING_CP932         length=110, text="そう、綾ってこうなのだ。\n妙に生真面目で、融通がきかなくって。\nもし、彼女とクラスが一緒だったら、ぼくは……。"
0x0000799B      1  FF                                                   TERMINATOR_FF             
0x0000799C      1  FF                                                   TERMINATOR_FF             
0x0000799D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000799F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000079A1      1  FF                                                   TERMINATOR_FF             
0x000079A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000079A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000079A6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000079A8      1  FF                                                   TERMINATOR_FF             
0x000079A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000079AB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000079AD      3  F3122E                                               IMM16_F3                  u16_be=4654, u16_le=11794
0x000079B0      1  FF                                                   TERMINATOR_FF             
0x000079B1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000079B3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000079B5     24  801681758DA193FA814195948A8882C882A282E682CB8176     LEN8_STRING_CP932         length=22, text="「今日、部活ないよね」"
0x000079CD      1  FF                                                   TERMINATOR_FF             
0x000079CE      1  FF                                                   TERMINATOR_FF             
0x000079CF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000079D1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000079D3      1  FF                                                   TERMINATOR_FF             
0x000079D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000079D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000079D8      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000079DA      1  FF                                                   TERMINATOR_FF             
0x000079DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000079DD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000079DF      3  F3122F                                               IMM16_F3                  u16_be=4655, u16_le=12050
0x000079E2      1  FF                                                   TERMINATOR_FF             
0x000079E3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000079E5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000079E7      9  800741593033353230                                   LEN8_STRING_CP932         length=7, text="AY03520"
0x000079F0      1  FF                                                   TERMINATOR_FF             
0x000079F1      1  FF                                                   TERMINATOR_FF             
0x000079F2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000079F4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000079F6     18  8010817582A68163816382A682C181488176                 LEN8_STRING_CP932         length=16, text="「え……えっ？」"
0x00007A08      1  FF                                                   TERMINATOR_FF             
0x00007A09      1  FF                                                   TERMINATOR_FF             
0x00007A0A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007A0C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007A0E      1  FF                                                   TERMINATOR_FF             
0x00007A0F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007A11      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007A13      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007A15      1  FF                                                   TERMINATOR_FF             
0x00007A16      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007A18      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007A1A      3  F31230                                               IMM16_F3                  u16_be=4656, u16_le=12306
0x00007A1D      1  FF                                                   TERMINATOR_FF             
0x00007A1E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007A20      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007A22     64  803E88BB82CD814189BD937882A982DC82CE82BD82AB82F0...  LEN8_STRING_CP932         length=62, text="綾は、何度かまばたきをする。\n突然のことばに、とまどっている。"
0x00007A62      1  FF                                                   TERMINATOR_FF             
0x00007A63      1  FF                                                   TERMINATOR_FF             
0x00007A64      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007A66      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007A68      1  FF                                                   TERMINATOR_FF             
0x00007A69      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007A6B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007A6D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007A6F      1  FF                                                   TERMINATOR_FF             
0x00007A70      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007A72      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007A74      3  F31231                                               IMM16_F3                  u16_be=4657, u16_le=12562
0x00007A77      1  FF                                                   TERMINATOR_FF             
0x00007A78      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007A7A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007A7C     54  8034817582E682A982C182BD82E7814182BF82E582C182C6...  LEN8_STRING_CP932         length=52, text="「よかったら、ちょっと遊びに行かないかな、と思って」"
0x00007AB2      1  FF                                                   TERMINATOR_FF             
0x00007AB3      1  FF                                                   TERMINATOR_FF             
0x00007AB4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007AB6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007AB8      1  FF                                                   TERMINATOR_FF             
0x00007AB9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007ABB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007ABD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007ABF      1  FF                                                   TERMINATOR_FF             
0x00007AC0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007AC2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007AC4      3  F3411A                                               IMM16_F3                  u16_be=16666, u16_le=6721
0x00007AC7      1  FF                                                   TERMINATOR_FF             
0x00007AC8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007ACA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007ACC    102  806482DA82AD82CD814182D982F182CC8FAD82B582D382B4...  LEN8_STRING_CP932         length=100, text="ぼくは、ほんの少しふざけた気分で彼女を誘う。面くらう彼女の姿が見たかった……という\n気持ちもあった。"
0x00007B32      1  FF                                                   TERMINATOR_FF             
0x00007B33      1  FF                                                   TERMINATOR_FF             
0x00007B34      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007B36      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007B38      1  FF                                                   TERMINATOR_FF             
0x00007B39      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007B3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007B3D      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00007B3F      1  FF                                                   TERMINATOR_FF             
0x00007B40      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007B42      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007B44      3  F31232                                               IMM16_F3                  u16_be=4658, u16_le=12818
0x00007B47      1  FF                                                   TERMINATOR_FF             
0x00007B48      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007B4A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00007B4C      9  800741593033353330                                   LEN8_STRING_CP932         length=7, text="AY03530"
0x00007B55      1  FF                                                   TERMINATOR_FF             
0x00007B56      1  FF                                                   TERMINATOR_FF             
0x00007B57      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007B59      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007B5B     48  802E817582C7814182C782A482B582C481638163814282C5...  LEN8_STRING_CP932         length=46, text="「ど、どうして……。でも……だって、急に……」"
0x00007B8B      1  FF                                                   TERMINATOR_FF             
0x00007B8C      1  FF                                                   TERMINATOR_FF             
0x00007B8D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007B8F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007B91      1  FF                                                   TERMINATOR_FF             
0x00007B92      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007B94      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007B96      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007B98      1  FF                                                   TERMINATOR_FF             
0x00007B99      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007B9B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007B9D      3  F31233                                               IMM16_F3                  u16_be=4659, u16_le=13074
0x00007BA0      1  FF                                                   TERMINATOR_FF             
0x00007BA1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007BA3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007BA5     22  80148DA2986682B5814182A482C282DE82AD88BB8142         LEN8_STRING_CP932         length=20, text="困惑し、うつむく綾。"
0x00007BBB      1  FF                                                   TERMINATOR_FF             
0x00007BBC      1  FF                                                   TERMINATOR_FF             
0x00007BBD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007BBF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007BC1      1  FF                                                   TERMINATOR_FF             
0x00007BC2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007BC4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007BC6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007BC8      1  FF                                                   TERMINATOR_FF             
0x00007BC9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007BCB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007BCD      3  F31234                                               IMM16_F3                  u16_be=4660, u16_le=13330
0x00007BD0      1  FF                                                   TERMINATOR_FF             
0x00007BD1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007BD3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007BD5     74  804894DE8F9782CD82A282C282C582E082BB82A482BE8142...  LEN8_STRING_CP932         length=72, text="彼女はいつでもそうだ。いろいろなことに対応できるよう準備をおこたらない。"
0x00007C1F      1  FF                                                   TERMINATOR_FF             
0x00007C20      1  FF                                                   TERMINATOR_FF             
0x00007C21      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007C23      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007C25      1  FF                                                   TERMINATOR_FF             
0x00007C26      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007C28      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007C2A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007C2C      1  FF                                                   TERMINATOR_FF             
0x00007C2D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007C2F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007C31      3  F31235                                               IMM16_F3                  u16_be=4661, u16_le=13586
0x00007C34      1  FF                                                   TERMINATOR_FF             
0x00007C35      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007C37      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007C39     54  803482BB82EA82CD814193CB94AD934982C88E9691D482C9...  LEN8_STRING_CP932         length=52, text="それは、突発的な事態に、うまく対応できない\nからだ。"
0x00007C6F      1  FF                                                   TERMINATOR_FF             
0x00007C70      1  FF                                                   TERMINATOR_FF             
0x00007C71      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007C73      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007C75      1  FF                                                   TERMINATOR_FF             
0x00007C76      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007C78      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007C7A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007C7C      1  FF                                                   TERMINATOR_FF             
0x00007C7D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007C7F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007C81      3  F31236                                               IMM16_F3                  u16_be=4662, u16_le=13842
0x00007C84      1  FF                                                   TERMINATOR_FF             
0x00007C85      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007C87      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007C89     58  803882C782B182A982AA816381635C6E82B782B282AD9573...  LEN8_STRING_CP932         length=56, text="どこかが……\nすごく不器用だし、彼女もそれを知っている。"
0x00007CC3      1  FF                                                   TERMINATOR_FF             
0x00007CC4      1  FF                                                   TERMINATOR_FF             
0x00007CC5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007CC7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007CC9      1  FF                                                   TERMINATOR_FF             
0x00007CCA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007CCC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00007CCE      8  800653452D533635                                     LEN8_STRING_CP932         length=6, text="SE-S65"
0x00007CD6      1  FF                                                   TERMINATOR_FF             
0x00007CD7      1  FF                                                   TERMINATOR_FF             
0x00007CD8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007CDA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007CDC      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00007CDE      1  FF                                                   TERMINATOR_FF             
0x00007CDF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007CE1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007CE3      1  FF                                                   TERMINATOR_FF             
0x00007CE4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007CE6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007CE8      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00007CEA      1  FF                                                   TERMINATOR_FF             
0x00007CEB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007CED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007CEF      3  F30129                                               IMM16_F3                  u16_be=297, u16_le=10497
0x00007CF2      1  FF                                                   TERMINATOR_FF             
0x00007CF3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007CF5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007CF7      1  FF                                                   TERMINATOR_FF             
0x00007CF8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007CFA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007CFC      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00007CFE      1  FF                                                   TERMINATOR_FF             
0x00007CFF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D01      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007D03      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00007D05      1  FF                                                   TERMINATOR_FF             
0x00007D06      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D08      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00007D0A      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00007D0C      1  FF                                                   TERMINATOR_FF             
0x00007D0D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007D0F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007D11      1  FF                                                   TERMINATOR_FF             
0x00007D12      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D14      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D16      2  F221                                                 IMM8_F2                   u8=33, s8=33
0x00007D18      1  FF                                                   TERMINATOR_FF             
0x00007D19      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D1B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007D1D      2  F214                                                 IMM8_F2                   u8=20, s8=20
0x00007D1F      1  FF                                                   TERMINATOR_FF             
0x00007D20      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D22      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00007D24      2  F233                                                 IMM8_F2                   u8=51, s8=51
0x00007D26      1  FF                                                   TERMINATOR_FF             
0x00007D27      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D29      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00007D2B      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00007D2D      1  FF                                                   TERMINATOR_FF             
0x00007D2E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007D30      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007D32      1  FF                                                   TERMINATOR_FF             
0x00007D33      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D35      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D37      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007D39      1  FF                                                   TERMINATOR_FF             
0x00007D3A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D3C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007D3E      3  F31237                                               IMM16_F3                  u16_be=4663, u16_le=14098
0x00007D41      1  FF                                                   TERMINATOR_FF             
0x00007D42      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007D44      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007D46     54  803482DA82AD82CD88BB82CC8EE882F082C682C182BD8142...  LEN8_STRING_CP932         length=52, text="ぼくは綾の手をとった。\nあの光の中でふれた彼女の手。"
0x00007D7C      1  FF                                                   TERMINATOR_FF             
0x00007D7D      1  FF                                                   TERMINATOR_FF             
0x00007D7E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007D80      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007D82      1  FF                                                   TERMINATOR_FF             
0x00007D83      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D85      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D87      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00007D89      1  FF                                                   TERMINATOR_FF             
0x00007D8A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007D8C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007D8E      3  F31238                                               IMM16_F3                  u16_be=4664, u16_le=14354
0x00007D91      1  FF                                                   TERMINATOR_FF             
0x00007D92      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007D94      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00007D96      9  800741593033353430                                   LEN8_STRING_CP932         length=7, text="AY03540"
0x00007D9F      1  FF                                                   TERMINATOR_FF             
0x00007DA0      1  FF                                                   TERMINATOR_FF             
0x00007DA1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007DA3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007DA5     12  800A81758163816382A08176                             LEN8_STRING_CP932         length=10, text="「……あ」"
0x00007DB1      1  FF                                                   TERMINATOR_FF             
0x00007DB2      1  FF                                                   TERMINATOR_FF             
0x00007DB3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007DB5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007DB7      1  FF                                                   TERMINATOR_FF             
0x00007DB8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007DBA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007DBC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007DBE      1  FF                                                   TERMINATOR_FF             
0x00007DBF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007DC1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007DC3      3  F31239                                               IMM16_F3                  u16_be=4665, u16_le=14610
0x00007DC6      1  FF                                                   TERMINATOR_FF             
0x00007DC7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007DC9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007DCB    116  8072817582C782B182A9975682D182C98D7382B182A48142...  LEN8_STRING_CP932         length=114, text="「どこか遊びに行こう。遊園地でも、映画館でも。本を読みたいなら、図書館へ行けばいいし、海が見たいなら電車に乗ろう」"
0x00007E3F      1  FF                                                   TERMINATOR_FF             
0x00007E40      1  FF                                                   TERMINATOR_FF             
0x00007E41      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007E43      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007E45      1  FF                                                   TERMINATOR_FF             
0x00007E46      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007E48      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007E4A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007E4C      1  FF                                                   TERMINATOR_FF             
0x00007E4D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007E4F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007E51      3  F3123A                                               IMM16_F3                  u16_be=4666, u16_le=14866
0x00007E54      1  FF                                                   TERMINATOR_FF             
0x00007E55      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007E57      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007E59     98  8060817582628263835683878362837682CC8E8E92AE8352...  LEN8_STRING_CP932         length=96, text="「ＣＤショップの試聴コーナーで新譜を聞き\nたおしてもいいし、公園をぶらぶらしてるだけ\nでもいい」"
0x00007EBB      1  FF                                                   TERMINATOR_FF             
0x00007EBC      1  FF                                                   TERMINATOR_FF             
0x00007EBD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007EBF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007EC1      1  FF                                                   TERMINATOR_FF             
0x00007EC2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007EC4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007EC6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007EC8      1  FF                                                   TERMINATOR_FF             
0x00007EC9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007ECB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007ECD      3  F3411B                                               IMM16_F3                  u16_be=16667, u16_le=6977
0x00007ED0      1  FF                                                   TERMINATOR_FF             
0x00007ED1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007ED3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007ED5     32  801E817582BE82A982E7814182D982E78141899397B682B5...  LEN8_STRING_CP932         length=30, text="「だから、ほら、遠慮しないで」"
0x00007EF5      1  FF                                                   TERMINATOR_FF             
0x00007EF6      1  FF                                                   TERMINATOR_FF             
0x00007EF7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007EF9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007EFB      1  FF                                                   TERMINATOR_FF             
0x00007EFC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007EFE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007F00      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007F02      1  FF                                                   TERMINATOR_FF             
0x00007F03      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007F05      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007F07      3  F3123B                                               IMM16_F3                  u16_be=4667, u16_le=15122
0x00007F0A      1  FF                                                   TERMINATOR_FF             
0x00007F0B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007F0D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007F0F     88  805688BB82CD81418DA182C982E08B8382AB82BE82B582BB...  LEN8_STRING_CP932         length=86, text="綾は、今にも泣きだしそうな困り顔をしていた。口をへの字にして、ぼくをにらみつけている。"
0x00007F67      1  FF                                                   TERMINATOR_FF             
0x00007F68      1  FF                                                   TERMINATOR_FF             
0x00007F69      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007F6B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007F6D      1  FF                                                   TERMINATOR_FF             
0x00007F6E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007F70      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007F72      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00007F74      1  FF                                                   TERMINATOR_FF             
0x00007F75      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007F77      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007F79      3  F3123C                                               IMM16_F3                  u16_be=4668, u16_le=15378
0x00007F7C      1  FF                                                   TERMINATOR_FF             
0x00007F7D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007F7F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00007F81      9  800741593033353530                                   LEN8_STRING_CP932         length=7, text="AY03550"
0x00007F8A      1  FF                                                   TERMINATOR_FF             
0x00007F8B      1  FF                                                   TERMINATOR_FF             
0x00007F8C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007F8E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007F90     48  802E817582BE82C182C48163816382BB82EA82C182C48163...  LEN8_STRING_CP932         length=46, text="「だって……それって……デートみたいだし……」"
0x00007FC0      1  FF                                                   TERMINATOR_FF             
0x00007FC1      1  FF                                                   TERMINATOR_FF             
0x00007FC2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00007FC4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00007FC6      1  FF                                                   TERMINATOR_FF             
0x00007FC7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007FC9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007FCB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00007FCD      1  FF                                                   TERMINATOR_FF             
0x00007FCE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00007FD0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00007FD2      3  F3411C                                               IMM16_F3                  u16_be=16668, u16_le=7233
0x00007FD5      1  FF                                                   TERMINATOR_FF             
0x00007FD6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007FD8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00007FDA     38  802482C882F182BE814182BB82F182C882B182C682F08B43...  LEN8_STRING_CP932         length=36, text="なんだ、そんなことを気にしてたんだ。"
0x00008000      1  FF                                                   TERMINATOR_FF             
0x00008001      1  FF                                                   TERMINATOR_FF             
0x00008002      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008004      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008006      1  FF                                                   TERMINATOR_FF             
0x00008007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008009      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000800B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000800D      1  FF                                                   TERMINATOR_FF             
0x0000800E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008010      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00008012      3  F3123D                                               IMM16_F3                  u16_be=4669, u16_le=15634
0x00008015      1  FF                                                   TERMINATOR_FF             
0x00008016      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008018      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000801A     34  8020817582B682E182A081418366815B836782C582A282A2...  LEN8_STRING_CP932         length=32, text="「じゃあ、デートでいいじゃない」"
0x0000803C      1  FF                                                   TERMINATOR_FF             
0x0000803D      1  FF                                                   TERMINATOR_FF             
0x0000803E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008040      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008042      1  FF                                                   TERMINATOR_FF             
0x00008043      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008045      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008047      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00008049      1  FF                                                   TERMINATOR_FF             
0x0000804A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000804C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000804E      3  F3123E                                               IMM16_F3                  u16_be=4670, u16_le=15890
0x00008051      1  FF                                                   TERMINATOR_FF             
0x00008052      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008054      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00008056      9  800741593033353630                                   LEN8_STRING_CP932         length=7, text="AY03560"
0x0000805F      1  FF                                                   TERMINATOR_FF             
0x00008060      1  FF                                                   TERMINATOR_FF             
0x00008061      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008063      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008065     14  800C817582C582E0816381638176                         LEN8_STRING_CP932         length=12, text="「でも……」"
0x00008073      1  FF                                                   TERMINATOR_FF             
0x00008074      1  FF                                                   TERMINATOR_FF             
0x00008075      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008077      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008079      1  FF                                                   TERMINATOR_FF             
0x0000807A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000807C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000807E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00008080      1  FF                                                   TERMINATOR_FF             
0x00008081      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008083      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00008085      3  F3123F                                               IMM16_F3                  u16_be=4671, u16_le=16146
0x00008088      1  FF                                                   TERMINATOR_FF             
0x00008089      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000808B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000808D     64  803E88BB82CD82A482C282DE82A282C481419573969E82BB...  LEN8_STRING_CP932         length=62, text="綾はうつむいて、不満そうな、困ったような\n表情を浮かべていた。"
0x000080CD      1  FF                                                   TERMINATOR_FF             
0x000080CE      1  FF                                                   TERMINATOR_FF             
0x000080CF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000080D1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000080D3      1  FF                                                   TERMINATOR_FF             
0x000080D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000080D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000080D8      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000080DA      1  FF                                                   TERMINATOR_FF             
0x000080DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000080DD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000080DF      3  F3412A                                               IMM16_F3                  u16_be=16682, u16_le=10817
0x000080E2      1  FF                                                   TERMINATOR_FF             
0x000080E3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000080E5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000080E7     62  803C82BD82D482F1814182B182F182C882CC82C182C482C7...  LEN8_STRING_CP932         length=60, text="たぶん、こんなのってどうだろう、と真面目に\n考えているのだ。"
0x00008125      1  FF                                                   TERMINATOR_FF             
0x00008126      1  FF                                                   TERMINATOR_FF             
0x00008127      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008129      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000812B      1  FF                                                   TERMINATOR_FF             
0x0000812C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000812E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008130      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00008132      1  FF                                                   TERMINATOR_FF             
0x00008133      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008135      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00008137      3  F31240                                               IMM16_F3                  u16_be=4672, u16_le=16402
0x0000813A      1  FF                                                   TERMINATOR_FF             
0x0000813B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000813D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000813F    104  806682AB82C182C688BB82CC928682C582CD814182A882C2...  LEN8_STRING_CP932         length=102, text="きっと綾の中では、おつきあいしてくださいと\nいわれて、それについて真剣に考えて、友だち\nにも相談して。"
0x000081A7      1  FF                                                   TERMINATOR_FF             
0x000081A8      1  FF                                                   TERMINATOR_FF             
0x000081A9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000081AB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000081AD      1  FF                                                   TERMINATOR_FF             
0x000081AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000081B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000081B2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000081B4      1  FF                                                   TERMINATOR_FF             
0x000081B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000081B7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000081B9      3  F31241                                               IMM16_F3                  u16_be=4673, u16_le=16658
0x000081BC      1  FF                                                   TERMINATOR_FF             
0x000081BD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000081BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000081C1     76  804A82BF82E182F182C682A882C282AB82A082A282B582DC...  LEN8_STRING_CP932         length=74, text="ちゃんとおつきあいします、という返事をして\nからするのがデートなのだろう。"
0x0000820D      1  FF                                                   TERMINATOR_FF             
0x0000820E      1  FF                                                   TERMINATOR_FF             
0x0000820F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008211      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008213      1  FF                                                   TERMINATOR_FF             
0x00008214      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008216      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008218      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000821A      1  FF                                                   TERMINATOR_FF             
0x0000821B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000821D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000821F      3  F31242                                               IMM16_F3                  u16_be=4674, u16_le=16914
0x00008222      1  FF                                                   TERMINATOR_FF             
0x00008223      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008225      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008227     64  803E82AF82EA82C7814182DA82AD82BD82BF82C982CD8141...  LEN8_STRING_CP932         length=62, text="けれど、ぼくたちには、あまり多くの時間は\n残されていなかった。"
0x00008267      1  FF                                                   TERMINATOR_FF             
0x00008268      1  FF                                                   TERMINATOR_FF             
0x00008269      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000826B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000826D      1  FF                                                   TERMINATOR_FF             
0x0000826E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008270      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008272      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00008274      1  FF                                                   TERMINATOR_FF             
0x00008275      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008277      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00008279      3  F3412B                                               IMM16_F3                  u16_be=16683, u16_le=11073
0x0000827C      1  FF                                                   TERMINATOR_FF             
0x0000827D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000827F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008281     74  804896B2928682C982C882C182C4814182C882C982A982F0...  LEN8_STRING_CP932         length=72, text="夢中になって、なにかをすれば、すぐに\nなくなってしまうくらいの時間しか。"
0x000082CB      1  FF                                                   TERMINATOR_FF             
0x000082CC      1  FF                                                   TERMINATOR_FF             
0x000082CD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000082CF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000082D1      1  FF                                                   TERMINATOR_FF             
0x000082D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000082D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000082D6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000082D8      1  FF                                                   TERMINATOR_FF             
0x000082D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000082DB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000082DD      3  F31243                                               IMM16_F3                  u16_be=4675, u16_le=17170
0x000082E0      1  FF                                                   TERMINATOR_FF             
0x000082E1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000082E3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000082E5    100  806282BE82A982E781418FAD82B58BAD88F882BE82C682B5...  LEN8_STRING_CP932         length=98, text="だから、少し強引だとしても、彼女が困惑して\nいたとしても、ぼくは彼女と一緒にどこかへ行きたかった。"
0x00008349      1  FF                                                   TERMINATOR_FF             
0x0000834A      1  FF                                                   TERMINATOR_FF             
0x0000834B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000834D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000834F      1  FF                                                   TERMINATOR_FF             
0x00008350      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008352      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008354      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00008356      1  FF                                                   TERMINATOR_FF             
0x00008357      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008359      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000835B      3  F31244                                               IMM16_F3                  u16_be=4676, u16_le=17426
0x0000835E      1  FF                                                   TERMINATOR_FF             
0x0000835F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008361      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008363     68  8042958192CA82CC8D828D5A90B682AA8141958192CA82C9...  LEN8_STRING_CP932         length=66, text="普通の高校生が、普通にするように、ふたりで\n一緒にすごしたかった。"
0x000083A7      1  FF                                                   TERMINATOR_FF             
0x000083A8      1  FF                                                   TERMINATOR_FF             
0x000083A9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000083AB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000083AD      1  FF                                                   TERMINATOR_FF             
0x000083AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000083B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000083B2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000083B4      1  FF                                                   TERMINATOR_FF             
0x000083B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000083B7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000083B9      3  F31245                                               IMM16_F3                  u16_be=4677, u16_le=17682
0x000083BC      1  FF                                                   TERMINATOR_FF             
0x000083BD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000083BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000083C1     38  8024817582B682E181418D7382AD82E681428DC58F8982CD...  LEN8_STRING_CP932         length=36, text="「じゃ、行くよ。最初はどこがいい？」"
0x000083E7      1  FF                                                   TERMINATOR_FF             
0x000083E8      1  FF                                                   TERMINATOR_FF             
0x000083E9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000083EB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000083ED      1  FF                                                   TERMINATOR_FF             
0x000083EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000083F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000083F2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000083F4      1  FF                                                   TERMINATOR_FF             
0x000083F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000083F7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000083F9      3  F3412C                                               IMM16_F3                  u16_be=16684, u16_le=11329
0x000083FC      1  FF                                                   TERMINATOR_FF             
0x000083FD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000083FF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008401     36  802288BB82CD82A482C282DE82A282BD82DC82DC81418FAC...  LEN8_STRING_CP932         length=34, text="綾はうつむいたまま、小さくいった。"
0x00008425      1  FF                                                   TERMINATOR_FF             
0x00008426      1  FF                                                   TERMINATOR_FF             
0x00008427      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008429      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000842B      1  FF                                                   TERMINATOR_FF             
0x0000842C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000842E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008430      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00008432      1  FF                                                   TERMINATOR_FF             
0x00008433      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008435      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00008437      3  F31246                                               IMM16_F3                  u16_be=4678, u16_le=17938
0x0000843A      1  FF                                                   TERMINATOR_FF             
0x0000843B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000843D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000843F      9  800741593033353730                                   LEN8_STRING_CP932         length=7, text="AY03570"
0x00008448      1  FF                                                   TERMINATOR_FF             
0x00008449      1  FF                                                   TERMINATOR_FF             
0x0000844A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000844C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000844E     38  8024817582A48163816382A482F181428163816382DC82A9...  LEN8_STRING_CP932         length=36, text="「う……うん。……まかせるけど……」"
0x00008474      1  FF                                                   TERMINATOR_FF             
0x00008475      1  FF                                                   TERMINATOR_FF             
0x00008476      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008478      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000847A      1  FF                                                   TERMINATOR_FF             
0x0000847B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000847D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000847F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00008481      1  FF                                                   TERMINATOR_FF             
0x00008482      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008484      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00008486      3  F3412D                                               IMM16_F3                  u16_be=16685, u16_le=11585
0x00008489      1  FF                                                   TERMINATOR_FF             
0x0000848A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000848C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000848E     74  804882DA82AD82CC8BAD88F882B382C9814182D982F182CC...  LEN8_STRING_CP932         length=72, text="ぼくの強引さに、ほんの少し抗議の意味をこめて、かすかな不満をにじませる。"
0x000084D8      1  FF                                                   TERMINATOR_FF             
0x000084D9      1  FF                                                   TERMINATOR_FF             
0x000084DA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000084DC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000084DE      1  FF                                                   TERMINATOR_FF             
0x000084DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000084E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000084E3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000084E5      1  FF                                                   TERMINATOR_FF             
0x000084E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000084E8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000084EA      3  F3412E                                               IMM16_F3                  u16_be=16686, u16_le=11841
0x000084ED      1  FF                                                   TERMINATOR_FF             
0x000084EE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000084F0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000084F2     56  803682BB82A482B982B882C982CD82A282E782EA82C882A2...  LEN8_STRING_CP932         length=54, text="そうせずにはいられない性格の綾。\n綾……という女の子。"
0x0000852A      1  FF                                                   TERMINATOR_FF             
0x0000852B      1  FF                                                   TERMINATOR_FF             
0x0000852C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000852E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008530      1  FF                                                   TERMINATOR_FF             
0x00008531      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008533      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008535      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00008537      1  FF                                                   TERMINATOR_FF             
0x00008538      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000853A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000853C      3  F31247                                               IMM16_F3                  u16_be=4679, u16_le=18194
0x0000853F      1  FF                                                   TERMINATOR_FF             
0x00008540      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008542      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008544     42  802882BB82B582C4814182DA82AD82CD88BB82CC8EE882F0...  LEN8_STRING_CP932         length=40, text="そして、ぼくは綾の手をひいて歩き始める。"
0x0000856E      1  FF                                                   TERMINATOR_FF             
0x0000856F      1  FF                                                   TERMINATOR_FF             
0x00008570      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008572      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008574      1  FF                                                   TERMINATOR_FF             
0x00008575      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008577      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008579      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000857B      1  FF                                                   TERMINATOR_FF             
0x0000857C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000857E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00008580      3  F3412F                                               IMM16_F3                  u16_be=16687, u16_le=12097
0x00008583      1  FF                                                   TERMINATOR_FF             
0x00008584      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008586      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008588     54  803488BB82CD81418F61815882C682A282C182BD976C8E71...  LEN8_STRING_CP932         length=52, text="綾は、渋々といった様子でぼくについてくる。でも……。"
0x000085BE      1  FF                                                   TERMINATOR_FF             
0x000085BF      1  FF                                                   TERMINATOR_FF             
0x000085C0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000085C2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000085C4      1  FF                                                   TERMINATOR_FF             
0x000085C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000085C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000085C9      2  F221                                                 IMM8_F2                   u8=33, s8=33
0x000085CB      1  FF                                                   TERMINATOR_FF             
0x000085CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000085CE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000085D0      2  F214                                                 IMM8_F2                   u8=20, s8=20
0x000085D2      1  FF                                                   TERMINATOR_FF             
0x000085D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000085D5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000085D7      2  F233                                                 IMM8_F2                   u8=51, s8=51
0x000085D9      1  FF                                                   TERMINATOR_FF             
0x000085DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000085DC      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x000085DE      2  F20F                                                 IMM8_F2                   u8=15, s8=15
0x000085E0      1  FF                                                   TERMINATOR_FF             
0x000085E1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000085E3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000085E5      1  FF                                                   TERMINATOR_FF             
0x000085E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000085E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000085EA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000085EC      1  FF                                                   TERMINATOR_FF             
0x000085ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000085EF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000085F1      3  F34130                                               IMM16_F3                  u16_be=16688, u16_le=12353
0x000085F4      1  FF                                                   TERMINATOR_FF             
0x000085F5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000085F7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000085F9     42  802882DA82AD82CC8EE882F0814188BB82CC82AB82E182B5...  LEN8_STRING_CP932         length=40, text="ぼくの手を、綾のきゃしゃな手が握り返す。"
0x00008623      1  FF                                                   TERMINATOR_FF             
0x00008624      1  FF                                                   TERMINATOR_FF             
0x00008625      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008627      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008629      1  FF                                                   TERMINATOR_FF             
0x0000862A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000862C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000862E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00008630      1  FF                                                   TERMINATOR_FF             
0x00008631      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008633      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00008635      3  F31248                                               IMM16_F3                  u16_be=4680, u16_le=18450
0x00008638      1  FF                                                   TERMINATOR_FF             
0x00008639      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000863B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000863D     54  803482BB82EA82AA814195738AED977082C894DE8F9782C9...  LEN8_STRING_CP932         length=52, text="それが、不器用な彼女にできる精一杯の意思表示だった。"
0x00008673      1  FF                                                   TERMINATOR_FF             
0x00008674      1  FF                                                   TERMINATOR_FF             
0x00008675      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008677      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008679      1  FF                                                   TERMINATOR_FF             
0x0000867A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000867C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000867E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00008680      1  FF                                                   TERMINATOR_FF             
0x00008681      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008683      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00008685      3  F34131                                               IMM16_F3                  u16_be=16689, u16_le=12609
0x00008688      1  FF                                                   TERMINATOR_FF             
0x00008689      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000868B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000868D     26  8018916692BC82B682E182C882A282E682CB814188BB82CD...  LEN8_STRING_CP932         length=24, text="素直じゃないよね、綾は。"
0x000086A7      1  FF                                                   TERMINATOR_FF             
0x000086A8      1  FF                                                   TERMINATOR_FF             
0x000086A9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000086AB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000086AD      1  FF                                                   TERMINATOR_FF             
0x000086AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000086B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000086B2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000086B4      1  FF                                                   TERMINATOR_FF             
0x000086B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000086B7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000086B9      3  F34132                                               IMM16_F3                  u16_be=16690, u16_le=12865
0x000086BC      1  FF                                                   TERMINATOR_FF             
0x000086BD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000086BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000086C1     80  804E82BB82A48E7682C182BD82C682AB814182DA82AD82CD...  LEN8_STRING_CP932         length=78, text="そう思ったとき、ぼくは彼女といられるわずかな時間を本当に大切にしようと思った。"
0x00008711      1  FF                                                   TERMINATOR_FF             
0x00008712      1  FF                                                   TERMINATOR_FF             
0x00008713      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008715      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008717      1  FF                                                   TERMINATOR_FF             
0x00008718      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000871A      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x0000871C      8  80068CF6898082D6                                     LEN8_STRING_CP932         length=6, text="公園へ"
0x00008724      1  FF                                                   TERMINATOR_FF             
0x00008725      1  FF                                                   TERMINATOR_FF             
0x00008726      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008728      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x0000872A      3  F31249                                               IMM16_F3                  u16_be=4681, u16_le=18706
0x0000872D      1  FF                                                   TERMINATOR_FF             
0x0000872E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00008730      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00008732     10  800897568980926E82D6                                 LEN8_STRING_CP932         length=8, text="遊園地へ"
0x0000873C      1  FF                                                   TERMINATOR_FF             
0x0000873D      1  FF                                                   TERMINATOR_FF             
0x0000873E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008740      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00008742      3  F3124A                                               IMM16_F3                  u16_be=4682, u16_le=18962
0x00008745      1  FF                                                   TERMINATOR_FF             
0x00008746      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008748      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x0000874A      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000874C      1  FF                                                   TERMINATOR_FF             
0x0000874D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000874F      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00008751      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008753      1  FF                                                   TERMINATOR_FF             
0x00008754      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008756      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008758      2  F211                                                 IMM8_F2                   u8=17, s8=17
0x0000875A      1  FF                                                   TERMINATOR_FF             
0x0000875B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000875D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000875F      1  FF                                                   TERMINATOR_FF             
0x00008760      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00008762      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00008765      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008767      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00008768      1  FF                                                   TERMINATOR_FF             
0x00008769      2  0087                                                 WORD_00XX                 u16_be=135, low_byte=135
0x0000876B      1  78                                                   OPAQUE_RAW_BYTES          bytes=78
0x0000876C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000876E      2  0059                                                 WORD_00XX                 u16_be=89, low_byte=89
0x00008770      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00008772      1  FF                                                   TERMINATOR_FF             
0x00008773      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00008775      2  0087                                                 WORD_00XX                 u16_be=135, low_byte=135
0x00008777      1  8B                                                   OPAQUE_RAW_BYTES          bytes=8B
0x00008778      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000877A      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x0000877D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000877F      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00008780      1  FF                                                   TERMINATOR_FF             
0x00008781      2  0087                                                 WORD_00XX                 u16_be=135, low_byte=135
0x00008783      1  8B                                                   OPAQUE_RAW_BYTES          bytes=8B
0x00008784      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008786      2  0059                                                 WORD_00XX                 u16_be=89, low_byte=89
0x00008788      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000878A      1  FF                                                   TERMINATOR_FF             
0x0000878B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000878D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000878F      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00008791      1  FF                                                   TERMINATOR_FF             
0x00008792      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00008794      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00008796      1  FF                                                   TERMINATOR_FF             
0x00008797      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00008799      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000879B      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x0000879D      1  FF                                                   TERMINATOR_FF             
0x0000879E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000087A0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000087A2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000087A4      1  FF                                                   TERMINATOR_FF             
0x000087A5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000087A7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000087A9      1  FF                                                   TERMINATOR_FF             
0x000087AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000087AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000087AE      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000087B0      1  FF                                                   TERMINATOR_FF             
0x000087B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000087B3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000087B5      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x000087B7      1  FF                                                   TERMINATOR_FF             
0x000087B8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000087BA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000087BC      1  FF                                                   TERMINATOR_FF             
0x000087BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000087BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000087C1      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000087C3      1  FF                                                   TERMINATOR_FF             
0x000087C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000087C6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000087C8      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000087CA      1  FF                                                   TERMINATOR_FF             
0x000087CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000087CD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000087CF      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000087D1      1  FF                                                   TERMINATOR_FF             
0x000087D2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000087D4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000087D6      1  FF                                                   TERMINATOR_FF             
0x000087D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000087D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000087DB      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000087DD      1  FF                                                   TERMINATOR_FF             
0x000087DE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000087E0      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000087E2     14  800C50533241303537612E62696E                         LEN8_STRING_CP932         length=12, text="PS2A057a.bin"
0x000087F0      1  FF                                                   TERMINATOR_FF             
0x000087F1      1  FF                                                   TERMINATOR_FF             
0x000087F2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000087F4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000087F6      1  FF                                                   TERMINATOR_FF             
0x000087F7      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000087F9      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000087FB      1  FF                                                   TERMINATOR_FF             
0x000087FC      2  0088                                                 WORD_00XX                 u16_be=136, low_byte=136
0x000087FE      1  06                                                   OPAQUE_RAW_BYTES          bytes=06
0x000087FF      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x00008801      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00008803      2  0087                                                 WORD_00XX                 u16_be=135, low_byte=135
0x00008805      1  F7                                                   OPAQUE_RAW_BYTES          bytes=F7
0x00008806      1  FF                                                   TERMINATOR_FF             
0x00008807      1  FF                                                   TERMINATOR_FF             
