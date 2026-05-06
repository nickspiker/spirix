#!/usr/bin/env bash
# Build external-clock silicon Fmax test for spirix DUTs (Colorlight 5A-75B v8.0)
# Drives DUT from RC oscillator (pot between FPGA pins E16 and F15).
# BRAM-driven test vectors, sticky-fail latch on output mismatch, LED status.
# Usage: bash scripts/build_extclk.sh [--program] [--flash]
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/.."
SPIRIX_ROOT="$ROOT/.."
BUILD="$ROOT/build_extclk"
LPF="$ROOT/constraints/extclk.lpf"
VEC_MEM="$BUILD/div_p4_vectors.mem"
SEED="${SEED:-4}"

mkdir -p "$BUILD"

# Generate BRAM init (always regenerate — fast, deterministic from LFSR seed)
echo "--- Generate BRAM vectors ---"
cd "$SPIRIX_ROOT/paper/comparison"
cargo build --release --bin gen_div_p4_bram 2>&1 | tail -3
./target/release/gen_div_p4_bram "$VEC_MEM"
cd - > /dev/null

PROGRAM=0; FLASH=0
for arg in "$@"; do
    case "$arg" in
        --program) PROGRAM=1 ;;
        --flash)   FLASH=1   ;;
    esac
done

DUT_VERILOG="$SPIRIX_ROOT/paper/comparison/verilog/spirix_divide.v"
TOP_VERILOG="$ROOT/bench/top_extclk.v"
I2C_VERILOG="$ROOT/bench/ssd1306_i2c.v"
GLYPH_MEM_SRC="$ROOT/data/decimal_glyphs.mem"

# Stage glyph ROM and BRAM vectors in build dir for $readmemh
cp -f "$GLYPH_MEM_SRC" "$BUILD/decimal_glyphs.mem"

echo ""
echo "--- Synthesise ---"
cd "$BUILD"
yosys -p "
    read_verilog $I2C_VERILOG
    read_verilog $DUT_VERILOG
    read_verilog $TOP_VERILOG
    synth_ecp5 -nowidelut -abc2 -top top_extclk -json $BUILD/extclk.json
    stat
" 2>&1 | tee "$BUILD/extclk_yosys.log" | grep -E "(LUT4|CCU2C|TRELLIS_FF|Warning|Error)" | head -10
cd - > /dev/null

echo ""
echo "--- Place & Route ---"
nextpnr-ecp5 --25k --package CABGA256 --speed 6 --seed "$SEED" \
    --lpf "$LPF" \
    --json "$BUILD/extclk.json" \
    --textcfg "$BUILD/extclk.config" \
    --timing-allow-fail \
    2>&1 | tee "$BUILD/extclk_pnr.log" | grep -E "(Max frequency|Total LUT4|Warning|Error|Info: Device utilisation)" | head -15

echo ""
echo "--- Pack bitstream ---"
ecppack --svf "$BUILD/extclk.svf" "$BUILD/extclk.config" "$BUILD/extclk.bit"
echo "Bitstream: $BUILD/extclk.bit"

if [ "$PROGRAM" -eq 1 ]; then
    echo ""
    echo "--- Program SRAM ---"
    openFPGALoader -c ft232 "$BUILD/extclk.svf"
fi

if [ "$FLASH" -eq 1 ]; then
    echo ""
    echo "--- Write flash ---"
    openFPGALoader -c ft232 --write-flash "$BUILD/extclk.bit"
fi

echo ""
echo "Done."
