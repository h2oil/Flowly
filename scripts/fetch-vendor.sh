#!/usr/bin/env bash
# Populate apps/desktop-shell/vendor: the Vosk Windows runtime (from the PyPI
# wheel — GitHub releases may be blocked in some environments), an MSVC import
# library generated from the DLL's export table, and the small English model.
set -euo pipefail
cd "$(dirname "$0")/../apps/desktop-shell"
mkdir -p vendor && cd vendor

WHEEL_URL=$(curl -sS https://pypi.org/pypi/vosk/0.3.45/json | python3 -c "
import json,sys
for f in json.load(sys.stdin)['urls']:
    if 'win_amd64' in f['filename']:
        print(f['url']); break")
curl -sSL -o vosk-wheel.whl "$WHEEL_URL"
unzip -o -q vosk-wheel.whl -d wheel
find wheel -name "*.dll" -exec cp {} . \;
rm -rf wheel vosk-wheel.whl

python3 "$(dirname "$0")/../scripts/pe_exports.py" libvosk.dll vosk.def 2>/dev/null || python3 - <<'PYEOF'
import struct
data = open("libvosk.dll", "rb").read()
pe = struct.unpack_from("<I", data, 0x3C)[0]
n_sec, = struct.unpack_from("<H", data, pe + 6)
opt_size, = struct.unpack_from("<H", data, pe + 20)
opt = pe + 24
exp_rva, _ = struct.unpack_from("<II", data, opt + 112)
secs = []
for i in range(n_sec):
    o = opt + opt_size + i * 40
    va, = struct.unpack_from("<I", data, o + 12)
    vsz, = struct.unpack_from("<I", data, o + 8)
    raw, = struct.unpack_from("<I", data, o + 20)
    secs.append((va, vsz, raw))
def r2o(rva):
    for va, vsz, raw in secs:
        if va <= rva < va + vsz: return raw + rva - va
    raise ValueError
e = r2o(exp_rva)
n, = struct.unpack_from("<I", data, e + 24)
names_off = r2o(struct.unpack_from("<I", data, e + 32)[0])
with open("vosk.def", "w") as f:
    f.write("LIBRARY libvosk.dll\nEXPORTS\n")
    for i in range(n):
        o = r2o(struct.unpack_from("<I", data, names_off + i * 4)[0])
        f.write("    " + data[o:data.index(b'\0', o)].decode() + "\n")
PYEOF
LLVM_DLLTOOL=$(ls /usr/bin/llvm-dlltool* | head -1)
"$LLVM_DLLTOOL" -m i386:x86-64 -d vosk.def -D libvosk.dll -l vosk.lib
cp vosk.lib libvosk.lib

curl -sSL -o model.zip https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip
unzip -q -o model.zip && rm -f model.zip
rm -rf model && mv vosk-model-small-en-us-0.15 model
echo "vendor ready:" && ls
