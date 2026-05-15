#!/bin/bash
# Synthesize all paper-comparison modules and report cell counts.
#
# Two passes per module:
#   - Pure LUT4 (-nodsp -nowidelut): ASIC-equivalent area count
#   - Default (DSP allowed):         FPGA-realistic area count
#
# Output: TSV table to stdout. Source RTL files unchanged.
set -e
cd "$(dirname "${BASH_SOURCE[0]}")/.."

SPIRIX_DIR=verilog
FPNEW_DIR=../../fpga/bench/fpnew_v
TMP=/tmp/synth_paper_$$
mkdir -p "$TMP"

extract_counts() {
    # $1 = log file, $2 = top module
    # Returns "LUT4 TRELLIS_FF CCU2C DP16KD MULT18X18D"
    local log="$1" top="$2"
    awk -v top="=== $top ===" '
        $0 == top { in_block=1; have_local=0; next }
        in_block && /^\s*\+-+Local/ { have_local=1; next }
        in_block && have_local && /^=== / { in_block=0 }
        in_block && have_local && /^\s+[0-9]+\s+(LUT4|TRELLIS_FF|CCU2C|DP16KD|MULT18X18D)\s*$/ {
            count = $1; gsub(/^\s+|\s+$/, "", count)
            cell = $2
            counts[cell] = count
        }
        END {
            printf "%s\t%s\t%s\t%s\t%s\n",
                counts["LUT4"]+0, counts["TRELLIS_FF"]+0, counts["CCU2C"]+0,
                counts["DP16KD"]+0, counts["MULT18X18D"]+0
        }
    ' "$log"
}

synth_module() {
    # $1 = label (printed)
    # $2 = top module
    # $3 = rtl file(s) (space-separated)
    # $4 = chparam name=value (e.g., "PARALLEL=4"); empty for none
    # $5 = synth flags (e.g., "-nodsp -nowidelut")
    local label="$1" top="$2" rtls="$3" chparam="$4" flags="$5"
    local log="$TMP/${label// /_}_${flags// /}.log"
    local read_cmds=""
    for f in $rtls; do read_cmds="$read_cmds read_verilog $f;"; done
    local cp_cmd=""
    if [ -n "$chparam" ]; then
        local pname="${chparam%=*}" pval="${chparam##*=}"
        cp_cmd="chparam -set $pname $pval $top;"
    fi
    yosys -p "
        $read_cmds
        $cp_cmd
        synth_ecp5 $flags -top $top
        stat -top $top
    " > "$log" 2>&1 || { echo "yosys FAILED for $label" >&2; cat "$log" >&2; return 1; }
    extract_counts "$log" "$top"
}

printf "%-30s\t%-22s\t%6s\t%4s\t%5s\t%6s\t%6s\n" \
    "Module" "Flags" "LUT4" "FF" "CCU2C" "DP16KD" "MULT18X18D"
echo "---------------------------------------------------------------------------------------------------"

# Spirix paper-comparison modules ------------------------------------------
declare -a SPIRIX_MODS=(
    "spirix_addsub|$SPIRIX_DIR/spirix_addsub.v|"
    "spirix_multiply|$SPIRIX_DIR/spirix_multiply.v|"
    "spirix_divide P=4|$SPIRIX_DIR/spirix_divide.v|PARALLEL=4"
    "spirix_sqrt P=4|$SPIRIX_DIR/spirix_sqrt.v|PARALLEL=4"
)
for entry in "${SPIRIX_MODS[@]}"; do
    IFS='|' read -r label rtl chparam <<< "$entry"
    top=$(echo "$label" | awk '{print $1}')
    for flags in "-nodsp -nowidelut" ""; do
        result=$(synth_module "$label" "$top" "$rtl" "$chparam" "$flags")
        IFS=$'\t' read -r lut ff ccu dp mul <<< "$result"
        printf "%-30s\t%-22s\t%6s\t%4s\t%5s\t%6s\t%6s\n" \
            "$label" "${flags:-default}" "$lut" "$ff" "$ccu" "$dp" "$mul"
    done
done

# FPnew modules ------------------------------------------------------------
# fpnew_fma covers add/mul/fma; fpnew_divsqrt covers div/sqrt. Synth as full
# module (no operand const-prop) — gives the unified-datapath area.
declare -a FPN_MODS=(
    "fpnew_fma|fpnew_fma|$FPNEW_DIR/fpnew_fma_fp32.v|"
    "fpnew_divsqrt FP32|fpnew_divsqrt_fp32_wrap|$FPNEW_DIR/fpnew_divsqrt_fp32.v $SPIRIX_DIR/fpnew_divsqrt_fp32_wrap.v|"
    "fpnew_divsqrt multi|div_sqrt_mvp_wrapper|$FPNEW_DIR/fpnew_divsqrt_fp32.v|"
)
for entry in "${FPN_MODS[@]}"; do
    IFS='|' read -r label top rtl chparam <<< "$entry"
    for flags in "-nodsp -nowidelut" ""; do
        result=$(synth_module "$label" "$top" "$rtl" "$chparam" "$flags")
        IFS=$'\t' read -r lut ff ccu dp mul <<< "$result"
        printf "%-30s\t%-22s\t%6s\t%4s\t%5s\t%6s\t%6s\n" \
            "$label" "${flags:-default}" "$lut" "$ff" "$ccu" "$dp" "$mul"
    done
done

echo ""
echo "Logs: $TMP/"
