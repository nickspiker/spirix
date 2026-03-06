#!/bin/bash
# Full synthesis profile of all Spirix RTL modules.
# Target: ECP5-25F CABGA256 speed-6, FRAC=25 EXP=8.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FPGA_DIR="$SCRIPT_DIR/.."
RTL="$FPGA_DIR/rtl"
cd "$FPGA_DIR"
mkdir -p sim/profile

FRAC=25
EXP=8
SUMMARY="sim/profile/summary.txt"
> "$SUMMARY"

echo "============================================================"
echo " Spirix RTL Synthesis Profile — ECP5-25F, FRAC=$FRAC EXP=$EXP"
echo "============================================================"
echo ""

#---------------------------------------------------------------------------
# Helper: synthesize a pipelined module (has its own clk)
#---------------------------------------------------------------------------
synth_pipelined() {
    local NAME=$1
    local MODULE=$2
    local VFILES=$3    # space-separated list of verilog files
    local TOP=$4       # top-level wrapper module name
    local WRAPPER=$5   # path to wrapper file
    local OUT="sim/profile/${NAME}"

    echo "--- $NAME ---"

    # Yosys synthesis
    local READ_CMD=""
    for f in $VFILES; do
        READ_CMD="$READ_CMD read_verilog $f;"
    done
    READ_CMD="$READ_CMD read_verilog $WRAPPER;"

    yosys -p "
        $READ_CMD
        synth_ecp5 -top $TOP -json ${OUT}.json
        stat
    " > ${OUT}_yosys.log 2>&1

    local LUTS=$(grep -E '^\s+[0-9]+ +LUT4$' ${OUT}_yosys.log | awk '{print $1}' | tail -1)
    local FFS=$(grep -E '^\s+[0-9]+ +TRELLIS_FF$' ${OUT}_yosys.log | awk '{print $1}' | tail -1)
    local DSPS=$(grep -E '^\s+[0-9]+ +MULT18X18D$' ${OUT}_yosys.log | awk '{print $1}' | tail -1)
    local CCUS=$(grep -E '^\s+[0-9]+ +CCU2C$' ${OUT}_yosys.log | awk '{print $1}' | tail -1)

    # Place and route
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

    printf "  LUT4=%-5s FF=%-4s DSP=%-3s CCU2C=%-4s Fmax=%-7s Logic=%-6s Route=%-6s\n" \
        "${LUTS:--}" "${FFS:--}" "${DSPS:--}" "${CCUS:--}" "${FMAX:--}" "${LOGIC_NS:--}" "${ROUTE_NS:--}"

    printf "%-28s %5s %5s %4s %5s %8s %7s %7s\n" \
        "$NAME" "${LUTS:--}" "${FFS:--}" "${DSPS:--}" "${CCUS:--}" "${FMAX:--}" "${LOGIC_NS:--}" "${ROUTE_NS:--}" \
        >> "$SUMMARY"
}

#---------------------------------------------------------------------------
# Helper: synthesize a combinational module (wrap with input/output regs)
#---------------------------------------------------------------------------
synth_combinational() {
    local NAME=$1
    local MODULE=$2
    local VFILES=$3
    local PORTS=$4     # "2in" for a+b operands, "1in" for single input
    local OUT="sim/profile/${NAME}"

    echo "--- $NAME ---"

    # Generate registered wrapper
    local WTOP="synth_reg_${NAME}"
    local WFILE="/tmp/${WTOP}.v"

    if [ "$PORTS" = "1in" ]; then
        cat > "$WFILE" << EOFWRAP
module ${WTOP} (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac,
    input  wire signed [${EXP}-1:0]  a_exp,
    output reg  signed [${FRAC}-1:0] result_frac,
    output reg  signed [${EXP}-1:0]  result_exp
);
    reg signed [${FRAC}-1:0] a_frac_r;
    reg signed [${EXP}-1:0]  a_exp_r;
    always @(posedge clk) begin a_frac_r <= a_frac; a_exp_r <= a_exp; end
    wire signed [${FRAC}-1:0] w_frac;
    wire signed [${EXP}-1:0]  w_exp;
    ${MODULE} #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP})) dut (
        .a_frac(a_frac_r), .a_exp(a_exp_r),
        .result_frac(w_frac), .result_exp(w_exp)
    );
    always @(posedge clk) begin result_frac <= w_frac; result_exp <= w_exp; end
endmodule
EOFWRAP
    else
        cat > "$WFILE" << EOFWRAP
module ${WTOP} (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac,
    input  wire signed [${EXP}-1:0]  a_exp,
    input  wire signed [${FRAC}-1:0] b_frac,
    input  wire signed [${EXP}-1:0]  b_exp,
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
    ${MODULE} #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP})) dut (
        .a_frac(a_frac_r), .a_exp(a_exp_r),
        .b_frac(b_frac_r), .b_exp(b_exp_r),
        .result_frac(w_frac), .result_exp(w_exp)
    );
    always @(posedge clk) begin result_frac <= w_frac; result_exp <= w_exp; end
endmodule
EOFWRAP
    fi

    # Build read commands
    local READ_CMD=""
    for f in $VFILES; do
        READ_CMD="$READ_CMD read_verilog $f;"
    done
    READ_CMD="$READ_CMD read_verilog $WFILE;"

    yosys -p "
        $READ_CMD
        synth_ecp5 -top $WTOP -json ${OUT}.json
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

    printf "  LUT4=%-5s FF=%-4s DSP=%-3s CCU2C=%-4s Fmax=%-7s Logic=%-6s Route=%-6s\n" \
        "${LUTS:--}" "${FFS:--}" "${DSPS:--}" "${CCUS:--}" "${FMAX:--}" "${LOGIC_NS:--}" "${ROUTE_NS:--}"

    printf "%-28s %5s %5s %4s %5s %8s %7s %7s\n" \
        "$NAME" "${LUTS:--}" "${FFS:--}" "${DSPS:--}" "${CCUS:--}" "${FMAX:--}" "${LOGIC_NS:--}" "${ROUTE_NS:--}" \
        >> "$SUMMARY"
}

