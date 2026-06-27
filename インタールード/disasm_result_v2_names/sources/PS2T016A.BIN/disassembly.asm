; Full conservative disassembly for PS2T016A.BIN
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
0x00000066      3  F3322A                                               IMM16_F3                  u16_be=12842, u16_le=10802
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
0x000000F0      3  F3322B                                               IMM16_F3                  u16_be=12843, u16_le=11058
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
0x0000015A      3  F3322C                                               IMM16_F3                  u16_be=12844, u16_le=11314
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
0x000001AA      3  F3322D                                               IMM16_F3                  u16_be=12845, u16_le=11570
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
0x00000222      3  F3322E                                               IMM16_F3                  u16_be=12846, u16_le=11826
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
0x000002B6      3  F3322F                                               IMM16_F3                  u16_be=12847, u16_le=12082
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
0x000002EE      3  F33230                                               IMM16_F3                  u16_be=12848, u16_le=12338
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
0x00000327      3  F33231                                               IMM16_F3                  u16_be=12849, u16_le=12594
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
0x00000384      3  F33232                                               IMM16_F3                  u16_be=12850, u16_le=12850
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
0x000003DB      3  F33233                                               IMM16_F3                  u16_be=12851, u16_le=13106
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
0x0000043E      3  F33234                                               IMM16_F3                  u16_be=12852, u16_le=13362
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
0x00000477      3  F33235                                               IMM16_F3                  u16_be=12853, u16_le=13618
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
0x000004D4      3  F33236                                               IMM16_F3                  u16_be=12854, u16_le=13874
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
0x00000519      3  F33237                                               IMM16_F3                  u16_be=12855, u16_le=14130
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
0x000005A0      3  F33238                                               IMM16_F3                  u16_be=12856, u16_le=14386
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
0x00000605      3  F33239                                               IMM16_F3                  u16_be=12857, u16_le=14642
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
0x00000662      3  F3323A                                               IMM16_F3                  u16_be=12858, u16_le=14898
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
0x000006B1      3  F3323B                                               IMM16_F3                  u16_be=12859, u16_le=15154
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
0x000006D5      3  F3323C                                               IMM16_F3                  u16_be=12860, u16_le=15410
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
0x00000736      3  F3323D                                               IMM16_F3                  u16_be=12861, u16_le=15666
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
0x000007B3      3  F3323E                                               IMM16_F3                  u16_be=12862, u16_le=15922
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
0x000007FD      3  F3323F                                               IMM16_F3                  u16_be=12863, u16_le=16178
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
0x0000085F      3  F33240                                               IMM16_F3                  u16_be=12864, u16_le=16434
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
0x000008AF      3  F33241                                               IMM16_F3                  u16_be=12865, u16_le=16690
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
0x0000090F      3  F33242                                               IMM16_F3                  u16_be=12866, u16_le=16946
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
0x00000982      3  F33243                                               IMM16_F3                  u16_be=12867, u16_le=17202
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
0x000009DB      3  F33244                                               IMM16_F3                  u16_be=12868, u16_le=17458
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
0x00000A4E      3  F33245                                               IMM16_F3                  u16_be=12869, u16_le=17714
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
0x00000A89      3  F33246                                               IMM16_F3                  u16_be=12870, u16_le=17970
0x00000A8C      1  FF                                                   TERMINATOR_FF             
0x00000A8D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A8F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A91     42  8028817582BB815B82BE82E682C8815B814282BB82F182C8...  LEN8_STRING_CP932         length=40, text="「そーだよなー。そんな記憶ないもんなー」"
0x00000ABB      1  FF                                                   TERMINATOR_FF             
0x00000ABC      1  FF                                                   TERMINATOR_FF             
0x00000ABD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000ABF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AC1      1  FF                                                   TERMINATOR_FF             
0x00000AC2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AC4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000AC6      8  800673652D653031                                     LEN8_STRING_CP932         length=6, text="se-e01"
0x00000ACE      1  FF                                                   TERMINATOR_FF             
0x00000ACF      1  FF                                                   TERMINATOR_FF             
0x00000AD0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AD2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AD4      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00000AD6      1  FF                                                   TERMINATOR_FF             
0x00000AD7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000AD9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000ADB      1  FF                                                   TERMINATOR_FF             
0x00000ADC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000ADE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AE0      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000AE2      1  FF                                                   TERMINATOR_FF             
0x00000AE3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AE5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AE7      3  F301B4                                               IMM16_F3                  u16_be=436, u16_le=46081
0x00000AEA      1  FF                                                   TERMINATOR_FF             
0x00000AEB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AED      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000AEF      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000AF1      1  FF                                                   TERMINATOR_FF             
0x00000AF2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000AF4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AF6      1  FF                                                   TERMINATOR_FF             
0x00000AF7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AF9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AFB      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000AFD      1  FF                                                   TERMINATOR_FF             
0x00000AFE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B00      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B02      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000B04      1  FF                                                   TERMINATOR_FF             
0x00000B05      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B07      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000B09      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000B0B      1  FF                                                   TERMINATOR_FF             
0x00000B0C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B0E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B10      1  FF                                                   TERMINATOR_FF             
0x00000B11      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B13      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B15      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000B17      1  FF                                                   TERMINATOR_FF             
0x00000B18      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B1A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B1C      3  F33247                                               IMM16_F3                  u16_be=12871, u16_le=18226
0x00000B1F      1  FF                                                   TERMINATOR_FF             
0x00000B20      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B22      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000B24      9  80074D543035303930                                   LEN8_STRING_CP932         length=7, text="MT05090"
0x00000B2D      1  FF                                                   TERMINATOR_FF             
0x00000B2E      1  FF                                                   TERMINATOR_FF             
0x00000B2F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B31      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B33     28  801A817583568383815B8379839382A8815B82BF815B82BD...  LEN8_STRING_CP932         length=26, text="「シャーペンおーちーたー」"
0x00000B4F      1  FF                                                   TERMINATOR_FF             
0x00000B50      1  FF                                                   TERMINATOR_FF             
0x00000B51      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B53      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B55      1  FF                                                   TERMINATOR_FF             
0x00000B56      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B58      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B5A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B5C      1  FF                                                   TERMINATOR_FF             
0x00000B5D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B5F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B61      3  F33248                                               IMM16_F3                  u16_be=12872, u16_le=18482
0x00000B64      1  FF                                                   TERMINATOR_FF             
0x00000B65      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B67      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B69     14  800C817582D082EB82A682E68176                         LEN8_STRING_CP932         length=12, text="「ひろえよ」"
0x00000B77      1  FF                                                   TERMINATOR_FF             
0x00000B78      1  FF                                                   TERMINATOR_FF             
0x00000B79      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B7B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B7D      1  FF                                                   TERMINATOR_FF             
0x00000B7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B82      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000B84      1  FF                                                   TERMINATOR_FF             
0x00000B85      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B87      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B89      3  F33249                                               IMM16_F3                  u16_be=12873, u16_le=18738
0x00000B8C      1  FF                                                   TERMINATOR_FF             
0x00000B8D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B8F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000B91      9  80074D543035313030                                   LEN8_STRING_CP932         length=7, text="MT05100"
0x00000B9A      1  FF                                                   TERMINATOR_FF             
0x00000B9B      1  FF                                                   TERMINATOR_FF             
0x00000B9C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B9E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BA0     40  8026817582D0815B82EB815B82A6815B82C8815B82A2815B...  LEN8_STRING_CP932         length=38, text="「ひーろーえーなーいー。おーちーたー」"
0x00000BC8      1  FF                                                   TERMINATOR_FF             
0x00000BC9      1  FF                                                   TERMINATOR_FF             
0x00000BCA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000BCC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000BCE      1  FF                                                   TERMINATOR_FF             
0x00000BCF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BD1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BD3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000BD5      1  FF                                                   TERMINATOR_FF             
0x00000BD6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BD8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000BDA      3  F3324A                                               IMM16_F3                  u16_be=12874, u16_le=18994
0x00000BDD      1  FF                                                   TERMINATOR_FF             
0x00000BDE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BE0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BE2     32  801E817582BB82EA82AD82E782A28EA995AA82C582D082EB...  LEN8_STRING_CP932         length=30, text="「それくらい自分でひろえって」"
0x00000C02      1  FF                                                   TERMINATOR_FF             
0x00000C03      1  FF                                                   TERMINATOR_FF             
0x00000C04      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C06      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C08      1  FF                                                   TERMINATOR_FF             
0x00000C09      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C0B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C0D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000C0F      1  FF                                                   TERMINATOR_FF             
0x00000C10      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C12      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C14      3  F3324B                                               IMM16_F3                  u16_be=12875, u16_le=19250
0x00000C17      1  FF                                                   TERMINATOR_FF             
0x00000C18      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C1A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C1C     36  8022817582A68160814182D082EB82C182C482AD82F182C8...  LEN8_STRING_CP932         length=34, text="「え～、ひろってくんなきゃやだー」"
0x00000C40      1  FF                                                   TERMINATOR_FF             
0x00000C41      1  FF                                                   TERMINATOR_FF             
0x00000C42      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C44      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000C46      9  80074D543035313130                                   LEN8_STRING_CP932         length=7, text="MT05110"
0x00000C4F      1  FF                                                   TERMINATOR_FF             
0x00000C50      1  FF                                                   TERMINATOR_FF             
0x00000C51      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C53      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00000C55     12  800A8F4582C182C482E282E9                             LEN8_STRING_CP932         length=10, text="拾ってやる"
0x00000C61      1  FF                                                   TERMINATOR_FF             
0x00000C62      1  FF                                                   TERMINATOR_FF             
0x00000C63      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C65      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x00000C67      3  F3324C                                               IMM16_F3                  u16_be=12876, u16_le=19506
0x00000C6A      1  FF                                                   TERMINATOR_FF             
0x00000C6B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C6D      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00000C6F     12  800A8EA995AA82C58F4582A6                             LEN8_STRING_CP932         length=10, text="自分で拾え"
0x00000C7B      1  FF                                                   TERMINATOR_FF             
0x00000C7C      1  FF                                                   TERMINATOR_FF             
0x00000C7D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C7F      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00000C81      3  F3324D                                               IMM16_F3                  u16_be=12877, u16_le=19762
0x00000C84      1  FF                                                   TERMINATOR_FF             
0x00000C85      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C87      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x00000C89      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000C8B      1  FF                                                   TERMINATOR_FF             
0x00000C8C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C8E      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00000C90      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C92      1  FF                                                   TERMINATOR_FF             
0x00000C93      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C95      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C97      2  F213                                                 IMM8_F2                   u8=19, s8=19
0x00000C99      1  FF                                                   TERMINATOR_FF             
0x00000C9A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C9C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C9E      1  FF                                                   TERMINATOR_FF             
0x00000C9F      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000CA1      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00000CA4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CA6      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000CA7      1  FF                                                   TERMINATOR_FF             
0x00000CA8      2  000D                                                 WORD_00XX                 u16_be=13, low_byte=13
0x00000CAA      1  F4                                                   OPAQUE_RAW_BYTES          bytes=F4
0x00000CAB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CAD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CAF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000CB1      1  FF                                                   TERMINATOR_FF             
0x00000CB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CB4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000CB6      3  F3324E                                               IMM16_F3                  u16_be=12878, u16_le=20018
0x00000CB9      1  FF                                                   TERMINATOR_FF             
0x00000CBA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CBC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CBE     34  8020817582B582E5815B82AA82CB815B82C8815B814182DC...  LEN8_STRING_CP932         length=32, text="「しょーがねーなー、まったくー」"
0x00000CE0      1  FF                                                   TERMINATOR_FF             
0x00000CE1      1  FF                                                   TERMINATOR_FF             
0x00000CE2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CE4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CE6      1  FF                                                   TERMINATOR_FF             
0x00000CE7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CE9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CEB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000CED      1  FF                                                   TERMINATOR_FF             
0x00000CEE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CF0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000CF2      3  F3324F                                               IMM16_F3                  u16_be=12879, u16_le=20274
0x00000CF5      1  FF                                                   TERMINATOR_FF             
0x00000CF6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CF8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CFA     68  80428FB082C9935D82AA82E9834A837E834F837D83568383...  LEN8_STRING_CP932         length=66, text="床に転がるカミグマシャーペン（お気に入り）をひろって、玉に手渡す。"
0x00000D3E      1  FF                                                   TERMINATOR_FF             
0x00000D3F      1  FF                                                   TERMINATOR_FF             
0x00000D40      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D42      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D44      1  FF                                                   TERMINATOR_FF             
0x00000D45      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D47      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D49      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000D4B      1  FF                                                   TERMINATOR_FF             
0x00000D4C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D4E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D50      3  F33250                                               IMM16_F3                  u16_be=12880, u16_le=20530
0x00000D53      1  FF                                                   TERMINATOR_FF             
0x00000D54      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D56      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000D58      9  80074D543035313230                                   LEN8_STRING_CP932         length=7, text="MT05120"
0x00000D61      1  FF                                                   TERMINATOR_FF             
0x00000D62      1  FF                                                   TERMINATOR_FF             
0x00000D63      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D65      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D67     18  8010817582D082EB82ED82EA82BD815B8176                 LEN8_STRING_CP932         length=16, text="「ひろわれたー」"
0x00000D79      1  FF                                                   TERMINATOR_FF             
0x00000D7A      1  FF                                                   TERMINATOR_FF             
0x00000D7B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D7D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D7F      1  FF                                                   TERMINATOR_FF             
0x00000D80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D82      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D84      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000D86      1  FF                                                   TERMINATOR_FF             
0x00000D87      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D89      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D8B      3  F33251                                               IMM16_F3                  u16_be=12881, u16_le=20786
0x00000D8E      1  FF                                                   TERMINATOR_FF             
0x00000D8F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D91      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D93     30  801C817582A8914F82AA82D082EB82A682C182C482A282C1...  LEN8_STRING_CP932         length=28, text="「お前がひろえっていったの」"
0x00000DB1      1  FF                                                   TERMINATOR_FF             
0x00000DB2      1  FF                                                   TERMINATOR_FF             
0x00000DB3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000DB5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000DB7      1  FF                                                   TERMINATOR_FF             
0x00000DB8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DBA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DBC      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000DBE      1  FF                                                   TERMINATOR_FF             
0x00000DBF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DC1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000DC3      3  F33252                                               IMM16_F3                  u16_be=12882, u16_le=21042
0x00000DC6      1  FF                                                   TERMINATOR_FF             
0x00000DC7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DC9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000DCB      9  80074D543035313330                                   LEN8_STRING_CP932         length=7, text="MT05130"
0x00000DD4      1  FF                                                   TERMINATOR_FF             
0x00000DD5      1  FF                                                   TERMINATOR_FF             
0x00000DD6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DD8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DDA     14  800C817582D38160816082F18176                         LEN8_STRING_CP932         length=12, text="「ふ～～ん」"
0x00000DE8      1  FF                                                   TERMINATOR_FF             
0x00000DE9      1  FF                                                   TERMINATOR_FF             
0x00000DEA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000DEC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000DEE      1  FF                                                   TERMINATOR_FF             
0x00000DEF      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000DF1      2  000F                                                 WORD_00XX                 u16_be=15, low_byte=15
0x00000DF3      1  AD                                                   OPAQUE_RAW_BYTES          bytes=AD
0x00000DF4      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000DF6      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00000DF9      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000DFB      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000DFC      1  FF                                                   TERMINATOR_FF             
0x00000DFD      2  000F                                                 WORD_00XX                 u16_be=15, low_byte=15
0x00000DFF      1  AD                                                   OPAQUE_RAW_BYTES          bytes=AD
0x00000E00      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E02      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E04      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000E06      1  FF                                                   TERMINATOR_FF             
0x00000E07      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E09      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E0B      3  F33253                                               IMM16_F3                  u16_be=12883, u16_le=21298
0x00000E0E      1  FF                                                   TERMINATOR_FF             
0x00000E0F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E11      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000E13      9  80074D543935313430                                   LEN8_STRING_CP932         length=7, text="MT95140"
0x00000E1C      1  FF                                                   TERMINATOR_FF             
0x00000E1D      1  FF                                                   TERMINATOR_FF             
0x00000E1E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E20      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E22     12  800A817582DE82A3815B8176                             LEN8_STRING_CP932         length=10, text="「むぅー」"
0x00000E2E      1  FF                                                   TERMINATOR_FF             
0x00000E2F      1  FF                                                   TERMINATOR_FF             
0x00000E30      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E32      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E34      1  FF                                                   TERMINATOR_FF             
0x00000E35      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E37      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E39      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000E3B      1  FF                                                   TERMINATOR_FF             
0x00000E3C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E3E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E40      3  F33254                                               IMM16_F3                  u16_be=12884, u16_le=21554
0x00000E43      1  FF                                                   TERMINATOR_FF             
0x00000E44      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E46      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E48     60  803A8BCA82CD82BE82E782BE82E782C69573969E82A282C1...  LEN8_STRING_CP932         length=58, text="玉はだらだらと不満いっぱいで、シャーペンを\nひろいあげる。"
0x00000E84      1  FF                                                   TERMINATOR_FF             
0x00000E85      1  FF                                                   TERMINATOR_FF             
0x00000E86      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E88      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E8A      1  FF                                                   TERMINATOR_FF             
0x00000E8B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E8D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E8F      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000E91      1  FF                                                   TERMINATOR_FF             
0x00000E92      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E94      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E96      3  F301B0                                               IMM16_F3                  u16_be=432, u16_le=45057
0x00000E99      1  FF                                                   TERMINATOR_FF             
0x00000E9A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E9C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000E9E      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000EA0      1  FF                                                   TERMINATOR_FF             
0x00000EA1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000EA3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000EA5      1  FF                                                   TERMINATOR_FF             
0x00000EA6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EA8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EAA      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000EAC      1  FF                                                   TERMINATOR_FF             
0x00000EAD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EAF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000EB1      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000EB3      1  FF                                                   TERMINATOR_FF             
0x00000EB4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EB6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000EB8      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000EBA      1  FF                                                   TERMINATOR_FF             
0x00000EBB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000EBD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000EBF      1  FF                                                   TERMINATOR_FF             
0x00000EC0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EC2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EC4      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000EC6      1  FF                                                   TERMINATOR_FF             
0x00000EC7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EC9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000ECB      3  F33255                                               IMM16_F3                  u16_be=12885, u16_le=21810
0x00000ECE      1  FF                                                   TERMINATOR_FF             
0x00000ECF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000ED1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000ED3      9  80074D543035313530                                   LEN8_STRING_CP932         length=7, text="MT05150"
0x00000EDC      1  FF                                                   TERMINATOR_FF             
0x00000EDD      1  FF                                                   TERMINATOR_FF             
0x00000EDE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000EE0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000EE2     24  8016817582E2815B82B382B582AD82C8815B82A2815B8176     LEN8_STRING_CP932         length=22, text="「やーさしくなーいー」"
0x00000EFA      1  FF                                                   TERMINATOR_FF             
0x00000EFB      1  FF                                                   TERMINATOR_FF             
0x00000EFC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000EFE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F00      1  FF                                                   TERMINATOR_FF             
0x00000F01      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F03      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F05      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000F07      1  FF                                                   TERMINATOR_FF             
0x00000F08      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F0A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F0C      3  F33256                                               IMM16_F3                  u16_be=12886, u16_le=22066
0x00000F0F      1  FF                                                   TERMINATOR_FF             
0x00000F10      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F12      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F14     36  8022817582BB82EA82AD82E782A282CD8EA995AA82C582B7...  LEN8_STRING_CP932         length=34, text="「それくらいは自分でするもんなの」"
0x00000F38      1  FF                                                   TERMINATOR_FF             
0x00000F39      1  FF                                                   TERMINATOR_FF             
0x00000F3A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F3C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F3E      1  FF                                                   TERMINATOR_FF             
0x00000F3F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F41      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F43      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000F45      1  FF                                                   TERMINATOR_FF             
0x00000F46      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F48      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F4A      3  F301B4                                               IMM16_F3                  u16_be=436, u16_le=46081
0x00000F4D      1  FF                                                   TERMINATOR_FF             
0x00000F4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F50      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000F52      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000F54      1  FF                                                   TERMINATOR_FF             
0x00000F55      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F57      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F59      1  FF                                                   TERMINATOR_FF             
0x00000F5A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F5E      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000F60      1  FF                                                   TERMINATOR_FF             
0x00000F61      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F63      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F65      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000F67      1  FF                                                   TERMINATOR_FF             
0x00000F68      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F6A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000F6C      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000F6E      1  FF                                                   TERMINATOR_FF             
0x00000F6F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F71      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F73      1  FF                                                   TERMINATOR_FF             
0x00000F74      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F76      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F78      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000F7A      1  FF                                                   TERMINATOR_FF             
0x00000F7B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F7D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F7F      3  F33257                                               IMM16_F3                  u16_be=12887, u16_le=22322
0x00000F82      1  FF                                                   TERMINATOR_FF             
0x00000F83      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F85      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000F87      9  80074D543035313630                                   LEN8_STRING_CP932         length=7, text="MT05160"
0x00000F90      1  FF                                                   TERMINATOR_FF             
0x00000F91      1  FF                                                   TERMINATOR_FF             
0x00000F92      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F94      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F96     16  800E817582C282DC82E7816082F18176                     LEN8_STRING_CP932         length=14, text="「つまら～ん」"
0x00000FA6      1  FF                                                   TERMINATOR_FF             
0x00000FA7      1  FF                                                   TERMINATOR_FF             
0x00000FA8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FAA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FAC      1  FF                                                   TERMINATOR_FF             
0x00000FAD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FAF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FB1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000FB3      1  FF                                                   TERMINATOR_FF             
0x00000FB4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FB6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000FB8      3  F33258                                               IMM16_F3                  u16_be=12888, u16_le=22578
0x00000FBB      1  FF                                                   TERMINATOR_FF             
0x00000FBC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FBE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FC0     22  8014817582A282A282A982E782E6814191B182AB8176         LEN8_STRING_CP932         length=20, text="「いいからよ、続き」"
0x00000FD6      1  FF                                                   TERMINATOR_FF             
0x00000FD7      1  FF                                                   TERMINATOR_FF             
0x00000FD8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FDA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FDC      1  FF                                                   TERMINATOR_FF             
0x00000FDD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FDF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FE1      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000FE3      1  FF                                                   TERMINATOR_FF             
0x00000FE4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FE6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FE8      3  F301B3                                               IMM16_F3                  u16_be=435, u16_le=45825
0x00000FEB      1  FF                                                   TERMINATOR_FF             
0x00000FEC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FEE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000FF0      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000FF2      1  FF                                                   TERMINATOR_FF             
0x00000FF3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FF5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FF7      1  FF                                                   TERMINATOR_FF             
0x00000FF8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FFA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FFC      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000FFE      1  FF                                                   TERMINATOR_FF             
0x00000FFF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001001      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001003      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001005      1  FF                                                   TERMINATOR_FF             
0x00001006      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001008      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000100A      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000100C      1  FF                                                   TERMINATOR_FF             
0x0000100D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000100F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001011      1  FF                                                   TERMINATOR_FF             
0x00001012      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001014      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001016      8  800653452D533935                                     LEN8_STRING_CP932         length=6, text="SE-S95"
0x0000101E      1  FF                                                   TERMINATOR_FF             
0x0000101F      1  FF                                                   TERMINATOR_FF             
0x00001020      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001022      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001024      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00001026      1  FF                                                   TERMINATOR_FF             
0x00001027      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001029      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000102B      1  FF                                                   TERMINATOR_FF             
0x0000102C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000102E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001030      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001032      1  FF                                                   TERMINATOR_FF             
0x00001033      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001035      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001037      3  F33259                                               IMM16_F3                  u16_be=12889, u16_le=22834
0x0000103A      1  FF                                                   TERMINATOR_FF             
0x0000103B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000103D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000103F      9  80074D543035313730                                   LEN8_STRING_CP932         length=7, text="MT05170"
0x00001048      1  FF                                                   TERMINATOR_FF             
0x00001049      1  FF                                                   TERMINATOR_FF             
0x0000104A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000104C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000104E     36  8022817582F1815B8163816333363582D482F182CC815B33...  LEN8_STRING_CP932         length=34, text="「んー……365ぶんのー364でー……」"
0x00001072      1  FF                                                   TERMINATOR_FF             
0x00001073      1  FF                                                   TERMINATOR_FF             
0x00001074      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001076      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001078      1  FF                                                   TERMINATOR_FF             
0x00001079      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000107B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000107D      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x0000107F      1  FF                                                   TERMINATOR_FF             
0x00001080      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001082      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001084      1  FF                                                   TERMINATOR_FF             
0x00001085      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001087      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001089      8  800673652D653031                                     LEN8_STRING_CP932         length=6, text="se-e01"
0x00001091      1  FF                                                   TERMINATOR_FF             
0x00001092      1  FF                                                   TERMINATOR_FF             
0x00001093      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001095      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001097      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00001099      1  FF                                                   TERMINATOR_FF             
0x0000109A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000109C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000109E      1  FF                                                   TERMINATOR_FF             
0x0000109F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010A3      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000010A5      1  FF                                                   TERMINATOR_FF             
0x000010A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010A8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010AA      3  F301B4                                               IMM16_F3                  u16_be=436, u16_le=46081
0x000010AD      1  FF                                                   TERMINATOR_FF             
0x000010AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010B0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000010B2      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000010B4      1  FF                                                   TERMINATOR_FF             
0x000010B5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010B7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010B9      1  FF                                                   TERMINATOR_FF             
0x000010BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010BE      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000010C0      1  FF                                                   TERMINATOR_FF             
0x000010C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010C3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010C5      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000010C7      1  FF                                                   TERMINATOR_FF             
0x000010C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010CA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000010CC      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000010CE      1  FF                                                   TERMINATOR_FF             
0x000010CF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010D1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010D3      1  FF                                                   TERMINATOR_FF             
0x000010D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010D8      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000010DA      1  FF                                                   TERMINATOR_FF             
0x000010DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010DD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000010DF      3  F3325A                                               IMM16_F3                  u16_be=12890, u16_le=23090
0x000010E2      1  FF                                                   TERMINATOR_FF             
0x000010E3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010E5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000010E7      9  80074D543035313830                                   LEN8_STRING_CP932         length=7, text="MT05180"
0x000010F0      1  FF                                                   TERMINATOR_FF             
0x000010F1      1  FF                                                   TERMINATOR_FF             
0x000010F2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010F4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010F6     26  8018817582DC815B82BD815B82A8815B82BF815B82BD815B...  LEN8_STRING_CP932         length=24, text="「まーたーおーちーたー」"
0x00001110      1  FF                                                   TERMINATOR_FF             
0x00001111      1  FF                                                   TERMINATOR_FF             
0x00001112      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001114      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001116      1  FF                                                   TERMINATOR_FF             
0x00001117      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001119      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000111B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000111D      1  FF                                                   TERMINATOR_FF             
0x0000111E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001120      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001122      3  F3325B                                               IMM16_F3                  u16_be=12891, u16_le=23346
0x00001125      1  FF                                                   TERMINATOR_FF             
0x00001126      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001128      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000112A     16  800E8175978E82C682B782C882E68176                     LEN8_STRING_CP932         length=14, text="「落とすなよ」"
0x0000113A      1  FF                                                   TERMINATOR_FF             
0x0000113B      1  FF                                                   TERMINATOR_FF             
0x0000113C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000113E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001140      1  FF                                                   TERMINATOR_FF             
0x00001141      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001143      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001145      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001147      1  FF                                                   TERMINATOR_FF             
0x00001148      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000114A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000114C      3  F301B2                                               IMM16_F3                  u16_be=434, u16_le=45569
0x0000114F      1  FF                                                   TERMINATOR_FF             
0x00001150      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001152      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001154      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00001156      1  FF                                                   TERMINATOR_FF             
0x00001157      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001159      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000115B      1  FF                                                   TERMINATOR_FF             
0x0000115C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000115E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001160      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001162      1  FF                                                   TERMINATOR_FF             
0x00001163      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001165      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001167      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001169      1  FF                                                   TERMINATOR_FF             
0x0000116A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000116C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000116E      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001170      1  FF                                                   TERMINATOR_FF             
0x00001171      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001173      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001175      1  FF                                                   TERMINATOR_FF             
0x00001176      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001178      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000117A      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000117C      1  FF                                                   TERMINATOR_FF             
0x0000117D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000117F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001181      3  F3325C                                               IMM16_F3                  u16_be=12892, u16_le=23602
0x00001184      1  FF                                                   TERMINATOR_FF             
0x00001185      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001187      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001189      9  80074D543035313930                                   LEN8_STRING_CP932         length=7, text="MT05190"
0x00001192      1  FF                                                   TERMINATOR_FF             
0x00001193      1  FF                                                   TERMINATOR_FF             
0x00001194      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001196      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001198     54  8034817582D082EB82C182C482AD82F182C882AB82E18141...  LEN8_STRING_CP932         length=52, text="「ひろってくんなきゃ、ベンキョー\nでーきーなーいー」"
0x000011CE      1  FF                                                   TERMINATOR_FF             
0x000011CF      1  FF                                                   TERMINATOR_FF             
0x000011D0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000011D2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000011D4      1  FF                                                   TERMINATOR_FF             
0x000011D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011D9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000011DB      1  FF                                                   TERMINATOR_FF             
0x000011DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011DE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000011E0      3  F3325D                                               IMM16_F3                  u16_be=12893, u16_le=23858
0x000011E3      1  FF                                                   TERMINATOR_FF             
0x000011E4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011E6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011E8     34  8020817583818393836882AD82B9815B82C8815B814182A8...  LEN8_STRING_CP932         length=32, text="「メンドくせーなー、お前はよお」"
0x0000120A      1  FF                                                   TERMINATOR_FF             
0x0000120B      1  FF                                                   TERMINATOR_FF             
0x0000120C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000120E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001210      1  FF                                                   TERMINATOR_FF             
0x00001211      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001213      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001215      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00001217      1  FF                                                   TERMINATOR_FF             
0x00001218      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000121A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000121C      2  F227                                                 IMM8_F2                   u8=39, s8=39
0x0000121E      1  FF                                                   TERMINATOR_FF             
0x0000121F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001221      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001223      1  FF                                                   TERMINATOR_FF             
0x00001224      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001226      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001228      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000122A      1  FF                                                   TERMINATOR_FF             
0x0000122B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000122D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000122F      3  F3325E                                               IMM16_F3                  u16_be=12894, u16_le=24114
0x00001232      1  FF                                                   TERMINATOR_FF             
0x00001233      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001235      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001237     76  804A8BCA82CD82C982E282C982E282C6814183568383815B...  LEN8_STRING_CP932         length=74, text="玉はにやにやと、シャーペンをひろうぼくを見ている。\nなにか悪だくみの予感。"
0x00001283      1  FF                                                   TERMINATOR_FF             
0x00001284      1  FF                                                   TERMINATOR_FF             
0x00001285      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001287      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001289      1  FF                                                   TERMINATOR_FF             
0x0000128A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000128C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000128E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001290      1  FF                                                   TERMINATOR_FF             
0x00001291      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001293      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001295      3  F3325F                                               IMM16_F3                  u16_be=12895, u16_le=24370
0x00001298      1  FF                                                   TERMINATOR_FF             
0x00001299      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000129B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000129D     26  8018817582D982E7814182E082A4978E82C682B782C882E6...  LEN8_STRING_CP932         length=24, text="「ほら、もう落とすなよ」"
0x000012B7      1  FF                                                   TERMINATOR_FF             
0x000012B8      1  FF                                                   TERMINATOR_FF             
0x000012B9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012BB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012BD      1  FF                                                   TERMINATOR_FF             
0x000012BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012C2      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000012C4      1  FF                                                   TERMINATOR_FF             
0x000012C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012C7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000012C9      3  F33260                                               IMM16_F3                  u16_be=12896, u16_le=24626
0x000012CC      1  FF                                                   TERMINATOR_FF             
0x000012CD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012CF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000012D1      9  80074D543035323030                                   LEN8_STRING_CP932         length=7, text="MT05200"
0x000012DA      1  FF                                                   TERMINATOR_FF             
0x000012DB      1  FF                                                   TERMINATOR_FF             
0x000012DC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012DE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012E0     14  800C817582F18160816381638176                         LEN8_STRING_CP932         length=12, text="「ん～……」"
0x000012EE      1  FF                                                   TERMINATOR_FF             
0x000012EF      1  FF                                                   TERMINATOR_FF             
0x000012F0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012F2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012F4      1  FF                                                   TERMINATOR_FF             
0x000012F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012F9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000012FB      1  FF                                                   TERMINATOR_FF             
0x000012FC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012FE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001300      3  F33261                                               IMM16_F3                  u16_be=12897, u16_le=24882
0x00001303      1  FF                                                   TERMINATOR_FF             
0x00001304      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001306      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001308     42  80288BCA82CD88AC82C182BD83568383815B8379839382F0...  LEN8_STRING_CP932         length=40, text="玉は握ったシャーペンをすぐに放り投げる。"
0x00001332      1  FF                                                   TERMINATOR_FF             
0x00001333      1  FF                                                   TERMINATOR_FF             
0x00001334      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001336      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001338      1  FF                                                   TERMINATOR_FF             
0x00001339      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000133B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000133D      8  800673652D653031                                     LEN8_STRING_CP932         length=6, text="se-e01"
0x00001345      1  FF                                                   TERMINATOR_FF             
0x00001346      1  FF                                                   TERMINATOR_FF             
0x00001347      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001349      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000134B      2  F21B                                                 IMM8_F2                   u8=27, s8=27
0x0000134D      1  FF                                                   TERMINATOR_FF             
0x0000134E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001350      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001352      1  FF                                                   TERMINATOR_FF             
0x00001353      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001355      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001357      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001359      1  FF                                                   TERMINATOR_FF             
0x0000135A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000135C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000135E      3  F33262                                               IMM16_F3                  u16_be=12898, u16_le=25138
0x00001361      1  FF                                                   TERMINATOR_FF             
0x00001362      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001364      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001366      9  80074D543035323130                                   LEN8_STRING_CP932         length=7, text="MT05210"
0x0000136F      1  FF                                                   TERMINATOR_FF             
0x00001370      1  FF                                                   TERMINATOR_FF             
0x00001371      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001373      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001375     18  8010817582A8815B82BF815B82BD815B8176                 LEN8_STRING_CP932         length=16, text="「おーちーたー」"
0x00001387      1  FF                                                   TERMINATOR_FF             
0x00001388      1  FF                                                   TERMINATOR_FF             
0x00001389      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000138B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000138D      1  FF                                                   TERMINATOR_FF             
0x0000138E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001390      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001392      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001394      1  FF                                                   TERMINATOR_FF             
0x00001395      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001397      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001399      3  F33263                                               IMM16_F3                  u16_be=12899, u16_le=25394
0x0000139C      1  FF                                                   TERMINATOR_FF             
0x0000139D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000139F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013A1     36  80228175978E82BF82BD82B682E182C882AD8141978E82C6...  LEN8_STRING_CP932         length=34, text="「落ちたじゃなく、落としてんだろ」"
0x000013C5      1  FF                                                   TERMINATOR_FF             
0x000013C6      1  FF                                                   TERMINATOR_FF             
0x000013C7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000013C9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000013CB      1  FF                                                   TERMINATOR_FF             
0x000013CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013D0      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000013D2      1  FF                                                   TERMINATOR_FF             
0x000013D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013D5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000013D7      3  F33264                                               IMM16_F3                  u16_be=12900, u16_le=25650
0x000013DA      1  FF                                                   TERMINATOR_FF             
0x000013DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013DD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000013DF      9  80074D543035323230                                   LEN8_STRING_CP932         length=7, text="MT05220"
0x000013E8      1  FF                                                   TERMINATOR_FF             
0x000013E9      1  FF                                                   TERMINATOR_FF             
0x000013EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013EC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013EE     84  8052817583568383815B8379839382C882A282C683788393...  LEN8_STRING_CP932         length=82, text="「シャーペンないとベンキョーできなーいー。\nあ～、マイ、ベンキョーしたかったなー」"
0x00001442      1  FF                                                   TERMINATOR_FF             
0x00001443      1  FF                                                   TERMINATOR_FF             
0x00001444      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001446      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001448      1  FF                                                   TERMINATOR_FF             
0x00001449      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000144B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000144D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000144F      1  FF                                                   TERMINATOR_FF             
0x00001450      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001452      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001454      3  F33265                                               IMM16_F3                  u16_be=12901, u16_le=25906
0x00001457      1  FF                                                   TERMINATOR_FF             
0x00001458      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000145A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000145C     24  801682B782B282AD82A482EA82B582BB82A482C88BCA8142     LEN8_STRING_CP932         length=22, text="すごくうれしそうな玉。"
0x00001474      1  FF                                                   TERMINATOR_FF             
0x00001475      1  FF                                                   TERMINATOR_FF             
0x00001476      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001478      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000147A      1  FF                                                   TERMINATOR_FF             
0x0000147B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000147D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000147F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001481      1  FF                                                   TERMINATOR_FF             
0x00001482      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001484      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001486      3  F33266                                               IMM16_F3                  u16_be=12902, u16_le=26162
0x00001489      1  FF                                                   TERMINATOR_FF             
0x0000148A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000148C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000148E    102  8064916692BC82C982D082EB82C182C482ED82BD82B582C4...  LEN8_STRING_CP932         length=100, text="素直にひろってわたしても、どーせまたすぐ\n落とすに決まってる。\nちょっと荒療治かもしれないけど……。"
0x000014F4      1  FF                                                   TERMINATOR_FF             
0x000014F5      1  FF                                                   TERMINATOR_FF             
0x000014F6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000014F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000014FA      1  FF                                                   TERMINATOR_FF             
0x000014FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014FF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001501      1  FF                                                   TERMINATOR_FF             
0x00001502      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001504      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001506      3  F33267                                               IMM16_F3                  u16_be=12903, u16_le=26418
0x00001509      1  FF                                                   TERMINATOR_FF             
0x0000150A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000150C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000150E     22  8014817582BF82E582C182C691D282C182C482EB8176         LEN8_STRING_CP932         length=20, text="「ちょっと待ってろ」"
0x00001524      1  FF                                                   TERMINATOR_FF             
0x00001525      1  FF                                                   TERMINATOR_FF             
0x00001526      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001528      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000152A      1  FF                                                   TERMINATOR_FF             
0x0000152B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000152D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000152F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001531      1  FF                                                   TERMINATOR_FF             
0x00001532      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001534      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001536      3  F33268                                               IMM16_F3                  u16_be=12904, u16_le=26674
0x00001539      1  FF                                                   TERMINATOR_FF             
0x0000153A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000153C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000153E      9  80074D543035323330                                   LEN8_STRING_CP932         length=7, text="MT05230"
0x00001547      1  FF                                                   TERMINATOR_FF             
0x00001548      1  FF                                                   TERMINATOR_FF             
0x00001549      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000154B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000154D     25  8017817582F1816081482082DE816082D3816082D3816081...  LEN8_STRING_CP932         length=23, text="「ん～？ む～ふ～ふ～」"
0x00001566      1  FF                                                   TERMINATOR_FF             
0x00001567      1  FF                                                   TERMINATOR_FF             
0x00001568      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000156A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000156C      1  FF                                                   TERMINATOR_FF             
0x0000156D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000156F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001571      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001573      1  FF                                                   TERMINATOR_FF             
0x00001574      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001576      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001578      3  F33269                                               IMM16_F3                  u16_be=12905, u16_le=26930
0x0000157B      1  FF                                                   TERMINATOR_FF             
0x0000157C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000157E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001580     88  80568BB38EBA8CE382EB82CC97708BEF94A082F08A4A82AF...  LEN8_STRING_CP932         length=86, text="教室後ろの用具箱を開ける。\n掃除道具一式にくわえて、細々とした備品が\nおさまっている。"
0x000015D8      1  FF                                                   TERMINATOR_FF             
0x000015D9      1  FF                                                   TERMINATOR_FF             
0x000015DA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000015DC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000015DE      1  FF                                                   TERMINATOR_FF             
0x000015DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015E3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000015E5      1  FF                                                   TERMINATOR_FF             
0x000015E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015E8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000015EA      3  F3326A                                               IMM16_F3                  u16_be=12906, u16_le=27186
0x000015ED      1  FF                                                   TERMINATOR_FF             
0x000015EE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015F0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015F2     18  8010817582A082CC82E682A881418BCA8176                 LEN8_STRING_CP932         length=16, text="「あのよお、玉」"
0x00001604      1  FF                                                   TERMINATOR_FF             
0x00001605      1  FF                                                   TERMINATOR_FF             
0x00001606      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001608      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000160A      1  FF                                                   TERMINATOR_FF             
0x0000160B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000160D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000160F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001611      1  FF                                                   TERMINATOR_FF             
0x00001612      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001614      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001616      3  F3326B                                               IMM16_F3                  u16_be=12907, u16_le=27442
0x00001619      1  FF                                                   TERMINATOR_FF             
0x0000161A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000161C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000161E      9  80074D543035323430                                   LEN8_STRING_CP932         length=7, text="MT05240"
0x00001627      1  FF                                                   TERMINATOR_FF             
0x00001628      1  FF                                                   TERMINATOR_FF             
0x00001629      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000162B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000162D     23  8015817582F1816081482082C8816082C9816081488176       LEN8_STRING_CP932         length=21, text="「ん～？ な～に～？」"
0x00001644      1  FF                                                   TERMINATOR_FF             
0x00001645      1  FF                                                   TERMINATOR_FF             
0x00001646      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001648      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000164A      1  FF                                                   TERMINATOR_FF             
0x0000164B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000164D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000164F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001651      1  FF                                                   TERMINATOR_FF             
0x00001652      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001654      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001656      3  F3326C                                               IMM16_F3                  u16_be=12908, u16_le=27698
0x00001659      1  FF                                                   TERMINATOR_FF             
0x0000165A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000165C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000165E     22  8014817582B1815B82B582C482E282E9815B21218176         LEN8_STRING_CP932         length=20, text="「こーしてやるー!!」"
0x00001674      1  FF                                                   TERMINATOR_FF             
0x00001675      1  FF                                                   TERMINATOR_FF             
0x00001676      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001678      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000167A      1  FF                                                   TERMINATOR_FF             
0x0000167B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000167D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000167F      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001681      1  FF                                                   TERMINATOR_FF             
0x00001682      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001684      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001686      3  F30213                                               IMM16_F3                  u16_be=531, u16_le=4866
0x00001689      1  FF                                                   TERMINATOR_FF             
0x0000168A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000168C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000168E      1  FF                                                   TERMINATOR_FF             
0x0000168F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001691      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001693      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001695      1  FF                                                   TERMINATOR_FF             
0x00001696      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001698      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000169A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000169C      1  FF                                                   TERMINATOR_FF             
0x0000169D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000169F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000016A1      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000016A3      1  FF                                                   TERMINATOR_FF             
0x000016A4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016A6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016A8      1  FF                                                   TERMINATOR_FF             
0x000016A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016AD      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x000016AF      1  FF                                                   TERMINATOR_FF             
0x000016B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016B4      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x000016B6      1  FF                                                   TERMINATOR_FF             
0x000016B7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016B9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016BB      1  FF                                                   TERMINATOR_FF             
0x000016BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016C0      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000016C2      1  FF                                                   TERMINATOR_FF             
0x000016C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016C5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016C7      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000016C9      1  FF                                                   TERMINATOR_FF             
0x000016CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016CC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000016CE      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000016D0      1  FF                                                   TERMINATOR_FF             
0x000016D1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016D3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016D5      1  FF                                                   TERMINATOR_FF             
0x000016D6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016D8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000016DA      8  800673652D653339                                     LEN8_STRING_CP932         length=6, text="se-e39"
0x000016E2      1  FF                                                   TERMINATOR_FF             
0x000016E3      1  FF                                                   TERMINATOR_FF             
0x000016E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016E8      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x000016EA      1  FF                                                   TERMINATOR_FF             
0x000016EB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016ED      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016EF      1  FF                                                   TERMINATOR_FF             
0x000016F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016F4      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000016F6      1  FF                                                   TERMINATOR_FF             
0x000016F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016F9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000016FB      3  F3326D                                               IMM16_F3                  u16_be=12909, u16_le=27954
0x000016FE      1  FF                                                   TERMINATOR_FF             
0x000016FF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001701      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001703      9  80074D543035323530                                   LEN8_STRING_CP932         length=7, text="MT05250"
0x0000170C      1  FF                                                   TERMINATOR_FF             
0x0000170D      1  FF                                                   TERMINATOR_FF             
0x0000170E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001710      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001712     62  803C817582A68160816082C13F212082AB82E1815B814182...  LEN8_STRING_CP932         length=60, text="「え～～っ?! きゃー、やだー!!\nな～にすんのー!! たすけてー」"
0x00001750      1  FF                                                   TERMINATOR_FF             
0x00001751      1  FF                                                   TERMINATOR_FF             
0x00001752      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001754      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001756      1  FF                                                   TERMINATOR_FF             
0x00001757      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001759      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000175B      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000175D      1  FF                                                   TERMINATOR_FF             
0x0000175E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001760      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001762      3  F30214                                               IMM16_F3                  u16_be=532, u16_le=5122
0x00001765      1  FF                                                   TERMINATOR_FF             
0x00001766      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001768      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000176A      1  FF                                                   TERMINATOR_FF             
0x0000176B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000176D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000176F      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001771      1  FF                                                   TERMINATOR_FF             
0x00001772      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001774      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001776      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001778      1  FF                                                   TERMINATOR_FF             
0x00001779      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000177B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000177D      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000177F      1  FF                                                   TERMINATOR_FF             
0x00001780      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001782      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001784      1  FF                                                   TERMINATOR_FF             
0x00001785      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001787      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001789      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000178B      1  FF                                                   TERMINATOR_FF             
0x0000178C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000178E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001790      3  F3326E                                               IMM16_F3                  u16_be=12910, u16_le=28210
0x00001793      1  FF                                                   TERMINATOR_FF             
0x00001794      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001796      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001798      9  80074D543035323630                                   LEN8_STRING_CP932         length=7, text="MT05260"
0x000017A1      1  FF                                                   TERMINATOR_FF             
0x000017A2      1  FF                                                   TERMINATOR_FF             
0x000017A3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017A5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017A7     33  801F817582A081608160816081482082C882C9814182B182...  LEN8_STRING_CP932         length=31, text="「あ～～～？ なに、これ～～？」"
0x000017C8      1  FF                                                   TERMINATOR_FF             
0x000017C9      1  FF                                                   TERMINATOR_FF             
0x000017CA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000017CC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000017CE      1  FF                                                   TERMINATOR_FF             
0x000017CF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017D3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000017D5      1  FF                                                   TERMINATOR_FF             
0x000017D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017D8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000017DA      3  F3326F                                               IMM16_F3                  u16_be=12911, u16_le=28466
0x000017DD      1  FF                                                   TERMINATOR_FF             
0x000017DE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017E0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017E2     52  80328BCA82CD834B8380836582C5834F838B834F838B8AAA...  LEN8_STRING_CP932         length=50, text="玉はガムテでグルグル巻きにされた右手を\n見つめる。"
0x00001816      1  FF                                                   TERMINATOR_FF             
0x00001817      1  FF                                                   TERMINATOR_FF             
0x00001818      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000181A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000181C      1  FF                                                   TERMINATOR_FF             
0x0000181D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000181F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001821      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001823      1  FF                                                   TERMINATOR_FF             
0x00001824      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001826      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001828      3  F33270                                               IMM16_F3                  u16_be=12912, u16_le=28722
0x0000182B      1  FF                                                   TERMINATOR_FF             
0x0000182C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000182E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001830     58  803883568383815B8379839382CD82AD82AD82E882C282AF...  LEN8_STRING_CP932         length=56, text="シャーペンはくくりつけられているので、もう\n落とせない。"
0x0000186A      1  FF                                                   TERMINATOR_FF             
0x0000186B      1  FF                                                   TERMINATOR_FF             
0x0000186C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000186E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001870      1  FF                                                   TERMINATOR_FF             
0x00001871      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001873      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001875      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001877      1  FF                                                   TERMINATOR_FF             
0x00001878      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000187A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000187C      3  F33271                                               IMM16_F3                  u16_be=12913, u16_le=28978
0x0000187F      1  FF                                                   TERMINATOR_FF             
0x00001880      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001882      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001884      9  80074D543035323730                                   LEN8_STRING_CP932         length=7, text="MT05270"
0x0000188D      1  FF                                                   TERMINATOR_FF             
0x0000188E      1  FF                                                   TERMINATOR_FF             
0x0000188F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001891      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001893     40  8026817582B0815B814182D082C182C7816082A28142837D...  LEN8_STRING_CP932         length=38, text="「げー、ひっど～い。マイ、かわいそー」"
0x000018BB      1  FF                                                   TERMINATOR_FF             
0x000018BC      1  FF                                                   TERMINATOR_FF             
0x000018BD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000018BF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000018C1      1  FF                                                   TERMINATOR_FF             
0x000018C2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018C6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000018C8      1  FF                                                   TERMINATOR_FF             
0x000018C9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018CB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000018CD      3  F33272                                               IMM16_F3                  u16_be=12914, u16_le=29234
0x000018D0      1  FF                                                   TERMINATOR_FF             
0x000018D1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018D3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018D5     42  8028817582BB82CC96E291E88F4982ED82E782B982BD82E7...  LEN8_STRING_CP932         length=40, text="「その問題終わらせたら、とってやるから」"
0x000018FF      1  FF                                                   TERMINATOR_FF             
0x00001900      1  FF                                                   TERMINATOR_FF             
0x00001901      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001903      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001905      1  FF                                                   TERMINATOR_FF             
0x00001906      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001908      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000190A      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000190C      1  FF                                                   TERMINATOR_FF             
0x0000190D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000190F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001911      3  F33273                                               IMM16_F3                  u16_be=12915, u16_le=29490
0x00001914      1  FF                                                   TERMINATOR_FF             
0x00001915      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001917      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001919      9  80074D543035323830                                   LEN8_STRING_CP932         length=7, text="MT05280"
0x00001922      1  FF                                                   TERMINATOR_FF             
0x00001923      1  FF                                                   TERMINATOR_FF             
0x00001924      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001926      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001928     39  8025817582A681608160816081482082A882ED82F182C882...  LEN8_STRING_CP932         length=37, text="「え～～～？ おわんないよ、きっとー」"
0x0000194F      1  FF                                                   TERMINATOR_FF             
0x00001950      1  FF                                                   TERMINATOR_FF             
0x00001951      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001953      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001955      1  FF                                                   TERMINATOR_FF             
0x00001956      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001958      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000195A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000195C      1  FF                                                   TERMINATOR_FF             
0x0000195D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000195F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001961      3  F33274                                               IMM16_F3                  u16_be=12916, u16_le=29746
0x00001964      1  FF                                                   TERMINATOR_FF             
0x00001965      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001967      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001969     16  800E81758F4982ED82E782B982EB8176                     LEN8_STRING_CP932         length=14, text="「終わらせろ」"
0x00001979      1  FF                                                   TERMINATOR_FF             
0x0000197A      1  FF                                                   TERMINATOR_FF             
0x0000197B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000197D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000197F      1  FF                                                   TERMINATOR_FF             
0x00001980      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001982      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001984      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001986      1  FF                                                   TERMINATOR_FF             
0x00001987      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001989      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000198B      3  F30215                                               IMM16_F3                  u16_be=533, u16_le=5378
0x0000198E      1  FF                                                   TERMINATOR_FF             
0x0000198F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001991      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001993      1  FF                                                   TERMINATOR_FF             
0x00001994      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001996      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001998      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000199A      1  FF                                                   TERMINATOR_FF             
0x0000199B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000199D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000199F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000019A1      1  FF                                                   TERMINATOR_FF             
0x000019A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019A4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000019A6      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000019A8      1  FF                                                   TERMINATOR_FF             
0x000019A9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000019AB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000019AD      1  FF                                                   TERMINATOR_FF             
0x000019AE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019B0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000019B2      7  800573652D3432                                       LEN8_STRING_CP932         length=5, text="se-42"
0x000019B9      1  FF                                                   TERMINATOR_FF             
0x000019BA      1  FF                                                   TERMINATOR_FF             
0x000019BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019BF      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x000019C1      1  FF                                                   TERMINATOR_FF             
0x000019C2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000019C4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000019C6      1  FF                                                   TERMINATOR_FF             
0x000019C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019C9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019CB      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000019CD      1  FF                                                   TERMINATOR_FF             
0x000019CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019D0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000019D2      3  F33275                                               IMM16_F3                  u16_be=12917, u16_le=30002
0x000019D5      1  FF                                                   TERMINATOR_FF             
0x000019D6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019D8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000019DA      9  80074D543035323930                                   LEN8_STRING_CP932         length=7, text="MT05290"
0x000019E3      1  FF                                                   TERMINATOR_FF             
0x000019E4      1  FF                                                   TERMINATOR_FF             
0x000019E5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019E9     60  803A817582E282BE815B814182A882ED82F182C882A2815B...  LEN8_STRING_CP932         length=58, text="「やだー、おわんないー、マイ、いっしょー\nこのまんまだー」"
0x00001A25      1  FF                                                   TERMINATOR_FF             
0x00001A26      1  FF                                                   TERMINATOR_FF             
0x00001A27      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A29      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A2B      1  FF                                                   TERMINATOR_FF             
0x00001A2C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A2E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A30      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001A32      1  FF                                                   TERMINATOR_FF             
0x00001A33      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A35      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001A37      3  F33276                                               IMM16_F3                  u16_be=12918, u16_le=30258
0x00001A3A      1  FF                                                   TERMINATOR_FF             
0x00001A3B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A3D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A3F     42  8028817582BE82A982E781418F4982ED82E782B982BD82E7...  LEN8_STRING_CP932         length=40, text="「だから、終わらせたら、とってやるって」"
0x00001A69      1  FF                                                   TERMINATOR_FF             
0x00001A6A      1  FF                                                   TERMINATOR_FF             
0x00001A6B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A6D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A6F      1  FF                                                   TERMINATOR_FF             
0x00001A70      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A72      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A74      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001A76      1  FF                                                   TERMINATOR_FF             
0x00001A77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A79      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001A7B      3  F33277                                               IMM16_F3                  u16_be=12919, u16_le=30514
0x00001A7E      1  FF                                                   TERMINATOR_FF             
0x00001A7F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A81      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001A83      9  80074D543035333030                                   LEN8_STRING_CP932         length=7, text="MT05300"
0x00001A8C      1  FF                                                   TERMINATOR_FF             
0x00001A8D      1  FF                                                   TERMINATOR_FF             
0x00001A8E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A90      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A92     54  8034817582E282BE815B814182A882ED82F182C882A2815B...  LEN8_STRING_CP932         length=52, text="「やだー、おわんないー。おわんない\nおわんなーいー」"
0x00001AC8      1  FF                                                   TERMINATOR_FF             
0x00001AC9      1  FF                                                   TERMINATOR_FF             
0x00001ACA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001ACC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001ACE      1  FF                                                   TERMINATOR_FF             
0x00001ACF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001AD1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001AD3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001AD5      1  FF                                                   TERMINATOR_FF             
0x00001AD6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001AD8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001ADA      3  F33278                                               IMM16_F3                  u16_be=12920, u16_le=30770
0x00001ADD      1  FF                                                   TERMINATOR_FF             
0x00001ADE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001AE0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001AE2     58  80388BCA82CD8357835E836F835E82C682AE82B882E891B1...  LEN8_STRING_CP932         length=56, text="玉はジタバタとぐずり続ける。\nしまった、逆効果だったか。"
0x00001B1C      1  FF                                                   TERMINATOR_FF             
0x00001B1D      1  FF                                                   TERMINATOR_FF             
0x00001B1E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B20      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B22      1  FF                                                   TERMINATOR_FF             
0x00001B23      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B25      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B27      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00001B29      1  FF                                                   TERMINATOR_FF             
0x00001B2A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B2C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B2E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B30      1  FF                                                   TERMINATOR_FF             
0x00001B31      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B33      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B35      1  FF                                                   TERMINATOR_FF             
0x00001B36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B38      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B3A      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001B3C      1  FF                                                   TERMINATOR_FF             
0x00001B3D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B3F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B41      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00001B43      1  FF                                                   TERMINATOR_FF             
0x00001B44      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B46      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B48      1  FF                                                   TERMINATOR_FF             
0x00001B49      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B4B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B4D      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00001B4F      1  FF                                                   TERMINATOR_FF             
0x00001B50      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B52      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B54      1  FF                                                   TERMINATOR_FF             
0x00001B55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B57      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B59      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001B5B      1  FF                                                   TERMINATOR_FF             
0x00001B5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B5E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B60      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001B62      1  FF                                                   TERMINATOR_FF             
0x00001B63      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B65      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001B67      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001B69      1  FF                                                   TERMINATOR_FF             
0x00001B6A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B6C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B6E      1  FF                                                   TERMINATOR_FF             
0x00001B6F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B71      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B73      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00001B75      1  FF                                                   TERMINATOR_FF             
0x00001B76      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B78      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B7A      1  FF                                                   TERMINATOR_FF             
0x00001B7B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B7D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B7F      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00001B81      1  FF                                                   TERMINATOR_FF             
0x00001B82      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B84      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00001B86     14  800C50533254303137612E62696E                         LEN8_STRING_CP932         length=12, text="PS2T017a.bin"
0x00001B94      1  FF                                                   TERMINATOR_FF             
0x00001B95      1  FF                                                   TERMINATOR_FF             
0x00001B96      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B98      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B9A      1  FF                                                   TERMINATOR_FF             
0x00001B9B      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00001B9D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001B9F      1  FF                                                   TERMINATOR_FF             
0x00001BA0      2  001B                                                 WORD_00XX                 u16_be=27, low_byte=27
0x00001BA2      1  AA                                                   OPAQUE_RAW_BYTES          bytes=AA
0x00001BA3      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x00001BA5      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00001BA7      2  001B                                                 WORD_00XX                 u16_be=27, low_byte=27
0x00001BA9      1  9B                                                   OPAQUE_RAW_BYTES          bytes=9B
0x00001BAA      1  FF                                                   TERMINATOR_FF             
0x00001BAB      1  FF                                                   TERMINATOR_FF             
