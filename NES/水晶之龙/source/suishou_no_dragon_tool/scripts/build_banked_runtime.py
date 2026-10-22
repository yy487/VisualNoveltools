"""Assemble V3 banked-page dispatcher; final Rust CLI embeds only this new code."""
import json
import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(ROOT / "workspaces/suishou_no_dragon/scripts"))
from sekiryuuou_make_16x16_test import Asm6502
from fds_probe import parse_fds

OUT = Path(__file__).resolve().parents[1] / "assets"
REPORT = Path(__file__).resolve().parents[1] / "work/suishou/first_sentence_nes/build.json"
r = json.loads(REPORT.read_text(encoding="utf-8"))
old = r["runtime_labels"]
m = r["mapper_labels"]
a = Asm6502(0xF300)
MODE, SHADOW, SAVED = 0x9FD0, 0x9FD1, 0x9FD2
DIR, DBANK, PBANK, COUNT, ROOTPTR, TMP = 0x9FD4, 0x9FD6, 0x9FD7, 0x9FD8, 0x9FDA, 0x9FDC
BUFFER = 0xA120
UIFLAG, UICOUNT, KEY, ITEMNAME, ITEMPTR, RESTPTR = 0x9FDD, 0x9FE1, 0x9FE2, 0x9FE4, 0x9FE6, 0x9FE8
UIREF = 0xA170

def op(c, p): a.emit(c, p & 255, p >> 8)
def call(n): a.abs_ref(0x20, n)
def jump(n): a.abs_ref(0x4C, n)
def direct(c, p): op(c, p)
def far(c, n): a.emit(c ^ 0x20, 3); jump(n)

a.label("set_bank")
op(0x8D, SHADOW); op(0x8D, 0x5116); a.emit(0x60)
a.label("boot_return")
a.emit(0xA9, 0)
for p in [MODE, 0xA10A, 0xA10E, 0xA10F, UIFLAG]: op(0x8D, p)
a.emit(0xA9, 3); call("set_bank"); a.emit(0x58); direct(0x4C, 0x6000)
a.label("nmi_enter")
op(0xAD, SHADOW); op(0x8D, SAVED)
a.emit(0xA9, 3); call("set_bank")
# Replay overwritten A9 10 / STA 2000 of the existing mapper routine.
a.emit(0xA9, 0x10); op(0x8D, 0x2000); direct(0x4C, m["nmi_arm"] + 5)
a.label("nmi_finish")
a.emit(0xA9, 0); op(0x8D, 0x9D5B)
op(0xAD, SAVED); call("set_bank"); a.emit(0x60)
a.label("video_begin")
op(0xAD, MODE); a.branch(0xF0, "video_v2")
op(0xAD, PBANK); a.emit(0x09, 0x80); a.branch(0xD0, "video_map")
a.label("video_v2"); a.emit(0xA9, 0xA0)
a.label("video_map"); call("set_bank"); direct(0x20, m["chr_font"])
op(0xAD, 0xA106); direct(0x4C, old["upload"] + 3)
a.label("video_end")
op(0x8D, 0x2000); a.emit(0xA9, 3); call("set_bank")
direct(0x20, m["chr_top"]); a.emit(0xA9, 0x10); op(0x8D, 0x2000); a.emit(0x60)
a.label("dispatch")
a.emit(0xA9,0); op(0x8D,UIFLAG); jump("dispatch_entry")
a.label("dispatch_ui")
a.emit(0xA9,1); op(0x8D,UIFLAG)
a.label("dispatch_entry")
op(0xAD, 0x9D50); a.emit(0x85, 0); op(0x8D, ROOTPTR)
op(0xAD, 0x9D51); a.emit(0x85, 1); op(0x8D, ROOTPTR+1)
for y, b in enumerate(b"\x02ZH\x03"):
    a.emit(0xA0, y, 0xB1, 0, 0xC9, b); far(0xD0, "interface_check")
