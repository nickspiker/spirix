# Timing numbers from a nextpnr-ecp5 log, sourced by the synthesis scripts.
# Each prints the first match in the log, or nothing when there is none (or no
# log), so callers fall back with ${VAR:--}.
#
#   pnr_fmax LOG      Fmax in MHz from the first "Max frequency for clock …: N MHz"
#   pnr_ns LOG logic  ns from the first "N ns logic"
#   pnr_ns LOG routing ns from the first "N ns routing"

pnr_fmax() {
    awk '
        BEGIN { key = "Max frequency for clock" }
        {
            at = index($0, key)
            if (!at) next
            rest = substr($0, at + length(key))
            if (match(rest, /:[[:space:]]*[0-9]+\.[0-9]+[[:space:]]*MHz/)) {
                n = substr(rest, RSTART, RLENGTH)
                sub(/^:[[:space:]]*/, "", n)
                sub(/[[:space:]]*MHz$/, "", n)
                print n
                exit
            }
        }
    ' "$1" 2>/dev/null
}

pnr_ns() {
    awk -v what="$2" '
        match($0, "[0-9]+\\.[0-9]+ ns " what) {
            n = substr($0, RSTART, RLENGTH)
            sub(/ .*/, "", n)
            print n
            exit
        }
    ' "$1" 2>/dev/null
}
