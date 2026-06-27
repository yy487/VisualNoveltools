; Full conservative disassembly for PS2A011A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      2  F250                                                 IMM8_F2                   u8=80, s8=80
0x0000000D      1  FF                                                   TERMINATOR_FF             
0x0000000E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000010      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000012      1  FF                                                   TERMINATOR_FF             
0x00000013      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000015      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000017      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000019      1  FF                                                   TERMINATOR_FF             
0x0000001A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000001C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000001E      3  F301B3                                               IMM16_F3                  u16_be=435, u16_le=45825
0x00000021      1  FF                                                   TERMINATOR_FF             
0x00000022      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000024      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000026      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000028      1  FF                                                   TERMINATOR_FF             
0x00000029      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000002B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000002D      1  FF                                                   TERMINATOR_FF             
0x0000002E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000030      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000032      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00000034      1  FF                                                   TERMINATOR_FF             
0x00000035      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000037      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000039      2  F221                                                 IMM8_F2                   u8=33, s8=33
0x0000003B      1  FF                                                   TERMINATOR_FF             
0x0000003C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000003E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000040      1  FF                                                   TERMINATOR_FF             
0x00000041      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000043      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000045      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000047      1  FF                                                   TERMINATOR_FF             
0x00000048      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000004A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000004C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000004E      1  FF                                                   TERMINATOR_FF             
0x0000004F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000051      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000053      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000055      1  FF                                                   TERMINATOR_FF             
0x00000056      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000058      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000005A      1  FF                                                   TERMINATOR_FF             
0x0000005B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000005D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000005F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000061      1  FF                                                   TERMINATOR_FF             
0x00000062      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000064      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000066      3  F302FB                                               IMM16_F3                  u16_be=763, u16_le=64258
0x00000069      1  FF                                                   TERMINATOR_FF             
0x0000006A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000006C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000006E    112  806E8BCA82CD82E682A482E282AD93A682B08F6F82B382C8...  LEN8_STRING_CP932         length=110, text="玉はようやく逃げ出さないようになった。\nバイトの前、バイトの休みは、とりあえず教室に残るよーにはなってくれた。"
0x000000DE      1  FF                                                   TERMINATOR_FF             
0x000000DF      1  FF                                                   TERMINATOR_FF             
0x000000E0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000E2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000E4      1  FF                                                   TERMINATOR_FF             
0x000000E5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000000EB      1  FF                                                   TERMINATOR_FF             
0x000000EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000EE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000000F0      3  F302FC                                               IMM16_F3                  u16_be=764, u16_le=64514
0x000000F3      1  FF                                                   TERMINATOR_FF             
0x000000F4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000F6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000F8     80  804E82E082BF82EB82F1836F8343836782CC95FB82AA8A79...  LEN8_STRING_CP932         length=78, text="もちろんバイトの方が楽しいみたいだし、\nあいかわらず、やる気はないのだけれど。"
0x00000148      1  FF                                                   TERMINATOR_FF             
0x00000149      1  FF                                                   TERMINATOR_FF             
0x0000014A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000014C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000014E      1  FF                                                   TERMINATOR_FF             
0x0000014F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000151      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000153      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000155      1  FF                                                   TERMINATOR_FF             
0x00000156      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000158      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000015A      3  F302FD                                               IMM16_F3                  u16_be=765, u16_le=64770
0x0000015D      1  FF                                                   TERMINATOR_FF             
0x0000015E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000160      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000162     54  803482BB82EA82C982B582C482E081418BCA82CC95D78BAD...  LEN8_STRING_CP932         length=52, text="それにしても、玉の勉強できなさ加減には\n驚かされる。"
0x00000198      1  FF                                                   TERMINATOR_FF             
0x00000199      1  FF                                                   TERMINATOR_FF             
0x0000019A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000019C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000019E      1  FF                                                   TERMINATOR_FF             
0x0000019F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001A3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001A5      1  FF                                                   TERMINATOR_FF             
0x000001A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001A8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001AA      3  F302FE                                               IMM16_F3                  u16_be=766, u16_le=65026
0x000001AD      1  FF                                                   TERMINATOR_FF             
0x000001AE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001B2     68  804292868A7790B682C882DD82CC8A7797CD82C681418FAC...  LEN8_STRING_CP932         length=66, text="中学生なみの学力と、小学校低学年なみの\n集中力を思う存分発揮する。"
0x000001F6      1  FF                                                   TERMINATOR_FF             
0x000001F7      1  FF                                                   TERMINATOR_FF             
0x000001F8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001FA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001FC      1  FF                                                   TERMINATOR_FF             
0x000001FD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001FF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000201      8  800653452D533935                                     LEN8_STRING_CP932         length=6, text="SE-S95"
0x00000209      1  FF                                                   TERMINATOR_FF             
0x0000020A      1  FF                                                   TERMINATOR_FF             
0x0000020B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000020D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000020F      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00000211      1  FF                                                   TERMINATOR_FF             
0x00000212      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000214      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000216      1  FF                                                   TERMINATOR_FF             
0x00000217      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000219      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000021B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000021D      1  FF                                                   TERMINATOR_FF             
0x0000021E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000220      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000222      3  F302FF                                               IMM16_F3                  u16_be=767, u16_le=65282
0x00000225      1  FF                                                   TERMINATOR_FF             
0x00000226      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000228      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000022A      9  80074D543034393830                                   LEN8_STRING_CP932         length=7, text="MT04980"
0x00000233      1  FF                                                   TERMINATOR_FF             
0x00000234      1  FF                                                   TERMINATOR_FF             
0x00000235      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000237      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000239    107  8069817582AD82E782B782AA815B333882C982F182BE82A9...  LEN8_STRING_CP932         length=105, text="「くらすがー38にんだからー、１ねんがー\nえっとー365にちでー……ん～……\n２がつの29にちってどーするー？」"
0x000002A4      1  FF                                                   TERMINATOR_FF             
0x000002A5      1  FF                                                   TERMINATOR_FF             
0x000002A6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002A8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002AA      1  FF                                                   TERMINATOR_FF             
0x000002AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002AF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000002B1      1  FF                                                   TERMINATOR_FF             
0x000002B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000002B6      3  F30300                                               IMM16_F3                  u16_be=768, u16_le=3
0x000002B9      1  FF                                                   TERMINATOR_FF             
0x000002BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002BC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002BE     30  801C817582BB82EA82CD8D6C82A682C882AD82C482A282A2...  LEN8_STRING_CP932         length=28, text="「それは考えなくていいから」"
0x000002DC      1  FF                                                   TERMINATOR_FF             
0x000002DD      1  FF                                                   TERMINATOR_FF             
0x000002DE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002E0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002E2      1  FF                                                   TERMINATOR_FF             
0x000002E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002E5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002E7      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000002E9      1  FF                                                   TERMINATOR_FF             
0x000002EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002EC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000002EE      3  F30301                                               IMM16_F3                  u16_be=769, u16_le=259
0x000002F1      1  FF                                                   TERMINATOR_FF             
0x000002F2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002F4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002F6      9  80074D543034393930                                   LEN8_STRING_CP932         length=7, text="MT04990"
0x000002FF      1  FF                                                   TERMINATOR_FF             
0x00000300      1  FF                                                   TERMINATOR_FF             
0x00000301      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000303      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000305     16  800E81758163816382D3815B82F18176                     LEN8_STRING_CP932         length=14, text="「……ふーん」"
0x00000315      1  FF                                                   TERMINATOR_FF             
0x00000316      1  FF                                                   TERMINATOR_FF             
0x00000317      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000319      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000031B      1  FF                                                   TERMINATOR_FF             
0x0000031C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000031E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000320      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000322      1  FF                                                   TERMINATOR_FF             
0x00000323      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000325      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000327      3  F30302                                               IMM16_F3                  u16_be=770, u16_le=515
0x0000032A      1  FF                                                   TERMINATOR_FF             
0x0000032B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000032D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000032F     14  800C817581638163816381638176                         LEN8_STRING_CP932         length=12, text="「…………」"
0x0000033D      1  FF                                                   TERMINATOR_FF             
0x0000033E      1  FF                                                   TERMINATOR_FF             
0x0000033F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000341      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000343      1  FF                                                   TERMINATOR_FF             
0x00000344      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000346      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000348      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000034A      1  FF                                                   TERMINATOR_FF             
0x0000034B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000034D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000034F      3  F301B2                                               IMM16_F3                  u16_be=434, u16_le=45569
0x00000352      1  FF                                                   TERMINATOR_FF             
0x00000353      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000355      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000357      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000359      1  FF                                                   TERMINATOR_FF             
0x0000035A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000035C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000035E      1  FF                                                   TERMINATOR_FF             
0x0000035F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000361      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000363      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000365      1  FF                                                   TERMINATOR_FF             
0x00000366      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000368      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000036A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000036C      1  FF                                                   TERMINATOR_FF             
0x0000036D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000036F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000371      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000373      1  FF                                                   TERMINATOR_FF             
0x00000374      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000376      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000378      1  FF                                                   TERMINATOR_FF             
0x00000379      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000037B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000037D      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000037F      1  FF                                                   TERMINATOR_FF             
0x00000380      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000382      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000384      3  F30303                                               IMM16_F3                  u16_be=771, u16_le=771
0x00000387      1  FF                                                   TERMINATOR_FF             
0x00000388      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000038A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000038C      9  80074D543035303030                                   LEN8_STRING_CP932         length=7, text="MT05000"
0x00000395      1  FF                                                   TERMINATOR_FF             
0x00000396      1  FF                                                   TERMINATOR_FF             
0x00000397      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000399      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000039B     46  802C8175838C8366815B82CD82BD82F182C982F182BE82A9...  LEN8_STRING_CP932         length=44, text="「レデーはたんにんだから、クラスいれるー？」"
0x000003C9      1  FF                                                   TERMINATOR_FF             
0x000003CA      1  FF                                                   TERMINATOR_FF             
0x000003CB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003CD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003CF      1  FF                                                   TERMINATOR_FF             
0x000003D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003D4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000003D6      1  FF                                                   TERMINATOR_FF             
0x000003D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003D9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000003DB      3  F30304                                               IMM16_F3                  u16_be=772, u16_le=1027
0x000003DE      1  FF                                                   TERMINATOR_FF             
0x000003DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003E1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003E3     20  8012817582A282EA82C882AD82C482A282A28176             LEN8_STRING_CP932         length=18, text="「いれなくていい」"
0x000003F7      1  FF                                                   TERMINATOR_FF             
0x000003F8      1  FF                                                   TERMINATOR_FF             
0x000003F9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003FB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003FD      1  FF                                                   TERMINATOR_FF             
0x000003FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000400      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000402      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000404      1  FF                                                   TERMINATOR_FF             
0x00000405      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000407      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000409      3  F301B3                                               IMM16_F3                  u16_be=435, u16_le=45825
0x0000040C      1  FF                                                   TERMINATOR_FF             
0x0000040D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000040F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000411      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000413      1  FF                                                   TERMINATOR_FF             
0x00000414      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000416      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000418      1  FF                                                   TERMINATOR_FF             
0x00000419      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000041B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000041D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000041F      1  FF                                                   TERMINATOR_FF             
0x00000420      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000422      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000424      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000426      1  FF                                                   TERMINATOR_FF             
0x00000427      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000429      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000042B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000042D      1  FF                                                   TERMINATOR_FF             
0x0000042E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000430      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000432      1  FF                                                   TERMINATOR_FF             
0x00000433      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000435      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000437      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000439      1  FF                                                   TERMINATOR_FF             
0x0000043A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000043C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000043E      3  F30305                                               IMM16_F3                  u16_be=773, u16_le=1283
0x00000441      1  FF                                                   TERMINATOR_FF             
0x00000442      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000444      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000446      9  80074D543035303130                                   LEN8_STRING_CP932         length=7, text="MT05010"
0x0000044F      1  FF                                                   TERMINATOR_FF             
0x00000450      1  FF                                                   TERMINATOR_FF             
0x00000451      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000453      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000455     16  800E81758163816382D3815B82F18176                     LEN8_STRING_CP932         length=14, text="「……ふーん」"
0x00000465      1  FF                                                   TERMINATOR_FF             
0x00000466      1  FF                                                   TERMINATOR_FF             
0x00000467      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000469      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000046B      1  FF                                                   TERMINATOR_FF             
0x0000046C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000046E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000470      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000472      1  FF                                                   TERMINATOR_FF             
0x00000473      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000475      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000477      3  F30306                                               IMM16_F3                  u16_be=774, u16_le=1539
0x0000047A      1  FF                                                   TERMINATOR_FF             
0x0000047B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000047D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000047F     14  800C817581638163816381638176                         LEN8_STRING_CP932         length=12, text="「…………」"
0x0000048D      1  FF                                                   TERMINATOR_FF             
0x0000048E      1  FF                                                   TERMINATOR_FF             
0x0000048F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000491      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000493      1  FF                                                   TERMINATOR_FF             
0x00000494      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000496      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000498      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000049A      1  FF                                                   TERMINATOR_FF             
0x0000049B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000049D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000049F      3  F301B2                                               IMM16_F3                  u16_be=434, u16_le=45569
0x000004A2      1  FF                                                   TERMINATOR_FF             
0x000004A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004A5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000004A7      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000004A9      1  FF                                                   TERMINATOR_FF             
0x000004AA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004AC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004AE      1  FF                                                   TERMINATOR_FF             
0x000004AF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004B3      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000004B5      1  FF                                                   TERMINATOR_FF             
0x000004B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004B8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004BA      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000004BC      1  FF                                                   TERMINATOR_FF             
0x000004BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004BF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000004C1      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000004C3      1  FF                                                   TERMINATOR_FF             
0x000004C4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004C6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004C8      1  FF                                                   TERMINATOR_FF             
0x000004C9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004CD      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000004CF      1  FF                                                   TERMINATOR_FF             
0x000004D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004D2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000004D4      3  F30307                                               IMM16_F3                  u16_be=775, u16_le=1795
0x000004D7      1  FF                                                   TERMINATOR_FF             
0x000004D8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004DA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000004DC      9  80074D543035303230                                   LEN8_STRING_CP932         length=7, text="MT05020"
0x000004E5      1  FF                                                   TERMINATOR_FF             
0x000004E6      1  FF                                                   TERMINATOR_FF             
0x000004E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004E9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004EB     28  801A817582D382AD82BD82F182CC8343838F836D82CD815B...  LEN8_STRING_CP932         length=26, text="「ふくたんのイワノはー？」"
0x00000507      1  FF                                                   TERMINATOR_FF             
0x00000508      1  FF                                                   TERMINATOR_FF             
0x00000509      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000050B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000050D      1  FF                                                   TERMINATOR_FF             
0x0000050E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000510      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000512      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000514      1  FF                                                   TERMINATOR_FF             
0x00000515      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000517      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000519      3  F30308                                               IMM16_F3                  u16_be=776, u16_le=2051
0x0000051C      1  FF                                                   TERMINATOR_FF             
0x0000051D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000051F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000521     30  801C81758343838F836D839483428362836082E082A282EA...  LEN8_STRING_CP932         length=28, text="「イワノヴィッチもいれない」"
0x0000053F      1  FF                                                   TERMINATOR_FF             
0x00000540      1  FF                                                   TERMINATOR_FF             
0x00000541      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000543      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000545      1  FF                                                   TERMINATOR_FF             
0x00000546      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000548      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000054A      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000054C      1  FF                                                   TERMINATOR_FF             
0x0000054D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000054F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000551      3  F301B3                                               IMM16_F3                  u16_be=435, u16_le=45825
0x00000554      1  FF                                                   TERMINATOR_FF             
0x00000555      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000557      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000559      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000055B      1  FF                                                   TERMINATOR_FF             
0x0000055C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000055E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000560      1  FF                                                   TERMINATOR_FF             
0x00000561      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000563      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000565      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000567      1  FF                                                   TERMINATOR_FF             
0x00000568      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000056A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000056C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000056E      1  FF                                                   TERMINATOR_FF             
0x0000056F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000571      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000573      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000575      1  FF                                                   TERMINATOR_FF             
0x00000576      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000578      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000057A      1  FF                                                   TERMINATOR_FF             
0x0000057B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000057D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000057F      8  800653452D533935                                     LEN8_STRING_CP932         length=6, text="SE-S95"
0x00000587      1  FF                                                   TERMINATOR_FF             
0x00000588      1  FF                                                   TERMINATOR_FF             
0x00000589      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000058B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000058D      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x0000058F      1  FF                                                   TERMINATOR_FF             
0x00000590      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000592      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000594      1  FF                                                   TERMINATOR_FF             
0x00000595      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000597      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000599      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000059B      1  FF                                                   TERMINATOR_FF             
0x0000059C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000059E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000005A0      3  F30309                                               IMM16_F3                  u16_be=777, u16_le=2307
0x000005A3      1  FF                                                   TERMINATOR_FF             
0x000005A4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005A6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000005A8      9  80074D543035303330                                   LEN8_STRING_CP932         length=7, text="MT05030"
0x000005B1      1  FF                                                   TERMINATOR_FF             
0x000005B2      1  FF                                                   TERMINATOR_FF             
0x000005B3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005B5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005B7     60  803A817582D3815B82F18163816382A682C182C6815B82BD...  LEN8_STRING_CP932         length=58, text="「ふーん……えっとーたんじょーびがいっしょ\nなのはー……」"
0x000005F3      1  FF                                                   TERMINATOR_FF             
0x000005F4      1  FF                                                   TERMINATOR_FF             
0x000005F5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005F7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005F9      1  FF                                                   TERMINATOR_FF             
0x000005FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005FC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005FE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000600      1  FF                                                   TERMINATOR_FF             
0x00000601      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000603      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000605      3  F3030A                                               IMM16_F3                  u16_be=778, u16_le=2563
0x00000608      1  FF                                                   TERMINATOR_FF             
0x00000609      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000060B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000060D     14  800C817581638163816381638176                         LEN8_STRING_CP932         length=12, text="「…………」"
0x0000061B      1  FF                                                   TERMINATOR_FF             
0x0000061C      1  FF                                                   TERMINATOR_FF             
0x0000061D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000061F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000621      1  FF                                                   TERMINATOR_FF             
0x00000622      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000624      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000626      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000628      1  FF                                                   TERMINATOR_FF             
0x00000629      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000062B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000062D      3  F301B2                                               IMM16_F3                  u16_be=434, u16_le=45569
0x00000630      1  FF                                                   TERMINATOR_FF             
0x00000631      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000633      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000635      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000637      1  FF                                                   TERMINATOR_FF             
0x00000638      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000063A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000063C      1  FF                                                   TERMINATOR_FF             
0x0000063D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000063F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000641      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000643      1  FF                                                   TERMINATOR_FF             
0x00000644      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000646      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000648      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000064A      1  FF                                                   TERMINATOR_FF             
0x0000064B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000064D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000064F      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000651      1  FF                                                   TERMINATOR_FF             
0x00000652      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000654      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000656      1  FF                                                   TERMINATOR_FF             
0x00000657      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000659      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000065B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000065D      1  FF                                                   TERMINATOR_FF             
0x0000065E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000660      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000662      3  F3030B                                               IMM16_F3                  u16_be=779, u16_le=2819
0x00000665      1  FF                                                   TERMINATOR_FF             
0x00000666      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000668      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000066A      9  80074D543035303430                                   LEN8_STRING_CP932         length=7, text="MT05040"
0x00000673      1  FF                                                   TERMINATOR_FF             
0x00000674      1  FF                                                   TERMINATOR_FF             
0x00000675      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000677      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000679     38  802481758360838082C182C482CB815B814182B582B582B4...  LEN8_STRING_CP932         length=36, text="「チムってねー、ししざなんだってー」"
0x0000069F      1  FF                                                   TERMINATOR_FF             
0x000006A0      1  FF                                                   TERMINATOR_FF             
0x000006A1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006A3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006A5      1  FF                                                   TERMINATOR_FF             
0x000006A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006A8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006AA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000006AC      1  FF                                                   TERMINATOR_FF             
0x000006AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006AF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006B1      3  F3030C                                               IMM16_F3                  u16_be=780, u16_le=3075
0x000006B4      1  FF                                                   TERMINATOR_FF             
0x000006B5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006B7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006B9     10  8008817582D682A68176                                 LEN8_STRING_CP932         length=8, text="「へえ」"
0x000006C3      1  FF                                                   TERMINATOR_FF             
0x000006C4      1  FF                                                   TERMINATOR_FF             
0x000006C5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006C7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006C9      1  FF                                                   TERMINATOR_FF             
0x000006CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006CE      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000006D0      1  FF                                                   TERMINATOR_FF             
0x000006D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006D3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006D5      3  F3030D                                               IMM16_F3                  u16_be=781, u16_le=3331
0x000006D8      1  FF                                                   TERMINATOR_FF             
0x000006D9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006DB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000006DD      9  80074D543035303530                                   LEN8_STRING_CP932         length=7, text="MT05050"
0x000006E6      1  FF                                                   TERMINATOR_FF             
0x000006E7      1  FF                                                   TERMINATOR_FF             
0x000006E8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006EC     56  8036817582CD82E982B382F1814182D382BD82B282B4815B...  LEN8_STRING_CP932         length=54, text="「はるさん、ふたござー。ホントはふたりいる\nかもねー」"
0x00000724      1  FF                                                   TERMINATOR_FF             
0x00000725      1  FF                                                   TERMINATOR_FF             
0x00000726      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000728      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000072A      1  FF                                                   TERMINATOR_FF             
0x0000072B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000072D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000072F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000731      1  FF                                                   TERMINATOR_FF             
0x00000732      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000734      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000736      3  F3030E                                               IMM16_F3                  u16_be=782, u16_le=3587
0x00000739      1  FF                                                   TERMINATOR_FF             
0x0000073A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000073C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000073E     20  8012817582A282C882A282C68E7682A482E68176             LEN8_STRING_CP932         length=18, text="「いないと思うよ」"
0x00000752      1  FF                                                   TERMINATOR_FF             
0x00000753      1  FF                                                   TERMINATOR_FF             
0x00000754      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000756      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000758      1  FF                                                   TERMINATOR_FF             
0x00000759      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000075B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000075D      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000075F      1  FF                                                   TERMINATOR_FF             
0x00000760      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000762      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000764      3  F301B3                                               IMM16_F3                  u16_be=435, u16_le=45825
0x00000767      1  FF                                                   TERMINATOR_FF             
0x00000768      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000076A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000076C      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000076E      1  FF                                                   TERMINATOR_FF             
0x0000076F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000771      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000773      1  FF                                                   TERMINATOR_FF             
0x00000774      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000776      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000778      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000077A      1  FF                                                   TERMINATOR_FF             
0x0000077B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000077D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000077F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000781      1  FF                                                   TERMINATOR_FF             
0x00000782      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000784      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000786      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000788      1  FF                                                   TERMINATOR_FF             
0x00000789      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000078B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000078D      1  FF                                                   TERMINATOR_FF             
0x0000078E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000790      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000792      8  800653452D533935                                     LEN8_STRING_CP932         length=6, text="SE-S95"
0x0000079A      1  FF                                                   TERMINATOR_FF             
0x0000079B      1  FF                                                   TERMINATOR_FF             
0x0000079C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000079E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007A0      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x000007A2      1  FF                                                   TERMINATOR_FF             
0x000007A3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007A5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007A7      1  FF                                                   TERMINATOR_FF             
0x000007A8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007AC      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000007AE      1  FF                                                   TERMINATOR_FF             
0x000007AF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007B1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000007B3      3  F3030F                                               IMM16_F3                  u16_be=783, u16_le=3843
0x000007B6      1  FF                                                   TERMINATOR_FF             
0x000007B7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007B9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000007BB      9  80074D543035303630                                   LEN8_STRING_CP932         length=7, text="MT05060"
0x000007C4      1  FF                                                   TERMINATOR_FF             
0x000007C5      1  FF                                                   TERMINATOR_FF             
0x000007C6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007C8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007CA     33  801F817582D3815B82F18163816333363582AA815B333882...  LEN8_STRING_CP932         length=31, text="「ふーん……365がー38でー……」"
0x000007EB      1  FF                                                   TERMINATOR_FF             
0x000007EC      1  FF                                                   TERMINATOR_FF             
0x000007ED      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007EF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007F1      1  FF                                                   TERMINATOR_FF             
0x000007F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007F6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000007F8      1  FF                                                   TERMINATOR_FF             
0x000007F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007FB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000007FD      3  F30310                                               IMM16_F3                  u16_be=784, u16_le=4099
0x00000800      1  FF                                                   TERMINATOR_FF             
0x00000801      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000803      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000805     72  80468A6D97A682CC8C768E5A82D082C682C282C982B582C4...  LEN8_STRING_CP932         length=70, text="確率の計算ひとつにしても、\nあーでもないこーでもないと寄り道したがる。"
0x0000084D      1  FF                                                   TERMINATOR_FF             
0x0000084E      1  FF                                                   TERMINATOR_FF             
0x0000084F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000851      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000853      1  FF                                                   TERMINATOR_FF             
0x00000854      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000856      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000858      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000085A      1  FF                                                   TERMINATOR_FF             
0x0000085B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000085D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000085F      3  F30311                                               IMM16_F3                  u16_be=785, u16_le=4355
0x00000862      1  FF                                                   TERMINATOR_FF             
0x00000863      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000865      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000867     54  803482C7815B82E282E793AF82B682B182C682F091B182AF...  LEN8_STRING_CP932         length=52, text="どーやら同じことを続けるのが、ものすごく苦手らしい。"
0x0000089D      1  FF                                                   TERMINATOR_FF             
0x0000089E      1  FF                                                   TERMINATOR_FF             
0x0000089F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008A1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008A3      1  FF                                                   TERMINATOR_FF             
0x000008A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008A8      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000008AA      1  FF                                                   TERMINATOR_FF             
0x000008AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008AD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000008AF      3  F30312                                               IMM16_F3                  u16_be=786, u16_le=4611
0x000008B2      1  FF                                                   TERMINATOR_FF             
0x000008B3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008B5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008B7     70  804482BD82D482F1814182BB82A482C882F182BE82EB815B...  LEN8_STRING_CP932         length=68, text="たぶん、そうなんだろーなとは思っていたけど、まさかここまでとは……。"
0x000008FD      1  FF                                                   TERMINATOR_FF             
0x000008FE      1  FF                                                   TERMINATOR_FF             
0x000008FF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000901      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000903      1  FF                                                   TERMINATOR_FF             
0x00000904      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000906      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000908      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000090A      1  FF                                                   TERMINATOR_FF             
0x0000090B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000090D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000090F      3  F30313                                               IMM16_F3                  u16_be=787, u16_le=4867
0x00000912      1  FF                                                   TERMINATOR_FF             
0x00000913      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000915      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000917     36  8022817582A8914F814182E682AD8EF38CB18377815B834C...  LEN8_STRING_CP932         length=34, text="「お前、よく受験ヘーキだったよな」"
0x0000093B      1  FF                                                   TERMINATOR_FF             
0x0000093C      1  FF                                                   TERMINATOR_FF             
0x0000093D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000093F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000941      1  FF                                                   TERMINATOR_FF             
0x00000942      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000944      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000946      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000948      1  FF                                                   TERMINATOR_FF             
0x00000949      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000094B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000094D      3  F301B0                                               IMM16_F3                  u16_be=432, u16_le=45057
0x00000950      1  FF                                                   TERMINATOR_FF             
0x00000951      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000953      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000955      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000957      1  FF                                                   TERMINATOR_FF             
0x00000958      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000095A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000095C      1  FF                                                   TERMINATOR_FF             
0x0000095D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000095F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000961      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000963      1  FF                                                   TERMINATOR_FF             
0x00000964      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000966      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000968      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000096A      1  FF                                                   TERMINATOR_FF             
0x0000096B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000096D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000096F      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000971      1  FF                                                   TERMINATOR_FF             
0x00000972      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000974      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000976      1  FF                                                   TERMINATOR_FF             
0x00000977      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000979      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000097B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000097D      1  FF                                                   TERMINATOR_FF             
0x0000097E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000980      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000982      3  F30314                                               IMM16_F3                  u16_be=788, u16_le=5123
0x00000985      1  FF                                                   TERMINATOR_FF             
0x00000986      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000988      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000098A      9  80074D543035303730                                   LEN8_STRING_CP932         length=7, text="MT05070"
0x00000993      1  FF                                                   TERMINATOR_FF             
0x00000994      1  FF                                                   TERMINATOR_FF             
0x00000995      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000997      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000999     48  802E817582BE815B82A9815B82E7815B8141837D834382BE...  LEN8_STRING_CP932         length=46, text="「だーかーらー、マイだってやればでーきーるー」"
0x000009C9      1  FF                                                   TERMINATOR_FF             
0x000009CA      1  FF                                                   TERMINATOR_FF             
0x000009CB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009CD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009CF      1  FF                                                   TERMINATOR_FF             
0x000009D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009D4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000009D6      1  FF                                                   TERMINATOR_FF             
0x000009D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009D9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009DB      3  F30315                                               IMM16_F3                  u16_be=789, u16_le=5379
0x000009DE      1  FF                                                   TERMINATOR_FF             
0x000009DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009E1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009E3     36  8022817592868A77839382C682AB81418EF38CB195D78BAD...  LEN8_STRING_CP932         length=34, text="「中学ンとき、受験勉強したっけ？」"
0x00000A07      1  FF                                                   TERMINATOR_FF             
0x00000A08      1  FF                                                   TERMINATOR_FF             
0x00000A09      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A0B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A0D      1  FF                                                   TERMINATOR_FF             
0x00000A0E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A10      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A12      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000A14      1  FF                                                   TERMINATOR_FF             
0x00000A15      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A17      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A19      3  F301B2                                               IMM16_F3                  u16_be=434, u16_le=45569
0x00000A1C      1  FF                                                   TERMINATOR_FF             
0x00000A1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A1F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A21      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000A23      1  FF                                                   TERMINATOR_FF             
0x00000A24      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A26      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A28      1  FF                                                   TERMINATOR_FF             
0x00000A29      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A2B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A2D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000A2F      1  FF                                                   TERMINATOR_FF             
0x00000A30      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A32      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A34      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000A36      1  FF                                                   TERMINATOR_FF             
0x00000A37      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A39      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A3B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000A3D      1  FF                                                   TERMINATOR_FF             
0x00000A3E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A40      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A42      1  FF                                                   TERMINATOR_FF             
0x00000A43      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A45      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A47      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000A49      1  FF                                                   TERMINATOR_FF             
0x00000A4A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A4C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A4E      3  F30316                                               IMM16_F3                  u16_be=790, u16_le=5635
0x00000A51      1  FF                                                   TERMINATOR_FF             
0x00000A52      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A54      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A56      9  80074D543035303830                                   LEN8_STRING_CP932         length=7, text="MT05080"
0x00000A5F      1  FF                                                   TERMINATOR_FF             
0x00000A60      1  FF                                                   TERMINATOR_FF             
0x00000A61      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A63      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A65     18  8010817582BA815B82F182BA815B82F18176                 LEN8_STRING_CP932         length=16, text="「ぜーんぜーん」"
0x00000A77      1  FF                                                   TERMINATOR_FF             
0x00000A78      1  FF                                                   TERMINATOR_FF             
0x00000A79      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A7B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A7D      1  FF                                                   TERMINATOR_FF             
0x00000A7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A82      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000A84      1  FF                                                   TERMINATOR_FF             
0x00000A85      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A87      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A89      3  F30317                                               IMM16_F3                  u16_be=791, u16_le=5891
0x00000A8C      1  FF                                                   TERMINATOR_FF             
0x00000A8D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A8F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A91     42  8028817582BB815B82BE82E682C8815B814282BB82F182C8...  LEN8_STRING_CP932         length=40, text="「そーだよなー。そんな記憶ないもんなー」"
0x00000ABB      1  FF                                                   TERMINATOR_FF             
0x00000ABC      1  FF                                                   TERMINATOR_FF             
0x00000ABD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000ABF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AC1      1  FF                                                   TERMINATOR_FF             
0x00000AC2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AC4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AC6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000AC8      1  FF                                                   TERMINATOR_FF             
0x00000AC9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000ACB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000ACD      3  F33D4E                                               IMM16_F3                  u16_be=15694, u16_le=20029
0x00000AD0      1  FF                                                   TERMINATOR_FF             
0x00000AD1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AD3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AD5     58  80388BCA82CC836A834A836282C682B582BD8FCE8AE782F0...  LEN8_STRING_CP932         length=56, text="玉のニカッとした笑顔を見ながら、本当かよ……\nとか思う。"
0x00000B0F      1  FF                                                   TERMINATOR_FF             
0x00000B10      1  FF                                                   TERMINATOR_FF             
0x00000B11      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B13      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B15      1  FF                                                   TERMINATOR_FF             
0x00000B16      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B18      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B1A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B1C      1  FF                                                   TERMINATOR_FF             
0x00000B1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B1F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B21      3  F33D4F                                               IMM16_F3                  u16_be=15695, u16_le=20285
0x00000B24      1  FF                                                   TERMINATOR_FF             
0x00000B25      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B27      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B29    100  806282DC82A0814182B182A282C282CC82E4815B82B182C6...  LEN8_STRING_CP932         length=98, text="まあ、こいつのゆーこと信用しちゃうと\n痛いめにあうんだから、こっちはこっちで\n気をひきしめなきゃ。"
0x00000B8D      1  FF                                                   TERMINATOR_FF             
0x00000B8E      1  FF                                                   TERMINATOR_FF             
0x00000B8F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B91      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B93      1  FF                                                   TERMINATOR_FF             
0x00000B94      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B96      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B98      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B9A      1  FF                                                   TERMINATOR_FF             
0x00000B9B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B9D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B9F      3  F33D50                                               IMM16_F3                  u16_be=15696, u16_le=20541
0x00000BA2      1  FF                                                   TERMINATOR_FF             
0x00000BA3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BA5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BA7     88  805682A482F1814182A082DC82E282A982B582CD82B582C8...  LEN8_STRING_CP932         length=86, text="うん、あまやかしはしないぞ。\nビシビシ勉強させて、世の中を甘く見る癖を\nなおしてやる。"
0x00000BFF      1  FF                                                   TERMINATOR_FF             
0x00000C00      1  FF                                                   TERMINATOR_FF             
0x00000C01      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C03      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C05      1  FF                                                   TERMINATOR_FF             
0x00000C06      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C08      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C0A      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000C0C      1  FF                                                   TERMINATOR_FF             
0x00000C0D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C0F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C11      3  F30089                                               IMM16_F3                  u16_be=137, u16_le=35072
0x00000C14      1  FF                                                   TERMINATOR_FF             
0x00000C15      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C17      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C19      1  FF                                                   TERMINATOR_FF             
0x00000C1A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C1C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C1E      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000C20      1  FF                                                   TERMINATOR_FF             
0x00000C21      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C23      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C25      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C27      1  FF                                                   TERMINATOR_FF             
0x00000C28      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C2A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000C2C      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000C2E      1  FF                                                   TERMINATOR_FF             
0x00000C2F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C31      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C33      1  FF                                                   TERMINATOR_FF             
0x00000C34      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C38      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000C3A      1  FF                                                   TERMINATOR_FF             
0x00000C3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C3D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C3F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000C41      1  FF                                                   TERMINATOR_FF             
0x00000C42      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C44      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000C46      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000C48      1  FF                                                   TERMINATOR_FF             
0x00000C49      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C4B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C4D      1  FF                                                   TERMINATOR_FF             
0x00000C4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C50      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C52      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000C54      1  FF                                                   TERMINATOR_FF             
0x00000C55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C57      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C59      3  F33D51                                               IMM16_F3                  u16_be=15697, u16_le=20797
0x00000C5C      1  FF                                                   TERMINATOR_FF             
0x00000C5D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C5F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C61     22  801482E6815B82B5814182AA82F182CE82E982BC2121         LEN8_STRING_CP932         length=20, text="よーし、がんばるぞ!!"
0x00000C77      1  FF                                                   TERMINATOR_FF             
0x00000C78      1  FF                                                   TERMINATOR_FF             
0x00000C79      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C7B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C7D      1  FF                                                   TERMINATOR_FF             
0x00000C7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C82      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00000C84      1  FF                                                   TERMINATOR_FF             
0x00000C85      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C87      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C89      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C8B      1  FF                                                   TERMINATOR_FF             
0x00000C8C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C8E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C90      1  FF                                                   TERMINATOR_FF             
0x00000C91      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C93      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C95      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000C97      1  FF                                                   TERMINATOR_FF             
0x00000C98      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C9A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C9C      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00000C9E      1  FF                                                   TERMINATOR_FF             
0x00000C9F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CA1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CA3      1  FF                                                   TERMINATOR_FF             
0x00000CA4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CA6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CA8      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000CAA      1  FF                                                   TERMINATOR_FF             
0x00000CAB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CAD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CAF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CB1      1  FF                                                   TERMINATOR_FF             
0x00000CB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CB4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000CB6      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000CB8      1  FF                                                   TERMINATOR_FF             
0x00000CB9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CBB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CBD      1  FF                                                   TERMINATOR_FF             
0x00000CBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CC0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CC2      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000CC4      1  FF                                                   TERMINATOR_FF             
0x00000CC5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CC7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CC9      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000CCB      1  FF                                                   TERMINATOR_FF             
0x00000CCC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CCE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000CD0      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000CD2      1  FF                                                   TERMINATOR_FF             
0x00000CD3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CD5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CD7      1  FF                                                   TERMINATOR_FF             
0x00000CD8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CDA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CDC      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00000CDE      1  FF                                                   TERMINATOR_FF             
0x00000CDF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CE1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CE3      1  FF                                                   TERMINATOR_FF             
0x00000CE4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CE6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CE8      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000CEA      1  FF                                                   TERMINATOR_FF             
0x00000CEB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CED      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000CEF     14  800C50533241303133612E62696E                         LEN8_STRING_CP932         length=12, text="PS2A013a.bin"
0x00000CFD      1  FF                                                   TERMINATOR_FF             
0x00000CFE      1  FF                                                   TERMINATOR_FF             
0x00000CFF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D01      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D03      1  FF                                                   TERMINATOR_FF             
0x00000D04      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000D06      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000D08      1  FF                                                   TERMINATOR_FF             
0x00000D09      2  000D                                                 WORD_00XX                 u16_be=13, low_byte=13
0x00000D0B      1  13                                                   OPAQUE_RAW_BYTES          bytes=13
0x00000D0C      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x00000D0E      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000D10      2  000D                                                 WORD_00XX                 u16_be=13, low_byte=13
0x00000D12      1  04                                                   OPAQUE_RAW_BYTES          bytes=04
0x00000D13      1  FF                                                   TERMINATOR_FF             
0x00000D14      1  FF                                                   TERMINATOR_FF             