jump("packet")
a.label("interface_check")
op(0xAD,UIFLAG); a.branch(0xD0,"interface_lookup")
op(0xAD,ROOTPTR+1); a.emit(0xC9,0x93); far(0xD0,"legacy")
op(0xAD,ROOTPTR); a.emit(0xC9,0xB0); far(0xB0,"legacy")
a.label("interface_lookup")
call("lookup"); far(0x90,"legacy")
op(0xAD,ROOTPTR+1);a.emit(0xC9,0x93);a.branch(0xD0,"interface_ready")
op(0xAD,ROOTPTR);a.emit(0xC9,0x81);a.branch(0xD0,"interface_ready")
# Loading text is automatic; it must not introduce an A-key wait.
a.emit(0xA9,1);op(0x8D,UIFLAG)
a.label("interface_ready")
a.emit(0xA9,UIREF&255,0x85,0,0xA9,UIREF>>8,0x85,1)
a.label("packet")
for y, p in [(4, DBANK), (5, DIR), (6, DIR+1), (7, COUNT), (8, COUNT+1)]:
    a.emit(0xA0, y, 0xB1, 0); op(0x8D, p)
a.emit(0xA9, 1); op(0x8D, MODE)
a.label("next_page")
call("directory_get"); op(0x8D, PBANK)
call("directory_get"); a.emit(0x85, 0)
call("directory_get"); a.emit(0x85, 1)
op(0xAD, PBANK); a.emit(0x09, 0x80); call("set_bank")
# Copy one V2 page to stable RAM before enabling NMI / waiting for frames.
a.emit(0xA0, 9, 0xB1, 0, 0x18, 0x69, 15); op(0x8D, TMP)
a.emit(0xA0, 0)
a.label("copy_page")
a.emit(0xB1, 0); op(0x99, BUFFER); a.emit(0xC8); op(0xCC, TMP); a.branch(0xD0, "copy_page")
a.emit(0xA9, 3); call("set_bank")
a.emit(0xA9, BUFFER & 255); op(0x8D, 0x9D50)
a.emit(0xA9, BUFFER >> 8); op(0x8D, 0x9D51)
direct(0x20, old["dispatch"])
op(0xAD, COUNT); a.branch(0xD0, "dec_low"); op(0xCE, COUNT+1)
a.label("dec_low"); op(0xCE, COUNT)
op(0xAD, COUNT); op(0x0D, COUNT+1); a.branch(0xF0, "done")
a.label("wait_a")
direct(0x20, old["frame"]); direct(0x20, 0x6360)
a.emit(0xA5, 0x1A, 0x45, 0x1B, 0x25, 0x1A, 0x29, 0x80); a.branch(0xF0, "wait_a")
jump("next_page")
a.label("done")
op(0xAD,UIFLAG); a.branch(0xF0,"last_wait_a")
# Refresh an A press consumed by paging before returning to the inventory handler.
direct(0x20,0x6360);jump("return_packet")
a.label("last_wait_a")
direct(0x20, old["frame"]); direct(0x20, 0x6360)
a.emit(0xA5, 0x1A, 0x45, 0x1B, 0x25, 0x1A, 0x29, 0x80); a.branch(0xF0, "last_wait_a")
a.label("return_packet")
op(0xAD, ROOTPTR); op(0x8D, 0x9D50)
op(0xAD, ROOTPTR+1); op(0x8D, 0x9D51); a.emit(0x60)
a.label("legacy")
call("restore");a.emit(0xA9, 0); op(0x8D, MODE); direct(0x4C, old["dispatch"])
a.label("directory_get")
# Zero page 0/1 is intentionally untouched: callers keep a page pointer there.
op(0xAD, SHADOW); a.emit(0x48)
op(0xAD, DBANK); a.emit(0x09, 0x80); call("set_bank")
op(0xAD, DIR); a.emit(0x85, 0x14)
op(0xAD, DIR+1); a.emit(0x85, 0x15, 0xA0, 0, 0xB1, 0x14); op(0x8D, TMP)
op(0xEE, DIR); a.branch(0xD0, "get_restore")
op(0xEE, DIR+1); op(0xAD, DIR+1); a.emit(0xC9, 0xE0); a.branch(0xD0, "get_restore")
a.emit(0xA9, 0xC0); op(0x8D, DIR+1); op(0xEE, DBANK)
a.label("get_restore")
a.emit(0x68); call("set_bank"); op(0xAD, TMP); a.emit(0x60)

