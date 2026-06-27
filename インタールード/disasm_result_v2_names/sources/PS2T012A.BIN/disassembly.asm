; Full conservative disassembly for PS2T012A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      3  F30212                                               IMM16_F3                  u16_be=530, u16_le=4610
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
0x0000001F      2  F221                                                 IMM8_F2                   u8=33, s8=33
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
0x0000004C      3  F331A5                                               IMM16_F3                  u16_be=12709, u16_le=42289
0x0000004F      1  FF                                                   TERMINATOR_FF             
0x00000050      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000052      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000054     36  8022817582C882F182C593A682B082E982A982C8815B8141...  LEN8_STRING_CP932         length=34, text="「なんで逃げるかなー、お前はよー」"
0x00000078      1  FF                                                   TERMINATOR_FF             
0x00000079      1  FF                                                   TERMINATOR_FF             
0x0000007A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000007C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000007E      1  FF                                                   TERMINATOR_FF             
0x0000007F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000081      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000083      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000085      1  FF                                                   TERMINATOR_FF             
0x00000086      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000088      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000008A      3  F331A6                                               IMM16_F3                  u16_be=12710, u16_le=42545
0x0000008D      1  FF                                                   TERMINATOR_FF             
0x0000008E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000090      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000092      9  80074D543031353830                                   LEN8_STRING_CP932         length=7, text="MT01580"
0x0000009B      1  FF                                                   TERMINATOR_FF             
0x0000009C      1  FF                                                   TERMINATOR_FF             
0x0000009D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000009F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000A1     16  800E817582DE82A38160816381638176                     LEN8_STRING_CP932         length=14, text="「むぅ～……」"
0x000000B1      1  FF                                                   TERMINATOR_FF             
0x000000B2      1  FF                                                   TERMINATOR_FF             
0x000000B3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000B5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000B7      1  FF                                                   TERMINATOR_FF             
0x000000B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000BC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000000BE      1  FF                                                   TERMINATOR_FF             
0x000000BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000C1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000000C3      3  F331A7                                               IMM16_F3                  u16_be=12711, u16_le=42801
0x000000C6      1  FF                                                   TERMINATOR_FF             
0x000000C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000C9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000CB     20  8012817582B682E181418E6E82DF82E982A98176             LEN8_STRING_CP932         length=18, text="「じゃ、始めるか」"
0x000000DF      1  FF                                                   TERMINATOR_FF             
0x000000E0      1  FF                                                   TERMINATOR_FF             
0x000000E1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000E3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000E5      1  FF                                                   TERMINATOR_FF             
0x000000E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000EA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000000EC      1  FF                                                   TERMINATOR_FF             
0x000000ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000EF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000000F1      3  F331A8                                               IMM16_F3                  u16_be=12712, u16_le=43057
0x000000F4      1  FF                                                   TERMINATOR_FF             
0x000000F5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000F7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000000F9      9  80074D543031353930                                   LEN8_STRING_CP932         length=7, text="MT01590"
0x00000102      1  FF                                                   TERMINATOR_FF             
0x00000103      1  FF                                                   TERMINATOR_FF             
0x00000104      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000106      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000108     22  8014817583788393834C838782AB82E7816082A28176         LEN8_STRING_CP932         length=20, text="「ベンキョきら～い」"
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
0x00000130      3  F331A9                                               IMM16_F3                  u16_be=12713, u16_le=43313
0x00000133      1  FF                                                   TERMINATOR_FF             
0x00000134      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000136      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000138     16  800E817582A082AB82E782DF82EB8176                     LEN8_STRING_CP932         length=14, text="「あきらめろ」"
0x00000148      1  FF                                                   TERMINATOR_FF             
0x00000149      1  FF                                                   TERMINATOR_FF             
0x0000014A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000014C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000014E      1  FF                                                   TERMINATOR_FF             
0x0000014F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000151      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000153      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000155      1  FF                                                   TERMINATOR_FF             
0x00000156      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000158      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000015A      3  F331AA                                               IMM16_F3                  u16_be=12714, u16_le=43569
0x0000015D      1  FF                                                   TERMINATOR_FF             
0x0000015E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000160      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000162      9  80074D543031363030                                   LEN8_STRING_CP932         length=7, text="MT01600"
0x0000016B      1  FF                                                   TERMINATOR_FF             
0x0000016C      1  FF                                                   TERMINATOR_FF             
0x0000016D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000016F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000171     16  800E81758163816382DE82A381608176                     LEN8_STRING_CP932         length=14, text="「……むぅ～」"
0x00000181      1  FF                                                   TERMINATOR_FF             
0x00000182      1  FF                                                   TERMINATOR_FF             
0x00000183      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000185      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000187      1  FF                                                   TERMINATOR_FF             
0x00000188      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000018A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000018C      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000018E      1  FF                                                   TERMINATOR_FF             
0x0000018F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000191      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000193      2  F250                                                 IMM8_F2                   u8=80, s8=80
0x00000195      1  FF                                                   TERMINATOR_FF             
0x00000196      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000198      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000019A      1  FF                                                   TERMINATOR_FF             
0x0000019B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000019D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000019F      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000001A1      1  FF                                                   TERMINATOR_FF             
0x000001A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001A4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001A6      3  F301B0                                               IMM16_F3                  u16_be=432, u16_le=45057
0x000001A9      1  FF                                                   TERMINATOR_FF             
0x000001AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001AC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000001AE      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000001B0      1  FF                                                   TERMINATOR_FF             
0x000001B1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001B3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001B5      1  FF                                                   TERMINATOR_FF             
0x000001B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001BA      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000001BC      1  FF                                                   TERMINATOR_FF             
0x000001BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001C1      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000001C3      1  FF                                                   TERMINATOR_FF             
0x000001C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001C6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000001C8      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000001CA      1  FF                                                   TERMINATOR_FF             
0x000001CB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001CD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001CF      1  FF                                                   TERMINATOR_FF             
0x000001D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001D4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001D6      1  FF                                                   TERMINATOR_FF             
0x000001D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001D9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001DB      3  F331AB                                               IMM16_F3                  u16_be=12715, u16_le=43825
0x000001DE      1  FF                                                   TERMINATOR_FF             
0x000001DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001E1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001E3     72  80468BCA82CD834A836F839382A982E78141836D815B8367...  LEN8_STRING_CP932         length=70, text="玉はカバンから、ノートとペンケースをとりだす。\n露骨にいやそうな態度。"
0x0000022B      1  FF                                                   TERMINATOR_FF             
0x0000022C      1  FF                                                   TERMINATOR_FF             
0x0000022D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000022F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000231      1  FF                                                   TERMINATOR_FF             
0x00000232      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000234      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000236      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000238      1  FF                                                   TERMINATOR_FF             
0x00000239      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000023B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000023D      3  F331AC                                               IMM16_F3                  u16_be=12716, u16_le=44081
0x00000240      1  FF                                                   TERMINATOR_FF             
0x00000241      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000243      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000245      9  80074D543034353430                                   LEN8_STRING_CP932         length=7, text="MT04540"
0x0000024E      1  FF                                                   TERMINATOR_FF             
0x0000024F      1  FF                                                   TERMINATOR_FF             
0x00000250      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000252      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000254     34  8020817582F181608163816383788393834C8387815B82E2...  LEN8_STRING_CP932         length=32, text="「ん～……ベンキョーやーだなっ」"
0x00000276      1  FF                                                   TERMINATOR_FF             
0x00000277      1  FF                                                   TERMINATOR_FF             
0x00000278      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000027A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000027C      1  FF                                                   TERMINATOR_FF             
0x0000027D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000027F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000281      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000283      1  FF                                                   TERMINATOR_FF             
0x00000284      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000286      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000288      3  F331AD                                               IMM16_F3                  u16_be=12717, u16_le=44337
0x0000028B      1  FF                                                   TERMINATOR_FF             
0x0000028C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000028E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000290     18  8010817582A282E282C582E082E282E98176                 LEN8_STRING_CP932         length=16, text="「いやでもやる」"
0x000002A2      1  FF                                                   TERMINATOR_FF             
0x000002A3      1  FF                                                   TERMINATOR_FF             
0x000002A4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002A6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002A8      1  FF                                                   TERMINATOR_FF             
0x000002A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002AD      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000002AF      1  FF                                                   TERMINATOR_FF             
0x000002B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000002B4      3  F331AE                                               IMM16_F3                  u16_be=12718, u16_le=44593
0x000002B7      1  FF                                                   TERMINATOR_FF             
0x000002B8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002BA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002BC      9  80074D543034353530                                   LEN8_STRING_CP932         length=7, text="MT04550"
0x000002C5      1  FF                                                   TERMINATOR_FF             
0x000002C6      1  FF                                                   TERMINATOR_FF             
0x000002C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002C9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002CB     16  800E817582DE82A38160816381638176                     LEN8_STRING_CP932         length=14, text="「むぅ～……」"
0x000002DB      1  FF                                                   TERMINATOR_FF             
0x000002DC      1  FF                                                   TERMINATOR_FF             
0x000002DD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002DF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002E1      1  FF                                                   TERMINATOR_FF             
0x000002E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002E6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000002E8      1  FF                                                   TERMINATOR_FF             
0x000002E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002EB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000002ED      3  F331AF                                               IMM16_F3                  u16_be=12719, u16_le=44849
0x000002F0      1  FF                                                   TERMINATOR_FF             
0x000002F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002F3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002F5     94  805C82E282E98B4382C882AD8141837983938350815B8358...  LEN8_STRING_CP932         length=92, text="やる気なく、ペンケースをあさる。\nケシゴムを見つめたりする。\nしぶしぶシャーペンを手にする。"
0x00000353      1  FF                                                   TERMINATOR_FF             
0x00000354      1  FF                                                   TERMINATOR_FF             
0x00000355      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000357      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000359      1  FF                                                   TERMINATOR_FF             
0x0000035A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000035C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000035E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000360      1  FF                                                   TERMINATOR_FF             
0x00000361      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000363      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000365      3  F331B0                                               IMM16_F3                  u16_be=12720, u16_le=45105
0x00000368      1  FF                                                   TERMINATOR_FF             
0x00000369      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000036B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000036D     42  802882E282E98B4382C882AD836D815B836782C983568383...  LEN8_STRING_CP932         length=40, text="やる気なくノートにシャーペンを近づける。"
0x00000397      1  FF                                                   TERMINATOR_FF             
0x00000398      1  FF                                                   TERMINATOR_FF             
0x00000399      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000039B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000039D      1  FF                                                   TERMINATOR_FF             
0x0000039E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003A0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000003A2      8  800653452D533935                                     LEN8_STRING_CP932         length=6, text="SE-S95"
0x000003AA      1  FF                                                   TERMINATOR_FF             
0x000003AB      1  FF                                                   TERMINATOR_FF             
0x000003AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003B0      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x000003B2      1  FF                                                   TERMINATOR_FF             
0x000003B3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003B5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003B7      1  FF                                                   TERMINATOR_FF             
0x000003B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003BC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000003BE      1  FF                                                   TERMINATOR_FF             
0x000003BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003C1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000003C3      3  F331B1                                               IMM16_F3                  u16_be=12721, u16_le=45361
0x000003C6      1  FF                                                   TERMINATOR_FF             
0x000003C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003C9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003CB     54  803482C882C982A98F9182AB82CD82B682DF82BD82C88141...  LEN8_STRING_CP932         length=52, text="なにか書きはじめたな、と思うと、カミグマ\nだったり。"
0x00000401      1  FF                                                   TERMINATOR_FF             
0x00000402      1  FF                                                   TERMINATOR_FF             
0x00000403      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000405      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000407      1  FF                                                   TERMINATOR_FF             
0x00000408      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000040A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000040C      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000040E      1  FF                                                   TERMINATOR_FF             
0x0000040F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000411      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000413      3  F301B3                                               IMM16_F3                  u16_be=435, u16_le=45825
0x00000416      1  FF                                                   TERMINATOR_FF             
0x00000417      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000419      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000041B      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000041D      1  FF                                                   TERMINATOR_FF             
0x0000041E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000420      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000422      1  FF                                                   TERMINATOR_FF             
0x00000423      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000425      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000427      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000429      1  FF                                                   TERMINATOR_FF             
0x0000042A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000042C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000042E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000430      1  FF                                                   TERMINATOR_FF             
0x00000431      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000433      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000435      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000437      1  FF                                                   TERMINATOR_FF             
0x00000438      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000043A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000043C      1  FF                                                   TERMINATOR_FF             
0x0000043D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000043F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000441      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000443      1  FF                                                   TERMINATOR_FF             
0x00000444      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000446      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000448      3  F331B2                                               IMM16_F3                  u16_be=12722, u16_le=45617
0x0000044B      1  FF                                                   TERMINATOR_FF             
0x0000044C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000044E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000450     48  802E81758163816383568383815B8379839382C4814182C8...  LEN8_STRING_CP932         length=46, text="「……シャーペンて、なんでシャーペンてゆー？」"
0x00000480      1  FF                                                   TERMINATOR_FF             
0x00000481      1  FF                                                   TERMINATOR_FF             
0x00000482      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000484      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000486      9  80074D543034353630                                   LEN8_STRING_CP932         length=7, text="MT04560"
0x0000048F      1  FF                                                   TERMINATOR_FF             
0x00000490      1  FF                                                   TERMINATOR_FF             
0x00000491      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000493      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00000495     28  801A83568383815B8376837983938356838B82CC97AA82BE...  LEN8_STRING_CP932         length=26, text="シャープペンシルの略だから"
0x000004B1      1  FF                                                   TERMINATOR_FF             
0x000004B2      1  FF                                                   TERMINATOR_FF             
0x000004B3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004B5      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x000004B7      3  F331B3                                               IMM16_F3                  u16_be=12723, u16_le=45873
0x000004BA      1  FF                                                   TERMINATOR_FF             
0x000004BB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004BD      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x000004BF     16  800E8DA182CD95D78BAD928682BE82EB                     LEN8_STRING_CP932         length=14, text="今は勉強中だろ"
0x000004CF      1  FF                                                   TERMINATOR_FF             
0x000004D0      1  FF                                                   TERMINATOR_FF             
0x000004D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004D3      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x000004D5      3  F331B4                                               IMM16_F3                  u16_be=12724, u16_le=46129
0x000004D8      1  FF                                                   TERMINATOR_FF             
0x000004D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004DB      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x000004DD      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000004DF      1  FF                                                   TERMINATOR_FF             
0x000004E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004E2      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x000004E4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004E6      1  FF                                                   TERMINATOR_FF             
0x000004E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004EB      2  F213                                                 IMM8_F2                   u8=19, s8=19
0x000004ED      1  FF                                                   TERMINATOR_FF             
0x000004EE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004F0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004F2      1  FF                                                   TERMINATOR_FF             
0x000004F3      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000004F5      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x000004F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004FA      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000004FB      1  FF                                                   TERMINATOR_FF             
0x000004FC      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x000004FE      1  E6                                                   OPAQUE_RAW_BYTES          bytes=E6
0x000004FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000501      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000503      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000505      1  FF                                                   TERMINATOR_FF             
0x00000506      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000508      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000050A      3  F331B5                                               IMM16_F3                  u16_be=12725, u16_le=46385
0x0000050D      1  FF                                                   TERMINATOR_FF             
0x0000050E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000510      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000512     46  802C8175817783568383815B83768145837983938356838B...  LEN8_STRING_CP932         length=44, text="「『シャープ・ペンシル』を略してシャーペン」"
0x00000540      1  FF                                                   TERMINATOR_FF             
0x00000541      1  FF                                                   TERMINATOR_FF             
0x00000542      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000544      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000546      1  FF                                                   TERMINATOR_FF             
0x00000547      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000549      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000054B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000054D      1  FF                                                   TERMINATOR_FF             
0x0000054E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000550      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000552      3  F301B4                                               IMM16_F3                  u16_be=436, u16_le=46081
0x00000555      1  FF                                                   TERMINATOR_FF             
0x00000556      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000558      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000055A      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000055C      1  FF                                                   TERMINATOR_FF             
0x0000055D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000055F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000561      1  FF                                                   TERMINATOR_FF             
0x00000562      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000564      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000566      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000568      1  FF                                                   TERMINATOR_FF             
0x00000569      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000056B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000056D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000056F      1  FF                                                   TERMINATOR_FF             
0x00000570      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000572      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000574      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000576      1  FF                                                   TERMINATOR_FF             
0x00000577      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000579      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000057B      1  FF                                                   TERMINATOR_FF             
0x0000057C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000057E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000580      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000582      1  FF                                                   TERMINATOR_FF             
0x00000583      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000585      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000587      3  F331B6                                               IMM16_F3                  u16_be=12726, u16_le=46641
0x0000058A      1  FF                                                   TERMINATOR_FF             
0x0000058B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000058D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000058F      9  80074D543034353730                                   LEN8_STRING_CP932         length=7, text="MT04570"
0x00000598      1  FF                                                   TERMINATOR_FF             
0x00000599      1  FF                                                   TERMINATOR_FF             
0x0000059A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000059C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000059E     16  800E81758163816382D3815B82F18176                     LEN8_STRING_CP932         length=14, text="「……ふーん」"
0x000005AE      1  FF                                                   TERMINATOR_FF             
0x000005AF      1  FF                                                   TERMINATOR_FF             
0x000005B0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005B2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005B4      1  FF                                                   TERMINATOR_FF             
0x000005B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005B9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000005BB      1  FF                                                   TERMINATOR_FF             
0x000005BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005BE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000005C0      3  F331B7                                               IMM16_F3                  u16_be=12727, u16_le=46897
0x000005C3      1  FF                                                   TERMINATOR_FF             
0x000005C4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005C6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005C8     18  80109153915295B782A282C482C882A28142                 LEN8_STRING_CP932         length=16, text="全然聞いてない。"
0x000005DA      1  FF                                                   TERMINATOR_FF             
0x000005DB      1  FF                                                   TERMINATOR_FF             
0x000005DC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005DE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005E0      1  FF                                                   TERMINATOR_FF             
0x000005E1      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x000005E3      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x000005E5      1  3B                                                   OPAQUE_RAW_BYTES          bytes=3B
0x000005E6      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000005E8      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x000005EB      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000005ED      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000005EE      1  FF                                                   TERMINATOR_FF             
0x000005EF      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x000005F1      1  3B                                                   OPAQUE_RAW_BYTES          bytes=3B
0x000005F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005F6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000005F8      1  FF                                                   TERMINATOR_FF             
0x000005F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005FB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000005FD      3  F331B8                                               IMM16_F3                  u16_be=12728, u16_le=47153
0x00000600      1  FF                                                   TERMINATOR_FF             
0x00000601      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000603      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000605     42  80288175926D82E782C882A282AF82C782E6814195D78BAD...  LEN8_STRING_CP932         length=40, text="「知らないけどよ、勉強には関係ないから」"
0x0000062F      1  FF                                                   TERMINATOR_FF             
0x00000630      1  FF                                                   TERMINATOR_FF             
0x00000631      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000633      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000635      1  FF                                                   TERMINATOR_FF             
0x00000636      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000638      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000063A      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000063C      1  FF                                                   TERMINATOR_FF             
0x0000063D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000063F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000641      3  F301B0                                               IMM16_F3                  u16_be=432, u16_le=45057
0x00000644      1  FF                                                   TERMINATOR_FF             
0x00000645      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000647      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000649      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000064B      1  FF                                                   TERMINATOR_FF             
0x0000064C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000064E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000650      1  FF                                                   TERMINATOR_FF             
0x00000651      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000653      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000655      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000657      1  FF                                                   TERMINATOR_FF             
0x00000658      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000065A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000065C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000065E      1  FF                                                   TERMINATOR_FF             
0x0000065F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000661      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000663      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000665      1  FF                                                   TERMINATOR_FF             
0x00000666      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000668      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000066A      1  FF                                                   TERMINATOR_FF             
0x0000066B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000066D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000066F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000671      1  FF                                                   TERMINATOR_FF             
0x00000672      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000674      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000676      3  F331B9                                               IMM16_F3                  u16_be=12729, u16_le=47409
0x00000679      1  FF                                                   TERMINATOR_FF             
0x0000067A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000067C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000067E      9  80074D543034353830                                   LEN8_STRING_CP932         length=7, text="MT04580"
0x00000687      1  FF                                                   TERMINATOR_FF             
0x00000688      1  FF                                                   TERMINATOR_FF             
0x00000689      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000068B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000068D     79  804D817582DE82A3816021212082B682D482F182BE82C182...  LEN8_STRING_CP932         length=77, text="「むぅ～!! じぶんだってしらないことあるー。\nマイばっかベンキョーさせるなー」"
0x000006DC      1  FF                                                   TERMINATOR_FF             
0x000006DD      1  FF                                                   TERMINATOR_FF             
0x000006DE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006E0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006E2      1  FF                                                   TERMINATOR_FF             
0x000006E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006E5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006E7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000006E9      1  FF                                                   TERMINATOR_FF             
0x000006EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006EC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006EE      3  F331BA                                               IMM16_F3                  u16_be=12730, u16_le=47665
0x000006F1      1  FF                                                   TERMINATOR_FF             
0x000006F2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006F4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006F6     32  801E817582A282A282A982E781418FAD82B582CD906982DF...  LEN8_STRING_CP932         length=30, text="「いいから、少しは進めろって」"
0x00000716      1  FF                                                   TERMINATOR_FF             
0x00000717      1  FF                                                   TERMINATOR_FF             
0x00000718      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000071A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000071C      1  FF                                                   TERMINATOR_FF             
0x0000071D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000071F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000721      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000723      1  FF                                                   TERMINATOR_FF             
0x00000724      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000726      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000728      3  F301B3                                               IMM16_F3                  u16_be=435, u16_le=45825
0x0000072B      1  FF                                                   TERMINATOR_FF             
0x0000072C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000072E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000730      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000732      1  FF                                                   TERMINATOR_FF             
0x00000733      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000735      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000737      1  FF                                                   TERMINATOR_FF             
0x00000738      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000073A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000073C      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000073E      1  FF                                                   TERMINATOR_FF             
0x0000073F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000741      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000743      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000745      1  FF                                                   TERMINATOR_FF             
0x00000746      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000748      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000074A      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000074C      1  FF                                                   TERMINATOR_FF             
0x0000074D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000074F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000751      1  FF                                                   TERMINATOR_FF             
0x00000752      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000754      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000756      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000758      1  FF                                                   TERMINATOR_FF             
0x00000759      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000075B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000075D      3  F331BB                                               IMM16_F3                  u16_be=12731, u16_le=47921
0x00000760      1  FF                                                   TERMINATOR_FF             
0x00000761      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000763      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000765      9  80074D543034353930                                   LEN8_STRING_CP932         length=7, text="MT04590"
0x0000076E      1  FF                                                   TERMINATOR_FF             
0x0000076F      1  FF                                                   TERMINATOR_FF             
0x00000770      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000772      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000774     18  801081758163816382DE82A3816381638176                 LEN8_STRING_CP932         length=16, text="「……むぅ……」"
0x00000786      1  FF                                                   TERMINATOR_FF             
0x00000787      1  FF                                                   TERMINATOR_FF             
0x00000788      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000078A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000078C      1  FF                                                   TERMINATOR_FF             
0x0000078D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000078F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000791      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000793      1  FF                                                   TERMINATOR_FF             
0x00000794      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000796      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000798      3  F33D6E                                               IMM16_F3                  u16_be=15726, u16_le=28221
0x0000079B      1  FF                                                   TERMINATOR_FF             
0x0000079C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000079E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007A0     84  80529153915295B782A282C482C882A281425C6E82C882E7...  LEN8_STRING_CP932         length=82, text="全然聞いてない。\nなら聞くなよ、とか思うけど、そんなこと\nいってもしょーがないし。"
0x000007F4      1  FF                                                   TERMINATOR_FF             
0x000007F5      1  FF                                                   TERMINATOR_FF             
0x000007F6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007FA      1  FF                                                   TERMINATOR_FF             
0x000007FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007FF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000801      1  FF                                                   TERMINATOR_FF             
0x00000802      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000804      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000806      3  F33D6F                                               IMM16_F3                  u16_be=15727, u16_le=28477
0x00000809      1  FF                                                   TERMINATOR_FF             
0x0000080A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000080C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000080E     38  80248BCA82C682C282AB82A082A482C982CD8141944591CF...  LEN8_STRING_CP932         length=36, text="玉とつきあうには、忍耐が必要だから。"
0x00000834      1  FF                                                   TERMINATOR_FF             
0x00000835      1  FF                                                   TERMINATOR_FF             
0x00000836      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000838      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000083A      1  FF                                                   TERMINATOR_FF             
0x0000083B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000083D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000083F      8  800673652D653137                                     LEN8_STRING_CP932         length=6, text="se-e17"
0x00000847      1  FF                                                   TERMINATOR_FF             
0x00000848      1  FF                                                   TERMINATOR_FF             
0x00000849      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000084B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000084D      2  F214                                                 IMM8_F2                   u8=20, s8=20
0x0000084F      1  FF                                                   TERMINATOR_FF             
0x00000850      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000852      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000854      1  FF                                                   TERMINATOR_FF             
0x00000855      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000857      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000859      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000085B      1  FF                                                   TERMINATOR_FF             
0x0000085C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000085E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000860      3  F301B1                                               IMM16_F3                  u16_be=433, u16_le=45313
0x00000863      1  FF                                                   TERMINATOR_FF             
0x00000864      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000866      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000868      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000086A      1  FF                                                   TERMINATOR_FF             
0x0000086B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000086D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000086F      1  FF                                                   TERMINATOR_FF             
0x00000870      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000872      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000874      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000876      1  FF                                                   TERMINATOR_FF             
0x00000877      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000879      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000087B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000087D      1  FF                                                   TERMINATOR_FF             
0x0000087E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000880      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000882      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000884      1  FF                                                   TERMINATOR_FF             
0x00000885      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000887      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000889      1  FF                                                   TERMINATOR_FF             
0x0000088A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000088C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000088E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000890      1  FF                                                   TERMINATOR_FF             
0x00000891      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000893      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000895      3  F331BD                                               IMM16_F3                  u16_be=12733, u16_le=48433
0x00000898      1  FF                                                   TERMINATOR_FF             
0x00000899      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000089B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000089D      9  80074D543034363030                                   LEN8_STRING_CP932         length=7, text="MT04600"
0x000008A6      1  FF                                                   TERMINATOR_FF             
0x000008A7      1  FF                                                   TERMINATOR_FF             
0x000008A8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008AA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008AC     30  801C817582F1815B8163816382A0815B814182A6815B82B2...  LEN8_STRING_CP932         length=28, text="「んー……あー、えーごだー」"
0x000008CA      1  FF                                                   TERMINATOR_FF             
0x000008CB      1  FF                                                   TERMINATOR_FF             
0x000008CC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008CE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008D0      1  FF                                                   TERMINATOR_FF             
0x000008D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008D5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000008D7      1  FF                                                   TERMINATOR_FF             
0x000008D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008DA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000008DC      3  F331BC                                               IMM16_F3                  u16_be=12732, u16_le=48177
0x000008DF      1  FF                                                   TERMINATOR_FF             
0x000008E0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008E2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008E4     34  802082E282C182C682CC82B182C682C581418BB389C88F91...  LEN8_STRING_CP932         length=32, text="やっとのことで、教科書を開いた。"
0x00000906      1  FF                                                   TERMINATOR_FF             
0x00000907      1  FF                                                   TERMINATOR_FF             
0x00000908      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000090A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000090C      1  FF                                                   TERMINATOR_FF             
0x0000090D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000090F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000911      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000913      1  FF                                                   TERMINATOR_FF             
0x00000914      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000916      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000918      3  F331BE                                               IMM16_F3                  u16_be=12734, u16_le=48689
0x0000091B      1  FF                                                   TERMINATOR_FF             
0x0000091C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000091E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000920     22  8014817582A282DC82B282EB8B4382C382AD82C88176         LEN8_STRING_CP932         length=20, text="「いまごろ気づくな」"
0x00000936      1  FF                                                   TERMINATOR_FF             
0x00000937      1  FF                                                   TERMINATOR_FF             
0x00000938      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000093A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000093C      1  FF                                                   TERMINATOR_FF             
0x0000093D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000093F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000941      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000943      1  FF                                                   TERMINATOR_FF             
0x00000944      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000946      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000948      3  F301B3                                               IMM16_F3                  u16_be=435, u16_le=45825
0x0000094B      1  FF                                                   TERMINATOR_FF             
0x0000094C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000094E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000950      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000952      1  FF                                                   TERMINATOR_FF             
0x00000953      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000955      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000957      1  FF                                                   TERMINATOR_FF             
0x00000958      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000095A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000095C      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000095E      1  FF                                                   TERMINATOR_FF             
0x0000095F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000961      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000963      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000965      1  FF                                                   TERMINATOR_FF             
0x00000966      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000968      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000096A      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000096C      1  FF                                                   TERMINATOR_FF             
0x0000096D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000096F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000971      1  FF                                                   TERMINATOR_FF             
0x00000972      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000974      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000976      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000978      1  FF                                                   TERMINATOR_FF             
0x00000979      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000097B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000097D      3  F331BF                                               IMM16_F3                  u16_be=12735, u16_le=48945
0x00000980      1  FF                                                   TERMINATOR_FF             
0x00000981      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000983      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000985      9  80074D543034363130                                   LEN8_STRING_CP932         length=7, text="MT04610"
0x0000098E      1  FF                                                   TERMINATOR_FF             
0x0000098F      1  FF                                                   TERMINATOR_FF             
0x00000990      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000992      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000994     36  8022817582F1815B8163816382A282C182C682A282B882AD...  LEN8_STRING_CP932         length=34, text="「んー……いっといずくれあー……」"
0x000009B8      1  FF                                                   TERMINATOR_FF             
0x000009B9      1  FF                                                   TERMINATOR_FF             
0x000009BA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009BC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009BE      1  FF                                                   TERMINATOR_FF             
0x000009BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009C3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000009C5      1  FF                                                   TERMINATOR_FF             
0x000009C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009C8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009CA      3  F331C0                                               IMM16_F3                  u16_be=12736, u16_le=49201
0x000009CD      1  FF                                                   TERMINATOR_FF             
0x000009CE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009D0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009D2     11  80098175636C6561728176                               LEN8_STRING_CP932         length=9, text="「clear」"
0x000009DD      1  FF                                                   TERMINATOR_FF             
0x000009DE      1  FF                                                   TERMINATOR_FF             
0x000009DF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009E1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009E3      1  FF                                                   TERMINATOR_FF             
0x000009E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E8      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000009EA      1  FF                                                   TERMINATOR_FF             
0x000009EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009ED      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009EF      3  F331C1                                               IMM16_F3                  u16_be=12737, u16_le=49457
0x000009F2      1  FF                                                   TERMINATOR_FF             
0x000009F3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009F5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000009F7      9  80074D543034363230                                   LEN8_STRING_CP932         length=7, text="MT04620"
0x00000A00      1  FF                                                   TERMINATOR_FF             
0x00000A01      1  FF                                                   TERMINATOR_FF             
0x00000A02      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A04      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A06     46  802C817582A282C182C682A282B8815B82AD82E882A0815B...  LEN8_STRING_CP932         length=44, text="「いっといずーくりあーざっとひーはずー……」"
0x00000A34      1  FF                                                   TERMINATOR_FF             
0x00000A35      1  FF                                                   TERMINATOR_FF             
0x00000A36      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A38      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A3A      1  FF                                                   TERMINATOR_FF             
0x00000A3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A3D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A3F      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000A41      1  FF                                                   TERMINATOR_FF             
0x00000A42      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A44      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A46      3  F301B1                                               IMM16_F3                  u16_be=433, u16_le=45313
0x00000A49      1  FF                                                   TERMINATOR_FF             
0x00000A4A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A4C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A4E      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000A50      1  FF                                                   TERMINATOR_FF             
0x00000A51      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A53      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A55      1  FF                                                   TERMINATOR_FF             
0x00000A56      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A58      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A5A      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000A5C      1  FF                                                   TERMINATOR_FF             
0x00000A5D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A5F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A61      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000A63      1  FF                                                   TERMINATOR_FF             
0x00000A64      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A66      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A68      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000A6A      1  FF                                                   TERMINATOR_FF             
0x00000A6B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A6D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A6F      1  FF                                                   TERMINATOR_FF             
0x00000A70      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A72      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A74      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000A76      1  FF                                                   TERMINATOR_FF             
0x00000A77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A79      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A7B      3  F331C2                                               IMM16_F3                  u16_be=12738, u16_le=49713
0x00000A7E      1  FF                                                   TERMINATOR_FF             
0x00000A7F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A81      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A83      9  80074D543034363330                                   LEN8_STRING_CP932         length=7, text="MT04630"
0x00000A8C      1  FF                                                   TERMINATOR_FF             
0x00000A8D      1  FF                                                   TERMINATOR_FF             
0x00000A8E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A90      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A92     26  8018817582A282C182C682C182C482C8815B82C9815B8148...  LEN8_STRING_CP932         length=24, text="「いっとってなーにー？」"
0x00000AAC      1  FF                                                   TERMINATOR_FF             
0x00000AAD      1  FF                                                   TERMINATOR_FF             
0x00000AAE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000AB0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AB2      1  FF                                                   TERMINATOR_FF             
0x00000AB3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AB5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AB7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000AB9      1  FF                                                   TERMINATOR_FF             
0x00000ABA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000ABC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000ABE      3  F331C3                                               IMM16_F3                  u16_be=12739, u16_le=49969
0x00000AC1      1  FF                                                   TERMINATOR_FF             
0x00000AC2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AC4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AC6     34  802082A482ED81418D828D5A90B682C682CD8E7682A682C8...  LEN8_STRING_CP932         length=32, text="うわ、高校生とは思えない英語力。"
0x00000AE8      1  FF                                                   TERMINATOR_FF             
0x00000AE9      1  FF                                                   TERMINATOR_FF             
0x00000AEA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000AEC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AEE      1  FF                                                   TERMINATOR_FF             
0x00000AEF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AF1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AF3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000AF5      1  FF                                                   TERMINATOR_FF             
0x00000AF6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AF8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000AFA      3  F331C4                                               IMM16_F3                  u16_be=12740, u16_le=50225
0x00000AFD      1  FF                                                   TERMINATOR_FF             
0x00000AFE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B00      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B02     50  8030817582A682C182C6814182B182B182CD7468617488C8...  LEN8_STRING_CP932         length=48, text="「えっと、ここはthat以下を強調する\n主語で……」"
0x00000B34      1  FF                                                   TERMINATOR_FF             
0x00000B35      1  FF                                                   TERMINATOR_FF             
0x00000B36      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B38      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B3A      1  FF                                                   TERMINATOR_FF             
0x00000B3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B3D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B3F      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000B41      1  FF                                                   TERMINATOR_FF             
0x00000B42      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B44      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B46      3  F301B2                                               IMM16_F3                  u16_be=434, u16_le=45569
0x00000B49      1  FF                                                   TERMINATOR_FF             
0x00000B4A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B4C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000B4E      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000B50      1  FF                                                   TERMINATOR_FF             
0x00000B51      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B53      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B55      1  FF                                                   TERMINATOR_FF             
0x00000B56      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B58      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B5A      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000B5C      1  FF                                                   TERMINATOR_FF             
0x00000B5D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B5F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B61      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000B63      1  FF                                                   TERMINATOR_FF             
0x00000B64      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B66      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000B68      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000B6A      1  FF                                                   TERMINATOR_FF             
0x00000B6B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B6D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B6F      1  FF                                                   TERMINATOR_FF             
0x00000B70      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B72      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B74      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000B76      1  FF                                                   TERMINATOR_FF             
0x00000B77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B79      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B7B      3  F331C5                                               IMM16_F3                  u16_be=12741, u16_le=50481
0x00000B7E      1  FF                                                   TERMINATOR_FF             
0x00000B7F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B81      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000B83      9  80074D543034363430                                   LEN8_STRING_CP932         length=7, text="MT04640"
0x00000B8C      1  FF                                                   TERMINATOR_FF             
0x00000B8D      1  FF                                                   TERMINATOR_FF             
0x00000B8E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B90      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B92     30  801C817582B782C182B2815B82A282C182C482A982F182B6...  LEN8_STRING_CP932         length=28, text="「すっごーいってかんじー？」"
0x00000BB0      1  FF                                                   TERMINATOR_FF             
0x00000BB1      1  FF                                                   TERMINATOR_FF             
0x00000BB2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000BB4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000BB6      1  FF                                                   TERMINATOR_FF             
0x00000BB7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BB9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BBB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000BBD      1  FF                                                   TERMINATOR_FF             
0x00000BBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BC0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000BC2      3  F331C6                                               IMM16_F3                  u16_be=12742, u16_le=50737
0x00000BC5      1  FF                                                   TERMINATOR_FF             
0x00000BC6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BC8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BCA     30  801C817582DC82A08163816382BB82EA82C582E082A282A2...  LEN8_STRING_CP932         length=28, text="「まあ……それでもいいかな」"
0x00000BE8      1  FF                                                   TERMINATOR_FF             
0x00000BE9      1  FF                                                   TERMINATOR_FF             
0x00000BEA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000BEC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000BEE      1  FF                                                   TERMINATOR_FF             
0x00000BEF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BF1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BF3      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000BF5      1  FF                                                   TERMINATOR_FF             
0x00000BF6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BF8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000BFA      3  F331C7                                               IMM16_F3                  u16_be=12743, u16_le=50993
0x00000BFD      1  FF                                                   TERMINATOR_FF             
0x00000BFE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C00      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000C02      9  80074D543034363530                                   LEN8_STRING_CP932         length=7, text="MT04650"
0x00000C0B      1  FF                                                   TERMINATOR_FF             
0x00000C0C      1  FF                                                   TERMINATOR_FF             
0x00000C0D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C0F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C11     90  8058817582B682E182A0814182A282C182C682B682E182C8...  LEN8_STRING_CP932         length=88, text="「じゃあ、いっとじゃなく、すごーいにすれば\nいーじゃーん。すごーいってなんてゆーうー？」"
0x00000C6B      1  FF                                                   TERMINATOR_FF             
0x00000C6C      1  FF                                                   TERMINATOR_FF             
0x00000C6D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C6F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C71      1  FF                                                   TERMINATOR_FF             
0x00000C72      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C74      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C76      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000C78      1  FF                                                   TERMINATOR_FF             
0x00000C79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C7B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C7D      3  F331C8                                               IMM16_F3                  u16_be=12744, u16_le=51249
0x00000C80      1  FF                                                   TERMINATOR_FF             
0x00000C81      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C83      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C85     44  802A817582A682C182C682CB816381635C6E677265617482...  LEN8_STRING_CP932         length=42, text="「えっとね……\ngreatとか、marvelousとか」"
0x00000CB1      1  FF                                                   TERMINATOR_FF             
0x00000CB2      1  FF                                                   TERMINATOR_FF             
0x00000CB3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CB5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CB7      1  FF                                                   TERMINATOR_FF             
0x00000CB8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CBA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CBC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000CBE      1  FF                                                   TERMINATOR_FF             
0x00000CBF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CC1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000CC3      3  F331C9                                               IMM16_F3                  u16_be=12745, u16_le=51505
0x00000CC6      1  FF                                                   TERMINATOR_FF             
0x00000CC7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CC9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CCB     14  800C8163816382A982C882A08142                         LEN8_STRING_CP932         length=12, text="……かなあ。"
0x00000CD9      1  FF                                                   TERMINATOR_FF             
0x00000CDA      1  FF                                                   TERMINATOR_FF             
0x00000CDB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CDD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CDF      1  FF                                                   TERMINATOR_FF             
0x00000CE0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CE2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CE4      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000CE6      1  FF                                                   TERMINATOR_FF             
0x00000CE7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CE9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000CEB      3  F331CA                                               IMM16_F3                  u16_be=12746, u16_le=51761
0x00000CEE      1  FF                                                   TERMINATOR_FF             
0x00000CEF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CF1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000CF3      9  80074D543034363630                                   LEN8_STRING_CP932         length=7, text="MT04660"
0x00000CFC      1  FF                                                   TERMINATOR_FF             
0x00000CFD      1  FF                                                   TERMINATOR_FF             
0x00000CFE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D00      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D02    112  806E817582B682E182B382B682E182B3814182AE82EA815B...  LEN8_STRING_CP932         length=110, text="「じゃさじゃさ、ぐれーといずくりやーざっと\nひいいずでいーじゃーん。なんでいっとなのか\nぜんぜんわかんなーい」"
0x00000D72      1  FF                                                   TERMINATOR_FF             
0x00000D73      1  FF                                                   TERMINATOR_FF             
0x00000D74      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D76      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D78      1  FF                                                   TERMINATOR_FF             
0x00000D79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D7B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D7D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000D7F      1  FF                                                   TERMINATOR_FF             
0x00000D80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D82      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D84      3  F331CB                                               IMM16_F3                  u16_be=12747, u16_le=52017
0x00000D87      1  FF                                                   TERMINATOR_FF             
0x00000D88      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D8A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D8C     40  802682DE81425C6E8BCA82CC82AD82B982C9814193EF82B5...  LEN8_STRING_CP932         length=38, text="む。\n玉のくせに、難しいことゆーなー。"
0x00000DB4      1  FF                                                   TERMINATOR_FF             
0x00000DB5      1  FF                                                   TERMINATOR_FF             
0x00000DB6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000DB8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000DBA      1  FF                                                   TERMINATOR_FF             
0x00000DBB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DBD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DBF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000DC1      1  FF                                                   TERMINATOR_FF             
0x00000DC2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DC4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000DC6      3  F331CC                                               IMM16_F3                  u16_be=12748, u16_le=52273
0x00000DC9      1  FF                                                   TERMINATOR_FF             
0x00000DCA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DCC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DCE     68  804282C682AB82C782AB814182B182A282C293AA82A282A2...  LEN8_STRING_CP932         length=66, text="ときどき、こいつ頭いいのか悪いのか、\nわかんなくなることがあるな。"
0x00000E12      1  FF                                                   TERMINATOR_FF             
0x00000E13      1  FF                                                   TERMINATOR_FF             
0x00000E14      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E16      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E18      1  FF                                                   TERMINATOR_FF             
0x00000E19      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E1B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E1D      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000E1F      1  FF                                                   TERMINATOR_FF             
0x00000E20      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E22      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E24      3  F331CD                                               IMM16_F3                  u16_be=12749, u16_le=52529
0x00000E27      1  FF                                                   TERMINATOR_FF             
0x00000E28      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E2A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000E2C      9  80074D543034363731                                   LEN8_STRING_CP932         length=7, text="MT04671"
0x00000E35      1  FF                                                   TERMINATOR_FF             
0x00000E36      1  FF                                                   TERMINATOR_FF             
0x00000E37      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E39      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E3B     44  802A817582BE82C182C482B3815B814182A282C182C682C1...  LEN8_STRING_CP932         length=42, text="「だってさー、いっとって『それ』じゃーん」"
0x00000E67      1  FF                                                   TERMINATOR_FF             
0x00000E68      1  FF                                                   TERMINATOR_FF             
0x00000E69      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E6B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E6D      1  FF                                                   TERMINATOR_FF             
0x00000E6E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E70      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E72      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000E74      1  FF                                                   TERMINATOR_FF             
0x00000E75      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E77      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E79      3  F331CE                                               IMM16_F3                  u16_be=12750, u16_le=52785
0x00000E7C      1  FF                                                   TERMINATOR_FF             
0x00000E7D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E7F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000E81      9  80074D543034363732                                   LEN8_STRING_CP932         length=7, text="MT04672"
0x00000E8A      1  FF                                                   TERMINATOR_FF             
0x00000E8B      1  FF                                                   TERMINATOR_FF             
0x00000E8C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E8E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E90    126  807C817582A882A9815B82B382F18141837D8343817782BB...  LEN8_STRING_CP932         length=124, text="「おかーさん、マイ『それとってー』って\nいったら『それじゃわかんない』ってゆーよー。だから、それじゃダメだよねー、きっとー」"
0x00000F0E      1  FF                                                   TERMINATOR_FF             
0x00000F0F      1  FF                                                   TERMINATOR_FF             
0x00000F10      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F12      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F14      1  FF                                                   TERMINATOR_FF             
0x00000F15      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F17      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F19      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000F1B      1  FF                                                   TERMINATOR_FF             
0x00000F1C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F1E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F20      3  F331CF                                               IMM16_F3                  u16_be=12751, u16_le=53041
0x00000F23      1  FF                                                   TERMINATOR_FF             
0x00000F24      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F26      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F28     38  802482B182E882E1835F838182BE814282DA82AD82CC8EE8...  LEN8_STRING_CP932         length=36, text="こりゃダメだ。ぼくの手にはおえない。"
0x00000F4E      1  FF                                                   TERMINATOR_FF             
0x00000F4F      1  FF                                                   TERMINATOR_FF             
0x00000F50      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F52      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F54      1  FF                                                   TERMINATOR_FF             
0x00000F55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F57      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F59      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000F5B      1  FF                                                   TERMINATOR_FF             
0x00000F5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F5E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F60      3  F331D0                                               IMM16_F3                  u16_be=12752, u16_le=53297
0x00000F63      1  FF                                                   TERMINATOR_FF             
0x00000F64      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F66      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F68     46  802C817582B782DC82F1814193A1945695D382C995B782A2...  LEN8_STRING_CP932         length=44, text="「すまん、藤之辺に聞いとくよ。だから、次ね」"
0x00000F96      1  FF                                                   TERMINATOR_FF             
0x00000F97      1  FF                                                   TERMINATOR_FF             
0x00000F98      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F9A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F9C      1  FF                                                   TERMINATOR_FF             
0x00000F9D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F9F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FA1      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000FA3      1  FF                                                   TERMINATOR_FF             
0x00000FA4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FA6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FA8      3  F301B0                                               IMM16_F3                  u16_be=432, u16_le=45057
0x00000FAB      1  FF                                                   TERMINATOR_FF             
0x00000FAC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FAE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000FB0      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000FB2      1  FF                                                   TERMINATOR_FF             
0x00000FB3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FB5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FB7      1  FF                                                   TERMINATOR_FF             
0x00000FB8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FBA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FBC      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000FBE      1  FF                                                   TERMINATOR_FF             
0x00000FBF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FC1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FC3      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000FC5      1  FF                                                   TERMINATOR_FF             
0x00000FC6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FC8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000FCA      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000FCC      1  FF                                                   TERMINATOR_FF             
0x00000FCD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FCF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FD1      1  FF                                                   TERMINATOR_FF             
0x00000FD2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FD4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FD6      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000FD8      1  FF                                                   TERMINATOR_FF             
0x00000FD9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FDB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000FDD      3  F331D1                                               IMM16_F3                  u16_be=12753, u16_le=53553
0x00000FE0      1  FF                                                   TERMINATOR_FF             
0x00000FE1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FE3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000FE5      9  80074D543034363830                                   LEN8_STRING_CP932         length=7, text="MT04680"
0x00000FEE      1  FF                                                   TERMINATOR_FF             
0x00000FEF      1  FF                                                   TERMINATOR_FF             
0x00000FF0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FF2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FF4     45  802B817582A682A581608163816382CD82E982B382F182C9...  LEN8_STRING_CP932         length=43, text="「えぇ～……はるさんにきくの～？ ズルだー」"
0x00001021      1  FF                                                   TERMINATOR_FF             
0x00001022      1  FF                                                   TERMINATOR_FF             
0x00001023      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001025      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001027      1  FF                                                   TERMINATOR_FF             
0x00001028      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000102A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000102C      8  800653452D533935                                     LEN8_STRING_CP932         length=6, text="SE-S95"
0x00001034      1  FF                                                   TERMINATOR_FF             
0x00001035      1  FF                                                   TERMINATOR_FF             
0x00001036      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001038      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000103A      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x0000103C      1  FF                                                   TERMINATOR_FF             
0x0000103D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000103F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001041      1  FF                                                   TERMINATOR_FF             
0x00001042      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001044      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001046      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001048      1  FF                                                   TERMINATOR_FF             
0x00001049      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000104B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000104D      3  F331D2                                               IMM16_F3                  u16_be=12754, u16_le=53809
0x00001050      1  FF                                                   TERMINATOR_FF             
0x00001051      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001053      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001055     66  80408BCA82CD8375815B8375815B82A282A282C882AA82E7...  LEN8_STRING_CP932         length=64, text="玉はブーブーいいながらも、構文をヘロヘロ\n文字でノートにうつす。"
0x00001097      1  FF                                                   TERMINATOR_FF             
0x00001098      1  FF                                                   TERMINATOR_FF             
0x00001099      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000109B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000109D      1  FF                                                   TERMINATOR_FF             
0x0000109E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010A2      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000010A4      1  FF                                                   TERMINATOR_FF             
0x000010A5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010A7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010A9      3  F301B3                                               IMM16_F3                  u16_be=435, u16_le=45825
0x000010AC      1  FF                                                   TERMINATOR_FF             
0x000010AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010AF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000010B1      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000010B3      1  FF                                                   TERMINATOR_FF             
0x000010B4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010B6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010B8      1  FF                                                   TERMINATOR_FF             
0x000010B9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010BD      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000010BF      1  FF                                                   TERMINATOR_FF             
0x000010C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010C2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010C4      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000010C6      1  FF                                                   TERMINATOR_FF             
0x000010C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010C9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000010CB      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000010CD      1  FF                                                   TERMINATOR_FF             
0x000010CE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010D0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010D2      1  FF                                                   TERMINATOR_FF             
0x000010D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010D7      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000010D9      1  FF                                                   TERMINATOR_FF             
0x000010DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010DC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000010DE      3  F331D3                                               IMM16_F3                  u16_be=12755, u16_le=54065
0x000010E1      1  FF                                                   TERMINATOR_FF             
0x000010E2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010E4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000010E6      9  80074D543034363930                                   LEN8_STRING_CP932         length=7, text="MT04690"
0x000010EF      1  FF                                                   TERMINATOR_FF             
0x000010F0      1  FF                                                   TERMINATOR_FF             
0x000010F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010F3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010F5     36  8022817582F1815B8163816383418343835882BD82D782BD...  LEN8_STRING_CP932         length=34, text="「んー……アイスたべたかったなー」"
0x00001119      1  FF                                                   TERMINATOR_FF             
0x0000111A      1  FF                                                   TERMINATOR_FF             
0x0000111B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000111D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000111F      1  FF                                                   TERMINATOR_FF             
0x00001120      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001122      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001124      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001126      1  FF                                                   TERMINATOR_FF             
0x00001127      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001129      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000112B      3  F331D4                                               IMM16_F3                  u16_be=12756, u16_le=54321
0x0000112E      1  FF                                                   TERMINATOR_FF             
0x0000112F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001131      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001133     16  800E81758DA1937882C8816381638176                     LEN8_STRING_CP932         length=14, text="「今度な……」"
0x00001143      1  FF                                                   TERMINATOR_FF             
0x00001144      1  FF                                                   TERMINATOR_FF             
0x00001145      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001147      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001149      1  FF                                                   TERMINATOR_FF             
0x0000114A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000114C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000114E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001150      1  FF                                                   TERMINATOR_FF             
0x00001151      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001153      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001155      3  F331D5                                               IMM16_F3                  u16_be=12757, u16_le=54577
0x00001158      1  FF                                                   TERMINATOR_FF             
0x00001159      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000115B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000115D      9  80074D543034373030                                   LEN8_STRING_CP932         length=7, text="MT04700"
0x00001166      1  FF                                                   TERMINATOR_FF             
0x00001167      1  FF                                                   TERMINATOR_FF             
0x00001168      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000116A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000116C     18  8010817582A482A381608160816381638176                 LEN8_STRING_CP932         length=16, text="「うぅ～～……」"
0x0000117E      1  FF                                                   TERMINATOR_FF             
0x0000117F      1  FF                                                   TERMINATOR_FF             
0x00001180      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001182      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001184      1  FF                                                   TERMINATOR_FF             
0x00001185      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001187      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001189      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000118B      1  FF                                                   TERMINATOR_FF             
0x0000118C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000118E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001190      3  F331D6                                               IMM16_F3                  u16_be=12758, u16_le=54833
0x00001193      1  FF                                                   TERMINATOR_FF             
0x00001194      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001196      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001198     72  80468BCA82CD837583638375836382A282A291B182AF8141...  LEN8_STRING_CP932         length=70, text="玉はブツブツいい続け、ときどき変な質問を\nしてくる。\n集中力０な感じ。"
0x000011E0      1  FF                                                   TERMINATOR_FF             
0x000011E1      1  FF                                                   TERMINATOR_FF             
0x000011E2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000011E4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000011E6      1  FF                                                   TERMINATOR_FF             
0x000011E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011EB      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000011ED      1  FF                                                   TERMINATOR_FF             
0x000011EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011F0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011F2      3  F301B3                                               IMM16_F3                  u16_be=435, u16_le=45825
0x000011F5      1  FF                                                   TERMINATOR_FF             
0x000011F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011F8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000011FA      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000011FC      1  FF                                                   TERMINATOR_FF             
0x000011FD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000011FF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001201      1  FF                                                   TERMINATOR_FF             
0x00001202      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001204      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001206      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001208      1  FF                                                   TERMINATOR_FF             
0x00001209      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000120B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000120D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000120F      1  FF                                                   TERMINATOR_FF             
0x00001210      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001212      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001214      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001216      1  FF                                                   TERMINATOR_FF             
0x00001217      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001219      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000121B      1  FF                                                   TERMINATOR_FF             
0x0000121C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000121E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001220      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001222      1  FF                                                   TERMINATOR_FF             
0x00001223      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001225      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001227      3  F331D7                                               IMM16_F3                  u16_be=12759, u16_le=55089
0x0000122A      1  FF                                                   TERMINATOR_FF             
0x0000122B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000122D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000122F      9  80074D543034373130                                   LEN8_STRING_CP932         length=7, text="MT04710"
0x00001238      1  FF                                                   TERMINATOR_FF             
0x00001239      1  FF                                                   TERMINATOR_FF             
0x0000123A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000123C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000123E     18  8010817582D482D382A38160816381638176                 LEN8_STRING_CP932         length=16, text="「ぶふぅ～……」"
0x00001250      1  FF                                                   TERMINATOR_FF             
0x00001251      1  FF                                                   TERMINATOR_FF             
0x00001252      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001254      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001256      1  FF                                                   TERMINATOR_FF             
0x00001257      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001259      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000125B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000125D      1  FF                                                   TERMINATOR_FF             
0x0000125E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001260      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001262      3  F331D8                                               IMM16_F3                  u16_be=12760, u16_le=55345
0x00001265      1  FF                                                   TERMINATOR_FF             
0x00001266      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001268      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000126A     16  800E817582C7815B82B582BD81488176                     LEN8_STRING_CP932         length=14, text="「どーした？」"
0x0000127A      1  FF                                                   TERMINATOR_FF             
0x0000127B      1  FF                                                   TERMINATOR_FF             
0x0000127C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000127E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001280      1  FF                                                   TERMINATOR_FF             
0x00001281      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001283      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001285      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001287      1  FF                                                   TERMINATOR_FF             
0x00001288      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000128A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000128C      3  F301B2                                               IMM16_F3                  u16_be=434, u16_le=45569
0x0000128F      1  FF                                                   TERMINATOR_FF             
0x00001290      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001292      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001294      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00001296      1  FF                                                   TERMINATOR_FF             
0x00001297      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001299      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000129B      1  FF                                                   TERMINATOR_FF             
0x0000129C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000129E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012A0      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000012A2      1  FF                                                   TERMINATOR_FF             
0x000012A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012A5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012A7      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000012A9      1  FF                                                   TERMINATOR_FF             
0x000012AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012AC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000012AE      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000012B0      1  FF                                                   TERMINATOR_FF             
0x000012B1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012B3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012B5      1  FF                                                   TERMINATOR_FF             
0x000012B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012BA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000012BC      1  FF                                                   TERMINATOR_FF             
0x000012BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012BF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000012C1      3  F331D9                                               IMM16_F3                  u16_be=12761, u16_le=55601
0x000012C4      1  FF                                                   TERMINATOR_FF             
0x000012C5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012C7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000012C9      9  80074D543034373230                                   LEN8_STRING_CP932         length=7, text="MT04720"
0x000012D2      1  FF                                                   TERMINATOR_FF             
0x000012D3      1  FF                                                   TERMINATOR_FF             
0x000012D4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012D6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012D8     24  8016817582E0815B82A982A682C182C482A2815B81488176     LEN8_STRING_CP932         length=22, text="「もーかえっていー？」"
0x000012F0      1  FF                                                   TERMINATOR_FF             
0x000012F1      1  FF                                                   TERMINATOR_FF             
0x000012F2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012F4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012F6      1  FF                                                   TERMINATOR_FF             
0x000012F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012FB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000012FD      1  FF                                                   TERMINATOR_FF             
0x000012FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001300      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001302      3  F331DA                                               IMM16_F3                  u16_be=12762, u16_le=55857
0x00001305      1  FF                                                   TERMINATOR_FF             
0x00001306      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001308      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000130A    112  806E817582A8914F82CB82A681418EA995AA82CC82B182C6...  LEN8_STRING_CP932         length=110, text="「お前ねえ、自分のことなんだから、少しくらいがんばれよ。それに今日、バイトあるだろ。\n帰っちゃダメだから……」"
0x0000137A      1  FF                                                   TERMINATOR_FF             
0x0000137B      1  FF                                                   TERMINATOR_FF             
0x0000137C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000137E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001380      1  FF                                                   TERMINATOR_FF             
0x00001381      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001383      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001385      8  800673652D653339                                     LEN8_STRING_CP932         length=6, text="se-e39"
0x0000138D      1  FF                                                   TERMINATOR_FF             
0x0000138E      1  FF                                                   TERMINATOR_FF             
0x0000138F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001391      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001393      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x00001395      1  FF                                                   TERMINATOR_FF             
0x00001396      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001398      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000139A      1  FF                                                   TERMINATOR_FF             
0x0000139B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000139D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000139F      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000013A1      1  FF                                                   TERMINATOR_FF             
0x000013A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013A4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013A6      3  F301AB                                               IMM16_F3                  u16_be=427, u16_le=43777
0x000013A9      1  FF                                                   TERMINATOR_FF             
0x000013AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013AC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000013AE      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000013B0      1  FF                                                   TERMINATOR_FF             
0x000013B1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000013B3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000013B5      1  FF                                                   TERMINATOR_FF             
0x000013B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013BA      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000013BC      1  FF                                                   TERMINATOR_FF             
0x000013BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013C1      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000013C3      1  FF                                                   TERMINATOR_FF             
0x000013C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013C6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000013C8      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000013CA      1  FF                                                   TERMINATOR_FF             
0x000013CB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000013CD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000013CF      1  FF                                                   TERMINATOR_FF             
0x000013D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013D4      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000013D6      1  FF                                                   TERMINATOR_FF             
0x000013D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013D9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000013DB      3  F331DB                                               IMM16_F3                  u16_be=12763, u16_le=56113
0x000013DE      1  FF                                                   TERMINATOR_FF             
0x000013DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013E1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000013E3      9  80074D543034373330                                   LEN8_STRING_CP932         length=7, text="MT04730"
0x000013EC      1  FF                                                   TERMINATOR_FF             
0x000013ED      1  FF                                                   TERMINATOR_FF             
0x000013EE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013F0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013F2     53  8033817582AB82E182A03F2120837D83438141836F834383...  LEN8_STRING_CP932         length=51, text="「きゃあ?! マイ、バイトーだあ!!\nチコクするぅぅ!!」"
0x00001427      1  FF                                                   TERMINATOR_FF             
0x00001428      1  FF                                                   TERMINATOR_FF             
0x00001429      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000142B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000142D      1  FF                                                   TERMINATOR_FF             
0x0000142E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001430      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001432      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001434      1  FF                                                   TERMINATOR_FF             
0x00001435      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001437      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001439      3  F331DC                                               IMM16_F3                  u16_be=12764, u16_le=56369
0x0000143C      1  FF                                                   TERMINATOR_FF             
0x0000143D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000143F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001441     36  8022817582A6814182BE82C182C482DC82BE8E9E8AD482A0...  LEN8_STRING_CP932         length=34, text="「え、だってまだ時間あるから……」"
0x00001465      1  FF                                                   TERMINATOR_FF             
0x00001466      1  FF                                                   TERMINATOR_FF             
0x00001467      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001469      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000146B      1  FF                                                   TERMINATOR_FF             
0x0000146C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000146E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001470      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001472      1  FF                                                   TERMINATOR_FF             
0x00001473      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001475      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001477      3  F3019D                                               IMM16_F3                  u16_be=413, u16_le=40193
0x0000147A      1  FF                                                   TERMINATOR_FF             
0x0000147B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000147D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000147F      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00001481      1  FF                                                   TERMINATOR_FF             
0x00001482      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001484      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001486      1  FF                                                   TERMINATOR_FF             
0x00001487      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001489      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000148B      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000148D      1  FF                                                   TERMINATOR_FF             
0x0000148E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001490      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001492      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001494      1  FF                                                   TERMINATOR_FF             
0x00001495      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001497      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001499      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000149B      1  FF                                                   TERMINATOR_FF             
0x0000149C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000149E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000014A0      1  FF                                                   TERMINATOR_FF             
0x000014A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014A5      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000014A7      1  FF                                                   TERMINATOR_FF             
0x000014A8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014AA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000014AC      3  F331DD                                               IMM16_F3                  u16_be=12765, u16_le=56625
0x000014AF      1  FF                                                   TERMINATOR_FF             
0x000014B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014B2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000014B4      9  80074D543034373430                                   LEN8_STRING_CP932         length=7, text="MT04740"
0x000014BD      1  FF                                                   TERMINATOR_FF             
0x000014BE      1  FF                                                   TERMINATOR_FF             
0x000014BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014C1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014C3     36  8022817582BD82A282D682F182BE815B8141836F83438367...  LEN8_STRING_CP932         length=34, text="「たいへんだー、バイトバイト～!!」"
0x000014E7      1  FF                                                   TERMINATOR_FF             
0x000014E8      1  FF                                                   TERMINATOR_FF             
0x000014E9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000014EB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000014ED      1  FF                                                   TERMINATOR_FF             
0x000014EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014F2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000014F4      1  FF                                                   TERMINATOR_FF             
0x000014F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014F7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000014F9      3  F331DE                                               IMM16_F3                  u16_be=12766, u16_le=56881
0x000014FC      1  FF                                                   TERMINATOR_FF             
0x000014FD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014FF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001501     94  805C8BCA82CD836D815B836782E28BB389C88F9182F09790...  LEN8_STRING_CP932         length=92, text="玉はノートや教科書を乱暴な手つきでカバンに\nなげいれる。\nざまあみろって気持ちがこもってる。"
0x0000155F      1  FF                                                   TERMINATOR_FF             
0x00001560      1  FF                                                   TERMINATOR_FF             
0x00001561      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001563      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001565      1  FF                                                   TERMINATOR_FF             
0x00001566      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001568      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000156A      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000156C      1  FF                                                   TERMINATOR_FF             
0x0000156D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000156F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001571      3  F301AE                                               IMM16_F3                  u16_be=430, u16_le=44545
0x00001574      1  FF                                                   TERMINATOR_FF             
0x00001575      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001577      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001579      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000157B      1  FF                                                   TERMINATOR_FF             
0x0000157C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000157E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001580      1  FF                                                   TERMINATOR_FF             
0x00001581      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001583      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001585      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001587      1  FF                                                   TERMINATOR_FF             
0x00001588      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000158A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000158C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000158E      1  FF                                                   TERMINATOR_FF             
0x0000158F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001591      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001593      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001595      1  FF                                                   TERMINATOR_FF             
0x00001596      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001598      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000159A      1  FF                                                   TERMINATOR_FF             
0x0000159B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000159D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000159F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000015A1      1  FF                                                   TERMINATOR_FF             
0x000015A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015A4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000015A6      3  F331DF                                               IMM16_F3                  u16_be=12767, u16_le=57137
0x000015A9      1  FF                                                   TERMINATOR_FF             
0x000015AA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015AC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000015AE      9  80074D543034373530                                   LEN8_STRING_CP932         length=7, text="MT04750"
0x000015B7      1  FF                                                   TERMINATOR_FF             
0x000015B8      1  FF                                                   TERMINATOR_FF             
0x000015B9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015BB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015BD     30  801C817582B682E182A082CB8141837D83438141836F8343...  LEN8_STRING_CP932         length=28, text="「じゃあね、マイ、バイトッ」"
0x000015DB      1  FF                                                   TERMINATOR_FF             
0x000015DC      1  FF                                                   TERMINATOR_FF             
0x000015DD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000015DF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000015E1      1  FF                                                   TERMINATOR_FF             
0x000015E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015E6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000015E8      1  FF                                                   TERMINATOR_FF             
0x000015E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015EB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000015ED      3  F331E0                                               IMM16_F3                  u16_be=12768, u16_le=57393
0x000015F0      1  FF                                                   TERMINATOR_FF             
0x000015F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015F3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015F5     50  803082B582DC82C182BD82A081425C6E82E682AF82A282C8...  LEN8_STRING_CP932         length=48, text="しまったあ。\nよけいなこと、思い出させちゃった。"
0x00001627      1  FF                                                   TERMINATOR_FF             
0x00001628      1  FF                                                   TERMINATOR_FF             
0x00001629      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000162B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000162D      1  FF                                                   TERMINATOR_FF             
0x0000162E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001630      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001632      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001634      1  FF                                                   TERMINATOR_FF             
0x00001635      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001637      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001639      3  F331E1                                               IMM16_F3                  u16_be=12769, u16_le=57649
0x0000163C      1  FF                                                   TERMINATOR_FF             
0x0000163D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000163F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001641      9  80074D543034373630                                   LEN8_STRING_CP932         length=7, text="MT04760"
0x0000164A      1  FF                                                   TERMINATOR_FF             
0x0000164B      1  FF                                                   TERMINATOR_FF             
0x0000164C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000164E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001650     97  805F817582A082C682CB814182DE82A982A682AB82C482E6...  LEN8_STRING_CP932         length=95, text="「あとね、むかえきてよねー。\nやーくーそーくー!! きてくんなかったら、\nもーベンキョーしないー」"
0x000016B1      1  FF                                                   TERMINATOR_FF             
0x000016B2      1  FF                                                   TERMINATOR_FF             
0x000016B3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016B5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016B7      1  FF                                                   TERMINATOR_FF             
0x000016B8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016BA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000016BC      8  800673652D653239                                     LEN8_STRING_CP932         length=6, text="se-e29"
0x000016C4      1  FF                                                   TERMINATOR_FF             
0x000016C5      1  FF                                                   TERMINATOR_FF             
0x000016C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016CA      2  F21F                                                 IMM8_F2                   u8=31, s8=31
0x000016CC      1  FF                                                   TERMINATOR_FF             
0x000016CD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016CF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016D1      1  FF                                                   TERMINATOR_FF             
0x000016D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016D6      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000016D8      1  FF                                                   TERMINATOR_FF             
0x000016D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016DD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016DF      1  FF                                                   TERMINATOR_FF             
0x000016E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016E2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000016E4      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000016E6      1  FF                                                   TERMINATOR_FF             
0x000016E7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016E9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016EB      1  FF                                                   TERMINATOR_FF             
0x000016EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016F0      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000016F2      1  FF                                                   TERMINATOR_FF             
0x000016F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016F5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016F7      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000016F9      1  FF                                                   TERMINATOR_FF             
0x000016FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016FC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000016FE      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001700      1  FF                                                   TERMINATOR_FF             
0x00001701      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001703      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001705      1  FF                                                   TERMINATOR_FF             
0x00001706      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001708      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000170A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000170C      1  FF                                                   TERMINATOR_FF             
0x0000170D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000170F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001711      3  F331E2                                               IMM16_F3                  u16_be=12770, u16_le=57905
0x00001714      1  FF                                                   TERMINATOR_FF             
0x00001715      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001717      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001719     54  80348BCA82CD93A682B08F6F82B782E682A482C890A882A2...  LEN8_STRING_CP932         length=52, text="玉は逃げ出すような勢いで、教室を飛び出して\nいった。"
0x0000174F      1  FF                                                   TERMINATOR_FF             
0x00001750      1  FF                                                   TERMINATOR_FF             
0x00001751      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001753      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001755      1  FF                                                   TERMINATOR_FF             
0x00001756      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001758      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000175A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000175C      1  FF                                                   TERMINATOR_FF             
0x0000175D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000175F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001761      3  F331E3                                               IMM16_F3                  u16_be=12771, u16_le=58161
0x00001764      1  FF                                                   TERMINATOR_FF             
0x00001765      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001767      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001769     48  802E82AA82E782F182C682B582BD8BB38EBA82C9814182DA...  LEN8_STRING_CP932         length=46, text="がらんとした教室に、ぼくはひとり取り残される。"
0x00001799      1  FF                                                   TERMINATOR_FF             
0x0000179A      1  FF                                                   TERMINATOR_FF             
0x0000179B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000179D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000179F      1  FF                                                   TERMINATOR_FF             
0x000017A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017A4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000017A6      1  FF                                                   TERMINATOR_FF             
0x000017A7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017A9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000017AB      3  F331E4                                               IMM16_F3                  u16_be=12772, u16_le=58417
0x000017AE      1  FF                                                   TERMINATOR_FF             
0x000017AF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017B1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017B3     18  801081758163816382D382A3816381638176                 LEN8_STRING_CP932         length=16, text="「……ふぅ……」"
0x000017C5      1  FF                                                   TERMINATOR_FF             
0x000017C6      1  FF                                                   TERMINATOR_FF             
0x000017C7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000017C9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000017CB      1  FF                                                   TERMINATOR_FF             
0x000017CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017D0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000017D2      1  FF                                                   TERMINATOR_FF             
0x000017D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017D5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000017D7      3  F331E5                                               IMM16_F3                  u16_be=12773, u16_le=58673
0x000017DA      1  FF                                                   TERMINATOR_FF             
0x000017DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017DD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017DF     98  806082C682E882A082A682B88141825093FA96DA82CD82C8...  LEN8_STRING_CP932         length=96, text="とりあえず、１日目はなんとか終了した。\nでも……まだ１日目だ。\nこれを毎日続けることになるのか？"
0x00001841      1  FF                                                   TERMINATOR_FF             
0x00001842      1  FF                                                   TERMINATOR_FF             
0x00001843      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001845      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001847      1  FF                                                   TERMINATOR_FF             
0x00001848      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000184A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000184C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000184E      1  FF                                                   TERMINATOR_FF             
0x0000184F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001851      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001853      3  F331E6                                               IMM16_F3                  u16_be=12774, u16_le=58929
0x00001856      1  FF                                                   TERMINATOR_FF             
0x00001857      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001859      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000185B     42  80288BCA82E091E595CF82BE82EB82A482AF82C7814182B1...  LEN8_STRING_CP932         length=40, text="玉も大変だろうけど、こっちも……大変だ。"
0x00001885      1  FF                                                   TERMINATOR_FF             
0x00001886      1  FF                                                   TERMINATOR_FF             
0x00001887      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001889      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000188B      1  FF                                                   TERMINATOR_FF             
0x0000188C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000188E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001890      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00001892      1  FF                                                   TERMINATOR_FF             
0x00001893      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001895      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001897      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001899      1  FF                                                   TERMINATOR_FF             
0x0000189A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000189C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000189E      1  FF                                                   TERMINATOR_FF             
0x0000189F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018A3      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000018A5      1  FF                                                   TERMINATOR_FF             
0x000018A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018A8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018AA      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x000018AC      1  FF                                                   TERMINATOR_FF             
0x000018AD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000018AF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000018B1      1  FF                                                   TERMINATOR_FF             
0x000018B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018B4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018B6      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000018B8      1  FF                                                   TERMINATOR_FF             
0x000018B9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018BB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018BD      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000018BF      1  FF                                                   TERMINATOR_FF             
0x000018C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018C2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000018C4      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000018C6      1  FF                                                   TERMINATOR_FF             
0x000018C7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000018C9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000018CB      1  FF                                                   TERMINATOR_FF             
0x000018CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018D0      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x000018D2      1  FF                                                   TERMINATOR_FF             
0x000018D3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000018D5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000018D7      1  FF                                                   TERMINATOR_FF             
0x000018D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018DC      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000018DE      1  FF                                                   TERMINATOR_FF             
0x000018DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018E1      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000018E3     14  800C50533254303134612E62696E                         LEN8_STRING_CP932         length=12, text="PS2T014a.bin"
0x000018F1      1  FF                                                   TERMINATOR_FF             
0x000018F2      1  FF                                                   TERMINATOR_FF             
0x000018F3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000018F5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000018F7      1  FF                                                   TERMINATOR_FF             
0x000018F8      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000018FA      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000018FC      1  FF                                                   TERMINATOR_FF             
0x000018FD      2  0019                                                 WORD_00XX                 u16_be=25, low_byte=25
0x000018FF      1  07                                                   OPAQUE_RAW_BYTES          bytes=07
0x00001900      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x00001902      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00001904      2  0018                                                 WORD_00XX                 u16_be=24, low_byte=24
0x00001906      1  F8                                                   OPAQUE_RAW_BYTES          bytes=F8
0x00001907      1  FF                                                   TERMINATOR_FF             
0x00001908      1  FF                                                   TERMINATOR_FF             
