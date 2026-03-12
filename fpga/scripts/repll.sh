#!/bin/bash
# Re-clock an existing build by patching PLL divisors in the .config file.
# Skips Yosys and nextpnr — only ecppll + sed + ecppack + program.
#
# Usage: ./repll.sh FREQ_MHZ [--program]
#
# Requires a prior build (fpga/build/ntsc.config must exist).
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD="$SCRIPT_DIR/../build"
CONFIG="$BUILD/ntsc.config"

FREQ="${1:?Usage: repll.sh FREQ_MHZ [--program]}"
PROGRAM="${2:-}"

if [ ! -f "$CONFIG" ]; then
    echo "ERROR: $CONFIG not found. Run build_ntsc.sh first."
    exit 1
fi

# Compute PLL parameters
PLL_OUT=$(ecppll -i 25 -o "$FREQ" -f /dev/null 2>&1)
CLKI=$(echo "$PLL_OUT"  | awk '/Refclk divisor:/  {print $3}')
CLKFB=$(echo "$PLL_OUT" | awk '/Feedback divisor:/ {print $3}')
CLKOP=$(echo "$PLL_OUT" | awk '/clkout0 divisor:/  {print $3}')
ACTUAL=$(echo "$PLL_OUT" | awk '/clkout0 frequency:/ {print $3}')
FVCO=$(echo "$PLL_OUT"  | awk '/VCO frequency:/    {print $3}')

if [ -z "$CLKI" ] || [ -z "$CLKFB" ] || [ -z "$CLKOP" ]; then
    echo "ERROR: ecppll failed for ${FREQ} MHz"
    echo "$PLL_OUT"
    exit 1
fi

CPHASE=$(( (CLKOP - 1) / 2 ))

# Convert to 7-bit binary strings for the config file
to_bin7() { printf '%07d' "$(echo "obase=2; $1" | bc)"; }

CLKI_BIN=$(to_bin7 "$CLKI")
CLKFB_BIN=$(to_bin7 "$CLKFB")
CLKOP_BIN=$(to_bin7 "$CLKOP")
CPHASE_BIN=$(to_bin7 "$CPHASE")

echo "Re-PLL: ${ACTUAL} MHz (CLKI=$CLKI CLKFB=$CLKFB CLKOP=$CLKOP CPHASE=$CPHASE VCO=${FVCO})"

# Verify PLL exists in config
if ! grep -q '^word: CLKI_DIV' "$CONFIG"; then
    echo "ERROR: No PLL found in $CONFIG (was it built at 25 MHz no-PLL mode?)"
    echo "Rebuild with a frequency > 25 to include a PLL."
    exit 1
fi

# Patch PLL divisors in-place
sed -i \
    -e "s/^word: CLKI_DIV .*/word: CLKI_DIV $CLKI_BIN/" \
    -e "s/^word: CLKFB_DIV .*/word: CLKFB_DIV $CLKFB_BIN/" \
    -e "s/^word: CLKOP_DIV .*/word: CLKOP_DIV $CLKOP_BIN/" \
    -e "s/^word: CLKOP_CPHASE .*/word: CLKOP_CPHASE $CPHASE_BIN/" \
    "$CONFIG"

# Re-pack bitstream
ecppack --svf "$BUILD/ntsc.svf" "$CONFIG" "$BUILD/ntsc.bit"
echo "Bitstream: $BUILD/ntsc.bit ($(stat -c%s "$BUILD/ntsc.bit") bytes)"

if [ "$PROGRAM" = "--program" ]; then
    openFPGALoader -c ft232 "$BUILD/ntsc.svf"
    echo "Done @ ${ACTUAL} MHz"
fi
