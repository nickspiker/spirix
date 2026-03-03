#!/bin/bash
# Comprehensive Spirix vs HardFloat synthesis comparison.
# Target: ECP5-25F CABGA256 speed-6, binary32-equivalent precision.
# Spirix: FRAC=25, EXP=8 | HardFloat: expWidth=8, sigWidth=24
#
# Each module is synthesized twice:
#   normal   — uses PFUMX/L6MUX wide LUTs + CCU2C carry chains
#   -nowidelut — pure LUT4 only (fairer cross-platform comparison)
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FPGA_DIR="$SCRIPT_DIR/.."
RTL="$FPGA_DIR/rtl"
HF="$RTL/hardfloat"
cd "$FPGA_DIR"
mkdir -p sim/compare_all

FRAC=25
EXP=8
OUT_DIR="sim/compare_all"
SUMMARY="$OUT_DIR/summary.txt"
> "$SUMMARY"

# HardFloat common Verilog dependencies
HF_COMMON="$HF/HardFloat_primitives.v $HF/HardFloat_rawFN.v $HF/HardFloat_specialize.v $HF/isSigNaNRecFN.v"

echo "================================================================"
echo " Spirix vs HardFloat — ECP5-25F speed-6, binary32 precision"
echo " Spirix: FRAC=$FRAC EXP=$EXP | HardFloat: expWidth=8 sigWidth=24"
echo "================================================================"
echo ""

