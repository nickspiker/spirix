#!/bin/bash
# Build USPTO-ready DOCX of patent specification + claims (figures excluded).
#
# Workflow:
#   1. Run xelatex once on the FULL patent to populate the .aux cross-reference
#      table (figure numbers, etc.). The PDF byproduct is harmless.
#   2. Clip the .tex at the start of the figures section.
#   3. Strip the pdfpages package (xelatex-only; tex4ht stumbles on it) and
#      inject \newlabel definitions for the figures so \ref{fig:foo}
#      resolves to the right number instead of "??".
#   4. make4ht -x -f odt → patent_clip.odt.
#   5. soffice --convert-to docx → patent_clip.docx.
#   6. docx_tidy (Rust, ./docx_tidy) post-processes the .docx: ragged-right,
#      no field shading, no blank paragraphs or \hline filler rows.
#
# Output: patent/<basename> spec+claims.docx
set -e
cd "$(dirname "$0")"

SRC="patent - rev3 non-provisional.tex"
AUX="patent - rev3 non-provisional.aux"
OUT_DOCX="patent - rev3 spec+claims.docx"

# Where the figures start: the line before the \newpage that precedes the first
# lstlisting (828 in the filed rev3; the July filing-details edit moved it).
CLIP_LINE=$(( $(awk '/^\\newpage/ { np = NR } /^\\begin\{lstlisting\}/ { print np; exit }' "$SRC") - 1 ))

# ── 1. ensure .aux has figure labels (full xelatex pass) ─────────────────────
if [ ! -f "$AUX" ] || ! grep -q "newlabel{fig:" "$AUX"; then
    echo "Building $AUX via xelatex..."
    xelatex -interaction=nonstopmode "$SRC" > /tmp/patent_aux.log 2>&1 || true
    xelatex -interaction=nonstopmode "$SRC" > /tmp/patent_aux.log 2>&1 || true
fi
echo "Figure labels in .aux: $(grep -c 'newlabel{fig:' "$AUX")"

# ── 2-3. clip + strip pdfpages + inject figure labels into preamble ──────────
# Labels MUST be defined before any \ref in the body — LaTeX resolves refs
# at expansion time during the single pass. We inject \newlabel into the
# preamble (right before \begin{document}) so they're seen before any body
# reference. Backslashes don't survive shell var round-trips, so we stage
# the injection into a temp file and have awk read it.
CLIP=/tmp/patent_clip.tex
LABELS_FILE=/tmp/patent_clip_labels.tex
{
    echo "% Injected figure labels"
    grep "newlabel{fig:" "$AUX" \
        | sed -E 's/^(\\newlabel\{fig:[^}]+\})\{\{([0-9]+)\}.*$/\1{{\2}{}{}{}{}}/'
} > "$LABELS_FILE"
head -$CLIP_LINE "$SRC" \
    | sed '/\\usepackage{pdfpages}/d' \
    | sed 's/\\newpage//g' \
    | awk -v labels_file="$LABELS_FILE" '
        /^\\begin\{document\}/ {
            while ((getline line < labels_file) > 0) print line
            close(labels_file)
            print ""
        }
        { print }
    ' > "$CLIP"
echo "\\end{document}" >> "$CLIP"

# ── 4. make4ht to ODT ────────────────────────────────────────────────────────
rm -f /tmp/patent_clip.{4ct,4tc,aux,idv,lg,log,xdv,xref,odt}
rm -f /tmp/patent_clip-m*.tmp
( cd /tmp && make4ht -x -f odt patent_clip.tex > /tmp/make4ht.log 2>&1 )
if [ ! -f /tmp/patent_clip.odt ]; then
    echo "ERROR: make4ht did not produce ODT. See /tmp/make4ht.log"
    exit 1
fi

# ── 5. ODT → DOCX ────────────────────────────────────────────────────────────
rm -f /tmp/patent_clip.docx
# Use a dedicated user profile so this works even when the user has another
# LibreOffice instance open (e.g. viewing the previous .docx in Writer).
SOFFICE_PROFILE=$(mktemp -d /tmp/soffice_profile.XXXXXX)
soffice "-env:UserInstallation=file://$SOFFICE_PROFILE" --headless \
    --convert-to docx /tmp/patent_clip.odt --outdir /tmp > /tmp/soffice.log 2>&1
rm -rf "$SOFFICE_PROFILE"
if [ ! -f /tmp/patent_clip.docx ]; then
    echo "ERROR: soffice did not produce DOCX. See /tmp/soffice.log"
    exit 1
fi

# ── 6. clean up: ragged-right + strip tex4ht whitespace artifacts ────────────
# docx_tidy (./docx_tidy, Rust) rewrites word/document.xml and word/styles.xml
# in the unpacked DOCX; the parts are zipped back in their original order.
TIDY=/tmp/patent_clip_tidy
rm -rf "$TIDY" /tmp/patent_clip_rr.docx
mkdir "$TIDY"
unzip -q /tmp/patent_clip.docx -d "$TIDY"
( cd docx_tidy && cargo run --release --quiet -- "$TIDY" )
( cd "$TIDY" && unzip -Z1 /tmp/patent_clip.docx | zip -q -X -D -nw -@ /tmp/patent_clip_rr.docx )
mv /tmp/patent_clip_rr.docx /tmp/patent_clip.docx
rm -rf "$TIDY"

cp /tmp/patent_clip.docx "$OUT_DOCX"
echo ""
echo "Built: $(pwd)/$OUT_DOCX ($(du -h "$OUT_DOCX" | cut -f1))"
