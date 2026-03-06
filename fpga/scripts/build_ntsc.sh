#!/bin/bash
# Build NTSC CRT self-test for Colorlight 5A-75B (ECP5-25F).
#
# Usage:
#   ./build_ntsc.sh [FREQ_MHZ] [--program]
#
# FREQ_MHZ=25 uses raw clock (no PLL). Any other frequency uses PLL.
#
# Examples:
#   ./build_ntsc.sh                 # build at 25 MHz (no PLL)
#   ./build_ntsc.sh 50              # build at 50 MHz (PLL)
#   ./build_ntsc.sh 50 --program    # build + program
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FPGA_DIR="$SCRIPT_DIR/.."
RTL="$FPGA_DIR/rtl"
LPF="$FPGA_DIR/constraints/colorlight_5a75b_v8.lpf"
BUILD="$FPGA_DIR/build"

FREQ="${1:-25}"
PROGRAM="${2:-}"

mkdir -p "$BUILD"
cd "$FPGA_DIR"

# -------------------------------------------------------------------------
# Compute PLL parameters (skip if 25 MHz)
# -------------------------------------------------------------------------
PLL_DEFINES=""
if [ "$FREQ" != "25" ]; then
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
    echo " NTSC self-test @ ${ACTUAL_FREQ} MHz"
    echo " PLL: CLKI=$CLKI CLKFB=$CLKFB CLKOP=$CLKOP (VCO=${FVCO} MHz)"
    echo "================================================================"
else
    echo "================================================================"
    echo " NTSC self-test @ 25 MHz (no PLL)"
    echo "================================================================"
fi

# -------------------------------------------------------------------------
# DUT selection: DUT=hf_fma | hf_mul | hf_add | spirix_fma (default)
# -------------------------------------------------------------------------
DUT="${DUT:-}"
DUT_DEFINE=""
HF_FILES=""
HF_DIR="$RTL/hardfloat"

case "$DUT" in
    hf_fma)
        DUT_DEFINE="-DDUT_HF_FMA"
        HF_FILES="read_verilog -I$HF_DIR $HF_DIR/HardFloat_primitives.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/HardFloat_rawFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/HardFloat_specialize.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/isSigNaNRecFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/fNToRecFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/recFNToFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/mulAddRecFN.v;"
        echo "  DUT: HardFloat mulAddRecFN (FMA) with IEEE 754 I/O"
        ;;
    hf_mul)
        DUT_DEFINE="-DDUT_HF_MUL"
        HF_FILES="read_verilog -I$HF_DIR $HF_DIR/HardFloat_primitives.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/HardFloat_rawFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/HardFloat_specialize.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/isSigNaNRecFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/fNToRecFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/recFNToFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/mulRecFN.v;"
        echo "  DUT: HardFloat mulRecFN (multiply) with IEEE 754 I/O"
        ;;
    hf_add)
        DUT_DEFINE="-DDUT_HF_ADD"
        HF_FILES="read_verilog -I$HF_DIR $HF_DIR/HardFloat_primitives.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/HardFloat_rawFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/HardFloat_specialize.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/isSigNaNRecFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/fNToRecFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/recFNToFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/addRecFN.v;"
        echo "  DUT: HardFloat addRecFN (add/sub) with IEEE 754 I/O"
        ;;
    fpn_fma)
        DUT_DEFINE="-DDUT_FPN_FMA"
        FPN_V="$SCRIPT_DIR/../rtl/fpnew_v/fpnew_fma_fp32.v"
        HF_FILES="read_verilog $FPN_V;"
        echo "  DUT: FPnew FMA (native IEEE 754)"
        ;;
    fpn_mul)
        DUT_DEFINE="-DDUT_FPN_MUL"
        FPN_V="$SCRIPT_DIR/../rtl/fpnew_v/fpnew_fma_fp32.v"
        HF_FILES="read_verilog $FPN_V;"
        echo "  DUT: FPnew multiply (native IEEE 754)"
        ;;
    fpn_add)
        DUT_DEFINE="-DDUT_FPN_ADD"
        FPN_V="$SCRIPT_DIR/../rtl/fpnew_v/fpnew_fma_fp32.v"
        HF_FILES="read_verilog $FPN_V;"
        echo "  DUT: FPnew add/sub (native IEEE 754)"
        ;;
    spirix_mul)
        DUT_DEFINE="-DDUT_SPIRIX_MUL"
        echo "  DUT: Spirix multiply (standalone)"
        ;;
    spirix_mul_pipe2)
        DUT_DEFINE="-DDUT_SPIRIX_MUL_PIPE2"
        echo "  DUT: Spirix multiply_pipe2 (2-stage pipeline)"
        ;;
    spirix_div_iter)
        DUT_DEFINE="-DDUT_SPIRIX_DIV_ITER"
        echo "  DUT: Spirix divide_iter (iterative, 0 DSP)"
        ;;
    spirix_divmod_nr)
        DUT_DEFINE="-DDUT_SPIRIX_DIVMOD_NR"
        echo "  DUT: Spirix divmod_nr (8-stage pipeline, 20 DSP)"
        ;;
    spirix_sqrt_nr)
        DUT_DEFINE="-DDUT_SPIRIX_SQRT_NR"
        echo "  DUT: Spirix sqrt_nr (10-stage pipeline, 27 DSP)"
        ;;
    spirix_sqrt_iter)
        DUT_DEFINE="-DDUT_SPIRIX_SQRT_ITER"
        echo "  DUT: Spirix sqrt_iter (iterative, 0 DSP)"
        ;;
    hf_div)
        DUT_DEFINE="-DDUT_HF_DIV"
        HF_FILES="read_verilog -I$HF_DIR $HF_DIR/HardFloat_primitives.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/HardFloat_rawFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/HardFloat_specialize.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/isSigNaNRecFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/fNToRecFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/recFNToFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/divSqrtRecFN_small.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/div_f32.v;"
        echo "  DUT: HardFloat div (divSqrtRecFN_small) with IEEE 754 I/O"
        ;;
    hf_sqrt)
        DUT_DEFINE="-DDUT_HF_SQRT"
        HF_FILES="read_verilog -I$HF_DIR $HF_DIR/HardFloat_primitives.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/HardFloat_rawFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/HardFloat_specialize.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/isSigNaNRecFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/fNToRecFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/recFNToFN.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/divSqrtRecFN_small.v;"
        HF_FILES="$HF_FILES read_verilog -I$HF_DIR $HF_DIR/sqrt_f32.v;"
        echo "  DUT: HardFloat sqrt (divSqrtRecFN_small) with IEEE 754 I/O"
        ;;
    fpn_div)
        DUT_DEFINE="-DDUT_FPN_DIV"
        FPN_V="$SCRIPT_DIR/../rtl/fpnew_v/fpnew_divsqrt_fp32.v"
        HF_FILES="read_verilog $FPN_V;"
        echo "  DUT: FPnew div (fpnew_divsqrt_th_32, op=DIV)"
        ;;
    fpn_sqrt)
        DUT_DEFINE="-DDUT_FPN_SQRT"
        FPN_V="$SCRIPT_DIR/../rtl/fpnew_v/fpnew_divsqrt_fp32.v"
        HF_FILES="read_verilog $FPN_V;"
        echo "  DUT: FPnew sqrt (fpnew_divsqrt_th_32, op=SQRT)"
        ;;
    "")
        echo "  DUT: Spirix FMA (default)"
        ;;
    *)
        echo "ERROR: unknown DUT='$DUT'. Use: hf_fma/mul/add, fpn_fma/mul/add/div/sqrt, spirix_mul/mul_pipe2/div_iter/divmod_nr/sqrt_nr/sqrt_iter, hf_div/sqrt, or empty."
        exit 1
        ;;