#---------------------------------------------------------------------------
# Generate wrappers for pipelined modules
#---------------------------------------------------------------------------

# addsub_pipe2 wrapper
cat > /tmp/synth_addsub_pipe2.v << EOF
module synth_addsub_pipe2 (
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

# multiply_pipe2 wrapper
cat > /tmp/synth_mul_pipe2_w.v << EOF
module synth_mul_pipe2_w (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac, b_frac,
    input  wire signed [${EXP}-1:0]  a_exp, b_exp,
    output wire signed [${FRAC}-1:0] result_frac,
    output wire signed [${EXP}-1:0]  result_exp
);
    spirix_multiply_pipe2 #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP})) dut (
        .clk(clk), .ce(1'b1), .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp), .negate(1'b0),
        .result_frac(result_frac), .result_exp(result_exp));
endmodule
EOF


# divmod_nr MOD=1 wrapper
cat > /tmp/synth_divmod_mod1_w.v << EOF
module synth_divmod_mod1_w (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac, b_frac,
    input  wire signed [${EXP}-1:0]  a_exp, b_exp,
    output wire signed [${FRAC}-1:0] q_frac, mod_frac,
    output wire signed [${EXP}-1:0]  q_exp, mod_exp
);
    spirix_divmod_nr #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP}), .ENABLE_MOD(1)) dut (
        .clk(clk), .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .q_frac(q_frac), .q_exp(q_exp),
        .mod_frac(mod_frac), .mod_exp(mod_exp));
endmodule
EOF

# divmod_nr MOD=0 wrapper
cat > /tmp/synth_divmod_mod0_w.v << EOF
module synth_divmod_mod0_w (
    input  wire clk,
    input  wire signed [${FRAC}-1:0] a_frac, b_frac,
    input  wire signed [${EXP}-1:0]  a_exp, b_exp,
    output wire signed [${FRAC}-1:0] q_frac, mod_frac,
    output wire signed [${EXP}-1:0]  q_exp, mod_exp
);
    spirix_divmod_nr #(.FRAC_BITS(${FRAC}), .EXP_BITS(${EXP}), .ENABLE_MOD(0)) dut (
        .clk(clk), .a_frac(a_frac), .a_exp(a_exp),
        .b_frac(b_frac), .b_exp(b_exp),
        .q_frac(q_frac), .q_exp(q_exp),
        .mod_frac(mod_frac), .mod_exp(mod_exp));
endmodule
EOF

# sqrt_nr wrapper
cat > /tmp/synth_sqrt_nr_w.v << EOF
module synth_sqrt_nr_w (
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

# divide_iter wrapper
cat > /tmp/synth_divide_iter_w.v << EOF
module synth_divide_iter_w (
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

#---------------------------------------------------------------------------
# Print header
#---------------------------------------------------------------------------
printf "%-28s %5s %5s %4s %5s %8s %7s %7s\n" \
    "Module" "LUT4" "FF" "DSP" "CCU2C" "Fmax" "Logic" "Route" >> "$SUMMARY"
printf "%-28s %5s %5s %4s %5s %8s %7s %7s\n" \
    "----------------------------" "-----" "-----" "----" "-----" "--------" "-------" "-------" >> "$SUMMARY"

#---------------------------------------------------------------------------
# Run all syntheses
#---------------------------------------------------------------------------

# 1. addsub (combinational)
synth_combinational "addsub" "spirix_addsub" "$RTL/spirix_addsub.v" "2in"

# 2. addsub_pipe2
synth_pipelined "addsub_pipe2" "spirix_addsub_pipe2" "$RTL/spirix_addsub_pipe2.v" \
    "synth_addsub_pipe2" "/tmp/synth_addsub_pipe2.v"

# 3. multiply (combinational)
synth_combinational "multiply" "spirix_multiply" "$RTL/spirix_multiply.v" "2in"

# 4. multiply_pipe2
synth_pipelined "multiply_pipe2" "spirix_multiply_pipe2" "$RTL/spirix_multiply_pipe2.v" \
    "synth_mul_pipe2_w" "/tmp/synth_mul_pipe2_w.v"


# 6. divmod_nr MOD=1
synth_pipelined "divmod_nr_mod1" "spirix_divmod_nr" "$RTL/spirix_divmod_nr.v" \
    "synth_divmod_mod1_w" "/tmp/synth_divmod_mod1_w.v"

# 7. divmod_nr MOD=0
synth_pipelined "divmod_nr_mod0" "spirix_divmod_nr" "$RTL/spirix_divmod_nr.v" \
    "synth_divmod_mod0_w" "/tmp/synth_divmod_mod0_w.v"

# 8. sqrt_nr
synth_pipelined "sqrt_nr" "spirix_sqrt_nr" "$RTL/spirix_sqrt_nr.v" \
    "synth_sqrt_nr_w" "/tmp/synth_sqrt_nr_w.v"

# 9. divide_iter
synth_pipelined "divide_iter" "spirix_divide_iter" "$RTL/spirix_divide_iter.v" \
    "synth_divide_iter_w" "/tmp/synth_divide_iter_w.v"

#---------------------------------------------------------------------------
# Print summary
#---------------------------------------------------------------------------
echo ""
echo "============================================================"
echo " Summary (ECP5-25F speed-6, FRAC=$FRAC EXP=$EXP)"
echo "============================================================"
cat "$SUMMARY"
echo ""
echo "Logs in sim/profile/"
