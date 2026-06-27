; Full conservative disassembly for PS2A023A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      3  F300A0                                               IMM16_F3                  u16_be=160, u16_le=40960
0x0000000E      1  FF                                                   TERMINATOR_FF             
0x0000000F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000011      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000013      1  FF                                                   TERMINATOR_FF             
0x00000014      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000016      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000018      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000001A      1  FF                                                   TERMINATOR_FF             
0x0000001B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000001D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000001F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000021      1  FF                                                   TERMINATOR_FF             
0x00000022      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000024      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000026      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000028      1  FF                                                   TERMINATOR_FF             
0x00000029      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000002B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000002D      1  FF                                                   TERMINATOR_FF             
0x0000002E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000030      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000032      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000034      1  FF                                                   TERMINATOR_FF             
0x00000035      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000037      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000039      3  F304CD                                               IMM16_F3                  u16_be=1229, u16_le=52484
0x0000003C      1  FF                                                   TERMINATOR_FF             
0x0000003D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000003F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000041     66  804082BB82B182CD814196E982CC837A815B838082BE82C1...  LEN8_STRING_CP932         length=64, text="そこは、夜のホームだった。\nぼくはいつのまにか、駅まで来ていた。"
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
0x00000095      3  F304CE                                               IMM16_F3                  u16_be=1230, u16_le=52740
0x00000098      1  FF                                                   TERMINATOR_FF             
0x00000099      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000009B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000009D    108  806A82DC82BE8DAC979082B582C482A282E981425C6E82C7...  LEN8_STRING_CP932         length=106, text="まだ混乱している。\nどうやって学校を出たのかもおぼえていない。\nどうやってここまで来たのかおぼえていない。"
0x00000109      1  FF                                                   TERMINATOR_FF             
0x0000010A      1  FF                                                   TERMINATOR_FF             
0x0000010B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000010D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000010F      1  FF                                                   TERMINATOR_FF             
0x00000110      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000112      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000114      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000116      1  FF                                                   TERMINATOR_FF             
0x00000117      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000119      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000011B      3  F304CF                                               IMM16_F3                  u16_be=1231, u16_le=52996
0x0000011E      1  FF                                                   TERMINATOR_FF             
0x0000011F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000121      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000123     14  800C8DF68A6F82BE82C182BD8142                         LEN8_STRING_CP932         length=12, text="錯覚だった。"
0x00000131      1  FF                                                   TERMINATOR_FF             
0x00000132      1  FF                                                   TERMINATOR_FF             
0x00000133      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000135      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000137      1  FF                                                   TERMINATOR_FF             
0x00000138      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000013A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000013C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000013E      1  FF                                                   TERMINATOR_FF             
0x0000013F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000141      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000143      3  F304D0                                               IMM16_F3                  u16_be=1232, u16_le=53252
0x00000146      1  FF                                                   TERMINATOR_FF             
0x00000147      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000149      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000014B    116  80728B438E9D82BF82AA978E82BF82C282A982C882A282DC...  LEN8_STRING_CP932         length=114, text="気持ちが落ちつかないまま、ぼくは辺りを見回す。帰宅ラッシュの、当たり前の風景。\n胸から、ためていた息があふれ出す。"
0x000001BF      1  FF                                                   TERMINATOR_FF             
0x000001C0      1  FF                                                   TERMINATOR_FF             
0x000001C1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001C3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001C5      1  FF                                                   TERMINATOR_FF             
0x000001C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001CA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001CC      1  FF                                                   TERMINATOR_FF             
0x000001CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001CF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001D1      3  F304D1                                               IMM16_F3                  u16_be=1233, u16_le=53508
0x000001D4      1  FF                                                   TERMINATOR_FF             
0x000001D5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001D7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001D9     46  802C82C782A482A982B582C482A282BD81425C6E82C882C9...  LEN8_STRING_CP932         length=44, text="どうかしていた。\nなにを焦っていたのだろう。"
0x00000207      1  FF                                                   TERMINATOR_FF             
0x00000208      1  FF                                                   TERMINATOR_FF             
0x00000209      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000020B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000020D      1  FF                                                   TERMINATOR_FF             
0x0000020E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000210      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000212      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000214      1  FF                                                   TERMINATOR_FF             
0x00000215      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000217      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000219      3  F304D2                                               IMM16_F3                  u16_be=1234, u16_le=53764
0x0000021C      1  FF                                                   TERMINATOR_FF             
0x0000021D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000021F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000221     28  801A82C882C982AA82A082C182BD82C182C482A282A482F1...  LEN8_STRING_CP932         length=26, text="なにがあったっていうんだ？"
0x0000023D      1  FF                                                   TERMINATOR_FF             
0x0000023E      1  FF                                                   TERMINATOR_FF             
0x0000023F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000241      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000243      1  FF                                                   TERMINATOR_FF             
0x00000244      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000246      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000248      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000024A      1  FF                                                   TERMINATOR_FF             
0x0000024B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000024D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000024F      3  F304D3                                               IMM16_F3                  u16_be=1235, u16_le=54020
0x00000252      1  FF                                                   TERMINATOR_FF             
0x00000253      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000255      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000257     32  801E8163816382C882C982E08B4E82AB82C482C882F182A9...  LEN8_STRING_CP932         length=30, text="……なにも起きてなんかいない。"
0x00000277      1  FF                                                   TERMINATOR_FF             
0x00000278      1  FF                                                   TERMINATOR_FF             
0x00000279      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000027B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000027D      1  FF                                                   TERMINATOR_FF             
0x0000027E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000280      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000282      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000284      1  FF                                                   TERMINATOR_FF             
0x00000285      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000287      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000289      3  F304D4                                               IMM16_F3                  u16_be=1236, u16_le=54276
0x0000028C      1  FF                                                   TERMINATOR_FF             
0x0000028D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000028F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000291    104  806682DA82AD82AA814182DA82C182C682B582C482A282E9...  LEN8_STRING_CP932         length=102, text="ぼくが、ぼっとしているあいだに、みんなが\n帰ってしまっただけだ。\n玉とも、単にすれ違ってしまっただけ。"
0x000002F9      1  FF                                                   TERMINATOR_FF             
0x000002FA      1  FF                                                   TERMINATOR_FF             
0x000002FB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002FD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002FF      1  FF                                                   TERMINATOR_FF             
0x00000300      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000302      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000304      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000306      1  FF                                                   TERMINATOR_FF             
0x00000307      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000309      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000030B      3  F304D5                                               IMM16_F3                  u16_be=1237, u16_le=54532
0x0000030E      1  FF                                                   TERMINATOR_FF             
0x0000030F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000311      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000313     86  80548BCA82CC834A836F839382AA8FC182A682C482A282BD...  LEN8_STRING_CP932         length=84, text="玉のカバンが消えていたのも……。\n錯覚だったのだ。\nうん……確かめたわけではないし。"
0x00000369      1  FF                                                   TERMINATOR_FF             
0x0000036A      1  FF                                                   TERMINATOR_FF             
0x0000036B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000036D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000036F      1  FF                                                   TERMINATOR_FF             
0x00000370      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000372      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000374      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000376      1  FF                                                   TERMINATOR_FF             
0x00000377      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000379      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000037B      3  F304D6                                               IMM16_F3                  u16_be=1238, u16_le=54788
0x0000037E      1  FF                                                   TERMINATOR_FF             
0x0000037F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000381      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000383     54  80348CA982BD814182C68E7682C182C482A282BD82BE82AF...  LEN8_STRING_CP932         length=52, text="見た、と思っていただけで、最初っから\nなかったんだ。"
0x000003B9      1  FF                                                   TERMINATOR_FF             
0x000003BA      1  FF                                                   TERMINATOR_FF             
0x000003BB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003BD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003BF      1  FF                                                   TERMINATOR_FF             
0x000003C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003C2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003C4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000003C6      1  FF                                                   TERMINATOR_FF             
0x000003C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003C9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000003CB      3  F304D7                                               IMM16_F3                  u16_be=1239, u16_le=55044
0x000003CE      1  FF                                                   TERMINATOR_FF             
0x000003CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003D1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003D3     46  802C82BB82A48D6C82A682EA82CE814195738E768B6382C8...  LEN8_STRING_CP932         length=44, text="そう考えれば、不思議なことなんてなにもない。"
0x00000401      1  FF                                                   TERMINATOR_FF             
0x00000402      1  FF                                                   TERMINATOR_FF             
0x00000403      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000405      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000407      1  FF                                                   TERMINATOR_FF             
0x00000408      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000040A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000040C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000040E      1  FF                                                   TERMINATOR_FF             
0x0000040F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000411      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000413      3  F304D8                                               IMM16_F3                  u16_be=1240, u16_le=55300
0x00000416      1  FF                                                   TERMINATOR_FF             
0x00000417      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000419      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000041B     36  802282C582E08163816382A082CC82C682AB8AB482B682BD...  LEN8_STRING_CP932         length=34, text="でも……あのとき感じた気配は……。"
0x0000043F      1  FF                                                   TERMINATOR_FF             
0x00000440      1  FF                                                   TERMINATOR_FF             
0x00000441      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000443      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000445      1  FF                                                   TERMINATOR_FF             
0x00000446      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000448      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000044A      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000044C      1  FF                                                   TERMINATOR_FF             
0x0000044D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000044F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000451      3  F3027D                                               IMM16_F3                  u16_be=637, u16_le=32002
0x00000454      1  FF                                                   TERMINATOR_FF             
0x00000455      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000457      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000459      3  F300A0                                               IMM16_F3                  u16_be=160, u16_le=40960
0x0000045C      1  FF                                                   TERMINATOR_FF             
0x0000045D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000045F      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00000461      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00000463      1  FF                                                   TERMINATOR_FF             
0x00000464      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000466      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00000468      2  F23C                                                 IMM8_F2                   u8=60, s8=60
0x0000046A      1  FF                                                   TERMINATOR_FF             
0x0000046B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000046D      2  0006                                                 WORD_00XX                 u16_be=6, low_byte=6
0x0000046F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000471      1  FF                                                   TERMINATOR_FF             
0x00000472      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000474      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000476      8  800673652D653333                                     LEN8_STRING_CP932         length=6, text="se-e33"
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
0x00000490      3  F304D9                                               IMM16_F3                  u16_be=1241, u16_le=55556
0x00000493      1  FF                                                   TERMINATOR_FF             
0x00000494      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000496      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000498     84  805282BB82F182C882C682AB814182A082CC8FAD8F9782CC...  LEN8_STRING_CP932         length=82, text="そんなとき、あの少女の面影が浮かぶ。\nあの朝、ホームにひとりっきりだった『彼女』。"
0x000004EC      1  FF                                                   TERMINATOR_FF             
0x000004ED      1  FF                                                   TERMINATOR_FF             
0x000004EE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004F0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004F2      1  FF                                                   TERMINATOR_FF             
0x000004F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004F7      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000004F9      1  FF                                                   TERMINATOR_FF             
0x000004FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004FC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004FE      3  F300A0                                               IMM16_F3                  u16_be=160, u16_le=40960
0x00000501      1  FF                                                   TERMINATOR_FF             
0x00000502      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000504      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000506      1  FF                                                   TERMINATOR_FF             
0x00000507      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000509      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000050B      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000050D      1  FF                                                   TERMINATOR_FF             
0x0000050E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000510      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000512      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000514      1  FF                                                   TERMINATOR_FF             
0x00000515      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000517      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000519      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000051B      1  FF                                                   TERMINATOR_FF             
0x0000051C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000051E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000520      1  FF                                                   TERMINATOR_FF             
0x00000521      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000523      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000525      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000527      1  FF                                                   TERMINATOR_FF             
0x00000528      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000052A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000052C      3  F304DA                                               IMM16_F3                  u16_be=1242, u16_le=55812
0x0000052F      1  FF                                                   TERMINATOR_FF             
0x00000530      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000532      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000534     24  801682C782A482B582C482B182F182C882C682AB82C98148     LEN8_STRING_CP932         length=22, text="どうしてこんなときに？"
0x0000054C      1  FF                                                   TERMINATOR_FF             
0x0000054D      1  FF                                                   TERMINATOR_FF             
0x0000054E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000550      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000552      1  FF                                                   TERMINATOR_FF             
0x00000553      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000555      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000557      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000559      1  FF                                                   TERMINATOR_FF             
0x0000055A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000055C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000055E      3  F304DB                                               IMM16_F3                  u16_be=1243, u16_le=56068
0x00000561      1  FF                                                   TERMINATOR_FF             
0x00000562      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000564      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000566     30  801C82DC82B382A98141817794DE8F97817882AA82A282E9...  LEN8_STRING_CP932         length=28, text="まさか、『彼女』がいるのか？"
0x00000584      1  FF                                                   TERMINATOR_FF             
0x00000585      1  FF                                                   TERMINATOR_FF             
0x00000586      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000588      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000058A      1  FF                                                   TERMINATOR_FF             
0x0000058B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000058D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000058F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000591      1  FF                                                   TERMINATOR_FF             
0x00000592      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000594      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000596      3  F304DC                                               IMM16_F3                  u16_be=1244, u16_le=56324
0x00000599      1  FF                                                   TERMINATOR_FF             
0x0000059A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000059C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000059E     88  80568EB882A282A982AF82C482A282E990A28A4582CC8EE8...  LEN8_STRING_CP932         length=86, text="失いかけている世界の手ざわりが、『彼女』の\n瞳にはあった。\n目が『彼女』を捜していた。"
0x000005F6      1  FF                                                   TERMINATOR_FF             
0x000005F7      1  FF                                                   TERMINATOR_FF             
0x000005F8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005FA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005FC      1  FF                                                   TERMINATOR_FF             
0x000005FD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005FF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000601      8  800653452D533139                                     LEN8_STRING_CP932         length=6, text="SE-S19"
0x00000609      1  FF                                                   TERMINATOR_FF             
0x0000060A      1  FF                                                   TERMINATOR_FF             
0x0000060B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000060D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000060F      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00000611      1  FF                                                   TERMINATOR_FF             
0x00000612      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000614      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000616      1  FF                                                   TERMINATOR_FF             
0x00000617      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000619      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000061B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000061D      1  FF                                                   TERMINATOR_FF             
0x0000061E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000620      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000622      3  F304DD                                               IMM16_F3                  u16_be=1245, u16_le=56580
0x00000625      1  FF                                                   TERMINATOR_FF             
0x00000626      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000628      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000062A     80  804E975B95FB82CC837A815B838082CD82A282C282E082C6...  LEN8_STRING_CP932         length=78, text="夕方のホームはいつもと変わらぬ混雑。\nもちろん、『彼女』の姿があるわけもない。"
0x0000067A      1  FF                                                   TERMINATOR_FF             
0x0000067B      1  FF                                                   TERMINATOR_FF             
0x0000067C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000067E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000680      1  FF                                                   TERMINATOR_FF             
0x00000681      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000683      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000685      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000687      1  FF                                                   TERMINATOR_FF             
0x00000688      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000068A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000068C      3  F304DE                                               IMM16_F3                  u16_be=1246, u16_le=56836
0x0000068F      1  FF                                                   TERMINATOR_FF             
0x00000690      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000692      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000694     54  803482C782A482A982B582C482E981425C6E82DA82AD82CD...  LEN8_STRING_CP932         length=52, text="どうかしてる。\nぼくは……正気を失いかけているのか？"
0x000006CA      1  FF                                                   TERMINATOR_FF             
0x000006CB      1  FF                                                   TERMINATOR_FF             
0x000006CC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006CE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006D0      1  FF                                                   TERMINATOR_FF             
0x000006D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006D5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000006D7      1  FF                                                   TERMINATOR_FF             
0x000006D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006DA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006DC      3  F304DF                                               IMM16_F3                  u16_be=1247, u16_le=57092
0x000006DF      1  FF                                                   TERMINATOR_FF             
0x000006E0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006E2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006E4     70  804493648ED482AA837A815B838082C982B782D782E882B1...  LEN8_STRING_CP932         length=68, text="電車がホームにすべりこんでくる。\nその瞬間、目が異様な物を映し出す。"
0x0000072A      1  FF                                                   TERMINATOR_FF             
0x0000072B      1  FF                                                   TERMINATOR_FF             
0x0000072C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000072E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000730      1  FF                                                   TERMINATOR_FF             
0x00000731      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000733      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000735      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x00000737      1  FF                                                   TERMINATOR_FF             
0x00000738      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000073A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000073C      1  FF                                                   TERMINATOR_FF             
0x0000073D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000073F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000741      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000743      1  FF                                                   TERMINATOR_FF             
0x00000744      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000746      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000748      3  F300A1                                               IMM16_F3                  u16_be=161, u16_le=41216
0x0000074B      1  FF                                                   TERMINATOR_FF             
0x0000074C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000074E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000750      1  FF                                                   TERMINATOR_FF             
0x00000751      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000753      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000755      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00000757      1  FF                                                   TERMINATOR_FF             
0x00000758      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000075A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000075C      2  F21C                                                 IMM8_F2                   u8=28, s8=28
0x0000075E      1  FF                                                   TERMINATOR_FF             
0x0000075F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000761      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000763      1  FF                                                   TERMINATOR_FF             
0x00000764      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000766      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000768      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000076A      1  FF                                                   TERMINATOR_FF             
0x0000076B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000076D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000076F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000771      1  FF                                                   TERMINATOR_FF             
0x00000772      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000774      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000776      2  F23C                                                 IMM8_F2                   u8=60, s8=60
0x00000778      1  FF                                                   TERMINATOR_FF             
0x00000779      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000077B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000077D      1  FF                                                   TERMINATOR_FF             
0x0000077E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000780      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000782      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000784      1  FF                                                   TERMINATOR_FF             
0x00000785      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000787      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000789      3  F304E0                                               IMM16_F3                  u16_be=1248, u16_le=57348
0x0000078C      1  FF                                                   TERMINATOR_FF             
0x0000078D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000078F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000791     80  804E906C82B282DD82CC928682C9906C82C582CD82C882A2...  LEN8_STRING_CP932         length=78, text="人ごみの中に人ではないものがいた。\n怪物、妖怪、魔物。\n呼び名はどうでもいい。"
0x000007E1      1  FF                                                   TERMINATOR_FF             
0x000007E2      1  FF                                                   TERMINATOR_FF             
0x000007E3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007E5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007E7      1  FF                                                   TERMINATOR_FF             
0x000007E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007EC      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x000007EE      1  FF                                                   TERMINATOR_FF             
0x000007EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007F3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007F5      1  FF                                                   TERMINATOR_FF             
0x000007F6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007FA      1  FF                                                   TERMINATOR_FF             
0x000007FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007FF      2  F209                                                 IMM8_F2                   u8=9, s8=9
0x00000801      1  FF                                                   TERMINATOR_FF             
0x00000802      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000804      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000806      2  F273                                                 IMM8_F2                   u8=115, s8=115
0x00000808      1  FF                                                   TERMINATOR_FF             
0x00000809      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000080B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000080D      1  FF                                                   TERMINATOR_FF             
0x0000080E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000810      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000812      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000814      1  FF                                                   TERMINATOR_FF             
0x00000815      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000817      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000819      3  F300A0                                               IMM16_F3                  u16_be=160, u16_le=40960
0x0000081C      1  FF                                                   TERMINATOR_FF             
0x0000081D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000081F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000821      1  FF                                                   TERMINATOR_FF             
0x00000822      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000824      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000826      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000828      1  FF                                                   TERMINATOR_FF             
0x00000829      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000082B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000082D      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000082F      1  FF                                                   TERMINATOR_FF             
0x00000830      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000832      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000834      2  F23C                                                 IMM8_F2                   u8=60, s8=60
0x00000836      1  FF                                                   TERMINATOR_FF             
0x00000837      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000839      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000083B      1  FF                                                   TERMINATOR_FF             
0x0000083C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000083E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000840      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000842      1  FF                                                   TERMINATOR_FF             
0x00000843      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000845      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000847      3  F304E1                                               IMM16_F3                  u16_be=1249, u16_le=57604
0x0000084A      1  FF                                                   TERMINATOR_FF             
0x0000084B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000084D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000084F     52  8032906C8AD482C582CD82C882A2817782C882C982A98178...  LEN8_STRING_CP932         length=50, text="人間ではない『なにか』が人の中にまぎれこんでいた。"
0x00000883      1  FF                                                   TERMINATOR_FF             
0x00000884      1  FF                                                   TERMINATOR_FF             
0x00000885      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000887      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000889      1  FF                                                   TERMINATOR_FF             
0x0000088A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000088C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000088E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000890      1  FF                                                   TERMINATOR_FF             
0x00000891      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000893      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000895      3  F304E2                                               IMM16_F3                  u16_be=1250, u16_le=57860
0x00000898      1  FF                                                   TERMINATOR_FF             
0x00000899      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000089B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000089D     60  803A82AF82EA82C78141817782C882C982A9817882CD8141...  LEN8_STRING_CP932         length=58, text="けれど、『なにか』は、すぐに人ごみの中に\n消えてしまった。"
0x000008D9      1  FF                                                   TERMINATOR_FF             
0x000008DA      1  FF                                                   TERMINATOR_FF             
0x000008DB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008DD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008DF      1  FF                                                   TERMINATOR_FF             
0x000008E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008E4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000008E6      1  FF                                                   TERMINATOR_FF             
0x000008E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008E9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000008EB      3  F304E3                                               IMM16_F3                  u16_be=1251, u16_le=58116
0x000008EE      1  FF                                                   TERMINATOR_FF             
0x000008EF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008F3     64  803E82BB82EA82CD88EA8F7582CC8CB682BE82C182BD8142...  LEN8_STRING_CP932         length=62, text="それは一瞬の幻だった。\n周りの誰も『なにか』なんて見ていない。"
0x00000933      1  FF                                                   TERMINATOR_FF             
0x00000934      1  FF                                                   TERMINATOR_FF             
0x00000935      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000937      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000939      1  FF                                                   TERMINATOR_FF             
0x0000093A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000093C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000093E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000940      1  FF                                                   TERMINATOR_FF             
0x00000941      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000943      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000945      3  F304E4                                               IMM16_F3                  u16_be=1252, u16_le=58372
0x00000948      1  FF                                                   TERMINATOR_FF             
0x00000949      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000094B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000094D     56  80369153906782C982B582D182EA82CC82E682A482C88AB4...  LEN8_STRING_CP932         length=54, text="全身にしびれのような感覚が走る。\nこれは……いったい。"
0x00000985      1  FF                                                   TERMINATOR_FF             
0x00000986      1  FF                                                   TERMINATOR_FF             
0x00000987      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000989      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000098B      1  FF                                                   TERMINATOR_FF             
0x0000098C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000098E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000990      2  F209                                                 IMM8_F2                   u8=9, s8=9
0x00000992      1  FF                                                   TERMINATOR_FF             
0x00000993      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000995      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000997      2  F267                                                 IMM8_F2                   u8=103, s8=103
0x00000999      1  FF                                                   TERMINATOR_FF             
0x0000099A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000099C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000099E      1  FF                                                   TERMINATOR_FF             
0x0000099F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009A3      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000009A5      1  FF                                                   TERMINATOR_FF             
0x000009A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009A8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009AA      3  F30096                                               IMM16_F3                  u16_be=150, u16_le=38400
0x000009AD      1  FF                                                   TERMINATOR_FF             
0x000009AE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009B0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009B2      1  FF                                                   TERMINATOR_FF             
0x000009B3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009B7      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000009B9      1  FF                                                   TERMINATOR_FF             
0x000009BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009BC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009BE      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000009C0      1  FF                                                   TERMINATOR_FF             
0x000009C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009C3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000009C5      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x000009C7      1  FF                                                   TERMINATOR_FF             
0x000009C8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009CA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009CC      1  FF                                                   TERMINATOR_FF             
0x000009CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009CF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009D1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000009D3      1  FF                                                   TERMINATOR_FF             
0x000009D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009D6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009D8      3  F304E5                                               IMM16_F3                  u16_be=1253, u16_le=58628
0x000009DB      1  FF                                                   TERMINATOR_FF             
0x000009DC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009DE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009E0     32  801E817794DE8F97817882CC814182A082CC93B582F08E76...  LEN8_STRING_CP932         length=30, text="『彼女』の、あの瞳を思い出す。"
0x00000A00      1  FF                                                   TERMINATOR_FF             
0x00000A01      1  FF                                                   TERMINATOR_FF             
0x00000A02      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A04      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A06      1  FF                                                   TERMINATOR_FF             
0x00000A07      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A09      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A0B      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00000A0D      1  FF                                                   TERMINATOR_FF             
0x00000A0E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A10      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A12      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A14      1  FF                                                   TERMINATOR_FF             
0x00000A15      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A17      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A19      1  FF                                                   TERMINATOR_FF             
0x00000A1A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A1C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A1E      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000A20      1  FF                                                   TERMINATOR_FF             
0x00000A21      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A23      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A25      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00000A27      1  FF                                                   TERMINATOR_FF             
0x00000A28      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A2A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A2C      1  FF                                                   TERMINATOR_FF             
0x00000A2D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A2F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A31      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000A33      1  FF                                                   TERMINATOR_FF             
0x00000A34      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A36      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A38      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000A3A      1  FF                                                   TERMINATOR_FF             
0x00000A3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A3D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A3F      2  F23C                                                 IMM8_F2                   u8=60, s8=60
0x00000A41      1  FF                                                   TERMINATOR_FF             
0x00000A42      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A44      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A46      1  FF                                                   TERMINATOR_FF             
0x00000A47      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A49      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A4B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000A4D      1  FF                                                   TERMINATOR_FF             
0x00000A4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A50      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A52      3  F304E6                                               IMM16_F3                  u16_be=1254, u16_le=58884
0x00000A55      1  FF                                                   TERMINATOR_FF             
0x00000A56      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A58      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A5A     52  8032817794DE8F97817882CD8CB682BE82C182BD81425C6E...  LEN8_STRING_CP932         length=50, text="『彼女』は幻だった。\nでも、ここは……どうなんだ？"
0x00000A8E      1  FF                                                   TERMINATOR_FF             
0x00000A8F      1  FF                                                   TERMINATOR_FF             
0x00000A90      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A92      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A94      1  FF                                                   TERMINATOR_FF             
0x00000A95      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A97      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A99      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000A9B      1  FF                                                   TERMINATOR_FF             
0x00000A9C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A9E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000AA0      3  F33F31                                               IMM16_F3                  u16_be=16177, u16_le=12607
0x00000AA3      1  FF                                                   TERMINATOR_FF             
0x00000AA4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AA6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AA8     34  802082DA82AD82BD82BF82CC968893FA82CD814182A282C1...  LEN8_STRING_CP932         length=32, text="ぼくたちの毎日は、いったい……。"
0x00000ACA      1  FF                                                   TERMINATOR_FF             
0x00000ACB      1  FF                                                   TERMINATOR_FF             
0x00000ACC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000ACE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AD0      1  FF                                                   TERMINATOR_FF             
0x00000AD1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AD3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AD5      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00000AD7      1  FF                                                   TERMINATOR_FF             
0x00000AD8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000ADA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000ADC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000ADE      1  FF                                                   TERMINATOR_FF             
0x00000ADF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000AE1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AE3      1  FF                                                   TERMINATOR_FF             
0x00000AE4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AE6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AE8      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000AEA      1  FF                                                   TERMINATOR_FF             
0x00000AEB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AEF      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
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
0x00000B15      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00000B17      1  FF                                                   TERMINATOR_FF             
0x00000B18      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B1A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B1C      1  FF                                                   TERMINATOR_FF             
0x00000B1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B1F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B21      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000B23      1  FF                                                   TERMINATOR_FF             
0x00000B24      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B26      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000B28     14  800C50533241303234612E62696E                         LEN8_STRING_CP932         length=12, text="PS2A024a.bin"
0x00000B36      1  FF                                                   TERMINATOR_FF             
0x00000B37      1  FF                                                   TERMINATOR_FF             
0x00000B38      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B3A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B3C      1  FF                                                   TERMINATOR_FF             
0x00000B3D      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000B3F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000B41      1  FF                                                   TERMINATOR_FF             
0x00000B42      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00000B44      1  4C                                                   OPAQUE_RAW_BYTES          bytes=4C
0x00000B45      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x00000B47      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000B49      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00000B4B      1  3D                                                   OPAQUE_RAW_BYTES          bytes=3D
0x00000B4C      1  FF                                                   TERMINATOR_FF             
0x00000B4D      1  FF                                                   TERMINATOR_FF             
