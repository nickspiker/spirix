#!/bin/bash
# Build and program a self-test bitstream for the Colorlight 5A-75B (ECP5-25F).
#
# Usage:
#   ./build_selftest.sh [TOP_FILE] [FREQ_MHZ] [--program]
#
# FREQ_MHZ=25 uses raw clock (no PLL). Any other frequency uses PLL.
#
# Examples:
#   ./build_selftest.sh                                    # build at 25 MHz (no PLL)
#   ./build_selftest.sh rtl/top_selftest_mul.v 80          # build at 80 MHz (PLL)
#   ./build_selftest.sh rtl/top_selftest_mul.v 40 --program  # build + program
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FPGA_DIR="$SCRIPT_DIR/.."
RTL="$FPGA_DIR/rtl"
LPF="$FPGA_DIR/constraints/colorlight_5a75b_v8.lpf"
BUILD="$FPGA_DIR/build"

TOP_FILE="${1:-$RTL/top_selftest_mul.v}"
FREQ="${2:-25}"
PROGRAM="${3:-}"

mkdir -p "$BUILD"
cd "$FPGA_DIR"

# -------------------------------------------------------------------------
# Compute PLL parameters (skip if 25 MHz)
# Fvco = 25 * CLKFB / CLKI, must be [450, 750] MHz (conservative)
# Fout = Fvco / CLKOP
# -------------------------------------------------------------------------
PLL_DEFINES=""
if [ "$FREQ" != "25" ]; then
    # Use ecppll (Project Trellis) for known-good PLL parameters
    PLL_OUT=$(ecppll -i 25 -o "$FREQ" -f /dev/null 2>&1)
    CLKI=$(echo "$PLL_OUT"  | awk '/Refclk divisor:/  {print $3}')
    CLKFB=$(echo "$PLL_OUT" | awk '/Feedback divisor:/ {print $3}')
    CLKOP=$(echo "$PLL_OUT" | awk '/clkout0 divisor:/  {print $3}')
    ACTUAL_FREQ=$(echo "$PLL_OUT" | awk '/clkout0 frequency:/ {print $3}')
    FVCO=$(echo "$PLL_OUT"  | awk '/VCO frequency:/    {print $3}')
    if [ -z "$CLKI" ] || [ -z "$CLKFB" ] || [ -z "$CLKOP" ]; then
        echo "ERROR: ecppll failed for ${FREQ} MHz"
        echo "$PLL_OUT"
        exit 1
    fi
    CPHASE=$(( (CLKOP - 1) / 2 ))
    PLL_DEFINES="-DPLL_CLKI_DIV=$CLKI -DPLL_CLKFB_DIV=$CLKFB -DPLL_CLKOP_DIV=$CLKOP -DPLL_CLKOP_CPHASE=$CPHASE"
    echo "================================================================"
    echo " Self-test: $(basename $TOP_FILE) @ ${ACTUAL_FREQ} MHz"
    echo " PLL: CLKI=$CLKI CLKFB=$CLKFB CLKOP=$CLKOP (VCO=${FVCO} MHz)"
    echo "================================================================"
else
    echo "================================================================"
    echo " Self-test: $(basename $TOP_FILE) @ 25 MHz (no PLL)"
    echo "================================================================"
fi

# -------------------------------------------------------------------------
# Collect all RTL files needed (all spirix_*.v in rtl/)
# -------------------------------------------------------------------------
RTL_FILES=""
for f in "$RTL"/spirix_*.v; do
    RTL_FILES="$RTL_FILES read_verilog $f;"
done

# -------------------------------------------------------------------------
# Synthesize
# -------------------------------------------------------------------------
echo ""
echo "--- Synthesize ---"
yosys -p "
    $RTL_FILES
    read_verilog $PLL_DEFINES $TOP_FILE
    synth_ecp5 -top top_selftest -json $BUILD/selftest.json
    stat
" > $BUILD/selftest_yosys.log 2>&1

grep -E '^\s+[0-9]+ +(LUT4|TRELLIS_FF|MULT18X18D|CCU2C|EHXPLLL)$' $BUILD/selftest_yosys.log || true
if grep -q 'ERROR' $BUILD/selftest_yosys.log; then
    echo "SYNTHESIS FAILED — see $BUILD/selftest_yosys.log"
    exit 1
fi

# -------------------------------------------------------------------------
# Place and route
# -------------------------------------------------------------------------
echo ""
echo "--- Place & Route ---"
nextpnr-ecp5 --25k --package CABGA256 --speed 6 \
    --json "$BUILD/selftest.json" \
    --lpf "$LPF" \
    --textcfg "$BUILD/selftest.config" \
    > $BUILD/selftest_pnr.log 2>&1 || true

grep -E '(Max frequency|logic,)' $BUILD/selftest_pnr.log || true
if grep -q 'ERROR' $BUILD/selftest_pnr.log; then
    echo "PNR FAILED — see $BUILD/selftest_pnr.log"
    exit 1
fi

# -------------------------------------------------------------------------
# Pack bitstream + SVF
# -------------------------------------------------------------------------
echo ""
echo "--- Pack ---"
ecppack --svf "$BUILD/selftest.svf" "$BUILD/selftest.config" "$BUILD/selftest.bit"
echo "Bitstream: $BUILD/selftest.bit ($(stat -c%s "$BUILD/selftest.bit") bytes)"

# -------------------------------------------------------------------------
# Program via SVF (SRAM, volatile — power cycle resets to flash)
# -------------------------------------------------------------------------
if [ "$PROGRAM" = "--program" ]; then
    echo ""
    echo "--- Program (SVF/SRAM) ---"
    openFPGALoader -c ft232 "$BUILD/selftest.svf"
fi

echo ""
echo "Done. To program manually:"
echo "  openFPGALoader -c ft232 $BUILD/selftest.svf"