#---------------------------------------------------------------------------
# Core synthesis function
#   synth_one NAME TOP "VFILES..." WRAPPER [--nowidelut]
#---------------------------------------------------------------------------
synth_one() {
    local NAME=$1
    local TOP=$2
    local VFILES=$3
    local WRAPPER=$4
    local NWL=""
    [ "${5:-}" = "--nowidelut" ] && NWL="-nowidelut"

    local SUFFIX=""
    [ -n "$NWL" ] && SUFFIX="_nwl"
    local OUT="$OUT_DIR/${NAME}${SUFFIX}"

    local READ_CMD=""
    for f in $VFILES; do
        READ_CMD="$READ_CMD read_verilog -I${HF} $f;"
    done
    [ -n "$WRAPPER" ] && READ_CMD="$READ_CMD read_verilog -I${HF} $WRAPPER;"

    yosys -p "
        $READ_CMD
        synth_ecp5 ${NWL} -top $TOP -json ${OUT}.json
        stat
    " > ${OUT}_yosys.log 2>&1

    local LUTS=$(grep -E '^\s+[0-9]+ +LUT4$' ${OUT}_yosys.log | awk '{print $1}' | tail -1)
    local FFS=$(grep -E '^\s+[0-9]+ +TRELLIS_FF$' ${OUT}_yosys.log | awk '{print $1}' | tail -1)
    local DSPS=$(grep -E '^\s+[0-9]+ +MULT18X18D$' ${OUT}_yosys.log | awk '{print $1}' | tail -1)
    local CCUS=$(grep -E '^\s+[0-9]+ +CCU2C$' ${OUT}_yosys.log | awk '{print $1}' | tail -1)

    nextpnr-ecp5 --25k --package CABGA256 --speed 6 \
        --json ${OUT}.json \
        --textcfg ${OUT}.config \
        --lpf-allow-unconstrained \
        --timing-allow-fail \
        > ${OUT}_pnr.log 2>&1 || true

    local FMAX=$(python3 -c "
import re, sys
for line in open('${OUT}_pnr.log'):
    m = re.search(r'Max frequency for clock.*?:\s*(\d+\.\d+)\s*MHz', line)
    if m: print(m.group(1)); sys.exit()
print('-')
" 2>/dev/null)

    local LOGIC_NS=$(python3 -c "
import re, sys
for line in open('${OUT}_pnr.log'):
    m = re.search(r'(\d+\.\d+) ns logic', line)
    if m: print(m.group(1)); sys.exit()
print('-')
" 2>/dev/null)

    local ROUTE_NS=$(python3 -c "
import re, sys
for line in open('${OUT}_pnr.log'):
    m = re.search(r'(\d+\.\d+) ns routing', line)
    if m: print(m.group(1)); sys.exit()
print('-')
" 2>/dev/null)

    printf "  %-32s LUT4=%-5s FF=%-4s DSP=%-3s CCU2C=%-4s Fmax=%-8s Logic=%-6s Route=%-6s\n" \
        "${NAME}${SUFFIX}" "${LUTS:--}" "${FFS:--}" "${DSPS:--}" "${CCUS:--}" \
        "${FMAX:--}" "${LOGIC_NS:--}" "${ROUTE_NS:--}"

    printf "%-32s %5s %5s %4s %5s %8s %7s %7s\n" \
        "${NAME}${SUFFIX}" "${LUTS:--}" "${FFS:--}" "${DSPS:--}" "${CCUS:--}" \
        "${FMAX:--}" "${LOGIC_NS:--}" "${ROUTE_NS:--}" >> "$SUMMARY"
}

#---------------------------------------------------------------------------
# Helper: run a module with both normal and -nowidelut
#---------------------------------------------------------------------------
synth_both() {
    synth_one "$@"
    synth_one "$@" --nowidelut
}

#---------------------------------------------------------------------------
# Summary header
#---------------------------------------------------------------------------
printf "%-32s %5s %5s %4s %5s %8s %7s %7s\n" \
    "Module" "LUT4" "FF" "DSP" "CCU2C" "Fmax" "Logic" "Route" >> "$SUMMARY"
printf "%-32s %5s %5s %4s %5s %8s %7s %7s\n" \
    "--------------------------------" "-----" "-----" "----" "-----" "--------" "-------" "-------" >> "$SUMMARY"

#===========================================================================
# ADD / SUB
#===========================================================================
echo "--- ADD/SUB ---"
echo "" >> "$SUMMARY"
echo "--- ADD/SUB ---" >> "$SUMMARY"

# Spirix addsub (combinational, registered wrapper)
cat > /tmp/bench_spirix_addsub.v << EOF
module bench_spirix_addsub (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac, b_frac,
    input  wire signed [${EXP}-1:0]  a_exp, b_exp,
    input  wire sub,
    output reg  signed [${FRAC}-1:0] result_frac,
    output reg  signed [${EXP}-1:0]  result_exp
);
    reg signed [${FRAC}-1:0] a_frac_r, b_frac_r;
    reg signed [${EXP}-1:0]  a_exp_r, b_exp_r;
    reg sub_r;
    always @(posedge clk) begin
        a_frac_r <= a_frac; a_exp_r <= a_exp;
        b_frac_r <= b_frac; b_exp_r <= b_exp;
        sub_r <= sub;
    end
    wire signed [${FRAC}-1:0] w_frac;
    wire signed [${EXP}-1:0]  w_exp;
    spirix_addsub #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP})) dut (
        .a_frac(a_frac_r), .a_exp(a_exp_r),
        .b_frac(b_frac_r), .b_exp(b_exp_r),
        .sub(sub_r),
        .result_frac(w_frac), .result_exp(w_exp)
    );
    always @(posedge clk) begin result_frac <= w_frac; result_exp <= w_exp; end
endmodule
EOF

synth_both "spirix_addsub" "bench_spirix_addsub" \
    "$RTL/spirix_addsub.v" "/tmp/bench_spirix_addsub.v"

# Spirix addsub_pipe2
cat > /tmp/bench_spirix_addsub_pipe2.v << EOF
module bench_spirix_addsub_pipe2 (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac, b_frac,
    input  wire signed [${EXP}-1:0]  a_exp, b_exp,
    input  wire sub,
    output wire signed [${FRAC}-1:0] result_frac,
    output wire signed [${EXP}-1:0]  result_exp
);
    spirix_addsub_pipe2 #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP})) dut (
        .clk(clk), .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp), .sub(sub),
        .result_frac(result_frac), .result_exp(result_exp));
endmodule
EOF

synth_both "spirix_addsub_pipe2" "bench_spirix_addsub_pipe2" \
    "$RTL/spirix_addsub_pipe2.v" "/tmp/bench_spirix_addsub_pipe2.v"

# HardFloat add (combinational, existing bench wrapper)
synth_both "hardfloat_add" "bench_hardfloat_add" \
    "$HF_COMMON $HF/addRecFN.v $HF/add_f32.v" "$HF/bench_hardfloat_add.v"

#===========================================================================
# MULTIPLY
#===========================================================================
echo "--- MULTIPLY ---"
echo "" >> "$SUMMARY"
echo "--- MULTIPLY ---" >> "$SUMMARY"

