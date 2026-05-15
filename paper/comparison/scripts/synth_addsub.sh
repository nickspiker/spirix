#!/bin/bash
# Synthesize the comparison-paper spirix_addsub (banker's rounded, FRAC=24, EXP=8)
# at ECP5-25F speed-6 and report LUT4 / DSP / Fmax estimate.
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMP_DIR="$SCRIPT_DIR/.."
cd "$COMP_DIR"

mkdir -p sim
RTL="verilog/spirix_addsub.v"
OUT="sim/synth_addsub"

# Wrap the module so all I/O are top-level pins (synthesis sees realistic boundary).
cat > /tmp/synth_top_addsub.v << 'EOF'
module synth_top_addsub (
    input  wire signed [23:0] a_frac,
    input  wire signed [7:0]  a_exp,
    input  wire signed [23:0] b_frac,
    input  wire signed [7:0]  b_exp,
    input  wire               sub,
    output wire signed [23:0] result_frac,
    output wire signed [7:0]  result_exp
);
    spirix_addsub #(.FRAC_BITS(24), .EXP_BITS(8)) dut (
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .sub(sub),
        .result_frac(result_frac), .result_exp(result_exp)
    );
endmodule
EOF

echo "── Default (DSP allowed, widelut allowed) ──"
yosys -q -p "
    read_verilog $RTL
    read_verilog /tmp/synth_top_addsub.v
    synth_ecp5 -top synth_top_addsub -json $OUT.json
" 2>&1 | tail -25

echo ""
echo "── No DSP, no widelut (pure LUT4 area count for ASIC-equivalent) ──"
yosys -q -p "
    read_verilog $RTL
    read_verilog /tmp/synth_top_addsub.v
    synth_ecp5 -top synth_top_addsub -nodsp -nowidelut -json ${OUT}_nodsp.json
" 2>&1 | tail -25

echo ""
echo "── Place & route timing (default) ──"
nextpnr-ecp5 --25k --package CABGA256 --speed 6 --seed 4 \
    --json $OUT.json --timing-allow-fail --report ${OUT}_pnr.json \
    2>&1 | grep -E "Max frequency|Info: Device utilisation" | head -10
