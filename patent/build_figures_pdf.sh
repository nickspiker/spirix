#!/bin/bash
# Build standalone "drawings" PDF for USPTO submission.
#
# Combines the patent's preamble (document class, packages, custom commands)
# with just the figures section (lines 829+). Because lstlisting starts its
# counter fresh in a standalone document, the first figure becomes Figure 1.
#
# Output: patent/<basename> drawings.pdf
set -e
cd "$(dirname "$0")"

SRC="patent - rev3 non-provisional.tex"
OUT_PDF="patent - rev3 drawings.pdf"

# Lines where the figures section starts (right after CLAIMS \end{enumerate}).
FIGS_START=829
PREAMBLE_END=$(grep -n "^\\\\begin{document}" "$SRC" | head -1 | cut -d: -f1)

FIG=/tmp/patent_figures.tex
{
    # Preamble (everything before \begin{document})
    head -$((PREAMBLE_END - 1)) "$SRC"
    echo "\\begin{document}"
    # Figures content
    tail -n +$FIGS_START "$SRC" | sed '$d'  # drop the existing \end{document}
    echo "\\end{document}"
} > "$FIG"

cd /tmp
# Two passes for cross-references (the brief-description figure numbers etc).
xelatex -interaction=nonstopmode patent_figures.tex > /tmp/figs1.log 2>&1 || true
xelatex -interaction=nonstopmode patent_figures.tex > /tmp/figs2.log 2>&1 || true

if [ ! -f /tmp/patent_figures.pdf ]; then
    echo "ERROR: xelatex did not produce PDF. See /tmp/figs2.log"
    exit 1
fi
cd - > /dev/null

cp /tmp/patent_figures.pdf "$OUT_PDF"
PAGES=$(pdfinfo "$OUT_PDF" 2>/dev/null | grep -E "^Pages:" | awk '{print $2}')
echo ""
echo "Built: $(pwd)/$OUT_PDF ($(du -h "$OUT_PDF" | cut -f1), ${PAGES:-?} pages)"