a.label("lookup")
op(0xAD,0xFE4B);op(0x8D,UICOUNT);a.branch(0xF0,"lookup_missing")
op(0xAD,0xFE48);op(0x8D,DBANK)
op(0xAD,0xFE49);op(0x8D,DIR);op(0xAD,0xFE4A);op(0x8D,DIR+1)
a.label("lookup_record")
call("directory_get");op(0x8D,KEY)
call("directory_get");op(0xCD,0x9D51);a.branch(0xD0,"lookup_skip")
op(0xAD,KEY);op(0xCD,0x9D50);a.branch(0xD0,"lookup_skip")
a.emit(0xA2,0)
a.label("lookup_copy")
call("directory_get");op(0x9D,UIREF);a.emit(0xE8,0xE0,10);a.branch(0xD0,"lookup_copy");a.emit(0x38,0x60)
a.label("lookup_skip")
a.emit(0xA2,10)
a.label("lookup_skip_byte")
call("directory_get");a.emit(0xCA);a.branch(0xD0,"lookup_skip_byte")
op(0xCE,UICOUNT);a.branch(0xD0,"lookup_record")
a.label("lookup_missing");a.emit(0x18,0x60)

a.label("item_entry")
a.emit(0xA5,8);op(0x8D,ITEMPTR);a.emit(0xA5,9);op(0x8D,ITEMPTR+1)
a.emit(0x18,0xA5,8,0x69,2,0x85,8);op(0x8D,ITEMNAME)
a.emit(0xA5,9,0x69,0,0x85,9);op(0x8D,ITEMNAME+1)
a.emit(0xA0,0)
a.label("item_skip_name")
a.emit(0xB1,8,0xC9,255);a.branch(0xF0,"item_after_name");call("item_next");jump("item_skip_name")
a.label("item_after_name");call("item_next")
a.label("item_variant")
a.emit(0xB1,8,0xC9,255);a.branch(0xF0,"item_name_only")
op(0xAC,0x9D8B);op(0xD9,0x9D8C);a.branch(0xF0,"item_variant_found");a.emit(0xA0,0)
call("item_next")
a.label("item_skip_variant")
a.emit(0xB1,8,0xC9,255);a.branch(0xF0,"item_next_variant");call("item_next");jump("item_skip_variant")
a.label("item_next_variant");call("item_next");jump("item_variant")
a.label("item_variant_found");call("item_next")
a.emit(0xA5,8);op(0x8D,0x9D50);a.emit(0xA5,9);op(0x8D,0x9D51);jump("item_try")
a.label("item_name_only")
op(0xAD,ITEMNAME);op(0x8D,0x9D50);op(0xAD,ITEMNAME+1);op(0x8D,0x9D51)
a.label("item_try")
call("lookup");a.branch(0x90,"item_fallback")
call("dispatch_ui");call("item_restore_pointer");a.emit(0x60)
a.label("item_fallback")
call("item_restore_pointer")
# Replay the overwritten 65EF prologue and resume its original scan.
a.emit(0xA9,0x53);op(0x8D,0x9D50);direct(0x4C,0x65F4)
a.label("item_restore_pointer")
op(0xAD,ITEMPTR);a.emit(0x85,8);op(0xAD,ITEMPTR+1);a.emit(0x85,9,0x60)
a.label("item_next")
a.emit(0xE6,8);a.branch(0xD0,"item_next_done");a.emit(0xE6,9)
a.label("item_next_done");a.emit(0x60)

a.label("initialize")
# 7BEF contains the title and opening cutscene: cleanup belongs after it returns.
direct(0x20,0x7BEF);call("restore");a.emit(0xA9,0)
for p in [MODE,UIFLAG,0xA10A,0xA10E,0xA10F]:op(0x8D,p)
a.emit(0x60)
a.label("restore")
op(0xAD,0xA10A);op(0x0D,0xA10E);a.branch(0xF0,"restore_return")
op(0xAD,0x9D50);op(0x8D,RESTPTR);op(0xAD,0x9D51);op(0x8D,RESTPTR+1)
a.emit(0xA9,0)
for p in [MODE,0xA10E,0xA10F,0x9D52,0x7163]:op(0x8D,p)
a.emit(0xA9,0x80);op(0x8D,0x0101)
a.emit(0xA9,0x92);op(0x8D,0xDFFE);a.emit(0xA9,0x7D);op(0x8D,0xDFFF)
direct(0x20,0x68F9);direct(0x20,0x6909)
a.emit(0xA9,2);op(0x8D,0xA10A);a.emit(0xA9,18);op(0x8D,0x9D54)
a.label("restore_wait")
direct(0x20,old["frame"]);op(0xAD,0x9D54);a.branch(0xD0,"restore_wait")
a.emit(0xA9,0);op(0x8D,0xA10A)
op(0xAD,RESTPTR);op(0x8D,0x9D50);op(0xAD,RESTPTR+1);op(0x8D,0x9D51)
a.label("restore_return");a.emit(0x60)

