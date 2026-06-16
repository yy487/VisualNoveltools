# -*- coding: utf-8 -*-
"""Static RealLive SEEN crypt template.

This title XORs the packed SEEN code stream with a 256-byte table before/after
RealLive LZ compression.  The table below was recovered once from the original
RealLive.exe export at byte_596984.  Keeping it here makes extract/inject usable
without passing the IDA export directory every time.

For another RealLive build, replace STATIC_XOR_KEY with that executable's
byte_596984 table or pass --ida-export/legacy exe_export_dir to the tools.
"""

STATIC_XOR_KEY = bytes.fromhex(
    "8be55dc3a1e030440085c074095f5e33c05b8be55dc38b450c85c075148b55ec83c220"
    "526a00e8f528010083c40889450c8b45e46a006a005053ff1534b143008b451085c074"
    "058b4dec89088a45f084c07578a1e03044008b7de88b750c85c075448b1dd0b0430085"
    "ff763781ff000004006a0076438b45f88d55fc5268000004005650ff152cb143006a05"
    "ffd3a1e030440081ef0000040081c60000040085c074c58b5df853e8f4fbffff8b45"
    "0c83c4045f5e5b8be55dc38b55f88d4dfc51575652ff152cb14300ebd88b45e883c0"
    "20506a00e8472801008b7de88945f48bf0a1e030440083c40885c075568b1dd0b043"
    "0085ff764981ff000004006a0076"
)
