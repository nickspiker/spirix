#!/usr/bin/env bash
# Build the standalone frequency counter on Colorlight 5A-75B v8.0.
# RC oscillator wired between E16 (ext_clk) and F15 (osc_drive); FPGA inverts
# ext_clk back through the pot. OLED shows live frequency in MHz + the raw
# 32-bit count_per_frame as a binary ruler.
# Usage: bash scripts/build_extclk.sh [--program] [--flash]
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/.."
BUILD="$ROOT/build_extclk"
LPF="$ROOT/constraints/extclk.lpf"
SEED="${SEED:-4}"

mkdir -p "$BUILD"

PROGRAM=0; FLASH=0; PLL_TEST=0; PLL_FREQ=400; RING_TEST=0; RING_STAGES=7; FONT_TEST=0; BENCH=0; DDS_TEST=0; DDS_FREQ=250
for arg in "$@"; do
    case "$arg" in
        --program)   PROGRAM=1 ;;
        --flash)     FLASH=1   ;;
        --pll=*)     PLL_TEST=1; PLL_FREQ="${arg#--pll=}" ;;
        --pll)       PLL_TEST=1 ;;
        --ring=*)    RING_TEST=1; RING_STAGES="${arg#--ring=}" ;;
        --ring)      RING_TEST=1 ;;
        --font)      FONT_TEST=1 ;;
        --bench)     BENCH=1; PLL_TEST=1 ;;
        --dds=*)     DDS_TEST=1; DDS_FREQ="${arg#--dds=}" ;;
        --dds)       DDS_TEST=1 ;;
    esac
done

if [ "$RING_TEST" -eq 1 ] && [ $((RING_STAGES % 2)) -eq 0 ]; then
    echo "RING_STAGES must be odd for oscillation (got $RING_STAGES)"; exit 1
fi

YOSYS_DEFINES=""
if [ "$FONT_TEST" -eq 1 ]; then
    YOSYS_DEFINES="-DFONT_TEST"
    echo "FONT_TEST mode: top band shows digits 0-9 (debug glyph render)"
elif [ "$RING_TEST" -eq 1 ]; then
    YOSYS_DEFINES="-DRING_TEST -DRING_STAGES=$RING_STAGES"
    echo "RING_TEST mode: count_clk = ${RING_STAGES}-stage internal ring oscillator"
elif [ "$BENCH" -eq 1 ]; then
    PLL_FBDIV=$((PLL_FREQ / 25))
    if [ $((PLL_FBDIV * 25)) -ne "$PLL_FREQ" ]; then
        echo "PLL freq must be a multiple of 25 MHz (got $PLL_FREQ)"; exit 1
    fi
    YOSYS_DEFINES="-DPLL_TEST -DPLL_FBDIV=$PLL_FBDIV -DBENCH"
    echo "BENCH mode at ${PLL_FREQ} MHz: gold (CE=1/256) vs test (CE=1) on 32-bit passthru"
elif [ "$PLL_TEST" -eq 1 ]; then
    PLL_FBDIV=$((PLL_FREQ / 25))
    if [ $((PLL_FBDIV * 25)) -ne "$PLL_FREQ" ]; then
        echo "PLL freq must be a multiple of 25 MHz (got $PLL_FREQ)"; exit 1
    fi
    YOSYS_DEFINES="-DPLL_TEST -DPLL_FBDIV=$PLL_FBDIV"
    echo "PLL_TEST mode: count_clk = 25→${PLL_FREQ} MHz PLL output (FBDIV=$PLL_FBDIV)"
elif [ "$DDS_TEST" -eq 1 ]; then
    # PLL @ 1000 MHz feeds 32-bit DDS phase accumulator.
    # K = round(target_MHz * 2^32 / 1000)
    DDS_K=$(awk -v n="$DDS_FREQ" 'BEGIN { printf "%d", n * 4294967296 / 1000 + 0.5 }')
    YOSYS_DEFINES="-DDDS_TEST -DDDS_PLL_FBDIV=40 -DDDS_K=$DDS_K"
    echo "DDS_TEST mode: target ${DDS_FREQ} MHz (K=$DDS_K, PLL=1000 MHz)"
fi

TOP_VERILOG="$ROOT/bench/top_extclk.v"
I2C_VERILOG="$ROOT/bench/ssd1306_i2c.v"
GLYPH_MEM_SRC="$ROOT/data/decimal_glyphs.mem"

# Stage glyph ROM in build dir for $readmemh
cp -f "$GLYPH_MEM_SRC" "$BUILD/decimal_glyphs.mem"

echo "--- Synthesise ---"
cd "$BUILD"
yosys -p "
    read_verilog $I2C_VERILOG
    read_verilog $YOSYS_DEFINES $TOP_VERILOG
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
    --ignore-loops --timing-allow-fail \
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
