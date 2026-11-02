#!/usr/bin/env bash
# HEX1000 on its board, simulated for the blender extension's hex-run
# video: X_eTaL-fpga (work/fpga, at FPGA_COMMIT) builds its board's
# UART core as Verilog the way its board flow does (xetal-fpga verilog,
# the 9K's 27 MHz clock and 115200 baud; the tool built into
# work/fpga-target, never inside the clone), and iverilog runs the
# board top, that core and the machine (hex1000.v) under
# extensions/blender/demos/hex-run.tb.v, the text sent over the serial
# line. Writes DIR/hex-run.vcd (the signals the video shows) and
# DIR/sim.txt (the clocks and the display at the end). Needs iverilog.
#   scripts/hex1000-sim.sh TEXT DIR
#   scripts/hex1000-sim.sh "C0DE+BEEF=" extensions/blender/work/hexrun
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
text="${1:?usage: hex1000-sim.sh TEXT DIR}"
dir="${2:?usage: hex1000-sim.sh TEXT DIR}"
for t in iverilog vvp cargo; do
  command -v "$t" >/dev/null || { echo "hex1000-sim: $t is not installed (iverilog: brew install icarus-verilog)" >&2; exit 1; }
done
"$root/scripts/libraries.sh" >/dev/null
fpga="$root/work/fpga"
(cd "$fpga/components" && CARGO_TARGET_DIR="$root/work/fpga-target" cargo build -q --release -p xetal-fpga-cli)
mkdir -p "$dir"
dir="$(cd "$dir" && pwd)"
# the board's UART divisors, as X_eTaL-fpga's scripts/board-hex1000.py sets them
clocks=$(( 27000000 / 115200 ))
sed -e "s/%dm1 = const i16 2/%dm1 = const i16 $(( clocks - 1 ))/" \
    -e "s/%rxk = const i16 3/%rxk = const i16 $(( clocks + clocks / 2 - 1 ))/" \
    "$fpga/boards/examples/uart.xir" > "$dir/uart.xir"
"$root/work/fpga-target/release/xetal-fpga" verilog "$dir/uart.xir" > "$dir/uart.v"
cat "$fpga/boards/tangnano9k/hex1000-top.v" "$dir/uart.v" "$fpga/libs/Hex1000/rtl/hex1000.v" > "$dir/design.v"
iverilog -g2005-sv -s tb -o "$dir/hex-run.vvp" "$dir/design.v" "$root/extensions/blender/demos/hex-run.tb.v"
vvp -n "$dir/hex-run.vvp" "+text=$text" "+vcd=$dir/hex-run.vcd" | grep -v '^VCD info' | grep -v '\$finish' | tee "$dir/sim.txt"