code = a.resolve()
assert a.pc < 0xFD00
OUT.mkdir(exist_ok=True)
(OUT / "banked_runtime.bin").write_bytes(code)
image = (REPORT.parent / "suishou_first_sentence_mmc5.nes").read_bytes()
entry = next(e for e in r["resources"] if e["name"] == "ADVS")
core = bytearray(image[16+entry["prg_bank"]*8192:16+entry["prg_bank"]*8192+entry["size"]])
core[0x683B-0x6000:0x683E-0x6000] = bytes([0x4C])+a.labels["dispatch"].to_bytes(2,"little")
core[0x7612-0x6000:0x7617-0x6000] = bytes([0x20])+a.labels["nmi_finish"].to_bytes(2,"little")+b"\xea\xea"
core[0x65EF-0x6000:0x65F2-0x6000] = bytes([0x4C])+a.labels["item_entry"].to_bytes(2,"little")
fixed = bytearray(image[-8192:])
fixed[0x1300:0x1300+len(code)] = code
for p, target in [(m["video_begin"],"video_begin"),(m["video_end"],"video_end"),(m["nmi_arm"],"nmi_enter")]:
    fixed[p-0xE000:p-0xE000+3] = bytes([0x4C])+a.labels[target].to_bytes(2,"little")
boot = fixed.index(bytes.fromhex("58 4C 00 60"),0x1000,0x1300)
fixed[boot:boot+4] = bytes([0x4C])+a.labels["boot_return"].to_bytes(2,"little")+b"\xea"
fixed[0x1D00:0x1D00+260] = bytes(260)
fixed[0x1E40:0x1E60] = bytes(32)
abi = {"base":0xF300,"labels":a.labels,"legacy":old,"mapper":m,"boot_patch":boot,
       "core_sha256":hashlib.sha256(core).hexdigest(),"fixed_sha256":hashlib.sha256(fixed).hexdigest(),
       "resources":r["resources"]}
disk = (REPORT.parent / "source.fds").read_bytes()
manifest = parse_fds(disk)
original_core_entry = next(e for side in manifest["sides"] for e in side["files"] if e["name"] == "ADVS")
original_core = disk[original_core_entry["data_offset"]:original_core_entry["data_offset"]+original_core_entry["size"]]
demo_core = image[16+entry["prg_bank"]*8192:16+entry["prg_bank"]*8192+entry["size"]]
patches = []
pos = 0
while pos < len(original_core):
    if original_core[pos] == demo_core[pos]: pos += 1; continue
    start = pos
    while pos < len(original_core) and original_core[pos] != demo_core[pos]: pos += 1
    patches.append({"offset":start,"old":list(original_core[start:pos]),"new":list(demo_core[start:pos])})
abi["core_patches"] = patches
fixed_entry = next(e for e in r["resources"] if e["name"] == "FIXDAT")
runtime=bytearray(image[16+fixed_entry["prg_bank"]*8192+0x780:16+fixed_entry["prg_bank"]*8192+fixed_entry["size"]])
runtime[:3]=bytes([0x4C])+a.labels["initialize"].to_bytes(2,"little")
(OUT / "ram_runtime.bin").write_bytes(runtime)
abi["fixed_payload_sha256"]=hashlib.sha256(image[16+fixed_entry["prg_bank"]*8192:16+fixed_entry["prg_bank"]*8192+0x780]+runtime).hexdigest()
previous=json.loads((Path(__file__).resolve().parents[1]/"work/suishou/native_return_fix_20261014/before/banked_abi.json").read_text(encoding="utf-8"))
abi["previous_core_sha256"]=previous["core_sha256"];abi["previous_fixed_sha256"]=previous["fixed_sha256"]
(OUT / "mapper_runtime.bin").write_bytes((REPORT.parent / "fixed_driver.bin").read_bytes())
(OUT / "banked_abi.json").write_text(json.dumps(abi, indent=2)+"\n", encoding="utf-8")
print(json.dumps({"bytes": len(code), "end": hex(a.pc), "labels": a.labels}))