esac

# -------------------------------------------------------------------------
# Collect all RTL files
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
    $HF_FILES
    read_verilog $RTL/ntsc_framebuf.v
    read_verilog $PLL_DEFINES $DUT_DEFINE $RTL/top_ntsc.v
    synth_ecp5 ${NODSP:+-nodsp} -top top_ntsc -json $BUILD/ntsc.json
    stat
" > "$BUILD/ntsc_yosys.log" 2>&1

grep -E '^\s+[0-9]+ +(LUT4|TRELLIS_FF|MULT18X18D|CCU2C|EHXPLLL|DP16KD)$' "$BUILD/ntsc_yosys.log" || true
if grep -q 'ERROR' "$BUILD/ntsc_yosys.log"; then
    echo "YOSYS ERROR — see $BUILD/ntsc_yosys.log"
    exit 1
fi

# -------------------------------------------------------------------------
# Place and route
# -------------------------------------------------------------------------
echo ""
echo "--- Place & Route ---"
SEED="${SEED:-1}"
nextpnr-ecp5 --25k --package CABGA256 --speed 6 --seed "$SEED" \
    --json "$BUILD/ntsc.json" \
    --lpf "$LPF" \
    --textcfg "$BUILD/ntsc.config" \
    > "$BUILD/ntsc_pnr.log" 2>&1 || true

grep -E '(Max frequency|logic,)' "$BUILD/ntsc_pnr.log" || true
if grep -q 'ERROR' "$BUILD/ntsc_pnr.log"; then
    echo "PNR ERROR — see $BUILD/ntsc_pnr.log"
    exit 1
fi

# -------------------------------------------------------------------------
# Pack bitstream + SVF
# -------------------------------------------------------------------------
echo ""
echo "--- Pack ---"
ecppack --svf "$BUILD/ntsc.svf" "$BUILD/ntsc.config" "$BUILD/ntsc.bit"
echo "Bitstream: $BUILD/ntsc.bit ($(stat -c%s "$BUILD/ntsc.bit") bytes)"

# -------------------------------------------------------------------------
# Program
# -------------------------------------------------------------------------
if [ "$PROGRAM" = "--program" ]; then
    echo ""
    echo "--- Program (SVF/SRAM) ---"
    openFPGALoader -c ft232 "$BUILD/ntsc.svf"
    echo "Done. Check your CRT!"
else
    echo ""
    echo "Done. To program:"
    echo "  openFPGALoader -c ft232 $BUILD/ntsc.svf"
fi
