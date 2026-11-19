#!/bin/bash
# Build the rev4 reply to the 2026-09-30 Notice to File Missing Parts (application 19/678,144).
#
# One source, "patent - rev4 substitute.tex", compiles to one PDF holding the substitute specification, claims, abstract and replacement drawings.
# This script then splits that PDF into the separate documents Patent Center indexes:
#   rev4/substitute specification.pdf
#   rev4/claims.pdf
#   rev4/abstract.pdf
#   rev4/replacement drawings.pdf
# The computer program listing appendix lives in cpla/*.txt; its file names, sizes and creation date are written into cpla_macros.tex so the incorporation-by-reference paragraph always matches the files.
set -e
cd "$(dirname "$0")"

SRC="patent - rev4 substitute.tex"
JOB="patent - rev4 substitute"
OUT=rev4
CPLA_DATE="November 19, 2026"

# Appendix macros: the spec quotes each file's exact byte size, so regenerate them from the files themselves.
{
    echo "\\newcommand{\\cplaDate}{$CPLA_DATE}"
    echo "\\newcommand{\\cplaHardfloatName}{HARDFLOAT\\_ADDSUB.txt}"
    echo "\\newcommand{\\cplaHardfloatBytes}{$(stat -c %s cpla/HARDFLOAT_ADDSUB.txt | sed ':a;s/\B[0-9]\{3\}\>/,&/;ta')}"
    echo "\\newcommand{\\cplaScalarName}{SCALAR\\_ADDSUB.txt}"
    echo "\\newcommand{\\cplaScalarBytes}{$(stat -c %s cpla/SCALAR_ADDSUB.txt | sed ':a;s/\B[0-9]\{3\}\>/,&/;ta')}"
    echo "\\newcommand{\\cplaCircleName}{CIRCLE\\_DIVIDE.txt}"
    echo "\\newcommand{\\cplaCircleBytes}{$(stat -c %s cpla/CIRCLE_DIVIDE.txt | sed ':a;s/\B[0-9]\{3\}\>/,&/;ta')}"
} > cpla_macros.tex

for f in cpla/*.txt; do
    if LC_ALL=C grep -q -P '[^\x00-\x7F]' "$f"; then
        echo "ERROR: $f contains non-ASCII bytes; the appendix must be ASCII"
        exit 1
    fi
done

# Flowchart sheets: Graphviz sources in flowcharts/, shared styling in flowcharts/common.gvi.
for dotsrc in flowcharts/*.dot; do
    sed -e "/__COMMON__/r flowcharts/common.gvi" -e "/__COMMON__/d" "$dotsrc" | dot -Tpdf -o "${dotsrc%.dot}.pdf"
done

BUILD=$(mktemp -d /tmp/patent_rev4.XXXXXX)
# Three passes: aux labels, then \pageref{LastSheet} for the n/N sheet numbers, then settle.
for pass in 1 2 3; do
    xelatex -interaction=nonstopmode -output-directory="$BUILD" "$SRC" > "$BUILD/pass$pass.log" 2>&1 || true
done
FULL="$BUILD/$JOB.pdf"
if [ ! -f "$FULL" ]; then
    echo "ERROR: xelatex produced no PDF. See $BUILD/pass3.log"
    exit 1
fi
if grep -q -E 'undefined references|Reference .* undefined' "$BUILD/pass3.log"; then
    echo "WARNING: undefined references remain:"
    grep -E 'Reference .* undefined' "$BUILD/pass3.log" | sort -u
fi

PAGES=$(pdfinfo "$FULL" | awk '/^Pages:/{print $2}')

# Locate the document boundaries by the headings each part starts with.
first_page_matching() {
    local pattern="$1" pg
    for pg in $(seq 1 "$PAGES"); do
        if pdftotext -f "$pg" -l "$pg" "$FULL" - 2>/dev/null | grep -q -E "$pattern"; then
            echo "$pg"
            return
        fi
    done
}
CLAIMS_AT=$(first_page_matching '^CLAIMS$')
ABSTRACT_AT=$(first_page_matching '^ABSTRACT$')
DRAWINGS_AT=$(first_page_matching '^Replacement Sheet$')
echo "Pages: $PAGES total; claims start $CLAIMS_AT, abstract $ABSTRACT_AT, drawings $DRAWINGS_AT"
if [ -z "$CLAIMS_AT" ] || [ -z "$ABSTRACT_AT" ] || [ -z "$DRAWINGS_AT" ]; then
    echo "ERROR: could not find every document boundary"
    exit 1
fi

mkdir -p "$OUT"
rm -f "$OUT"/*.pdf
pdfseparate "$FULL" "$BUILD/p-%d.pdf" 2>/dev/null
unite_range() {
    local from=$1 to=$2 dest="$3" files=() pg
    for pg in $(seq "$from" "$to"); do files+=("$BUILD/p-$pg.pdf"); done
    pdfunite "${files[@]}" "$dest" 2>/dev/null
}
unite_range 1 $((CLAIMS_AT - 1)) "$OUT/substitute specification.pdf"
unite_range "$CLAIMS_AT" $((ABSTRACT_AT - 1)) "$OUT/claims.pdf"
unite_range "$ABSTRACT_AT" $((DRAWINGS_AT - 1)) "$OUT/abstract.pdf"
unite_range "$DRAWINGS_AT" "$PAGES" "$OUT/replacement drawings.pdf"
cp "$FULL" "$OUT/combined (review only).pdf"

SPEC_SHEETS=$((DRAWINGS_AT - 1))
DRAW_SHEETS=$((PAGES - DRAWINGS_AT + 1))
echo ""
for f in "$OUT"/*.pdf; do
    printf '%-40s %3s pages\n' "$(basename "$f")" "$(pdfinfo "$f" | awk '/^Pages:/{print $2}')"
done
echo "Specification + claims + abstract + drawings = $((SPEC_SHEETS + DRAW_SHEETS)) sheets (application size fee applies above 100)"
echo "Build directory: $BUILD"

# Response cover document: page counts come from this build so the enclosure list always matches.
{
    echo "\\newcommand{\\specPages}{$((CLAIMS_AT - 1))}"
    echo "\\newcommand{\\claimsPages}{$((ABSTRACT_AT - CLAIMS_AT))}"
    echo "\\newcommand{\\abstractPages}{$((DRAWINGS_AT - ABSTRACT_AT))}"
    echo "\\newcommand{\\drawingSheets}{$DRAW_SHEETS}"
} > rev4_counts.tex
for pass in 1 2; do
    xelatex -interaction=nonstopmode -output-directory="$BUILD" "patent - rev4 response.tex" > "$BUILD/response$pass.log" 2>&1 || true
done
if [ ! -f "$BUILD/patent - rev4 response.pdf" ]; then
    echo "ERROR: response did not build. See $BUILD/response2.log"
    exit 1
fi
cp "$BUILD/patent - rev4 response.pdf" "$OUT/response to notice.pdf"
printf '%-40s %3s pages\n' "response to notice.pdf" "$(pdfinfo "$OUT/response to notice.pdf" | awk '/^Pages:/{print $2}')"
