#!/bin/bash
# Synthesize Spirix and HardFloat binary32 subtractors, compare LUT counts and timing.
# Both use round-to-nearest-even (RNE). Target: ECP5-25F CABGA256.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FPGA_DIR="$SCRIPT_DIR/.."
HARDFLOAT_RTL="$FPGA_DIR/rtl/hardfloat"
cd "$FPGA_DIR"
mkdir -p sim/compare
rm -f sim/compare/summary.txt

echo "========================================"
echo " Spirix vs HardFloat — RNE, ECP5-25F"
echo "========================================"
echo ""

#---------------------------------------------------------------------------
# Synthesize + place-route a Spirix subtractor, extract LUTs and timing
#---------------------------------------------------------------------------
synth_spirix() {
    local NAME=$1
    local FRAC=$2
    local EXP=$3
    local OUT="sim/compare/${NAME}"

    echo "--- Spirix $NAME (FRAC_BITS=$FRAC, EXP_BITS=$EXP, RNE) ---"

    cat > /tmp/synth_top_${NAME}.v << EOF
module synth_top_${NAME} (
    input  wire signed [${FRAC}-1:0] a_frac,
    input  wire signed [${EXP}-1:0]  a_exp,
    input  wire signed [${FRAC}-1:0] b_frac,
    input  wire signed [${EXP}-1:0]  b_exp,
    output wire signed [${FRAC}-1:0] result_frac,
    output wire signed [${EXP}-1:0]  result_exp
);
    spirix_subtract #(
        .FRAC_BITS(${FRAC}),
        .EXP_BITS(${EXP})
    ) sub (
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(result_frac), .result_exp(result_exp)
    );
endmodule
EOF

    yosys -p "
        read_verilog rtl/spirix_subtract.v
        read_verilog /tmp/synth_top_${NAME}.v
        synth_ecp5 -top synth_top_${NAME} -json ${OUT}.json
        stat
    " > ${OUT}_yosys.log 2>&1

    local LUTS=$(grep -E "^\s+[0-9]+ +LUT4$" ${OUT}_yosys.log | grep -o '[0-9]*' | sort -n | tail -1)
    echo "  LUTs: ${LUTS:-unknown}"

    # Place and route to get timing (combinational block — no clock constraint needed)
    nextpnr-ecp5 --25k --package CABGA256 --speed 6 \
        --json ${OUT}.json \
        --textcfg ${OUT}.config \
        --lpf-allow-unconstrained \
        --timing-allow-fail \
        > ${OUT}_pnr.log 2>&1 || true

    local LOGIC_NS=$(python3 -c "
import re, sys
for line in open('${OUT}_pnr.log'):
    m = re.search(r'(\d+\.\d+) ns logic', line)
    if m: print(m.group(1)); sys.exit()
" 2>/dev/null)
    echo "  Logic critical path: ${LOGIC_NS:-?} ns"

    echo "Spirix_${NAME}  FRAC=${FRAC}  EXP=${EXP}  LUTs=${LUTS:-?}  LogicNs=${LOGIC_NS:-?}" >> sim/compare/summary.txt
}

#---------------------------------------------------------------------------
# Synthesize + place-route HardFloat binary32 subtractor
#---------------------------------------------------------------------------
synth_hardfloat_f32() {
    local NAME="HardFloat_f32"
    local OUT="sim/compare/${NAME}"

    echo "--- HardFloat binary32 (expWidth=8, sigWidth=24, RNE) ---"

    cat > /tmp/synth_top_${NAME}.v << 'EOF'
module synth_top_HardFloat_f32 (
    input  wire [32:0] a,
    input  wire [32:0] b,
    output wire [32:0] out,
    output wire [4:0]  exceptionFlags
);
    sub_f32 sub (
        .a            (a),
        .b            (b),
        .roundingMode (3'b000),   // RNE
        .out          (out),
        .exceptionFlags(exceptionFlags)
    );
endmodule
EOF

    yosys -p "
        read_verilog -I${HARDFLOAT_RTL} ${HARDFLOAT_RTL}/HardFloat_primitives.v
        read_verilog -I${HARDFLOAT_RTL} ${HARDFLOAT_RTL}/HardFloat_rawFN.v
        read_verilog -I${HARDFLOAT_RTL} ${HARDFLOAT_RTL}/HardFloat_specialize.v
        read_verilog -I${HARDFLOAT_RTL} ${HARDFLOAT_RTL}/isSigNaNRecFN.v
        read_verilog -I${HARDFLOAT_RTL} ${HARDFLOAT_RTL}/addRecFN.v
        read_verilog -I${HARDFLOAT_RTL} ${HARDFLOAT_RTL}/sub_f32.v
        read_verilog /tmp/synth_top_${NAME}.v
        synth_ecp5 -top synth_top_HardFloat_f32 -json ${OUT}.json
        stat
    " > ${OUT}_yosys.log 2>&1

    local LUTS=$(grep -E "^\s+[0-9]+ +LUT4$" ${OUT}_yosys.log | grep -o '[0-9]*' | sort -n | tail -1)
    echo "  LUTs: ${LUTS:-unknown}"

    nextpnr-ecp5 --25k --package CABGA256 --speed 6 \
        --json ${OUT}.json \
        --textcfg ${OUT}.config \
        --lpf-allow-unconstrained \
        --timing-allow-fail \
        > ${OUT}_pnr.log 2>&1 || true

    local LOGIC_NS=$(python3 -c "
import re, sys
for line in open('${OUT}_pnr.log'):
    m = re.search(r'(\d+\.\d+) ns logic', line)
    if m: print(m.group(1)); sys.exit()
" 2>/dev/null)
    echo "  Logic critical path: ${LOGIC_NS:-?} ns"

    echo "HardFloat_f32   expW=8  sigW=24  LUTs=${LUTS:-?}  LogicNs=${LOGIC_NS:-?}" >> sim/compare/summary.txt
}

# Run
synth_spirix "F4E4"  16  16
synth_spirix "i25i8"  25   8
synth_hardfloat_f32

echo ""
echo "========================================"
echo " Summary (RNE, ECP5-25F speed-6)"
echo "========================================"
cat sim/compare/summary.txt
echo ""
echo "Logs in sim/compare/"
