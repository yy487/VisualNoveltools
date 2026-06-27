; Full conservative disassembly for PS2I012A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      2  F243                                                 IMM8_F2                   u8=67, s8=67
0x0000000D      1  FF                                                   TERMINATOR_FF             
0x0000000E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000010      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000012      1  FF                                                   TERMINATOR_FF             
0x00000013      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000015      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000017      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000019      1  FF                                                   TERMINATOR_FF             
0x0000001A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000001C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000001E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000020      1  FF                                                   TERMINATOR_FF             
0x00000021      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000023      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000025      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000027      1  FF                                                   TERMINATOR_FF             
0x00000028      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000002A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000002C      1  FF                                                   TERMINATOR_FF             
0x0000002D      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000002F      3  F10054                                               IMM16_F1                  u16_be=84, u16_le=21504
0x00000032      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000034      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000035      1  FF                                                   TERMINATOR_FF             
0x00000036      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000038      1  E3                                                   OPAQUE_RAW_BYTES          bytes=E3
0x00000039      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000003B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000003D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000003F      1  FF                                                   TERMINATOR_FF             
0x00000040      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000042      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000044      3  F33E7D                                               IMM16_F3                  u16_be=15997, u16_le=32062
0x00000047      1  FF                                                   TERMINATOR_FF             
0x00000048      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000004A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000004C     64  803E82E682A482E282AD8141918D96B1825189DB82C68376...  LEN8_STRING_CP932         length=62, text="ようやく、総務２課とプレートのかかった\nオフィスにたどりつく。"
0x0000008C      1  FF                                                   TERMINATOR_FF             
0x0000008D      1  FF                                                   TERMINATOR_FF             
0x0000008E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000090      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000092      1  FF                                                   TERMINATOR_FF             
0x00000093      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000095      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000097      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000099      1  FF                                                   TERMINATOR_FF             
0x0000009A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000009C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000009E      3  F33E7E                                               IMM16_F3                  u16_be=15998, u16_le=32318
0x000000A1      1  FF                                                   TERMINATOR_FF             
0x000000A2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000A4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000A6     54  803494E082CD8A4A82AF82C195FA82B581425C6E82C882F1...  LEN8_STRING_CP932         length=52, text="扉は開けっ放し。\nなんとなく、まったりとした雰囲気。"
0x000000DC      1  FF                                                   TERMINATOR_FF             
0x000000DD      1  FF                                                   TERMINATOR_FF             
0x000000DE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000E0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000E2      1  FF                                                   TERMINATOR_FF             
0x000000E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000000E9      1  FF                                                   TERMINATOR_FF             
0x000000EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000EC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000000EE      3  F31D8B                                               IMM16_F3                  u16_be=7563, u16_le=35613
0x000000F1      1  FF                                                   TERMINATOR_FF             
0x000000F2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000F4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000F6     34  802082DA82AD82CD81418BDF82AD82CC904588F582C990BA...  LEN8_STRING_CP932         length=32, text="ぼくは、近くの職員に声をかけた。"
0x00000118      1  FF                                                   TERMINATOR_FF             
0x00000119      1  FF                                                   TERMINATOR_FF             
0x0000011A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000011C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000011E      1  FF                                                   TERMINATOR_FF             
0x0000011F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000121      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000123      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000125      1  FF                                                   TERMINATOR_FF             
0x00000126      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000128      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000012A      3  F31D8C                                               IMM16_F3                  u16_be=7564, u16_le=35869
0x0000012D      1  FF                                                   TERMINATOR_FF             
0x0000012E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000130      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000132     47  802D817582A082CC815B81418ADB93A12090F294FC82B382...  LEN8_STRING_CP932         length=45, text="「あのー、丸藤 泉美さん、お願いできますか？」"
0x00000161      1  FF                                                   TERMINATOR_FF             
0x00000162      1  FF                                                   TERMINATOR_FF             
0x00000163      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000165      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000167      1  FF                                                   TERMINATOR_FF             
0x00000168      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000016A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000016C      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000016E      1  FF                                                   TERMINATOR_FF             
0x0000016F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000171      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000173      3  F301F6                                               IMM16_F3                  u16_be=502, u16_le=62977
0x00000176      1  FF                                                   TERMINATOR_FF             
0x00000177      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000179      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000017B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000017D      1  FF                                                   TERMINATOR_FF             
0x0000017E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000180      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000182      1  FF                                                   TERMINATOR_FF             
0x00000183      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000185      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000187      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000189      1  FF                                                   TERMINATOR_FF             
0x0000018A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000018C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000018E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000190      1  FF                                                   TERMINATOR_FF             
0x00000191      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000193      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000195      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000197      1  FF                                                   TERMINATOR_FF             
0x00000198      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000019A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000019C      1  FF                                                   TERMINATOR_FF             
0x0000019D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000019F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001A1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001A3      1  FF                                                   TERMINATOR_FF             
0x000001A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001A6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001A8      3  F31D8D                                               IMM16_F3                  u16_be=7565, u16_le=36125
0x000001AB      1  FF                                                   TERMINATOR_FF             
0x000001AC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001AE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001B0    100  806282D382E88CFC82A282BD82BB82CC8F9790AB82CD8141...  LEN8_STRING_CP932         length=98, text="ふり向いたその女性は、褐色の肌、紫に染めた\nショートヘア。\nどう見ても、役所勤めには見えなかった。"
0x00000214      1  FF                                                   TERMINATOR_FF             
0x00000215      1  FF                                                   TERMINATOR_FF             
0x00000216      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000218      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000021A      1  FF                                                   TERMINATOR_FF             
0x0000021B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000021D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000021F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000221      1  FF                                                   TERMINATOR_FF             
0x00000222      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000224      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000226      3  F31D8E                                               IMM16_F3                  u16_be=7566, u16_le=36381
0x00000229      1  FF                                                   TERMINATOR_FF             
0x0000022A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000022C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000022E      8  8006593030383930                                     LEN8_STRING_CP932         length=6, text="Y00890"
0x00000236      1  FF                                                   TERMINATOR_FF             
0x00000237      1  FF                                                   TERMINATOR_FF             
0x00000238      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000023A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000023C     69  8043817582F1816081418ADB93A1816081482082A082A081...  LEN8_STRING_CP932         length=67, text="「ん～、丸藤～？ ああ、マルねー。丸なら、\n公園でさぼってるけどー」"
0x00000281      1  FF                                                   TERMINATOR_FF             
0x00000282      1  FF                                                   TERMINATOR_FF             
0x00000283      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000285      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000287      1  FF                                                   TERMINATOR_FF             
0x00000288      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000028A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000028C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000028E      1  FF                                                   TERMINATOR_FF             
0x0000028F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000291      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000293      3  F31D8F                                               IMM16_F3                  u16_be=7567, u16_le=36637
0x00000296      1  FF                                                   TERMINATOR_FF             
0x00000297      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000299      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000029B    114  807082BB82B582C4814182BB82CC8358838C8393835F815B...  LEN8_STRING_CP932         length=112, text="そして、そのスレンダーな女性（……丸藤さんがよく話してる『優輝ちゃん』とゆー気がした）は、ちょっと眉をひそめる。"
0x0000030D      1  FF                                                   TERMINATOR_FF             
0x0000030E      1  FF                                                   TERMINATOR_FF             
0x0000030F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000311      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000313      1  FF                                                   TERMINATOR_FF             
0x00000314      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000316      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000318      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000031A      1  FF                                                   TERMINATOR_FF             
0x0000031B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000031D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000031F      3  F301F7                                               IMM16_F3                  u16_be=503, u16_le=63233
0x00000322      1  FF                                                   TERMINATOR_FF             
0x00000323      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000325      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000327      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000329      1  FF                                                   TERMINATOR_FF             
0x0000032A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000032C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000032E      1  FF                                                   TERMINATOR_FF             
0x0000032F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000331      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000333      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000335      1  FF                                                   TERMINATOR_FF             
0x00000336      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000338      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000033A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000033C      1  FF                                                   TERMINATOR_FF             
0x0000033D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000033F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000341      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000343      1  FF                                                   TERMINATOR_FF             
0x00000344      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000346      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000348      1  FF                                                   TERMINATOR_FF             
0x00000349      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000034B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000034D      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000034F      1  FF                                                   TERMINATOR_FF             
0x00000350      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000352      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000354      3  F31D90                                               IMM16_F3                  u16_be=7568, u16_le=36893
0x00000357      1  FF                                                   TERMINATOR_FF             
0x00000358      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000035A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000035C      8  8006593030393030                                     LEN8_STRING_CP932         length=6, text="Y00900"
0x00000364      1  FF                                                   TERMINATOR_FF             
0x00000365      1  FF                                                   TERMINATOR_FF             
0x00000366      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000368      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000036A     22  8014817582C5814182A082F182BD82C882C981488176         LEN8_STRING_CP932         length=20, text="「で、あんたなに？」"
0x00000380      1  FF                                                   TERMINATOR_FF             
0x00000381      1  FF                                                   TERMINATOR_FF             
0x00000382      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000384      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000386      1  FF                                                   TERMINATOR_FF             
0x00000387      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000389      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000038B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000038D      1  FF                                                   TERMINATOR_FF             
0x0000038E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000390      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000392      3  F31D91                                               IMM16_F3                  u16_be=7569, u16_le=37149
0x00000395      1  FF                                                   TERMINATOR_FF             
0x00000396      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000398      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000039A     42  8028817582A282A6814182BF82E582C182C697708E9682AA...  LEN8_STRING_CP932         length=40, text="「いえ、ちょっと用事があったもんで……」"
0x000003C4      1  FF                                                   TERMINATOR_FF             
0x000003C5      1  FF                                                   TERMINATOR_FF             
0x000003C6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003C8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003CA      1  FF                                                   TERMINATOR_FF             
0x000003CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003CF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000003D1      1  FF                                                   TERMINATOR_FF             
0x000003D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003D4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000003D6      3  F31D92                                               IMM16_F3                  u16_be=7570, u16_le=37405
0x000003D9      1  FF                                                   TERMINATOR_FF             
0x000003DA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003DC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003DE     96  805E82DA82AD82CD814197448B5082BF82E182F1816982E7...  LEN8_STRING_CP932         length=94, text="ぼくは、優輝ちゃん（らしき女性）の品定めするような視線に、総務２課のオフィスから\n逃げ出した。"
0x0000043E      1  FF                                                   TERMINATOR_FF             
0x0000043F      1  FF                                                   TERMINATOR_FF             
0x00000440      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000442      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000444      1  FF                                                   TERMINATOR_FF             
0x00000445      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000447      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000449      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000044B      1  FF                                                   TERMINATOR_FF             
0x0000044C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000044E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000450      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000452      1  FF                                                   TERMINATOR_FF             
0x00000453      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000455      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000457      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000459      1  FF                                                   TERMINATOR_FF             
0x0000045A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000045C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000045E      1  FF                                                   TERMINATOR_FF             
0x0000045F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000461      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000463      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000465      1  FF                                                   TERMINATOR_FF             
0x00000466      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000468      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000046A      2  F246                                                 IMM8_F2                   u8=70, s8=70
0x0000046C      1  FF                                                   TERMINATOR_FF             
0x0000046D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000046F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000471      1  FF                                                   TERMINATOR_FF             
0x00000472      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000474      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000476      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000478      1  FF                                                   TERMINATOR_FF             
0x00000479      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000047B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000047D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000047F      1  FF                                                   TERMINATOR_FF             
0x00000480      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000482      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000484      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000486      1  FF                                                   TERMINATOR_FF             
0x00000487      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000489      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000048B      1  FF                                                   TERMINATOR_FF             
0x0000048C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000048E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000490      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000492      1  FF                                                   TERMINATOR_FF             
0x00000493      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000495      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000497      3  F31D93                                               IMM16_F3                  u16_be=7571, u16_le=37661
0x0000049A      1  FF                                                   TERMINATOR_FF             
0x0000049B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000049D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000049F     54  8034825282C282CC8C9A95A882CC8AD48141926E8FE38250...  LEN8_STRING_CP932         length=52, text="３つの建物の間、地上１０数階のフロアに公園があった。"
0x000004D5      1  FF                                                   TERMINATOR_FF             
0x000004D6      1  FF                                                   TERMINATOR_FF             
0x000004D7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004D9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004DB      1  FF                                                   TERMINATOR_FF             
0x000004DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004E0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000004E2      1  FF                                                   TERMINATOR_FF             
0x000004E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004E5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000004E7      3  F31D94                                               IMM16_F3                  u16_be=7572, u16_le=37917
0x000004EA      1  FF                                                   TERMINATOR_FF             
0x000004EB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004ED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004EF     88  8056924E82A982AA8141919D927A82CC9372928682C58358...  LEN8_STRING_CP932         length=86, text="誰かが、増築の途中でスペースの余分に気づいて、そこを市民に解放する公園にしたのだろう。"
0x00000547      1  FF                                                   TERMINATOR_FF             
0x00000548      1  FF                                                   TERMINATOR_FF             
0x00000549      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000054B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000054D      1  FF                                                   TERMINATOR_FF             
0x0000054E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000550      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000552      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000554      1  FF                                                   TERMINATOR_FF             
0x00000555      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000557      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000559      3  F31D95                                               IMM16_F3                  u16_be=7573, u16_le=38173
0x0000055C      1  FF                                                   TERMINATOR_FF             
0x0000055D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000055F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000561     56  80368ACF8CF582CC96BC8F8A82BE82C682A282ED82EA82C4...  LEN8_STRING_CP932         length=54, text="観光の名所だといわれているけれど、来たことはなかった。"
0x00000599      1  FF                                                   TERMINATOR_FF             
0x0000059A      1  FF                                                   TERMINATOR_FF             
0x0000059B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000059D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000059F      1  FF                                                   TERMINATOR_FF             
0x000005A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005A4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000005A6      1  FF                                                   TERMINATOR_FF             
0x000005A7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005A9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000005AB      3  F31D96                                               IMM16_F3                  u16_be=7574, u16_le=38429
0x000005AE      1  FF                                                   TERMINATOR_FF             
0x000005AF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005B1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005B3     60  803A82C582E0816381638BF3928682C9958282AD92EB8980...  LEN8_STRING_CP932         length=58, text="でも……空中に浮く庭園。\n優雅というか、貴族趣味というか。"
0x000005EF      1  FF                                                   TERMINATOR_FF             
0x000005F0      1  FF                                                   TERMINATOR_FF             
0x000005F1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005F3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005F5      1  FF                                                   TERMINATOR_FF             
0x000005F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005F8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005FA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000005FC      1  FF                                                   TERMINATOR_FF             
0x000005FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005FF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000601      3  F31D97                                               IMM16_F3                  u16_be=7575, u16_le=38685
0x00000604      1  FF                                                   TERMINATOR_FF             
0x00000605      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000607      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000609     36  802282DA82AD82CD8BF3928682C98D4C82AA82E98CF68980...  LEN8_STRING_CP932         length=34, text="ぼくは空中に広がる公園を見渡した。"
0x0000062D      1  FF                                                   TERMINATOR_FF             
0x0000062E      1  FF                                                   TERMINATOR_FF             
0x0000062F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000631      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000633      1  FF                                                   TERMINATOR_FF             
0x00000634      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000636      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000638      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000063A      1  FF                                                   TERMINATOR_FF             
0x0000063B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000063D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000063F      3  F31D98                                               IMM16_F3                  u16_be=7576, u16_le=38941
0x00000642      1  FF                                                   TERMINATOR_FF             
0x00000643      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000645      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000647     92  805A8BF382AA8D4C82A982C182BD81425C6E937389EF82CC...  LEN8_STRING_CP932         length=90, text="空が広かった。\n都会の騒音も、この高さまでは届かない。\n静かな……のんびりした場所だった。"
0x000006A3      1  FF                                                   TERMINATOR_FF             
0x000006A4      1  FF                                                   TERMINATOR_FF             
0x000006A5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006A7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006A9      1  FF                                                   TERMINATOR_FF             
0x000006AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006AE      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000006B0      1  FF                                                   TERMINATOR_FF             
0x000006B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006B3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006B5      3  F301D7                                               IMM16_F3                  u16_be=471, u16_le=55041
0x000006B8      1  FF                                                   TERMINATOR_FF             
0x000006B9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006BB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006BD      1  FF                                                   TERMINATOR_FF             
0x000006BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006C2      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000006C4      1  FF                                                   TERMINATOR_FF             
0x000006C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006C9      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000006CB      1  FF                                                   TERMINATOR_FF             
0x000006CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006CE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000006D0      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000006D2      1  FF                                                   TERMINATOR_FF             
0x000006D3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006D5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006D7      1  FF                                                   TERMINATOR_FF             
0x000006D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006DC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000006DE      1  FF                                                   TERMINATOR_FF             
0x000006DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006E1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006E3      3  F31D99                                               IMM16_F3                  u16_be=7577, u16_le=39197
0x000006E6      1  FF                                                   TERMINATOR_FF             
0x000006E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006E9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006EB     56  803682BB82B582C4814183788393836082C58ADB93A182B3...  LEN8_STRING_CP932         length=54, text="そして、ベンチで丸藤さんがくつろいでいるのを見つける。"
0x00000723      1  FF                                                   TERMINATOR_FF             
0x00000724      1  FF                                                   TERMINATOR_FF             
0x00000725      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000727      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000729      1  FF                                                   TERMINATOR_FF             
0x0000072A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000072C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000072E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000730      1  FF                                                   TERMINATOR_FF             
0x00000731      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000733      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000735      3  F31D9A                                               IMM16_F3                  u16_be=7578, u16_le=39453
0x00000738      1  FF                                                   TERMINATOR_FF             
0x00000739      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000073B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000073D    102  80648EE882C982CD8141837A834283628376834E838A815B...  LEN8_STRING_CP932         length=100, text="手には、ホィップクリームとイチゴのあま～い\nサンドウィッチ。\nホントに……あまいもの好きなんだから。"
0x000007A3      1  FF                                                   TERMINATOR_FF             
0x000007A4      1  FF                                                   TERMINATOR_FF             
0x000007A5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007A7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007A9      1  FF                                                   TERMINATOR_FF             
0x000007AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007AE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000007B0      1  FF                                                   TERMINATOR_FF             
0x000007B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007B3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000007B5      3  F31D9B                                               IMM16_F3                  u16_be=7579, u16_le=39709
0x000007B8      1  FF                                                   TERMINATOR_FF             
0x000007B9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007BB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007BD     38  80248ADB93A182B382F182CD814182DA82AD82C98B4382C3...  LEN8_STRING_CP932         length=36, text="丸藤さんは、ぼくに気づいて微笑んだ。"
0x000007E3      1  FF                                                   TERMINATOR_FF             
0x000007E4      1  FF                                                   TERMINATOR_FF             
0x000007E5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007E7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007E9      1  FF                                                   TERMINATOR_FF             
0x000007EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007EE      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000007F0      1  FF                                                   TERMINATOR_FF             
0x000007F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007F3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000007F5      3  F31D9C                                               IMM16_F3                  u16_be=7580, u16_le=39965
0x000007F8      1  FF                                                   TERMINATOR_FF             
0x000007F9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007FB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000007FD      9  8007495A3038313730                                   LEN8_STRING_CP932         length=7, text="IZ08170"
0x00000806      1  FF                                                   TERMINATOR_FF             
0x00000807      1  FF                                                   TERMINATOR_FF             
0x00000808      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000080A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000080C     28  801A817582A081608141975682D182C9978882BD82CC815B...  LEN8_STRING_CP932         length=26, text="「あ～、遊びに来たのー？」"
0x00000828      1  FF                                                   TERMINATOR_FF             
0x00000829      1  FF                                                   TERMINATOR_FF             
0x0000082A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000082C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000082E      1  FF                                                   TERMINATOR_FF             
0x0000082F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000831      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000833      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000835      1  FF                                                   TERMINATOR_FF             
0x00000836      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000838      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000083A      2  F246                                                 IMM8_F2                   u8=70, s8=70
0x0000083C      1  FF                                                   TERMINATOR_FF             
0x0000083D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000083F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000841      1  FF                                                   TERMINATOR_FF             
0x00000842      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000844      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000846      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000848      1  FF                                                   TERMINATOR_FF             
0x00000849      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000084B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000084D      3  F300BB                                               IMM16_F3                  u16_be=187, u16_le=47872
0x00000850      1  FF                                                   TERMINATOR_FF             
0x00000851      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000853      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000855      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000857      1  FF                                                   TERMINATOR_FF             
0x00000858      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000085A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000085C      1  FF                                                   TERMINATOR_FF             
0x0000085D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000085F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000861      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000863      1  FF                                                   TERMINATOR_FF             
0x00000864      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000866      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000868      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000086A      1  FF                                                   TERMINATOR_FF             
0x0000086B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000086D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000086F      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000871      1  FF                                                   TERMINATOR_FF             
0x00000872      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000874      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000876      1  FF                                                   TERMINATOR_FF             
0x00000877      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000879      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000087B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000087D      1  FF                                                   TERMINATOR_FF             
0x0000087E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000880      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000882      3  F31D9D                                               IMM16_F3                  u16_be=7581, u16_le=40221
0x00000885      1  FF                                                   TERMINATOR_FF             
0x00000886      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000888      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000088A     46  802C82A4816082F181425C6E8BCE96B18E9E8AD4928682C6...  LEN8_STRING_CP932         length=44, text="う～ん。\n勤務時間中とは思えない呑気な発言。"
0x000008B8      1  FF                                                   TERMINATOR_FF             
0x000008B9      1  FF                                                   TERMINATOR_FF             
0x000008BA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008BC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008BE      1  FF                                                   TERMINATOR_FF             
0x000008BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008C3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000008C5      1  FF                                                   TERMINATOR_FF             
0x000008C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008C8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000008CA      3  F31D9E                                               IMM16_F3                  u16_be=7582, u16_le=40477
0x000008CD      1  FF                                                   TERMINATOR_FF             
0x000008CE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008D0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008D2     46  802C817582A682C182C681418ADB93A182B382F1814182C7...  LEN8_STRING_CP932         length=44, text="「えっと、丸藤さん、どうしてるかなと思って」"
0x00000900      1  FF                                                   TERMINATOR_FF             
0x00000901      1  FF                                                   TERMINATOR_FF             
0x00000902      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000904      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000906      1  FF                                                   TERMINATOR_FF             
0x00000907      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000909      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000090B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000090D      1  FF                                                   TERMINATOR_FF             
0x0000090E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000910      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000912      3  F31D9F                                               IMM16_F3                  u16_be=7583, u16_le=40733
0x00000915      1  FF                                                   TERMINATOR_FF             
0x00000916      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000918      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000091A      9  8007495A3038313830                                   LEN8_STRING_CP932         length=7, text="IZ08180"
0x00000923      1  FF                                                   TERMINATOR_FF             
0x00000924      1  FF                                                   TERMINATOR_FF             
0x00000925      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000927      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000929     72  8046817582A6816082D682D68160814182BF82E582C182C6...  LEN8_STRING_CP932         length=70, text="「え～へへ～、ちょっと休憩～。お仕事ばっか\nだと、肩こっちゃうしねー」"
0x00000971      1  FF                                                   TERMINATOR_FF             
0x00000972      1  FF                                                   TERMINATOR_FF             
0x00000973      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000975      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000977      1  FF                                                   TERMINATOR_FF             
0x00000978      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000097A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000097C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000097E      1  FF                                                   TERMINATOR_FF             
0x0000097F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000981      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000983      3  F31DA0                                               IMM16_F3                  u16_be=7584, u16_le=40989
0x00000986      1  FF                                                   TERMINATOR_FF             
0x00000987      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000989      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000098B     30  801C817582A482ED814196F08F8A82C182C48A7982C582B7...  LEN8_STRING_CP932         length=28, text="「うわ、役所って楽ですねえ」"
0x000009A9      1  FF                                                   TERMINATOR_FF             
0x000009AA      1  FF                                                   TERMINATOR_FF             
0x000009AB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009AD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009AF      1  FF                                                   TERMINATOR_FF             
0x000009B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009B4      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000009B6      1  FF                                                   TERMINATOR_FF             
0x000009B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009B9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009BB      3  F31DA1                                               IMM16_F3                  u16_be=7585, u16_le=41245
0x000009BE      1  FF                                                   TERMINATOR_FF             
0x000009BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009C1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000009C3      9  8007495A3038313930                                   LEN8_STRING_CP932         length=7, text="IZ08190"
0x000009CC      1  FF                                                   TERMINATOR_FF             
0x000009CD      1  FF                                                   TERMINATOR_FF             
0x000009CE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009D0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009D2     54  8034817582A682A5816081418A7982C882B182C682C882A2...  LEN8_STRING_CP932         length=52, text="「えぇ～、楽なことないよー。ターイヘンなんだからあ」"
0x00000A08      1  FF                                                   TERMINATOR_FF             
0x00000A09      1  FF                                                   TERMINATOR_FF             
0x00000A0A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A0C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A0E      1  FF                                                   TERMINATOR_FF             
0x00000A0F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A11      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A13      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000A15      1  FF                                                   TERMINATOR_FF             
0x00000A16      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A18      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A1A      3  F31DA2                                               IMM16_F3                  u16_be=7586, u16_le=41501
0x00000A1D      1  FF                                                   TERMINATOR_FF             
0x00000A1E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A20      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A22     46  802C81758ADB93A182B382F18163816395CF82C882B182C6...  LEN8_STRING_CP932         length=44, text="「丸藤さん……変なこととか……ありません？」"
0x00000A50      1  FF                                                   TERMINATOR_FF             
0x00000A51      1  FF                                                   TERMINATOR_FF             
0x00000A52      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A54      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A56      1  FF                                                   TERMINATOR_FF             
0x00000A57      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A59      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A5B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000A5D      1  FF                                                   TERMINATOR_FF             
0x00000A5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A60      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A62      3  F31DA3                                               IMM16_F3                  u16_be=7587, u16_le=41757
0x00000A65      1  FF                                                   TERMINATOR_FF             
0x00000A66      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A68      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A6A      9  8007495A3038323031                                   LEN8_STRING_CP932         length=7, text="IZ08201"
0x00000A73      1  FF                                                   TERMINATOR_FF             
0x00000A74      1  FF                                                   TERMINATOR_FF             
0x00000A75      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A77      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A79     94  805C817582F18160814182B182CC82B282EB82CD8377815B...  LEN8_STRING_CP932         length=92, text="「ん～、このごろはヘーキかなあ。\nあ～、昨日ねえ、帰りの電車でヘンな酔っぱらいオヤジいたー」"
0x00000AD7      1  FF                                                   TERMINATOR_FF             
0x00000AD8      1  FF                                                   TERMINATOR_FF             
0x00000AD9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000ADB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000ADD      1  FF                                                   TERMINATOR_FF             
0x00000ADE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AE0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AE2      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000AE4      1  FF                                                   TERMINATOR_FF             
0x00000AE5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AE7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000AE9      3  F31DA4                                               IMM16_F3                  u16_be=7588, u16_le=42013
0x00000AEC      1  FF                                                   TERMINATOR_FF             
0x00000AED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AEF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000AF1      9  8007495A3038323032                                   LEN8_STRING_CP932         length=7, text="IZ08202"
0x00000AFA      1  FF                                                   TERMINATOR_FF             
0x00000AFB      1  FF                                                   TERMINATOR_FF             
0x00000AFC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AFE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B00    106  8068817582A8914F82E782CD82C882F182C982E0926D82E7...  LEN8_STRING_CP932         length=104, text="「お前らはなんにも知らないんだーって、\nおセッキョされそーになったからー、車輛\nかえたー。それくら～い」"
0x00000B6A      1  FF                                                   TERMINATOR_FF             
0x00000B6B      1  FF                                                   TERMINATOR_FF             
0x00000B6C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B6E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B70      1  FF                                                   TERMINATOR_FF             
0x00000B71      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B73      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B75      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B77      1  FF                                                   TERMINATOR_FF             
0x00000B78      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B7A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B7C      3  F31DA5                                               IMM16_F3                  u16_be=7589, u16_le=42269
0x00000B7F      1  FF                                                   TERMINATOR_FF             
0x00000B80      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B82      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B84     72  804682A482F18163816382BB82EA82AD82E782A282C882E7...  LEN8_STRING_CP932         length=70, text="うん……それくらいなら、丸藤さん的には何も\nおきていないといえるかな。"
0x00000BCC      1  FF                                                   TERMINATOR_FF             
0x00000BCD      1  FF                                                   TERMINATOR_FF             
0x00000BCE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000BD0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000BD2      1  FF                                                   TERMINATOR_FF             
0x00000BD3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BD5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BD7      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000BD9      1  FF                                                   TERMINATOR_FF             
0x00000BDA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BDC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000BDE      3  F31DA6                                               IMM16_F3                  u16_be=7590, u16_le=42525
0x00000BE1      1  FF                                                   TERMINATOR_FF             
0x00000BE2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BE4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000BE6      9  8007495A3038323130                                   LEN8_STRING_CP932         length=7, text="IZ08210"
0x00000BEF      1  FF                                                   TERMINATOR_FF             
0x00000BF0      1  FF                                                   TERMINATOR_FF             
0x00000BF1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BF3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BF5    136  8086817582A682A68160814182C7815B82B582BD82CC815B...  LEN8_STRING_CP932         length=134, text="「ええ～、どーしたのー？ な～に～、心配して\nくれたの～？ あ～、だって学校の時間でしょー。\nえ～、心配でぬけだして来てくれたとか～？」"
0x00000C7D      1  FF                                                   TERMINATOR_FF             
0x00000C7E      1  FF                                                   TERMINATOR_FF             
0x00000C7F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C81      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C83      1  FF                                                   TERMINATOR_FF             
0x00000C84      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C86      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C88      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000C8A      1  FF                                                   TERMINATOR_FF             
0x00000C8B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C8D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C8F      3  F31DA7                                               IMM16_F3                  u16_be=7591, u16_le=42781
0x00000C92      1  FF                                                   TERMINATOR_FF             
0x00000C93      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C95      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C97     80  804E8ADB93A182B382F182CD81418FAD82B582A282BD82B8...  LEN8_STRING_CP932         length=78, text="丸藤さんは、少しいたずらっぽく微笑んでいた。にやにやと、ぼくの目をのぞきこむ。"
0x00000CE7      1  FF                                                   TERMINATOR_FF             
0x00000CE8      1  FF                                                   TERMINATOR_FF             
0x00000CE9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CEB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CED      1  FF                                                   TERMINATOR_FF             
0x00000CEE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CF0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CF2      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000CF4      1  FF                                                   TERMINATOR_FF             
0x00000CF5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CF7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000CF9      3  F31DA8                                               IMM16_F3                  u16_be=7592, u16_le=43037
0x00000CFC      1  FF                                                   TERMINATOR_FF             
0x00000CFD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CFF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000D01      8  8006593030393130                                     LEN8_STRING_CP932         length=6, text="Y00910"
0x00000D09      1  FF                                                   TERMINATOR_FF             
0x00000D0A      1  FF                                                   TERMINATOR_FF             
0x00000D0B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D0D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D0F     44  802A817582ED8141837D838B82CC82E282C2814282B182C7...  LEN8_STRING_CP932         length=42, text="「わ、マルのやつ。こどもユーワクしてるよ」"
0x00000D3B      1  FF                                                   TERMINATOR_FF             
0x00000D3C      1  FF                                                   TERMINATOR_FF             
0x00000D3D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D3F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D41      1  FF                                                   TERMINATOR_FF             
0x00000D42      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D44      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D46      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000D48      1  FF                                                   TERMINATOR_FF             
0x00000D49      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D4B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D4D      3  F31DA9                                               IMM16_F3                  u16_be=7593, u16_le=43293
0x00000D50      1  FF                                                   TERMINATOR_FF             
0x00000D51      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D53      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000D55      9  80074B4F3031303130                                   LEN8_STRING_CP932         length=7, text="KO01010"
0x00000D5E      1  FF                                                   TERMINATOR_FF             
0x00000D5F      1  FF                                                   TERMINATOR_FF             
0x00000D60      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D62      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D64     58  8038817582DC82A0814197448B5082BF82E182F1814282BB...  LEN8_STRING_CP932         length=56, text="「まあ、優輝ちゃん。そんなこといっては\n失礼ですのよー」"
0x00000D9E      1  FF                                                   TERMINATOR_FF             
0x00000D9F      1  FF                                                   TERMINATOR_FF             
0x00000DA0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000DA2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000DA4      1  FF                                                   TERMINATOR_FF             
0x00000DA5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DA7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DA9      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000DAB      1  FF                                                   TERMINATOR_FF             
0x00000DAC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DAE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000DB0      3  F31DAA                                               IMM16_F3                  u16_be=7594, u16_le=43549
0x00000DB3      1  FF                                                   TERMINATOR_FF             
0x00000DB4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DB6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000DB8      8  8006593030393230                                     LEN8_STRING_CP932         length=6, text="Y00920"
0x00000DC0      1  FF                                                   TERMINATOR_FF             
0x00000DC1      1  FF                                                   TERMINATOR_FF             
0x00000DC2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DC4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DC6     98  8060817582A682C182C682CB814182A082EA82BE82C68E76...  LEN8_STRING_CP932         length=96, text="「えっとね、あれだと思うね、年下だと\nイニシアチブとりやすいんだと思うな。\n悪いオンナだよねー」"
0x00000E28      1  FF                                                   TERMINATOR_FF             
0x00000E29      1  FF                                                   TERMINATOR_FF             
0x00000E2A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E2C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E2E      1  FF                                                   TERMINATOR_FF             
0x00000E2F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E31      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E33      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000E35      1  FF                                                   TERMINATOR_FF             
0x00000E36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E38      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E3A      3  F301D8                                               IMM16_F3                  u16_be=472, u16_le=55297
0x00000E3D      1  FF                                                   TERMINATOR_FF             
0x00000E3E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E40      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E42      1  FF                                                   TERMINATOR_FF             
0x00000E43      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E45      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E47      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000E49      1  FF                                                   TERMINATOR_FF             
0x00000E4A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E4C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E4E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E50      1  FF                                                   TERMINATOR_FF             
0x00000E51      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E53      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000E55      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000E57      1  FF                                                   TERMINATOR_FF             
0x00000E58      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E5A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E5C      1  FF                                                   TERMINATOR_FF             
0x00000E5D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E5F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E61      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000E63      1  FF                                                   TERMINATOR_FF             
0x00000E64      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E66      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E68      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000E6A      1  FF                                                   TERMINATOR_FF             
0x00000E6B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E6D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000E6F      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000E71      1  FF                                                   TERMINATOR_FF             
0x00000E72      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E74      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E76      1  FF                                                   TERMINATOR_FF             
0x00000E77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E7B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000E7D      1  FF                                                   TERMINATOR_FF             
0x00000E7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E80      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E82      3  F31DAB                                               IMM16_F3                  u16_be=7595, u16_le=43805
0x00000E85      1  FF                                                   TERMINATOR_FF             
0x00000E86      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E88      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000E8A      9  8007495A3038323230                                   LEN8_STRING_CP932         length=7, text="IZ08220"
0x00000E93      1  FF                                                   TERMINATOR_FF             
0x00000E94      1  FF                                                   TERMINATOR_FF             
0x00000E95      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E97      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E99     30  801C817582E282BE814182E0815B814282CC82BC82A282C4...  LEN8_STRING_CP932         length=28, text="「やだ、もー。のぞいてるー」"
0x00000EB7      1  FF                                                   TERMINATOR_FF             
0x00000EB8      1  FF                                                   TERMINATOR_FF             
0x00000EB9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000EBB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000EBD      1  FF                                                   TERMINATOR_FF             
0x00000EBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EC0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EC2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000EC4      1  FF                                                   TERMINATOR_FF             
0x00000EC5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EC7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000EC9      3  F31DAC                                               IMM16_F3                  u16_be=7596, u16_le=44061
0x00000ECC      1  FF                                                   TERMINATOR_FF             
0x00000ECD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000ECF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000ED1     54  80348ADB93A182B382F182CD81418FAD82B5937B82C182BD...  LEN8_STRING_CP932         length=52, text="丸藤さんは、少し怒ったような表情をふたりに\n向ける。"
0x00000F07      1  FF                                                   TERMINATOR_FF             
0x00000F08      1  FF                                                   TERMINATOR_FF             
0x00000F09      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F0B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F0D      1  FF                                                   TERMINATOR_FF             
0x00000F0E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F10      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F12      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000F14      1  FF                                                   TERMINATOR_FF             
0x00000F15      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F17      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F19      3  F31DAD                                               IMM16_F3                  u16_be=7597, u16_le=44317
0x00000F1C      1  FF                                                   TERMINATOR_FF             
0x00000F1D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F1F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000F21      8  8006593030393330                                     LEN8_STRING_CP932         length=6, text="Y00930"
0x00000F29      1  FF                                                   TERMINATOR_FF             
0x00000F2A      1  FF                                                   TERMINATOR_FF             
0x00000F2B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F2D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F2F     40  8026817582ED81418B4382C382A982EA82BD81428C4F8E71...  LEN8_STRING_CP932         length=38, text="「わ、気づかれた。薫子さん、逃げよう」"
0x00000F57      1  FF                                                   TERMINATOR_FF             
0x00000F58      1  FF                                                   TERMINATOR_FF             
0x00000F59      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F5B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F5D      1  FF                                                   TERMINATOR_FF             
0x00000F5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F60      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F62      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000F64      1  FF                                                   TERMINATOR_FF             
0x00000F65      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F67      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F69      2  F246                                                 IMM8_F2                   u8=70, s8=70
0x00000F6B      1  FF                                                   TERMINATOR_FF             
0x00000F6C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F6E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F70      1  FF                                                   TERMINATOR_FF             
0x00000F71      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F73      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F75      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000F77      1  FF                                                   TERMINATOR_FF             
0x00000F78      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F7A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F7C      3  F30084                                               IMM16_F3                  u16_be=132, u16_le=33792
0x00000F7F      1  FF                                                   TERMINATOR_FF             
0x00000F80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F82      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000F84      2  F202                                                 IMM8_F2                   u8=2, s8=2
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
0x00000FAA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000FAC      1  FF                                                   TERMINATOR_FF             
0x00000FAD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FAF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000FB1      3  F31DAE                                               IMM16_F3                  u16_be=7598, u16_le=44573
0x00000FB4      1  FF                                                   TERMINATOR_FF             
0x00000FB5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FB7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000FB9      9  8007495A3038323330                                   LEN8_STRING_CP932         length=7, text="IZ08230"
0x00000FC2      1  FF                                                   TERMINATOR_FF             
0x00000FC3      1  FF                                                   TERMINATOR_FF             
0x00000FC4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FC6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FC8    104  8066817582B282DF82F182CB815B814182B982C182A982AD...  LEN8_STRING_CP932         length=102, text="「ごめんねー、せっかく来てくれたのにー。\n仕事もどるー。後でどんなウワサされちゃうか\nわかんないから」"
0x00001030      1  FF                                                   TERMINATOR_FF             
0x00001031      1  FF                                                   TERMINATOR_FF             
0x00001032      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001034      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001036      1  FF                                                   TERMINATOR_FF             
0x00001037      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001039      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000103B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000103D      1  FF                                                   TERMINATOR_FF             
0x0000103E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001040      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001042      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001044      1  FF                                                   TERMINATOR_FF             
0x00001045      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001047      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001049      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000104B      1  FF                                                   TERMINATOR_FF             
0x0000104C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000104E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001050      1  FF                                                   TERMINATOR_FF             
0x00001051      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001053      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001055      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001057      1  FF                                                   TERMINATOR_FF             
0x00001058      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000105A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000105C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000105E      1  FF                                                   TERMINATOR_FF             
0x0000105F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001061      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001063      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001065      1  FF                                                   TERMINATOR_FF             
0x00001066      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001068      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000106A      1  FF                                                   TERMINATOR_FF             
0x0000106B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000106D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000106F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001071      1  FF                                                   TERMINATOR_FF             
0x00001072      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001074      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001076      3  F31DAF                                               IMM16_F3                  u16_be=7599, u16_le=44829
0x00001079      1  FF                                                   TERMINATOR_FF             
0x0000107A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000107C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000107E     80  804E8ADB93A182B382F182CD814191E582AB82C882A8904B...  LEN8_STRING_CP932         length=78, text="丸藤さんは、大きなお尻をふりながら、\nいそぎ足で職員用エレベーターへ向かった。"
0x000010CE      1  FF                                                   TERMINATOR_FF             
0x000010CF      1  FF                                                   TERMINATOR_FF             
0x000010D0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010D2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010D4      1  FF                                                   TERMINATOR_FF             
0x000010D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010D9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000010DB      1  FF                                                   TERMINATOR_FF             
0x000010DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010DE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000010E0      3  F31DB0                                               IMM16_F3                  u16_be=7600, u16_le=45085
0x000010E3      1  FF                                                   TERMINATOR_FF             
0x000010E4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010E6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010E8     86  805482C682E882A082A682B881418ADB93A182B382F182AA...  LEN8_STRING_CP932         length=84, text="とりあえず、丸藤さんが無事なのは確認できたし。もう少し、ここでのんびりしていこうか。"
0x0000113E      1  FF                                                   TERMINATOR_FF             
0x0000113F      1  FF                                                   TERMINATOR_FF             
0x00001140      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001142      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001144      1  FF                                                   TERMINATOR_FF             
0x00001145      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001147      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001149      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x0000114B      1  FF                                                   TERMINATOR_FF             
0x0000114C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000114E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001150      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001152      1  FF                                                   TERMINATOR_FF             
0x00001153      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001155      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001157      1  FF                                                   TERMINATOR_FF             
0x00001158      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000115A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000115C      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000115E      1  FF                                                   TERMINATOR_FF             
0x0000115F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001161      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001163      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00001165      1  FF                                                   TERMINATOR_FF             
0x00001166      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001168      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000116A      1  FF                                                   TERMINATOR_FF             
0x0000116B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000116D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000116F      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001171      1  FF                                                   TERMINATOR_FF             
0x00001172      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001174      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001176      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001178      1  FF                                                   TERMINATOR_FF             
0x00001179      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000117B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000117D      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000117F      1  FF                                                   TERMINATOR_FF             
0x00001180      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001182      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001184      1  FF                                                   TERMINATOR_FF             
0x00001185      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001187      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001189      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x0000118B      1  FF                                                   TERMINATOR_FF             
0x0000118C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000118E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001190      1  FF                                                   TERMINATOR_FF             
0x00001191      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001193      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001195      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00001197      1  FF                                                   TERMINATOR_FF             
0x00001198      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000119A      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x0000119C     14  800C50533249303133612E62696E                         LEN8_STRING_CP932         length=12, text="PS2I013a.bin"
0x000011AA      1  FF                                                   TERMINATOR_FF             
0x000011AB      1  FF                                                   TERMINATOR_FF             
0x000011AC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000011AE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000011B0      1  FF                                                   TERMINATOR_FF             
0x000011B1      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000011B3      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000011B5      1  FF                                                   TERMINATOR_FF             
0x000011B6      2  0011                                                 WORD_00XX                 u16_be=17, low_byte=17
0x000011B8      1  C0                                                   OPAQUE_RAW_BYTES          bytes=C0
0x000011B9      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x000011BB      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x000011BD      2  0011                                                 WORD_00XX                 u16_be=17, low_byte=17
0x000011BF      1  B1                                                   OPAQUE_RAW_BYTES          bytes=B1
0x000011C0      1  FF                                                   TERMINATOR_FF             
0x000011C1      1  FF                                                   TERMINATOR_FF             