# Spirix multiply (combinational, registered wrapper)
cat > /tmp/bench_spirix_mul.v << EOF
module bench_spirix_mul (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac, b_frac,
    input  wire signed [${EXP}-1:0]  a_exp, b_exp,
    output reg  signed [${FRAC}-1:0] result_frac,
    output reg  signed [${EXP}-1:0]  result_exp
);
    reg signed [${FRAC}-1:0] a_frac_r, b_frac_r;
    reg signed [${EXP}-1:0]  a_exp_r, b_exp_r;
    always @(posedge clk) begin
        a_frac_r <= a_frac; a_exp_r <= a_exp;
        b_frac_r <= b_frac; b_exp_r <= b_exp;
    end
    wire signed [${FRAC}-1:0] w_frac;
    wire signed [${EXP}-1:0]  w_exp;
    spirix_multiply #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP})) dut (
        .a_frac(a_frac_r), .a_exp(a_exp_r),
        .b_frac(b_frac_r), .b_exp(b_exp_r),
        .result_frac(w_frac), .result_exp(w_exp)
    );
    always @(posedge clk) begin result_frac <= w_frac; result_exp <= w_exp; end
endmodule
EOF

synth_both "spirix_mul" "bench_spirix_mul" \
    "$RTL/spirix_multiply.v" "/tmp/bench_spirix_mul.v"

# Spirix multiply_pipe2
cat > /tmp/bench_spirix_mul_pipe2.v << EOF
module bench_spirix_mul_pipe2 (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac, b_frac,
    input  wire signed [${EXP}-1:0]  a_exp, b_exp,
    output wire signed [${FRAC}-1:0] result_frac,
    output wire signed [${EXP}-1:0]  result_exp
);
    spirix_multiply_pipe2 #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP})) dut (
        .clk(clk), .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(result_frac), .result_exp(result_exp));
endmodule
EOF

synth_both "spirix_mul_pipe2" "bench_spirix_mul_pipe2" \
    "$RTL/spirix_multiply_pipe2.v" "/tmp/bench_spirix_mul_pipe2.v"

# Spirix multiply_pipe3
cat > /tmp/bench_spirix_mul_pipe3.v << EOF
module bench_spirix_mul_pipe3 (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac, b_frac,
    input  wire signed [${EXP}-1:0]  a_exp, b_exp,
    output wire signed [${FRAC}-1:0] result_frac,
    output wire signed [${EXP}-1:0]  result_exp
);
    spirix_multiply_pipe3 #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP})) dut (
        .clk(clk), .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(result_frac), .result_exp(result_exp));
endmodule
EOF

synth_both "spirix_mul_pipe3" "bench_spirix_mul_pipe3" \
    "$RTL/spirix_multiply_pipe3.v" "/tmp/bench_spirix_mul_pipe3.v"

# HardFloat mul (combinational, registered wrapper)
cat > /tmp/bench_hardfloat_mul.v << 'EOF'
module bench_hardfloat_mul (
    input  wire        clk,
    input  wire [32:0] a_in,
    input  wire [32:0] b_in,
    output reg  [32:0] out_out,
    output reg  [4:0]  flags_out
);
    reg [32:0] a, b;
    always @(posedge clk) begin a <= a_in; b <= b_in; end
    wire [32:0] r_out;
    wire [4:0]  r_flags;
    mul_f32 mul (.a(a), .b(b), .roundingMode(3'b000), .out(r_out), .exceptionFlags(r_flags));
    always @(posedge clk) begin out_out <= r_out; flags_out <= r_flags; end
endmodule
EOF

synth_both "hardfloat_mul" "bench_hardfloat_mul" \
    "$HF_COMMON $HF/mulRecFN.v $HF/mul_f32.v" "/tmp/bench_hardfloat_mul.v"

#===========================================================================
# DIVIDE
#===========================================================================
echo "--- DIVIDE ---"
echo "" >> "$SUMMARY"
echo "--- DIVIDE ---" >> "$SUMMARY"

# Spirix divmod_nr MOD=0 (6-stage pipelined NR)
cat > /tmp/bench_spirix_div_nr.v << EOF
module bench_spirix_div_nr (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac, b_frac,
    input  wire signed [${EXP}-1:0]  a_exp, b_exp,
    output wire signed [${FRAC}-1:0] q_frac,
    output wire signed [${EXP}-1:0]  q_exp
);
    wire signed [${FRAC}-1:0] mod_frac;
    wire signed [${EXP}-1:0]  mod_exp;
    spirix_divmod_nr #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP}), .ENABLE_MOD(0)) dut (
        .clk(clk), .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .q_frac(q_frac), .q_exp(q_exp),
        .mod_frac(mod_frac), .mod_exp(mod_exp));
endmodule
EOF

synth_both "spirix_div_nr" "bench_spirix_div_nr" \
    "$RTL/spirix_divmod_nr.v" "/tmp/bench_spirix_div_nr.v"

# Spirix divide_iter (~26 cycles iterative)
cat > /tmp/bench_spirix_div_iter.v << EOF
module bench_spirix_div_iter (
    input  wire clk,
    input  wire start,
    input  wire signed [${FRAC}-1:0] a_frac, b_frac,
    input  wire signed [${EXP}-1:0]  a_exp, b_exp,
    output wire signed [${FRAC}-1:0] result_frac,
    output wire signed [${EXP}-1:0]  result_exp,
    output wire busy, done
);
    spirix_divide_iter #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP})) dut (
        .clk(clk), .start(start),
        .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .result_frac(result_frac), .result_exp(result_exp),
        .busy(busy), .done(done));
