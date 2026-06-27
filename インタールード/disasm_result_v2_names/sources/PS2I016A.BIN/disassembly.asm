; Full conservative disassembly for PS2I016A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      2  F228                                                 IMM8_F2                   u8=40, s8=40
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
0x0000001E      3  F300D8                                               IMM16_F3                  u16_be=216, u16_le=55296
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
0x00000039      2  F210                                                 IMM8_F2                   u8=16, s8=16
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
0x00000066      3  F31ED2                                               IMM16_F3                  u16_be=7890, u16_le=53790
0x00000069      1  FF                                                   TERMINATOR_FF             
0x0000006A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000006C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000006E     34  80208ADB93A182B382F182CD814182C982B182C982B1945C...  LEN8_STRING_CP932         length=32, text="丸藤さんは、にこにこ能天気状態。"
0x00000090      1  FF                                                   TERMINATOR_FF             
0x00000091      1  FF                                                   TERMINATOR_FF             
0x00000092      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000094      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000096      1  FF                                                   TERMINATOR_FF             
0x00000097      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000099      2  0047                                                 WORD_00XX                 u16_be=71, low_byte=71
0x0000009B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000009D      1  FF                                                   TERMINATOR_FF             
0x0000009E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000A2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000000A4      1  FF                                                   TERMINATOR_FF             
0x000000A5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000A7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000000A9      3  F31ED3                                               IMM16_F3                  u16_be=7891, u16_le=54046
0x000000AC      1  FF                                                   TERMINATOR_FF             
0x000000AD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000AF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000B1     36  8022817582C882C982A982A282A282B182C682C582E082A0...  LEN8_STRING_CP932         length=34, text="「なにかいいことでもありました？」"
0x000000D5      1  FF                                                   TERMINATOR_FF             
0x000000D6      1  FF                                                   TERMINATOR_FF             
0x000000D7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000D9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000DB      1  FF                                                   TERMINATOR_FF             
0x000000DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E0      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000000E2      1  FF                                                   TERMINATOR_FF             
0x000000E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000000E7      3  F31ED4                                               IMM16_F3                  u16_be=7892, u16_le=54302
0x000000EA      1  FF                                                   TERMINATOR_FF             
0x000000EB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000ED      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000000EF      9  8007495A3130313130                                   LEN8_STRING_CP932         length=7, text="IZ10110"
0x000000F8      1  FF                                                   TERMINATOR_FF             
0x000000F9      1  FF                                                   TERMINATOR_FF             
0x000000FA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000FC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000FE     32  801E817582F18160814195CA82C98160814282C7815B82B5...  LEN8_STRING_CP932         length=30, text="「ん～、別に～。どーしてえ？」"
0x0000011E      1  FF                                                   TERMINATOR_FF             
0x0000011F      1  FF                                                   TERMINATOR_FF             
0x00000120      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000122      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000124      1  FF                                                   TERMINATOR_FF             
0x00000125      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000127      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000129      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000012B      1  FF                                                   TERMINATOR_FF             
0x0000012C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000012E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000130      3  F31ED5                                               IMM16_F3                  u16_be=7893, u16_le=54558
0x00000133      1  FF                                                   TERMINATOR_FF             
0x00000134      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000136      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000138     40  8026817582A282E2816381638B408C9982E682B382BB815B...  LEN8_STRING_CP932         length=38, text="「いや……機嫌よさそーだなーと思って」"
0x00000160      1  FF                                                   TERMINATOR_FF             
0x00000161      1  FF                                                   TERMINATOR_FF             
0x00000162      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000164      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000166      1  FF                                                   TERMINATOR_FF             
0x00000167      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000169      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000016B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000016D      1  FF                                                   TERMINATOR_FF             
0x0000016E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000170      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000172      3  F300DF                                               IMM16_F3                  u16_be=223, u16_le=57088
0x00000175      1  FF                                                   TERMINATOR_FF             
0x00000176      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000178      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000017A      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000017C      1  FF                                                   TERMINATOR_FF             
0x0000017D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000017F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000181      1  FF                                                   TERMINATOR_FF             
0x00000182      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000184      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000186      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000188      1  FF                                                   TERMINATOR_FF             
0x00000189      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000018B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000018D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000018F      1  FF                                                   TERMINATOR_FF             
0x00000190      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000192      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000194      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000196      1  FF                                                   TERMINATOR_FF             
0x00000197      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000199      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000019B      1  FF                                                   TERMINATOR_FF             
0x0000019C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000019E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001A0      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000001A2      1  FF                                                   TERMINATOR_FF             
0x000001A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001A5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001A7      3  F31ED6                                               IMM16_F3                  u16_be=7894, u16_le=54814
0x000001AA      1  FF                                                   TERMINATOR_FF             
0x000001AB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001AD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000001AF      9  8007495A3130313230                                   LEN8_STRING_CP932         length=7, text="IZ10120"
0x000001B8      1  FF                                                   TERMINATOR_FF             
0x000001B9      1  FF                                                   TERMINATOR_FF             
0x000001BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001BC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001BE    126  807C817582A682A68160814182BE82C182C4815B83418374...  LEN8_STRING_CP932         length=124, text="「ええ～、だってーアフターファイブだも～ん。機嫌もよくなるよ～。むつかしー顔してんの、\n仕事中だけでじゅーぶんじゃな～い？」"
0x0000023C      1  FF                                                   TERMINATOR_FF             
0x0000023D      1  FF                                                   TERMINATOR_FF             
0x0000023E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000240      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000242      1  FF                                                   TERMINATOR_FF             
0x00000243      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000245      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000247      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000249      1  FF                                                   TERMINATOR_FF             
0x0000024A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000024C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000024E      3  F31ED7                                               IMM16_F3                  u16_be=7895, u16_le=55070
0x00000251      1  FF                                                   TERMINATOR_FF             
0x00000252      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000254      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000256     40  8026817582A081418E648E96928682CD82E282C182CF82E8...  LEN8_STRING_CP932         length=38, text="「あ、仕事中はやっぱり真剣なんですね」"
0x0000027E      1  FF                                                   TERMINATOR_FF             
0x0000027F      1  FF                                                   TERMINATOR_FF             
0x00000280      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000282      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000284      1  FF                                                   TERMINATOR_FF             
0x00000285      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000287      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000289      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000028B      1  FF                                                   TERMINATOR_FF             
0x0000028C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000028E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000290      3  F3020C                                               IMM16_F3                  u16_be=524, u16_le=3074
0x00000293      1  FF                                                   TERMINATOR_FF             
0x00000294      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000296      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000298      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000029A      1  FF                                                   TERMINATOR_FF             
0x0000029B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000029D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000029F      1  FF                                                   TERMINATOR_FF             
0x000002A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A4      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000002A6      1  FF                                                   TERMINATOR_FF             
0x000002A7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002AB      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002AD      1  FF                                                   TERMINATOR_FF             
0x000002AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002B2      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000002B4      1  FF                                                   TERMINATOR_FF             
0x000002B5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002B7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002B9      1  FF                                                   TERMINATOR_FF             
0x000002BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002BE      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000002C0      1  FF                                                   TERMINATOR_FF             
0x000002C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002C3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000002C5      3  F31ED8                                               IMM16_F3                  u16_be=7896, u16_le=55326
0x000002C8      1  FF                                                   TERMINATOR_FF             
0x000002C9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002CB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002CD      9  8007495A3130313331                                   LEN8_STRING_CP932         length=7, text="IZ10131"
0x000002D6      1  FF                                                   TERMINATOR_FF             
0x000002D7      1  FF                                                   TERMINATOR_FF             
0x000002D8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002DA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002DC     80  804E817582F1816081418AE782BE82AF8160814282DC82C1...  LEN8_STRING_CP932         length=78, text="「ん～、顔だけ～。まったりしてたら課長とか\n寄ってきて、変な仕事やらされる～」"
0x0000032C      1  FF                                                   TERMINATOR_FF             
0x0000032D      1  FF                                                   TERMINATOR_FF             
0x0000032E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000330      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000332      1  FF                                                   TERMINATOR_FF             
0x00000333      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000335      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000337      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000339      1  FF                                                   TERMINATOR_FF             
0x0000033A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000033C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000033E      3  F31ED9                                               IMM16_F3                  u16_be=7897, u16_le=55582
0x00000341      1  FF                                                   TERMINATOR_FF             
0x00000342      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000344      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000346      9  8007495A3130313332                                   LEN8_STRING_CP932         length=7, text="IZ10132"
0x0000034F      1  FF                                                   TERMINATOR_FF             
0x00000350      1  FF                                                   TERMINATOR_FF             
0x00000351      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000353      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000355     84  8052817582BE82A982E782CB815B814182A882F182C882B6...  LEN8_STRING_CP932         length=82, text="「だからねー、おんなじ計算５回くらいしてー、なるべく忙しそーなふりとかするんだー」"
0x000003A9      1  FF                                                   TERMINATOR_FF             
0x000003AA      1  FF                                                   TERMINATOR_FF             
0x000003AB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003AD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003AF      1  FF                                                   TERMINATOR_FF             
0x000003B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003B4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000003B6      1  FF                                                   TERMINATOR_FF             
0x000003B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003B9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000003BB      3  F31EDA                                               IMM16_F3                  u16_be=7898, u16_le=55838
0x000003BE      1  FF                                                   TERMINATOR_FF             
0x000003BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003C1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003C3     34  8020817582C282DC82E881418354837B836282C482E982ED...  LEN8_STRING_CP932         length=32, text="「つまり、サボッてるわけですね」"
0x000003E5      1  FF                                                   TERMINATOR_FF             
0x000003E6      1  FF                                                   TERMINATOR_FF             
0x000003E7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003E9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003EB      1  FF                                                   TERMINATOR_FF             
0x000003EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003F0      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000003F2      1  FF                                                   TERMINATOR_FF             
0x000003F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003F5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003F7      3  F300DF                                               IMM16_F3                  u16_be=223, u16_le=57088
0x000003FA      1  FF                                                   TERMINATOR_FF             
0x000003FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003FD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000003FF      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000401      1  FF                                                   TERMINATOR_FF             
0x00000402      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000404      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000406      1  FF                                                   TERMINATOR_FF             
0x00000407      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000409      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000040B      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000040D      1  FF                                                   TERMINATOR_FF             
0x0000040E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000410      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000412      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000414      1  FF                                                   TERMINATOR_FF             
0x00000415      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000417      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000419      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000041B      1  FF                                                   TERMINATOR_FF             
0x0000041C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000041E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000420      1  FF                                                   TERMINATOR_FF             
0x00000421      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000423      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000425      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000427      1  FF                                                   TERMINATOR_FF             
0x00000428      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000042A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000042C      3  F31EDB                                               IMM16_F3                  u16_be=7899, u16_le=56094
0x0000042F      1  FF                                                   TERMINATOR_FF             
0x00000430      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000432      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000434      9  8007495A3130313431                                   LEN8_STRING_CP932         length=7, text="IZ10141"
0x0000043D      1  FF                                                   TERMINATOR_FF             
0x0000043E      1  FF                                                   TERMINATOR_FF             
0x0000043F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000441      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000443     52  803281758354837B836282C482C882A282E682A881428365...  LEN8_STRING_CP932         length=50, text="「サボッてないよお。テーネーな仕事してる\nだけー」"
0x00000477      1  FF                                                   TERMINATOR_FF             
0x00000478      1  FF                                                   TERMINATOR_FF             
0x00000479      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000047B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000047D      1  FF                                                   TERMINATOR_FF             
0x0000047E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000480      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000482      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000484      1  FF                                                   TERMINATOR_FF             
0x00000485      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000487      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000489      3  F31EDC                                               IMM16_F3                  u16_be=7900, u16_le=56350
0x0000048C      1  FF                                                   TERMINATOR_FF             
0x0000048D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000048F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000491      9  8007495A3130313432                                   LEN8_STRING_CP932         length=7, text="IZ10142"
0x0000049A      1  FF                                                   TERMINATOR_FF             
0x0000049B      1  FF                                                   TERMINATOR_FF             
0x0000049C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000049E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004A0     90  8058817582BE82C182C482CB815B814182DD82C882B382F1...  LEN8_STRING_CP932         length=88, text="「だってねー、みなさんの税金を預かってるわけだから、うっかりさんはゆるされないもんねー」"
0x000004FA      1  FF                                                   TERMINATOR_FF             
0x000004FB      1  FF                                                   TERMINATOR_FF             
0x000004FC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004FE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000500      1  FF                                                   TERMINATOR_FF             
0x00000501      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000503      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000505      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000507      1  FF                                                   TERMINATOR_FF             
0x00000508      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000050A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000050C      3  F31EDD                                               IMM16_F3                  u16_be=7901, u16_le=56606
0x0000050F      1  FF                                                   TERMINATOR_FF             
0x00000510      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000512      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000514     40  802681758ED089EF906C82C182C4816381638A7790B682E6...  LEN8_STRING_CP932         length=38, text="「社会人って……学生より楽そうですね」"
0x0000053C      1  FF                                                   TERMINATOR_FF             
0x0000053D      1  FF                                                   TERMINATOR_FF             
0x0000053E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000540      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000542      1  FF                                                   TERMINATOR_FF             
0x00000543      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000545      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000547      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000549      1  FF                                                   TERMINATOR_FF             
0x0000054A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000054C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000054E      3  F300D8                                               IMM16_F3                  u16_be=216, u16_le=55296
0x00000551      1  FF                                                   TERMINATOR_FF             
0x00000552      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000554      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000556      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000558      1  FF                                                   TERMINATOR_FF             
0x00000559      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000055B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000055D      1  FF                                                   TERMINATOR_FF             
0x0000055E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000560      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000562      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000564      1  FF                                                   TERMINATOR_FF             
0x00000565      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000567      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000569      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000056B      1  FF                                                   TERMINATOR_FF             
0x0000056C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000056E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000570      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000572      1  FF                                                   TERMINATOR_FF             
0x00000573      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000575      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000577      1  FF                                                   TERMINATOR_FF             
0x00000578      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000057A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000057C      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000057E      1  FF                                                   TERMINATOR_FF             
0x0000057F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000581      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000583      3  F31EDE                                               IMM16_F3                  u16_be=7902, u16_le=56862
0x00000586      1  FF                                                   TERMINATOR_FF             
0x00000587      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000589      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000058B      9  8007495A3130313531                                   LEN8_STRING_CP932         length=7, text="IZ10151"
0x00000594      1  FF                                                   TERMINATOR_FF             
0x00000595      1  FF                                                   TERMINATOR_FF             
0x00000596      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000598      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000059A     60  803A817582E282A0816082BE814182BB82F182C882B182C6...  LEN8_STRING_CP932         length=58, text="「やあ～だ、そんなことないよおー。学生さん\n気楽だよお～」"
0x000005D6      1  FF                                                   TERMINATOR_FF             
0x000005D7      1  FF                                                   TERMINATOR_FF             
0x000005D8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005DA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005DC      1  FF                                                   TERMINATOR_FF             
0x000005DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005E1      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000005E3      1  FF                                                   TERMINATOR_FF             
0x000005E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005E6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000005E8      3  F31EDF                                               IMM16_F3                  u16_be=7903, u16_le=57118
0x000005EB      1  FF                                                   TERMINATOR_FF             
0x000005EC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005EE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000005F0      9  8007495A3130313532                                   LEN8_STRING_CP932         length=7, text="IZ10152"
0x000005F9      1  FF                                                   TERMINATOR_FF             
0x000005FA      1  FF                                                   TERMINATOR_FF             
0x000005FB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005FD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005FF     82  8050817582BE82C182C482B382A0814182BD82C682A682CE...  LEN8_STRING_CP932         length=80, text="「だってさあ、たとえばよお、気のあわない子がいたって話しなきゃいいだけだも～ん」"
0x00000651      1  FF                                                   TERMINATOR_FF             
0x00000652      1  FF                                                   TERMINATOR_FF             
0x00000653      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000655      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000657      1  FF                                                   TERMINATOR_FF             
0x00000658      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000065A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000065C      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000065E      1  FF                                                   TERMINATOR_FF             
0x0000065F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000661      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000663      3  F31EE0                                               IMM16_F3                  u16_be=7904, u16_le=57374
0x00000666      1  FF                                                   TERMINATOR_FF             
0x00000667      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000669      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000066B      9  8007495A3130313533                                   LEN8_STRING_CP932         length=7, text="IZ10153"
0x00000674      1  FF                                                   TERMINATOR_FF             
0x00000675      1  FF                                                   TERMINATOR_FF             
0x00000676      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000678      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000067A     90  8058817582C582E082E682A881418ED089EF906C82CD82CB...  LEN8_STRING_CP932         length=88, text="「でもよお、社会人はねえ、セクハラ上司だからシカト～ってわけにはいかないんですからねー」"
0x000006D4      1  FF                                                   TERMINATOR_FF             
0x000006D5      1  FF                                                   TERMINATOR_FF             
0x000006D6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006D8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006DA      1  FF                                                   TERMINATOR_FF             
0x000006DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006DF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000006E1      1  FF                                                   TERMINATOR_FF             
0x000006E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006E4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006E6      3  F31EE1                                               IMM16_F3                  u16_be=7905, u16_le=57630
0x000006E9      1  FF                                                   TERMINATOR_FF             
0x000006EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006EC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006EE     26  8018817582DC82A08163816382BB815B82C582B782A982CB...  LEN8_STRING_CP932         length=24, text="「まあ……そーですかね」"
0x00000708      1  FF                                                   TERMINATOR_FF             
0x00000709      1  FF                                                   TERMINATOR_FF             
0x0000070A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000070C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000070E      1  FF                                                   TERMINATOR_FF             
0x0000070F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000711      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000713      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000715      1  FF                                                   TERMINATOR_FF             
0x00000716      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000718      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000071A      3  F300DE                                               IMM16_F3                  u16_be=222, u16_le=56832
0x0000071D      1  FF                                                   TERMINATOR_FF             
0x0000071E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000720      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000722      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000724      1  FF                                                   TERMINATOR_FF             
0x00000725      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000727      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000729      1  FF                                                   TERMINATOR_FF             
0x0000072A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000072C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000072E      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000730      1  FF                                                   TERMINATOR_FF             
0x00000731      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000733      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000735      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000737      1  FF                                                   TERMINATOR_FF             
0x00000738      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000073A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000073C      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000073E      1  FF                                                   TERMINATOR_FF             
0x0000073F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000741      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000743      1  FF                                                   TERMINATOR_FF             
0x00000744      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000746      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000748      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000074A      1  FF                                                   TERMINATOR_FF             
0x0000074B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000074D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000074F      3  F31EE2                                               IMM16_F3                  u16_be=7906, u16_le=57886
0x00000752      1  FF                                                   TERMINATOR_FF             
0x00000753      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000755      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000757      9  8007495A3130313630                                   LEN8_STRING_CP932         length=7, text="IZ10160"
0x00000760      1  FF                                                   TERMINATOR_FF             
0x00000761      1  FF                                                   TERMINATOR_FF             
0x00000762      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000764      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000766     14  800C817582BB815B82E6815B8176                         LEN8_STRING_CP932         length=12, text="「そーよー」"
0x00000774      1  FF                                                   TERMINATOR_FF             
0x00000775      1  FF                                                   TERMINATOR_FF             
0x00000776      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000778      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000077A      1  FF                                                   TERMINATOR_FF             
0x0000077B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000077D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000077F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000781      1  FF                                                   TERMINATOR_FF             
0x00000782      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000784      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000786      3  F31EE3                                               IMM16_F3                  u16_be=7907, u16_le=58142
0x00000789      1  FF                                                   TERMINATOR_FF             
0x0000078A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000078C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000078E     64  803E817582B682E1816381638354837B836282C482C882A9...  LEN8_STRING_CP932         length=62, text="「じゃ……サボッてなかったってことで、\nこれからどーします？」"
0x000007CE      1  FF                                                   TERMINATOR_FF             
0x000007CF      1  FF                                                   TERMINATOR_FF             
0x000007D0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007D2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007D4      1  FF                                                   TERMINATOR_FF             
0x000007D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007D9      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000007DB      1  FF                                                   TERMINATOR_FF             
0x000007DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007DE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007E0      3  F300DF                                               IMM16_F3                  u16_be=223, u16_le=57088
0x000007E3      1  FF                                                   TERMINATOR_FF             
0x000007E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007E6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000007E8      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000007EA      1  FF                                                   TERMINATOR_FF             
0x000007EB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007ED      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007EF      1  FF                                                   TERMINATOR_FF             
0x000007F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007F4      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000007F6      1  FF                                                   TERMINATOR_FF             
0x000007F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007F9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007FB      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000007FD      1  FF                                                   TERMINATOR_FF             
0x000007FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000800      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000802      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000804      1  FF                                                   TERMINATOR_FF             
0x00000805      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000807      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000809      1  FF                                                   TERMINATOR_FF             
0x0000080A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000080C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000080E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000810      1  FF                                                   TERMINATOR_FF             
0x00000811      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000813      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000815      3  F31EE4                                               IMM16_F3                  u16_be=7908, u16_le=58398
0x00000818      1  FF                                                   TERMINATOR_FF             
0x00000819      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000081B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000081D      9  8007495A3130313730                                   LEN8_STRING_CP932         length=7, text="IZ10170"
0x00000826      1  FF                                                   TERMINATOR_FF             
0x00000827      1  FF                                                   TERMINATOR_FF             
0x00000828      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000082A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000082C     94  805C817582A0814182BB82A482BB82A481418DA193FA82CB...  LEN8_STRING_CP932         length=92, text="「あ、そうそう、今日ねえ午後いっぱい\nつかってー、王子さまリストつくったー。\nエクセルでー」"
0x0000088A      1  FF                                                   TERMINATOR_FF             
0x0000088B      1  FF                                                   TERMINATOR_FF             
0x0000088C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000088E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000890      1  FF                                                   TERMINATOR_FF             
0x00000891      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000893      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000895      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000897      1  FF                                                   TERMINATOR_FF             
0x00000898      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000089A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000089C      3  F31EE5                                               IMM16_F3                  u16_be=7909, u16_le=58654
0x0000089F      1  FF                                                   TERMINATOR_FF             
0x000008A0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008A2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008A4     76  804A82BE82A982E7814182BB82EA82CD8354837B836282C4...  LEN8_STRING_CP932         length=74, text="だから、それはサボッてないのかといおうとしたとき、妙に聞きなれた声がする。"
0x000008F0      1  FF                                                   TERMINATOR_FF             
0x000008F1      1  FF                                                   TERMINATOR_FF             
0x000008F2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008F4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008F6      1  FF                                                   TERMINATOR_FF             
0x000008F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008FB      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000008FD      1  FF                                                   TERMINATOR_FF             
0x000008FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000900      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000902      3  F31EE6                                               IMM16_F3                  u16_be=7910, u16_le=58910
0x00000905      1  FF                                                   TERMINATOR_FF             
0x00000906      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000908      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000090A     10  80084D54313436393054                                 LEN8_STRING_CP932         length=8, text="MT14690T"
0x00000914      1  FF                                                   TERMINATOR_FF             
0x00000915      1  FF                                                   TERMINATOR_FF             
0x00000916      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000918      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000091A     36  8022817582B182F182C882C682B182C5814182C8815B82C9...  LEN8_STRING_CP932         length=34, text="「こんなとこで、なーにしーてるー」"
0x0000093E      1  FF                                                   TERMINATOR_FF             
0x0000093F      1  FF                                                   TERMINATOR_FF             
0x00000940      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000942      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000944      1  FF                                                   TERMINATOR_FF             
0x00000945      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000947      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000949      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000094B      1  FF                                                   TERMINATOR_FF             
0x0000094C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000094E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000950      3  F301D6                                               IMM16_F3                  u16_be=470, u16_le=54785
0x00000953      1  FF                                                   TERMINATOR_FF             
0x00000954      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000956      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000958      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000095A      1  FF                                                   TERMINATOR_FF             
0x0000095B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000095D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000095F      1  FF                                                   TERMINATOR_FF             
0x00000960      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000962      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000964      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000966      1  FF                                                   TERMINATOR_FF             
0x00000967      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000969      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000096B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000096D      1  FF                                                   TERMINATOR_FF             
0x0000096E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000970      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000972      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000974      1  FF                                                   TERMINATOR_FF             
0x00000975      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000977      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000979      1  FF                                                   TERMINATOR_FF             
0x0000097A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000097C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000097E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000980      1  FF                                                   TERMINATOR_FF             
0x00000981      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000983      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000985      3  F31EE7                                               IMM16_F3                  u16_be=7911, u16_le=59166
0x00000988      1  FF                                                   TERMINATOR_FF             
0x00000989      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000098B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000098D     58  80388368834C836221215C6E82A882BB82E982A882BB82E9...  LEN8_STRING_CP932         length=56, text="ドキッ!!\nおそるおそるふり返ると……\nやっぱり玉だった。"
0x000009C7      1  FF                                                   TERMINATOR_FF             
0x000009C8      1  FF                                                   TERMINATOR_FF             
0x000009C9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009CB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009CD      1  FF                                                   TERMINATOR_FF             
0x000009CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009D2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000009D4      1  FF                                                   TERMINATOR_FF             
0x000009D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009D7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009D9      3  F31EE8                                               IMM16_F3                  u16_be=7912, u16_le=59422
0x000009DC      1  FF                                                   TERMINATOR_FF             
0x000009DD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009E1     14  800C817582A0816381638BCA8176                         LEN8_STRING_CP932         length=12, text="「あ……玉」"
0x000009EF      1  FF                                                   TERMINATOR_FF             
0x000009F0      1  FF                                                   TERMINATOR_FF             
0x000009F1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009F3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009F5      1  FF                                                   TERMINATOR_FF             
0x000009F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009F8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009FA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000009FC      1  FF                                                   TERMINATOR_FF             
0x000009FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009FF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A01      3  F31EE9                                               IMM16_F3                  u16_be=7913, u16_le=59678
0x00000A04      1  FF                                                   TERMINATOR_FF             
0x00000A05      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A07      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A09     10  80084D54313437303054                                 LEN8_STRING_CP932         length=8, text="MT14700T"
0x00000A13      1  FF                                                   TERMINATOR_FF             
0x00000A14      1  FF                                                   TERMINATOR_FF             
0x00000A15      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A17      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A19     18  80108175837D834382C182C48CC482D78176                 LEN8_STRING_CP932         length=16, text="「マイって呼べ」"
0x00000A2B      1  FF                                                   TERMINATOR_FF             
0x00000A2C      1  FF                                                   TERMINATOR_FF             
0x00000A2D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A2F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A31      1  FF                                                   TERMINATOR_FF             
0x00000A32      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A34      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A36      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000A38      1  FF                                                   TERMINATOR_FF             
0x00000A39      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A3B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A3D      3  F30266                                               IMM16_F3                  u16_be=614, u16_le=26114
0x00000A40      1  FF                                                   TERMINATOR_FF             
0x00000A41      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A43      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A45      1  FF                                                   TERMINATOR_FF             
0x00000A46      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A48      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A4A      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000A4C      1  FF                                                   TERMINATOR_FF             
0x00000A4D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A4F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A51      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A53      1  FF                                                   TERMINATOR_FF             
0x00000A54      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A56      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A58      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A5A      1  FF                                                   TERMINATOR_FF             
0x00000A5B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A5D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A5F      1  FF                                                   TERMINATOR_FF             
0x00000A60      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A62      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A64      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000A66      1  FF                                                   TERMINATOR_FF             
0x00000A67      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A69      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A6B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A6D      1  FF                                                   TERMINATOR_FF             
0x00000A6E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A70      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A72      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000A74      1  FF                                                   TERMINATOR_FF             
0x00000A75      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A77      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A79      1  FF                                                   TERMINATOR_FF             
0x00000A7A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A7C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A7E      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000A80      1  FF                                                   TERMINATOR_FF             
0x00000A81      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A83      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A85      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000A87      1  FF                                                   TERMINATOR_FF             
0x00000A88      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A8A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A8C      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000A8E      1  FF                                                   TERMINATOR_FF             
0x00000A8F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A91      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A93      1  FF                                                   TERMINATOR_FF             
0x00000A94      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A96      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A98      2  F221                                                 IMM8_F2                   u8=33, s8=33
0x00000A9A      1  FF                                                   TERMINATOR_FF             
0x00000A9B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A9D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A9F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AA1      1  FF                                                   TERMINATOR_FF             
0x00000AA2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AA4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000AA6      3  F300AA                                               IMM16_F3                  u16_be=170, u16_le=43520
0x00000AA9      1  FF                                                   TERMINATOR_FF             
0x00000AAA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AAC      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00000AAE      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00000AB0      1  FF                                                   TERMINATOR_FF             
0x00000AB1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000AB3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AB5      1  FF                                                   TERMINATOR_FF             
0x00000AB6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AB8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000ABA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000ABC      1  FF                                                   TERMINATOR_FF             
0x00000ABD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000ABF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000AC1      3  F31EEA                                               IMM16_F3                  u16_be=7914, u16_le=59934
0x00000AC4      1  FF                                                   TERMINATOR_FF             
0x00000AC5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AC7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AC9     60  803A82C682B182ED82A28AE782F082B582C882AA82E78BCA...  LEN8_STRING_CP932         length=58, text="とこわい顔をしながら玉が近づいて来て……\n腕にしがみつく。"
0x00000B05      1  FF                                                   TERMINATOR_FF             
0x00000B06      1  FF                                                   TERMINATOR_FF             
0x00000B07      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B09      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B0B      1  FF                                                   TERMINATOR_FF             
0x00000B0C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B0E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B10      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000B12      1  FF                                                   TERMINATOR_FF             
0x00000B13      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B15      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B17      3  F31EEB                                               IMM16_F3                  u16_be=7915, u16_le=60190
0x00000B1A      1  FF                                                   TERMINATOR_FF             
0x00000B1B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B1D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000B1F     10  80084D54313437313054                                 LEN8_STRING_CP932         length=8, text="MT14710T"
0x00000B29      1  FF                                                   TERMINATOR_FF             
0x00000B2A      1  FF                                                   TERMINATOR_FF             
0x00000B2B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B2D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B2F     56  8036817595E082A282C482F182CC8CA9815B82BD815B82A9...  LEN8_STRING_CP932         length=54, text="「歩いてんの見ーたーからー、マイ、走って\n来たんだあ」"
0x00000B67      1  FF                                                   TERMINATOR_FF             
0x00000B68      1  FF                                                   TERMINATOR_FF             
0x00000B69      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B6B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B6D      1  FF                                                   TERMINATOR_FF             
0x00000B6E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B70      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B72      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B74      1  FF                                                   TERMINATOR_FF             
0x00000B75      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B77      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B79      3  F31EEC                                               IMM16_F3                  u16_be=7916, u16_le=60446
0x00000B7C      1  FF                                                   TERMINATOR_FF             
0x00000B7D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B7F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B81     48  802E82C68ADB93A182B382F182C98CFC82A982C182C482ED...  LEN8_STRING_CP932         length=46, text="と丸藤さんに向かってわざとらしい笑顔を向ける。"
0x00000BB1      1  FF                                                   TERMINATOR_FF             
0x00000BB2      1  FF                                                   TERMINATOR_FF             
0x00000BB3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000BB5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000BB7      1  FF                                                   TERMINATOR_FF             
0x00000BB8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BBA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BBC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000BBE      1  FF                                                   TERMINATOR_FF             
0x00000BBF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BC1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000BC3      3  F31EED                                               IMM16_F3                  u16_be=7917, u16_le=60702
0x00000BC6      1  FF                                                   TERMINATOR_FF             
0x00000BC7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BC9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BCB     80  804E817582A0814182A082CC82CB8BCA814182B182BF82E7...  LEN8_STRING_CP932         length=78, text="「あ、あのね玉、こちら丸藤泉美さん。\nあ、丸藤さん、これは玉。おさななじ……」"
0x00000C1B      1  FF                                                   TERMINATOR_FF             
0x00000C1C      1  FF                                                   TERMINATOR_FF             
0x00000C1D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C1F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C21      1  FF                                                   TERMINATOR_FF             
0x00000C22      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C24      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C26      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000C28      1  FF                                                   TERMINATOR_FF             
0x00000C29      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C2B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C2D      3  F31EEE                                               IMM16_F3                  u16_be=7918, u16_le=60958
0x00000C30      1  FF                                                   TERMINATOR_FF             
0x00000C31      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C33      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000C35     10  80084D54313437323054                                 LEN8_STRING_CP932         length=8, text="MT14720T"
0x00000C3F      1  FF                                                   TERMINATOR_FF             
0x00000C40      1  FF                                                   TERMINATOR_FF             
0x00000C41      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C43      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C45     18  80108175837D834382C182C48CC482D78176                 LEN8_STRING_CP932         length=16, text="「マイって呼べ」"
0x00000C57      1  FF                                                   TERMINATOR_FF             
0x00000C58      1  FF                                                   TERMINATOR_FF             
0x00000C59      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C5B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C5D      1  FF                                                   TERMINATOR_FF             
0x00000C5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C60      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C62      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000C64      1  FF                                                   TERMINATOR_FF             
0x00000C65      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C67      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C69      3  F31EEF                                               IMM16_F3                  u16_be=7919, u16_le=61214
0x00000C6C      1  FF                                                   TERMINATOR_FF             
0x00000C6D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C6F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C71     24  801682C68BCA82AA82B182ED82A290BA82F08F6F82B78142     LEN8_STRING_CP932         length=22, text="と玉がこわい声を出す。"
0x00000C89      1  FF                                                   TERMINATOR_FF             
0x00000C8A      1  FF                                                   TERMINATOR_FF             
0x00000C8B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C8D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C8F      1  FF                                                   TERMINATOR_FF             
0x00000C90      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C92      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C94      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000C96      1  FF                                                   TERMINATOR_FF             
0x00000C97      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C99      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C9B      3  F31EF0                                               IMM16_F3                  u16_be=7920, u16_le=61470
0x00000C9E      1  FF                                                   TERMINATOR_FF             
0x00000C9F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CA1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000CA3      9  8007495A3130313830                                   LEN8_STRING_CP932         length=7, text="IZ10180"
0x00000CAC      1  FF                                                   TERMINATOR_FF             
0x00000CAD      1  FF                                                   TERMINATOR_FF             
0x00000CAE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CB0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CB2     78  804C817582A0814182A082CC81638163837D834382BF82E1...  LEN8_STRING_CP932         length=76, text="「あ、あの……マイちゃん、今ね、彼にね、\nちょっとお願いごとがあってね……」"
0x00000D00      1  FF                                                   TERMINATOR_FF             
0x00000D01      1  FF                                                   TERMINATOR_FF             
0x00000D02      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D04      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D06      1  FF                                                   TERMINATOR_FF             
0x00000D07      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D09      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D0B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000D0D      1  FF                                                   TERMINATOR_FF             
0x00000D0E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D10      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D12      3  F31EF1                                               IMM16_F3                  u16_be=7921, u16_le=61726
0x00000D15      1  FF                                                   TERMINATOR_FF             
0x00000D16      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D18      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D1A     54  803482C695D989F082B582E682A482C682B782E98ADB93A1...  LEN8_STRING_CP932         length=52, text="と弁解しようとする丸藤さんの話を、玉は聞きもしない。"
0x00000D50      1  FF                                                   TERMINATOR_FF             
0x00000D51      1  FF                                                   TERMINATOR_FF             
0x00000D52      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D54      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D56      1  FF                                                   TERMINATOR_FF             
0x00000D57      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D59      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D5B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000D5D      1  FF                                                   TERMINATOR_FF             
0x00000D5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D60      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D62      3  F31EF2                                               IMM16_F3                  u16_be=7922, u16_le=61982
0x00000D65      1  FF                                                   TERMINATOR_FF             
0x00000D66      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D68      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000D6A     10  80084D54313437333054                                 LEN8_STRING_CP932         length=8, text="MT14730T"
0x00000D74      1  FF                                                   TERMINATOR_FF             
0x00000D75      1  FF                                                   TERMINATOR_FF             
0x00000D76      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D78      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D7A     97  805F817582A682A6816082C13F2120837D834382DD82BD82...  LEN8_STRING_CP932         length=95, text="「ええ～っ?! マイみたいなかわいい子が\nいるのに、あ～んなおばさんと一緒に歩くなんてへ～～～ん」"
0x00000DDB      1  FF                                                   TERMINATOR_FF             
0x00000DDC      1  FF                                                   TERMINATOR_FF             
0x00000DDD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000DDF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000DE1      1  FF                                                   TERMINATOR_FF             
0x00000DE2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DE4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DE6      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000DE8      1  FF                                                   TERMINATOR_FF             
0x00000DE9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DEB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000DED      3  F31EF3                                               IMM16_F3                  u16_be=7923, u16_le=62238
0x00000DF0      1  FF                                                   TERMINATOR_FF             
0x00000DF1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DF3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000DF5      9  8007495A3130313930                                   LEN8_STRING_CP932         length=7, text="IZ10190"
0x00000DFE      1  FF                                                   TERMINATOR_FF             
0x00000DFF      1  FF                                                   TERMINATOR_FF             
0x00000E00      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E02      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E04     27  8019817582A68163816382A08163816382A882CE82B382F1...  LEN8_STRING_CP932         length=25, text="「え……あ……おばさん?」"
0x00000E1F      1  FF                                                   TERMINATOR_FF             
0x00000E20      1  FF                                                   TERMINATOR_FF             
0x00000E21      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E23      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E25      1  FF                                                   TERMINATOR_FF             
0x00000E26      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E28      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E2A      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000E2C      1  FF                                                   TERMINATOR_FF             
0x00000E2D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E2F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E31      3  F31EF4                                               IMM16_F3                  u16_be=7924, u16_le=62494
0x00000E34      1  FF                                                   TERMINATOR_FF             
0x00000E35      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E37      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000E39     10  80084D54313437343154                                 LEN8_STRING_CP932         length=8, text="MT14741T"
0x00000E43      1  FF                                                   TERMINATOR_FF             
0x00000E44      1  FF                                                   TERMINATOR_FF             
0x00000E45      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E47      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E49    112  806E8175835F8381835F8381835F83818141837D834388C8...  LEN8_STRING_CP932         length=110, text="「ダメダメダメ、マイ以外の人を見ちゃいやあ。きょーだって、一緒にごはんたべたいなって、\nさがしてたんだもーん」"
0x00000EB9      1  FF                                                   TERMINATOR_FF             
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
0x00000ECB      3  F31EF5                                               IMM16_F3                  u16_be=7925, u16_le=62750
0x00000ECE      1  FF                                                   TERMINATOR_FF             
0x00000ECF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000ED1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000ED3     10  80084D54313437343254                                 LEN8_STRING_CP932         length=8, text="MT14742T"
0x00000EDD      1  FF                                                   TERMINATOR_FF             
0x00000EDE      1  FF                                                   TERMINATOR_FF             
0x00000EDF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000EE1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000EE3     62  803C817582C882CC82C9814182DD82C282A982E782C882A2...  LEN8_STRING_CP932         length=60, text="「なのに、みつからないから、マイ、泣いちゃうかと思ったあ!!」"
0x00000F21      1  FF                                                   TERMINATOR_FF             
0x00000F22      1  FF                                                   TERMINATOR_FF             
0x00000F23      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F25      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F27      1  FF                                                   TERMINATOR_FF             
0x00000F28      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F2A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F2C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000F2E      1  FF                                                   TERMINATOR_FF             
0x00000F2F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F31      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F33      3  F31EF6                                               IMM16_F3                  u16_be=7926, u16_le=63006
0x00000F36      1  FF                                                   TERMINATOR_FF             
0x00000F37      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F39      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F3B     26  8018817582A0814182A6814182C582E08DA182CD81638163...  LEN8_STRING_CP932         length=24, text="「あ、え、でも今は……」"
0x00000F55      1  FF                                                   TERMINATOR_FF             
0x00000F56      1  FF                                                   TERMINATOR_FF             
0x00000F57      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F59      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F5B      1  FF                                                   TERMINATOR_FF             
0x00000F5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F60      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000F62      1  FF                                                   TERMINATOR_FF             
0x00000F63      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F65      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F67      3  F31EF7                                               IMM16_F3                  u16_be=7927, u16_le=63262
0x00000F6A      1  FF                                                   TERMINATOR_FF             
0x00000F6B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F6D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000F6F     10  80084D54313437353054                                 LEN8_STRING_CP932         length=8, text="MT14750T"
0x00000F79      1  FF                                                   TERMINATOR_FF             
0x00000F7A      1  FF                                                   TERMINATOR_FF             
0x00000F7B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F7D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F7F     45  802B817582A88AE882A2212120837D834382CC82B182C681...  LEN8_STRING_CP932         length=43, text="「お願い!! マイのこと、どっかつれてって～」"
0x00000FAC      1  FF                                                   TERMINATOR_FF             
0x00000FAD      1  FF                                                   TERMINATOR_FF             
0x00000FAE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FB0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FB2      1  FF                                                   TERMINATOR_FF             
0x00000FB3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FB5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FB7      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000FB9      1  FF                                                   TERMINATOR_FF             
0x00000FBA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FBC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000FBE      3  F31EF8                                               IMM16_F3                  u16_be=7928, u16_le=63518
0x00000FC1      1  FF                                                   TERMINATOR_FF             
0x00000FC2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FC4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000FC6      9  8007495A3130323031                                   LEN8_STRING_CP932         length=7, text="IZ10201"
0x00000FCF      1  FF                                                   TERMINATOR_FF             
0x00000FD0      1  FF                                                   TERMINATOR_FF             
0x00000FD1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FD3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FD5    132  8082817582A682C182C6814182A682C182C6816381638142...  LEN8_STRING_CP932         length=130, text="「えっと、えっと……。じゃ、私は帰ろっかな。あ、そうそう、今日は用事があるんだった。\n今日はどーもありがとうね。本当に助かったわ」"
0x00001059      1  FF                                                   TERMINATOR_FF             
0x0000105A      1  FF                                                   TERMINATOR_FF             
0x0000105B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000105D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000105F      1  FF                                                   TERMINATOR_FF             
0x00001060      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001062      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001064      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001066      1  FF                                                   TERMINATOR_FF             
0x00001067      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001069      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000106B      3  F31EF9                                               IMM16_F3                  u16_be=7929, u16_le=63774
0x0000106E      1  FF                                                   TERMINATOR_FF             
0x0000106F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001071      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001073      9  8007495A3130323032                                   LEN8_STRING_CP932         length=7, text="IZ10202"
0x0000107C      1  FF                                                   TERMINATOR_FF             
0x0000107D      1  FF                                                   TERMINATOR_FF             
0x0000107E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001080      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001082     38  8024817582A0814182A282AF82C882A2814192788D8F82B5...  LEN8_STRING_CP932         length=36, text="「あ、いけない、遅刻しちゃうかもお」"
0x000010A8      1  FF                                                   TERMINATOR_FF             
0x000010A9      1  FF                                                   TERMINATOR_FF             
0x000010AA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010AC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010AE      1  FF                                                   TERMINATOR_FF             
0x000010AF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010B1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000010B3      8  800673652D653037                                     LEN8_STRING_CP932         length=6, text="se-e07"
0x000010BB      1  FF                                                   TERMINATOR_FF             
0x000010BC      1  FF                                                   TERMINATOR_FF             
0x000010BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010C1      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x000010C3      1  FF                                                   TERMINATOR_FF             
0x000010C4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010C6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010C8      1  FF                                                   TERMINATOR_FF             
0x000010C9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010CD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000010CF      1  FF                                                   TERMINATOR_FF             
0x000010D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010D2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000010D4      3  F31EFA                                               IMM16_F3                  u16_be=7930, u16_le=64030
0x000010D7      1  FF                                                   TERMINATOR_FF             
0x000010D8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010DA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010DC     70  804482ED82B482C682E782B582A282A282A282ED82AF82F0...  LEN8_STRING_CP932         length=68, text="わざとらしいいいわけをしながら、丸藤さんは\n逃げるように立ち去った。"
0x00001122      1  FF                                                   TERMINATOR_FF             
0x00001123      1  FF                                                   TERMINATOR_FF             
0x00001124      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001126      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001128      1  FF                                                   TERMINATOR_FF             
0x00001129      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000112B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000112D      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x0000112F      1  FF                                                   TERMINATOR_FF             
0x00001130      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001132      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001134      1  FF                                                   TERMINATOR_FF             
0x00001135      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001137      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001139      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000113B      1  FF                                                   TERMINATOR_FF             
0x0000113C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000113E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001140      2  F228                                                 IMM8_F2                   u8=40, s8=40
0x00001142      1  FF                                                   TERMINATOR_FF             
0x00001143      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001145      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001147      1  FF                                                   TERMINATOR_FF             
0x00001148      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000114A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000114C      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000114E      1  FF                                                   TERMINATOR_FF             
0x0000114F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001151      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001153      3  F301D5                                               IMM16_F3                  u16_be=469, u16_le=54529
0x00001156      1  FF                                                   TERMINATOR_FF             
0x00001157      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001159      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000115B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000115D      1  FF                                                   TERMINATOR_FF             
0x0000115E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001160      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001162      1  FF                                                   TERMINATOR_FF             
0x00001163      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001165      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001167      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001169      1  FF                                                   TERMINATOR_FF             
0x0000116A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000116C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000116E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001170      1  FF                                                   TERMINATOR_FF             
0x00001171      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001173      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001175      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001177      1  FF                                                   TERMINATOR_FF             
0x00001178      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000117A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000117C      1  FF                                                   TERMINATOR_FF             
0x0000117D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000117F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001181      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001183      1  FF                                                   TERMINATOR_FF             
0x00001184      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001186      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001188      3  F31EFB                                               IMM16_F3                  u16_be=7931, u16_le=64286
0x0000118B      1  FF                                                   TERMINATOR_FF             
0x0000118C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000118E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001190     36  8022817582C882F182BE82E681418BCA81428B7D82C982C8...  LEN8_STRING_CP932         length=34, text="「なんだよ、玉。急になにすんだよ」"
0x000011B4      1  FF                                                   TERMINATOR_FF             
0x000011B5      1  FF                                                   TERMINATOR_FF             
0x000011B6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000011B8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000011BA      1  FF                                                   TERMINATOR_FF             
0x000011BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011BF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000011C1      1  FF                                                   TERMINATOR_FF             
0x000011C2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011C4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000011C6      3  F31EFC                                               IMM16_F3                  u16_be=7932, u16_le=64542
0x000011C9      1  FF                                                   TERMINATOR_FF             
0x000011CA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011CC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011CE     64  803E8BCA82CD82A982E782DD82C282A282BD987282A982E7...  LEN8_STRING_CP932         length=62, text="玉はからみついた腕からはなれて、じっと\nこっちをにらみつける。"
0x0000120E      1  FF                                                   TERMINATOR_FF             
0x0000120F      1  FF                                                   TERMINATOR_FF             
0x00001210      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001212      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001214      1  FF                                                   TERMINATOR_FF             
0x00001215      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001217      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001219      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000121B      1  FF                                                   TERMINATOR_FF             
0x0000121C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000121E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001220      3  F31EFD                                               IMM16_F3                  u16_be=7933, u16_le=64798
0x00001223      1  FF                                                   TERMINATOR_FF             
0x00001224      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001226      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001228     10  80084D54313437363054                                 LEN8_STRING_CP932         length=8, text="MT14760T"
0x00001232      1  FF                                                   TERMINATOR_FF             
0x00001233      1  FF                                                   TERMINATOR_FF             
0x00001234      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001236      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001238     20  8012817582A082EA814182C8815B82C981488176             LEN8_STRING_CP932         length=18, text="「あれ、なーに？」"
0x0000124C      1  FF                                                   TERMINATOR_FF             
0x0000124D      1  FF                                                   TERMINATOR_FF             
0x0000124E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001250      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001252      1  FF                                                   TERMINATOR_FF             
0x00001253      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001255      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001257      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001259      1  FF                                                   TERMINATOR_FF             
0x0000125A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000125C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000125E      3  F31EFE                                               IMM16_F3                  u16_be=7934, u16_le=65054
0x00001261      1  FF                                                   TERMINATOR_FF             
0x00001262      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001264      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001266     16  800E817582C882C982C182C481488176                     LEN8_STRING_CP932         length=14, text="「なにって？」"
0x00001276      1  FF                                                   TERMINATOR_FF             
0x00001277      1  FF                                                   TERMINATOR_FF             
0x00001278      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000127A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000127C      1  FF                                                   TERMINATOR_FF             
0x0000127D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000127F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001281      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001283      1  FF                                                   TERMINATOR_FF             
0x00001284      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001286      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001288      3  F301D7                                               IMM16_F3                  u16_be=471, u16_le=55041
0x0000128B      1  FF                                                   TERMINATOR_FF             
0x0000128C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000128E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001290      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001292      1  FF                                                   TERMINATOR_FF             
0x00001293      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001295      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001297      1  FF                                                   TERMINATOR_FF             
0x00001298      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000129A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000129C      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000129E      1  FF                                                   TERMINATOR_FF             
0x0000129F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012A1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012A3      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000012A5      1  FF                                                   TERMINATOR_FF             
0x000012A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012A8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000012AA      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000012AC      1  FF                                                   TERMINATOR_FF             
0x000012AD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012AF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012B1      1  FF                                                   TERMINATOR_FF             
0x000012B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012B4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012B6      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000012B8      1  FF                                                   TERMINATOR_FF             
0x000012B9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012BB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000012BD      3  F31EFF                                               IMM16_F3                  u16_be=7935, u16_le=65310
0x000012C0      1  FF                                                   TERMINATOR_FF             
0x000012C1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012C3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000012C5     10  80084D54313437373054                                 LEN8_STRING_CP932         length=8, text="MT14770T"
0x000012CF      1  FF                                                   TERMINATOR_FF             
0x000012D0      1  FF                                                   TERMINATOR_FF             
0x000012D1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012D3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012D5     74  8048817582A0815B82E4815B82CC82AA82B782AB82C882ED...  LEN8_STRING_CP932         length=72, text="「あーゆーのがすきなわけー？\nすげーうすぎたなー。フケツなかんじするー」"
0x0000131F      1  FF                                                   TERMINATOR_FF             
0x00001320      1  FF                                                   TERMINATOR_FF             
0x00001321      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001323      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001325      1  FF                                                   TERMINATOR_FF             
0x00001326      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001328      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000132A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000132C      1  FF                                                   TERMINATOR_FF             
0x0000132D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000132F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001331      3  F31F00                                               IMM16_F3                  u16_be=7936, u16_le=31
0x00001334      1  FF                                                   TERMINATOR_FF             
0x00001335      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001337      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001339     28  801A817582A282E282BB815B82E4815B82CC82B682E182C8...  LEN8_STRING_CP932         length=26, text="「いやそーゆーのじゃなく」"
0x00001355      1  FF                                                   TERMINATOR_FF             
0x00001356      1  FF                                                   TERMINATOR_FF             
0x00001357      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001359      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000135B      1  FF                                                   TERMINATOR_FF             
0x0000135C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000135E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001360      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001362      1  FF                                                   TERMINATOR_FF             
0x00001363      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001365      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001367      3  F301C4                                               IMM16_F3                  u16_be=452, u16_le=50177
0x0000136A      1  FF                                                   TERMINATOR_FF             
0x0000136B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000136D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000136F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001371      1  FF                                                   TERMINATOR_FF             
0x00001372      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001374      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001376      1  FF                                                   TERMINATOR_FF             
0x00001377      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001379      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000137B      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000137D      1  FF                                                   TERMINATOR_FF             
0x0000137E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001380      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001382      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001384      1  FF                                                   TERMINATOR_FF             
0x00001385      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001387      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001389      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000138B      1  FF                                                   TERMINATOR_FF             
0x0000138C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000138E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001390      1  FF                                                   TERMINATOR_FF             
0x00001391      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001393      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001395      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001397      1  FF                                                   TERMINATOR_FF             
0x00001398      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000139A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000139C      3  F31F01                                               IMM16_F3                  u16_be=7937, u16_le=287
0x0000139F      1  FF                                                   TERMINATOR_FF             
0x000013A0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013A2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000013A4     10  80084D54313437383054                                 LEN8_STRING_CP932         length=8, text="MT14780T"
0x000013AE      1  FF                                                   TERMINATOR_FF             
0x000013AF      1  FF                                                   TERMINATOR_FF             
0x000013B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013B4     74  8048817582CD82E982B382F182A282C182C482BD82CC837A...  LEN8_STRING_CP932         length=72, text="「はるさんいってたのホントだー。だんしってーユーワクにすっげーよわいー」"
0x000013FE      1  FF                                                   TERMINATOR_FF             
0x000013FF      1  FF                                                   TERMINATOR_FF             
0x00001400      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001402      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001404      1  FF                                                   TERMINATOR_FF             
0x00001405      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001407      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001409      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000140B      1  FF                                                   TERMINATOR_FF             
0x0000140C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000140E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001410      3  F31F02                                               IMM16_F3                  u16_be=7938, u16_le=543
0x00001413      1  FF                                                   TERMINATOR_FF             
0x00001414      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001416      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001418     24  801681759755986682C98EE382A282CC82A8914F21218176     LEN8_STRING_CP932         length=22, text="「誘惑に弱いのお前!!」"
0x00001430      1  FF                                                   TERMINATOR_FF             
0x00001431      1  FF                                                   TERMINATOR_FF             
0x00001432      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001434      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001436      1  FF                                                   TERMINATOR_FF             
0x00001437      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001439      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000143B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000143D      1  FF                                                   TERMINATOR_FF             
0x0000143E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001440      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001442      3  F301D4                                               IMM16_F3                  u16_be=468, u16_le=54273
0x00001445      1  FF                                                   TERMINATOR_FF             
0x00001446      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001448      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000144A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000144C      1  FF                                                   TERMINATOR_FF             
0x0000144D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000144F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001451      1  FF                                                   TERMINATOR_FF             
0x00001452      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001454      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001456      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001458      1  FF                                                   TERMINATOR_FF             
0x00001459      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000145B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000145D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000145F      1  FF                                                   TERMINATOR_FF             
0x00001460      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001462      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001464      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001466      1  FF                                                   TERMINATOR_FF             
0x00001467      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001469      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000146B      1  FF                                                   TERMINATOR_FF             
0x0000146C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000146E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001470      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001472      1  FF                                                   TERMINATOR_FF             
0x00001473      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001475      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001477      3  F31F03                                               IMM16_F3                  u16_be=7939, u16_le=799
0x0000147A      1  FF                                                   TERMINATOR_FF             
0x0000147B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000147D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000147F     10  80084D54313437393054                                 LEN8_STRING_CP932         length=8, text="MT14790T"
0x00001489      1  FF                                                   TERMINATOR_FF             
0x0000148A      1  FF                                                   TERMINATOR_FF             
0x0000148B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000148D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000148F     38  8024817582B782C182B282AD82B782C182B2815B82AD82E2...  LEN8_STRING_CP932         length=36, text="「すっごくすっごーくや～らしーいー」"
0x000014B5      1  FF                                                   TERMINATOR_FF             
0x000014B6      1  FF                                                   TERMINATOR_FF             
0x000014B7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000014B9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000014BB      1  FF                                                   TERMINATOR_FF             
0x000014BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014C0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000014C2      1  FF                                                   TERMINATOR_FF             
0x000014C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014C5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000014C7      3  F31F04                                               IMM16_F3                  u16_be=7940, u16_le=1055
0x000014CA      1  FF                                                   TERMINATOR_FF             
0x000014CB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014CD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014CF     46  802C82A282A282ED82AF82B782E982CC82CD82E282DF82BD...  LEN8_STRING_CP932         length=44, text="いいわけするのはやめた。\n火に油って感じだ。"
0x000014FD      1  FF                                                   TERMINATOR_FF             
0x000014FE      1  FF                                                   TERMINATOR_FF             
0x000014FF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001501      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001503      1  FF                                                   TERMINATOR_FF             
0x00001504      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001506      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001508      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000150A      1  FF                                                   TERMINATOR_FF             
0x0000150B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000150D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000150F      3  F301D7                                               IMM16_F3                  u16_be=471, u16_le=55041
0x00001512      1  FF                                                   TERMINATOR_FF             
0x00001513      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001515      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001517      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001519      1  FF                                                   TERMINATOR_FF             
0x0000151A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000151C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000151E      1  FF                                                   TERMINATOR_FF             
0x0000151F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001521      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001523      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001525      1  FF                                                   TERMINATOR_FF             
0x00001526      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001528      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000152A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000152C      1  FF                                                   TERMINATOR_FF             
0x0000152D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000152F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001531      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001533      1  FF                                                   TERMINATOR_FF             
0x00001534      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001536      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001538      1  FF                                                   TERMINATOR_FF             
0x00001539      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000153B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000153D      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000153F      1  FF                                                   TERMINATOR_FF             
0x00001540      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001542      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001544      3  F31F05                                               IMM16_F3                  u16_be=7941, u16_le=1311
0x00001547      1  FF                                                   TERMINATOR_FF             
0x00001548      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000154A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000154C     10  80084D54313438303054                                 LEN8_STRING_CP932         length=8, text="MT14800T"
0x00001556      1  FF                                                   TERMINATOR_FF             
0x00001557      1  FF                                                   TERMINATOR_FF             
0x00001558      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000155A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000155C     62  803C817582A882CE82B382F182CC82AD815B82B9815B82C9...  LEN8_STRING_CP932         length=60, text="「おばさんのくーせーに。あーゆーの好きってーやらしすーぎー」"
0x0000159A      1  FF                                                   TERMINATOR_FF             
0x0000159B      1  FF                                                   TERMINATOR_FF             
0x0000159C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000159E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000015A0      1  FF                                                   TERMINATOR_FF             
0x000015A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015A5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000015A7      1  FF                                                   TERMINATOR_FF             
0x000015A8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015AA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000015AC      3  F31F06                                               IMM16_F3                  u16_be=7942, u16_le=1567
0x000015AF      1  FF                                                   TERMINATOR_FF             
0x000015B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015B4     40  802682A0816082A08141837A835290E682AA82B182C182BF...  LEN8_STRING_CP932         length=38, text="あ～あ、ホコ先がこっちに向いちゃった。"
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
0x000015EE      3  F31F07                                               IMM16_F3                  u16_be=7943, u16_le=1823
0x000015F1      1  FF                                                   TERMINATOR_FF             
0x000015F2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015F4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015F6     58  803882B1815B82E4815B82B182C682CD82A882DA82A682C4...  LEN8_STRING_CP932         length=56, text="こーゆーことはおぼえてるヤツだから。\n長いぞ、きっと……"
0x00001630      1  FF                                                   TERMINATOR_FF             
0x00001631      1  FF                                                   TERMINATOR_FF             
0x00001632      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001634      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001636      1  FF                                                   TERMINATOR_FF             
0x00001637      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001639      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000163B      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x0000163D      1  FF                                                   TERMINATOR_FF             
0x0000163E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001640      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001642      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001644      1  FF                                                   TERMINATOR_FF             
0x00001645      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001647      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001649      1  FF                                                   TERMINATOR_FF             
0x0000164A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000164C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000164E      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00001650      1  FF                                                   TERMINATOR_FF             
0x00001651      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001653      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001655      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00001657      1  FF                                                   TERMINATOR_FF             
0x00001658      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000165A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000165C      1  FF                                                   TERMINATOR_FF             
0x0000165D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000165F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001661      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001663      1  FF                                                   TERMINATOR_FF             
0x00001664      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001666      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001668      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000166A      1  FF                                                   TERMINATOR_FF             
0x0000166B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000166D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000166F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001671      1  FF                                                   TERMINATOR_FF             
0x00001672      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001674      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001676      1  FF                                                   TERMINATOR_FF             
0x00001677      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001679      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000167B      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000167D      1  FF                                                   TERMINATOR_FF             
0x0000167E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001680      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001682      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001684      1  FF                                                   TERMINATOR_FF             
0x00001685      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001687      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001689      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000168B      1  FF                                                   TERMINATOR_FF             
0x0000168C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000168E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001690      1  FF                                                   TERMINATOR_FF             
0x00001691      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001693      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001695      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00001697      1  FF                                                   TERMINATOR_FF             
0x00001698      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000169A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000169C      1  FF                                                   TERMINATOR_FF             
0x0000169D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000169F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016A1      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000016A3      1  FF                                                   TERMINATOR_FF             
0x000016A4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016A6      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000016A8     14  800C50533254303230612E62696E                         LEN8_STRING_CP932         length=12, text="PS2T020a.bin"
0x000016B6      1  FF                                                   TERMINATOR_FF             
0x000016B7      1  FF                                                   TERMINATOR_FF             
0x000016B8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016BA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016BC      1  FF                                                   TERMINATOR_FF             
0x000016BD      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000016BF      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000016C1      1  FF                                                   TERMINATOR_FF             
0x000016C2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016C4      1  CC                                                   OPAQUE_RAW_BYTES          bytes=CC
0x000016C5      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x000016C7      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x000016C9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016CB      1  BD                                                   OPAQUE_RAW_BYTES          bytes=BD
0x000016CC      1  FF                                                   TERMINATOR_FF             
0x000016CD      1  FF                                                   TERMINATOR_FF             
