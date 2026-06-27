; Full conservative disassembly for PS2F003A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      2  F264                                                 IMM8_F2                   u8=100, s8=100
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
0x0000001E      2  F215                                                 IMM8_F2                   u8=21, s8=21
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
0x00000038      2  F21E                                                 IMM8_F2                   u8=30, s8=30
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
0x0000004B      3  F315FC                                               IMM16_F3                  u16_be=5628, u16_le=64533
0x0000004E      1  FF                                                   TERMINATOR_FF             
0x0000004F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000051      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000053     52  803282A0816082A0814182E082A48163816381425C6E975C...  LEN8_STRING_CP932         length=50, text="あ～あ、もう……。\n予測のつかないことが多すぎる。"
0x00000087      1  FF                                                   TERMINATOR_FF             
0x00000088      1  FF                                                   TERMINATOR_FF             
0x00000089      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000008B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000008D      1  FF                                                   TERMINATOR_FF             
0x0000008E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000090      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000092      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000094      1  FF                                                   TERMINATOR_FF             
0x00000095      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000097      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000099      3  F315FD                                               IMM16_F3                  u16_be=5629, u16_le=64789
0x0000009C      1  FF                                                   TERMINATOR_FF             
0x0000009D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000009F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000A1     56  803682DC82A081418BCA82BE82A982E782C182C482A282C1...  LEN8_STRING_CP932         length=54, text="まあ、玉だからっていっちゃえば、そうなんだ\nけど……。"
0x000000D9      1  FF                                                   TERMINATOR_FF             
0x000000DA      1  FF                                                   TERMINATOR_FF             
0x000000DB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000DD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000DF      1  FF                                                   TERMINATOR_FF             
0x000000E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000000E6      1  FF                                                   TERMINATOR_FF             
0x000000E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000000EB      3  F315FE                                               IMM16_F3                  u16_be=5630, u16_le=65045
0x000000EE      1  FF                                                   TERMINATOR_FF             
0x000000EF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000F3     94  805C8AB582EA82C482E982C282E082E882BE82C182BD82AF...  LEN8_STRING_CP932         length=92, text="慣れてるつもりだったけど。\nやっぱり疲れはするなあ。\nあいつ、なにしでかすか、わかんないし。"
0x00000151      1  FF                                                   TERMINATOR_FF             
0x00000152      1  FF                                                   TERMINATOR_FF             
0x00000153      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000155      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000157      1  FF                                                   TERMINATOR_FF             
0x00000158      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000015A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000015C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000015E      1  FF                                                   TERMINATOR_FF             
0x0000015F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000161      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000163      3  F315FF                                               IMM16_F3                  u16_be=5631, u16_le=65301
0x00000166      1  FF                                                   TERMINATOR_FF             
0x00000167      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000169      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000016B    122  80788AEE967B934982C988AB8B4382CD82C882A282F182BE...  LEN8_STRING_CP932         length=120, text="基本的に悪気はないんだけど、世間のみんなが\nそう思ってくれるとは限らないし。\nもう少しな、大人になってくれると楽だけど。"
0x000001E5      1  FF                                                   TERMINATOR_FF             
0x000001E6      1  FF                                                   TERMINATOR_FF             
0x000001E7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001E9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001EB      1  FF                                                   TERMINATOR_FF             
0x000001EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001F0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001F2      1  FF                                                   TERMINATOR_FF             
0x000001F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001F5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001F7      3  F31600                                               IMM16_F3                  u16_be=5632, u16_le=22
0x000001FA      1  FF                                                   TERMINATOR_FF             
0x000001FB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001FD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001FF     46  802C8BCA82C982BB82EA82F0965D82F182C582E08E6495FB...  LEN8_STRING_CP932         length=44, text="玉にそれを望んでも仕方ないし。\nそれに……。"
0x0000022D      1  FF                                                   TERMINATOR_FF             
0x0000022E      1  FF                                                   TERMINATOR_FF             
0x0000022F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000231      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000233      1  FF                                                   TERMINATOR_FF             
0x00000234      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000236      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000238      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000023A      1  FF                                                   TERMINATOR_FF             
0x0000023B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000023D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000023F      3  F31601                                               IMM16_F3                  u16_be=5633, u16_le=278
0x00000242      1  FF                                                   TERMINATOR_FF             
0x00000243      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000245      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000247     38  802491E5906C82C182C482A282C182C482E0816381638142...  LEN8_STRING_CP932         length=36, text="大人っていっても……。\n以外と、ね。"
0x0000026D      1  FF                                                   TERMINATOR_FF             
0x0000026E      1  FF                                                   TERMINATOR_FF             
0x0000026F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000271      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000273      1  FF                                                   TERMINATOR_FF             
0x00000274      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000276      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000278      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000027A      1  FF                                                   TERMINATOR_FF             
0x0000027B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000027D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000027F      2  F22D                                                 IMM8_F2                   u8=45, s8=45
0x00000281      1  FF                                                   TERMINATOR_FF             
0x00000282      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000284      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000286      1  FF                                                   TERMINATOR_FF             
0x00000287      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000289      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000028B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000028D      1  FF                                                   TERMINATOR_FF             
0x0000028E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000290      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000292      3  F3020B                                               IMM16_F3                  u16_be=523, u16_le=2818
0x00000295      1  FF                                                   TERMINATOR_FF             
0x00000296      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000298      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000029A      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000029C      1  FF                                                   TERMINATOR_FF             
0x0000029D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000029F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002A1      1  FF                                                   TERMINATOR_FF             
0x000002A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A6      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x000002A8      1  FF                                                   TERMINATOR_FF             
0x000002A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002AB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002AD      2  F210                                                 IMM8_F2                   u8=16, s8=16
0x000002AF      1  FF                                                   TERMINATOR_FF             
0x000002B0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002B2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002B4      1  FF                                                   TERMINATOR_FF             
0x000002B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B9      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000002BB      1  FF                                                   TERMINATOR_FF             
0x000002BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002BE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002C0      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002C2      1  FF                                                   TERMINATOR_FF             
0x000002C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002C5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002C7      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000002C9      1  FF                                                   TERMINATOR_FF             
0x000002CA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002CC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002CE      1  FF                                                   TERMINATOR_FF             
0x000002CF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002D3      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000002D5      1  FF                                                   TERMINATOR_FF             
0x000002D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002D8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000002DA      3  F31602                                               IMM16_F3                  u16_be=5634, u16_le=534
0x000002DD      1  FF                                                   TERMINATOR_FF             
0x000002DE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002E0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002E2      9  8007495A3930343030                                   LEN8_STRING_CP932         length=7, text="IZ90400"
0x000002EB      1  FF                                                   TERMINATOR_FF             
0x000002EC      1  FF                                                   TERMINATOR_FF             
0x000002ED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002EF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002F1    115  8071817582A081608141978882BD82AB82BD8160814282A8...  LEN8_STRING_CP932         length=113, text="「あ～、来たきた～。おそいねー。なになに～、\nデートだったのー？ え～、彼女いるんだー？\nえ～、どんな子なの～？」"
0x00000364      1  FF                                                   TERMINATOR_FF             
0x00000365      1  FF                                                   TERMINATOR_FF             
0x00000366      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000368      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000036A      1  FF                                                   TERMINATOR_FF             
0x0000036B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000036D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000036F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000371      1  FF                                                   TERMINATOR_FF             
0x00000372      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000374      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000376      3  F31603                                               IMM16_F3                  u16_be=5635, u16_le=790
0x00000379      1  FF                                                   TERMINATOR_FF             
0x0000037A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000037C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000037E    118  807488C8914F82A082DC82E082CC8B69928382C58ADB93A1...  LEN8_STRING_CP932         length=116, text="以前あまもの喫茶で丸藤さんに頼まれた、\n王子さまさがしを手伝って欲しいとゆーやつ。\nあれで急に呼びだされたんだけど。"
0x000003F4      1  FF                                                   TERMINATOR_FF             
0x000003F5      1  FF                                                   TERMINATOR_FF             
0x000003F6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003FA      1  FF                                                   TERMINATOR_FF             
0x000003FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003FF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000401      1  FF                                                   TERMINATOR_FF             
0x00000402      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000404      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000406      3  F31604                                               IMM16_F3                  u16_be=5636, u16_le=1046
0x00000409      1  FF                                                   TERMINATOR_FF             
0x0000040A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000040C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000040E    112  806E8ADB93A182B382F182CD81418CF6898082C582A88B43...  LEN8_STRING_CP932         length=110, text="丸藤さんは、公園でお気楽そうに待っていた。\n公務員っていってたけど……なんか働いている\n姿が想像できない人だ。"
0x0000047E      1  FF                                                   TERMINATOR_FF             
0x0000047F      1  FF                                                   TERMINATOR_FF             
0x00000480      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000482      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000484      1  FF                                                   TERMINATOR_FF             
0x00000485      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000487      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000489      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000048B      1  FF                                                   TERMINATOR_FF             
0x0000048C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000048E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000490      3  F31605                                               IMM16_F3                  u16_be=5637, u16_le=1302
0x00000493      1  FF                                                   TERMINATOR_FF             
0x00000494      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000496      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000498     46  802C817591539152814182BB82F182C882B182C682A282C1...  LEN8_STRING_CP932         length=44, text="「全然、そんなこといってないじゃないですか」"
0x000004C6      1  FF                                                   TERMINATOR_FF             
0x000004C7      1  FF                                                   TERMINATOR_FF             
0x000004C8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004CA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004CC      1  FF                                                   TERMINATOR_FF             
0x000004CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004CF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004D1      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000004D3      1  FF                                                   TERMINATOR_FF             
0x000004D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004D6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004D8      3  F300C2                                               IMM16_F3                  u16_be=194, u16_le=49664
0x000004DB      1  FF                                                   TERMINATOR_FF             
0x000004DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004DE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000004E0      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000004E2      1  FF                                                   TERMINATOR_FF             
0x000004E3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004E5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004E7      1  FF                                                   TERMINATOR_FF             
0x000004E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004EC      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000004EE      1  FF                                                   TERMINATOR_FF             
0x000004EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004F3      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000004F5      1  FF                                                   TERMINATOR_FF             
0x000004F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004F8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000004FA      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000004FC      1  FF                                                   TERMINATOR_FF             
0x000004FD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004FF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000501      1  FF                                                   TERMINATOR_FF             
0x00000502      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000504      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000506      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000508      1  FF                                                   TERMINATOR_FF             
0x00000509      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000050B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000050D      3  F31606                                               IMM16_F3                  u16_be=5638, u16_le=1558
0x00000510      1  FF                                                   TERMINATOR_FF             
0x00000511      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000513      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000515      9  8007495A3930343130                                   LEN8_STRING_CP932         length=7, text="IZ90410"
0x0000051E      1  FF                                                   TERMINATOR_FF             
0x0000051F      1  FF                                                   TERMINATOR_FF             
0x00000520      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000522      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000524     97  805F817582A68160814182C882C982C882C9814188EA8F8F...  LEN8_STRING_CP932         length=95, text="「え～、なになに、一緒に帰ったー？\nあー、家まで送ってったとかー？ きゃあ、\nカップルさんだー」"
0x00000585      1  FF                                                   TERMINATOR_FF             
0x00000586      1  FF                                                   TERMINATOR_FF             
0x00000587      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000589      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000058B      1  FF                                                   TERMINATOR_FF             
0x0000058C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000058E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000590      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000592      1  FF                                                   TERMINATOR_FF             
0x00000593      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000595      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000597      3  F31607                                               IMM16_F3                  u16_be=5639, u16_le=1814
0x0000059A      1  FF                                                   TERMINATOR_FF             
0x0000059B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000059D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000059F     32  801E817582BB82F182C882B182C682C882A282F182C582B7...  LEN8_STRING_CP932         length=30, text="「そんなことないんですってば」"
0x000005BF      1  FF                                                   TERMINATOR_FF             
0x000005C0      1  FF                                                   TERMINATOR_FF             
0x000005C1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005C3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005C5      1  FF                                                   TERMINATOR_FF             
0x000005C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005CA      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000005CC      1  FF                                                   TERMINATOR_FF             
0x000005CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005D1      3  F300C8                                               IMM16_F3                  u16_be=200, u16_le=51200
0x000005D4      1  FF                                                   TERMINATOR_FF             
0x000005D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005D7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000005D9      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000005DB      1  FF                                                   TERMINATOR_FF             
0x000005DC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005DE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005E0      1  FF                                                   TERMINATOR_FF             
0x000005E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005E5      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000005E7      1  FF                                                   TERMINATOR_FF             
0x000005E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005EC      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000005EE      1  FF                                                   TERMINATOR_FF             
0x000005EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005F1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000005F3      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000005F5      1  FF                                                   TERMINATOR_FF             
0x000005F6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005FA      1  FF                                                   TERMINATOR_FF             
0x000005FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005FF      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000601      1  FF                                                   TERMINATOR_FF             
0x00000602      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000604      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000606      3  F31608                                               IMM16_F3                  u16_be=5640, u16_le=2070
0x00000609      1  FF                                                   TERMINATOR_FF             
0x0000060A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000060C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000060E      9  8007495A3930343230                                   LEN8_STRING_CP932         length=7, text="IZ90420"
0x00000617      1  FF                                                   TERMINATOR_FF             
0x00000618      1  FF                                                   TERMINATOR_FF             
0x00000619      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000061B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000061D    116  8072817582A682C182D682C182D6815B81428FC682EA82C4...  LEN8_STRING_CP932         length=114, text="「えっへっへー。照れてるんでしょー。\nいーんだよー、恥ずかしがんなくてもー。\nどれどれ、お姉さんに話してみなさい」"
0x00000691      1  FF                                                   TERMINATOR_FF             
0x00000692      1  FF                                                   TERMINATOR_FF             
0x00000693      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000695      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000697      1  FF                                                   TERMINATOR_FF             
0x00000698      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000069A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000069C      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000069E      1  FF                                                   TERMINATOR_FF             
0x0000069F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006A1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006A3      3  F31609                                               IMM16_F3                  u16_be=5641, u16_le=2326
0x000006A6      1  FF                                                   TERMINATOR_FF             
0x000006A7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006A9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000006AB      9  8007495A3930343330                                   LEN8_STRING_CP932         length=7, text="IZ90430"
0x000006B4      1  FF                                                   TERMINATOR_FF             
0x000006B5      1  FF                                                   TERMINATOR_FF             
0x000006B6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006B8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006BA     58  8038817582A88E6F82B382F182AA82CB815B814197F688A4...  LEN8_STRING_CP932         length=56, text="「お姉さんがねー、恋愛相談の先生になって\nあげるからー」"
0x000006F4      1  FF                                                   TERMINATOR_FF             
0x000006F5      1  FF                                                   TERMINATOR_FF             
0x000006F6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006FA      1  FF                                                   TERMINATOR_FF             
0x000006FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006FF      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000701      1  FF                                                   TERMINATOR_FF             
0x00000702      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000704      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000706      3  F300C9                                               IMM16_F3                  u16_be=201, u16_le=51456
0x00000709      1  FF                                                   TERMINATOR_FF             
0x0000070A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000070C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000070E      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000710      1  FF                                                   TERMINATOR_FF             
0x00000711      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000713      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000715      1  FF                                                   TERMINATOR_FF             
0x00000716      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000718      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000071A      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000071C      1  FF                                                   TERMINATOR_FF             
0x0000071D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000071F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000721      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000723      1  FF                                                   TERMINATOR_FF             
0x00000724      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000726      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000728      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000072A      1  FF                                                   TERMINATOR_FF             
0x0000072B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000072D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000072F      1  FF                                                   TERMINATOR_FF             
0x00000730      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000732      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000734      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000736      1  FF                                                   TERMINATOR_FF             
0x00000737      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000739      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000073B      3  F3160A                                               IMM16_F3                  u16_be=5642, u16_le=2582
0x0000073E      1  FF                                                   TERMINATOR_FF             
0x0000073F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000741      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000743      9  8007495A3930343430                                   LEN8_STRING_CP932         length=7, text="IZ90440"
0x0000074C      1  FF                                                   TERMINATOR_FF             
0x0000074D      1  FF                                                   TERMINATOR_FF             
0x0000074E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000750      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000752     98  8060817582DC82B882CB814182AB82DD82CB814182C882C9...  LEN8_STRING_CP932         length=96, text="「まずね、きみね、なに座だっけ？\nあと、血液型～。それからね、お部屋の窓、\nどっち向いてる～？」"
0x000007B4      1  FF                                                   TERMINATOR_FF             
0x000007B5      1  FF                                                   TERMINATOR_FF             
0x000007B6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007B8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007BA      1  FF                                                   TERMINATOR_FF             
0x000007BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007BF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000007C1      1  FF                                                   TERMINATOR_FF             
0x000007C2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007C4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000007C6      3  F3160B                                               IMM16_F3                  u16_be=5643, u16_le=2838
0x000007C9      1  FF                                                   TERMINATOR_FF             
0x000007CA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007CC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007CE     54  8034817582BB82EA814182BA82F182BA82F197F688A4918A...  LEN8_STRING_CP932         length=52, text="「それ、ぜんぜん恋愛相談じゃないと思うんです\nけど」"
0x00000804      1  FF                                                   TERMINATOR_FF             
0x00000805      1  FF                                                   TERMINATOR_FF             
0x00000806      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000808      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000080A      1  FF                                                   TERMINATOR_FF             
0x0000080B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000080D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000080F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000811      1  FF                                                   TERMINATOR_FF             
0x00000812      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000814      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000816      3  F3160C                                               IMM16_F3                  u16_be=5644, u16_le=3094
0x00000819      1  FF                                                   TERMINATOR_FF             
0x0000081A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000081C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000081E     94  805C82C782A48D6C82A682C482E0814190E882A282C582B5...  LEN8_STRING_CP932         length=92, text="どう考えても、占いでしょ？\n女性誌に載ってるやつ。\nもう、この人の考え方ってわかりやすすぎ。"
0x0000087C      1  FF                                                   TERMINATOR_FF             
0x0000087D      1  FF                                                   TERMINATOR_FF             
0x0000087E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000880      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000882      1  FF                                                   TERMINATOR_FF             
0x00000883      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000885      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000887      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000889      1  FF                                                   TERMINATOR_FF             
0x0000088A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000088C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000088E      3  F300CB                                               IMM16_F3                  u16_be=203, u16_le=51968
0x00000891      1  FF                                                   TERMINATOR_FF             
0x00000892      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000894      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000896      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000898      1  FF                                                   TERMINATOR_FF             
0x00000899      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000089B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000089D      1  FF                                                   TERMINATOR_FF             
0x0000089E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008A2      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000008A4      1  FF                                                   TERMINATOR_FF             
0x000008A5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008A7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008A9      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000008AB      1  FF                                                   TERMINATOR_FF             
0x000008AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008AE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000008B0      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000008B2      1  FF                                                   TERMINATOR_FF             
0x000008B3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008B5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008B7      1  FF                                                   TERMINATOR_FF             
0x000008B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008BC      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000008BE      1  FF                                                   TERMINATOR_FF             
0x000008BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008C1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000008C3      3  F3160D                                               IMM16_F3                  u16_be=5645, u16_le=3350
0x000008C6      1  FF                                                   TERMINATOR_FF             
0x000008C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008C9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000008CB      9  8007495A3930343530                                   LEN8_STRING_CP932         length=7, text="IZ90450"
0x000008D4      1  FF                                                   TERMINATOR_FF             
0x000008D5      1  FF                                                   TERMINATOR_FF             
0x000008D6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008D8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008DA     38  8024817582C8816082C9814182A282C182C482E982CC815B...  LEN8_STRING_CP932         length=36, text="「な～に、いってるのー。あたるよー」"
0x00000900      1  FF                                                   TERMINATOR_FF             
0x00000901      1  FF                                                   TERMINATOR_FF             
0x00000902      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000904      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000906      1  FF                                                   TERMINATOR_FF             
0x00000907      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000909      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000090B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000090D      1  FF                                                   TERMINATOR_FF             
0x0000090E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000910      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000912      3  F3020B                                               IMM16_F3                  u16_be=523, u16_le=2818
0x00000915      1  FF                                                   TERMINATOR_FF             
0x00000916      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000918      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000091A      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000091C      1  FF                                                   TERMINATOR_FF             
0x0000091D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000091F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000921      1  FF                                                   TERMINATOR_FF             
0x00000922      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000924      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000926      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000928      1  FF                                                   TERMINATOR_FF             
0x00000929      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000092B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000092D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000092F      1  FF                                                   TERMINATOR_FF             
0x00000930      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000932      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000934      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000936      1  FF                                                   TERMINATOR_FF             
0x00000937      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000939      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000093B      1  FF                                                   TERMINATOR_FF             
0x0000093C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000093E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000940      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000942      1  FF                                                   TERMINATOR_FF             
0x00000943      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000945      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000947      3  F3160E                                               IMM16_F3                  u16_be=5646, u16_le=3606
0x0000094A      1  FF                                                   TERMINATOR_FF             
0x0000094B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000094D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000094F      9  8007495A3930343630                                   LEN8_STRING_CP932         length=7, text="IZ90460"
0x00000958      1  FF                                                   TERMINATOR_FF             
0x00000959      1  FF                                                   TERMINATOR_FF             
0x0000095A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000095C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000095E    128  807E817582BE82C182C4815B8DA18C8E815B814183588365...  LEN8_STRING_CP932         length=126, text="「だってー今月ー、ステキな恋にめぐりあえる\nかもって書いてあったもーん。だからねー、\n王子さまーさがしー、今日バッチリなのー」"
0x000009DE      1  FF                                                   TERMINATOR_FF             
0x000009DF      1  FF                                                   TERMINATOR_FF             
0x000009E0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009E2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009E4      1  FF                                                   TERMINATOR_FF             
0x000009E5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000009EB      1  FF                                                   TERMINATOR_FF             
0x000009EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009EE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009F0      3  F3160F                                               IMM16_F3                  u16_be=5647, u16_le=3862
0x000009F3      1  FF                                                   TERMINATOR_FF             
0x000009F4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009F6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009F8     66  804082A082A0814182BB82A4978882DC82B582BD82A98142...  LEN8_STRING_CP932         length=64, text="ああ、そう来ましたか。\nすっかり忘れてるものだと思ってましたが。"
0x00000A3A      1  FF                                                   TERMINATOR_FF             
0x00000A3B      1  FF                                                   TERMINATOR_FF             
0x00000A3C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A3E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A40      1  FF                                                   TERMINATOR_FF             
0x00000A41      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A43      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A45      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000A47      1  FF                                                   TERMINATOR_FF             
0x00000A48      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A4A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A4C      3  F31610                                               IMM16_F3                  u16_be=5648, u16_le=4118
0x00000A4F      1  FF                                                   TERMINATOR_FF             
0x00000A50      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A52      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A54     22  8014817582A08141905E96CA96DA82C582B782CB8176         LEN8_STRING_CP932         length=20, text="「あ、真面目ですね」"
0x00000A6A      1  FF                                                   TERMINATOR_FF             
0x00000A6B      1  FF                                                   TERMINATOR_FF             
0x00000A6C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A6E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A70      1  FF                                                   TERMINATOR_FF             
0x00000A71      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A73      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A75      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000A77      1  FF                                                   TERMINATOR_FF             
0x00000A78      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A7A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A7C      3  F300C2                                               IMM16_F3                  u16_be=194, u16_le=49664
0x00000A7F      1  FF                                                   TERMINATOR_FF             
0x00000A80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A82      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A84      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000A86      1  FF                                                   TERMINATOR_FF             
0x00000A87      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A89      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A8B      1  FF                                                   TERMINATOR_FF             
0x00000A8C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A8E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A90      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000A92      1  FF                                                   TERMINATOR_FF             
0x00000A93      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A95      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A97      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000A99      1  FF                                                   TERMINATOR_FF             
0x00000A9A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A9C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A9E      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000AA0      1  FF                                                   TERMINATOR_FF             
0x00000AA1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000AA3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AA5      1  FF                                                   TERMINATOR_FF             
0x00000AA6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AA8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AAA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000AAC      1  FF                                                   TERMINATOR_FF             
0x00000AAD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AAF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000AB1      3  F31611                                               IMM16_F3                  u16_be=5649, u16_le=4374
0x00000AB4      1  FF                                                   TERMINATOR_FF             
0x00000AB5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AB7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000AB9      9  8007495A3930343730                                   LEN8_STRING_CP932         length=7, text="IZ90470"
0x00000AC2      1  FF                                                   TERMINATOR_FF             
0x00000AC3      1  FF                                                   TERMINATOR_FF             
0x00000AC4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AC6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AC8     98  80608175837D8357838182BE82E6815B814182A282C282C5...  LEN8_STRING_CP932         length=96, text="「マジメだよー、いつでもー。じゃあね、\nさがし行こっ。今日はね、ぜ～ったい見つかると思うんだ～」"
0x00000B2A      1  FF                                                   TERMINATOR_FF             
0x00000B2B      1  FF                                                   TERMINATOR_FF             
0x00000B2C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B2E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B30      1  FF                                                   TERMINATOR_FF             
0x00000B31      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B33      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B35      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B37      1  FF                                                   TERMINATOR_FF             
0x00000B38      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B3A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B3C      3  F31612                                               IMM16_F3                  u16_be=5650, u16_le=4630
0x00000B3F      1  FF                                                   TERMINATOR_FF             
0x00000B40      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B42      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B44     82  805090E882A282CC82C682A882E882BE82C6814182CB8142...  LEN8_STRING_CP932         length=80, text="占いのとおりだと、ね。\n……しかも、めぐりあうとかじゃなく、見つかるなんだよな。"
0x00000B96      1  FF                                                   TERMINATOR_FF             
0x00000B97      1  FF                                                   TERMINATOR_FF             
0x00000B98      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B9A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B9C      1  FF                                                   TERMINATOR_FF             
0x00000B9D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B9F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BA1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000BA3      1  FF                                                   TERMINATOR_FF             
0x00000BA4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BA6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000BA8      3  F31613                                               IMM16_F3                  u16_be=5651, u16_le=4886
0x00000BAB      1  FF                                                   TERMINATOR_FF             
0x00000BAC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BAE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BB0     48  802E82C882F182BE82A982CB8141978E82C682B595A882B3...  LEN8_STRING_CP932         length=46, text="なんだかね、落とし物さがしに行くみたいだけど。"
0x00000BE0      1  FF                                                   TERMINATOR_FF             
0x00000BE1      1  FF                                                   TERMINATOR_FF             
0x00000BE2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000BE4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000BE6      1  FF                                                   TERMINATOR_FF             
0x00000BE7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BE9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BEB      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000BED      1  FF                                                   TERMINATOR_FF             
0x00000BEE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BF0      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000BF2     14  800C50533249303136612E62696E                         LEN8_STRING_CP932         length=12, text="PS2I016a.bin"
0x00000C00      1  FF                                                   TERMINATOR_FF             
0x00000C01      1  FF                                                   TERMINATOR_FF             
0x00000C02      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C04      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C06      1  FF                                                   TERMINATOR_FF             
0x00000C07      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000C09      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000C0B      1  FF                                                   TERMINATOR_FF             
0x00000C0C      2  000C                                                 WORD_00XX                 u16_be=12, low_byte=12
0x00000C0E      1  16                                                   OPAQUE_RAW_BYTES          bytes=16
0x00000C0F      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x00000C11      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000C13      2  000C                                                 WORD_00XX                 u16_be=12, low_byte=12
0x00000C15      1  07                                                   OPAQUE_RAW_BYTES          bytes=07
0x00000C16      1  FF                                                   TERMINATOR_FF             
0x00000C17      1  FF                                                   TERMINATOR_FF             