endmodule
EOF

synth_both "spirix_div_iter" "bench_spirix_div_iter" \
    "$RTL/spirix_divide_iter.v" "/tmp/bench_spirix_div_iter.v"

# HardFloat div (~26 cycles iterative)
cat > /tmp/bench_hardfloat_div.v << 'EOF'
module bench_hardfloat_div (
    input  wire        clock,
    input  wire [32:0] a,
    input  wire [32:0] b,
    output wire        inReady,
    output wire        outValid,
    output wire [32:0] out,
    output wire [4:0]  exceptionFlags
);
    div_f32 dut (
        .nReset       (1'b1),
        .clock        (clock),
        .inReady      (inReady),
        .inValid      (1'b1),
        .a            (a),
        .b            (b),
        .roundingMode (3'b000),
        .outValid     (outValid),
        .out          (out),
        .exceptionFlags(exceptionFlags)
    );
endmodule
EOF

synth_both "hardfloat_div" "bench_hardfloat_div" \
    "$HF_COMMON $HF/divSqrtRecFN_small.v $HF/div_f32.v" "/tmp/bench_hardfloat_div.v"

#===========================================================================
# SQRT
#===========================================================================
echo "--- SQRT ---"
echo "" >> "$SUMMARY"
echo "--- SQRT ---" >> "$SUMMARY"

# Spirix sqrt_nr (8-stage pipelined NR)
cat > /tmp/bench_spirix_sqrt_nr.v << EOF
module bench_spirix_sqrt_nr (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac,
    input  wire signed [${EXP}-1:0]  a_exp,
    output wire signed [${FRAC}-1:0] result_frac,
    output wire signed [${EXP}-1:0]  result_exp
);
    spirix_sqrt_nr #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP})) dut (
        .clk(clk), .a_frac(a_frac), .a_exp(a_exp),
        .result_frac(result_frac), .result_exp(result_exp));
endmodule
EOF

synth_both "spirix_sqrt_nr" "bench_spirix_sqrt_nr" \
    "$RTL/spirix_sqrt_nr.v" "/tmp/bench_spirix_sqrt_nr.v"

# HardFloat sqrt (~25 cycles iterative)
cat > /tmp/bench_hardfloat_sqrt.v << 'EOF'
module bench_hardfloat_sqrt (
    input  wire        clock,
    input  wire [32:0] a,
    output wire        inReady,
    output wire        outValid,
    output wire [32:0] out,
    output wire [4:0]  exceptionFlags
);
    sqrt_f32 dut (
        .nReset       (1'b1),
        .clock        (clock),
        .inReady      (inReady),
        .inValid      (1'b1),
        .a            (a),
        .roundingMode (3'b000),
        .outValid     (outValid),
        .out          (out),
        .exceptionFlags(exceptionFlags)
    );
endmodule
EOF

synth_both "hardfloat_sqrt" "bench_hardfloat_sqrt" \
    "$HF_COMMON $HF/divSqrtRecFN_small.v $HF/sqrt_f32.v" "/tmp/bench_hardfloat_sqrt.v"

#===========================================================================
# Print summary
#===========================================================================
echo ""
echo "================================================================"
echo " Summary (ECP5-25F speed-6, binary32 precision)"
echo " _nwl = -nowidelut (pure LUT4, no PFUMX/L6MUX)"
echo " Fmax in MHz. Logic and Route in nanoseconds."
echo "================================================================"
echo ""
printf "%-32s %5s %5s %4s %5s %8s %7s %7s\n" \
    "Module" "LUT4" "FF" "DSP" "CCU2C" "Fmax" "Logic" "Route"
printf "%-32s %5s %5s %4s %5s %8s %7s %7s\n" \
    "--------------------------------" "-----" "-----" "----" "-----" "--------" "-------" "-------"
cat "$SUMMARY"
echo ""
echo "Logs in $OUT_DIR/"
