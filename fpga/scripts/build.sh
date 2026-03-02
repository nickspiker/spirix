#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FPGA_DIR="$SCRIPT_DIR/.."

cd "$FPGA_DIR"
mkdir -p sim

echo "=== Synthesizing with Yosys ==="
yosys -p "
    read_verilog rtl/spirix_subtract_f4e4.v
    read_verilog rtl/top.v
    synth_ecp5 -top top -json sim/spirix.json
"

echo "=== Place and route with nextpnr ==="
nextpnr-ecp5 \
    --25k \
    --package CABGA256 \
    --speed 6 \
    --json sim/spirix.json \
    --lpf constraints/colorlight_5a75b_v8.lpf \
    --textcfg sim/spirix.config

echo "=== Packing bitstream ==="
ecppack sim/spirix.config sim/spirix.bit
ecppack sim/spirix.config --svf sim/spirix.svf

echo "=== Done: sim/spirix.bit / sim/spirix.svf ==="
echo ""
echo "Flash with:"
echo "  openFPGALoader --cable ft232 --freq 3000000 sim/spirix.svf"
