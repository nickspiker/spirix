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
#   6. Post-process the .docx to force left-aligned (ragged-right) paragraphs.
#
# Output: patent/<basename> spec+claims.docx
set -e
cd "$(dirname "$0")"

SRC="patent - rev3 non-provisional.tex"
AUX="patent - rev3 non-provisional.aux"
OUT_DOCX="patent - rev3 spec+claims.docx"

# Where the figures start. \newpage right before the lstlistings.
CLIP_LINE=828

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
python3 << 'PYEOF'
import zipfile, shutil, re
from lxml import etree

src = '/tmp/patent_clip.docx'
dst = '/tmp/patent_clip_rr.docx'
W = 'http://schemas.openxmlformats.org/wordprocessingml/2006/main'
NSMAP = {'w': W}

def clean_document_xml(data):
    """Strip tex4ht's whitespace-only runs and empty paragraphs."""
    root = etree.fromstring(data)
    body = root.find(f'{{{W}}}body')
    if body is None:
        return data

    for p in list(body.iter(f'{{{W}}}p')):
        # 0) Strip page breaks from paragraphs with no text content. (Forced
        #    page breaks belong inside text-bearing paragraphs; stray ones in
        #    empty paragraphs just produce blank pages in Word.)
        text_elems = list(p.iter(f'{{{W}}}t'))
        has_text = any((t.text or '').strip() for t in text_elems)
        if not has_text:
            for br in p.findall(f'.//{{{W}}}br'):
                if br.get(f'{{{W}}}type') == 'page':
                    br.getparent().remove(br)

        # 1) Remove trailing runs that contain only whitespace.
        runs = p.findall(f'{{{W}}}r')
        for r in reversed(runs):
            text_elems = r.findall(f'{{{W}}}t')
            has_break = r.find(f'{{{W}}}br') is not None
            if has_break:
                break  # keep — page break or line break
            if all((t.text or '').strip() == '' for t in text_elems):
                r.getparent().remove(r)
            else:
                break

        # 2) Strip trailing whitespace from the LAST <w:t> in the paragraph
        #    (tex4ht emits ". " at paragraph ends because LaTeX absorbs the
        #    newline-before-\par into a space). Walk back from the end of
        #    the text stream and rstrip the last non-empty <w:t>.
        text_elems = list(p.iter(f'{{{W}}}t'))
        for t in reversed(text_elems):
            txt = t.text or ''
            if txt == '':
                continue
            if txt.rstrip() != txt:
                t.text = txt.rstrip() or None
                # If we emptied it, drop the xml:space="preserve" attribute
                # to avoid an orphan preserve marker on a now-empty element.
                if t.text is None:
                    t.attrib.pop('{http://www.w3.org/XML/1998/namespace}space',
                                 None)
            break  # only the last non-empty <w:t>

        # 3) Inline-justify any remaining "both"/"justify" alignment to left.
        for jc in p.iter(f'{{{W}}}jc'):
            if jc.get(f'{{{W}}}val') in ('both', 'justify'):
                jc.set(f'{{{W}}}val', 'left')

        # 4) Drop the paragraph entirely if it has no visible content:
        #    no non-empty <w:t> text, no <w:br>, no <w:sectPr>.
        has_text = any((t.text or '').strip() != '' for t in p.iter(f'{{{W}}}t'))
        has_break = p.find(f'.//{{{W}}}br') is not None
        has_sect = p.find(f'.//{{{W}}}sectPr') is not None
        # tex4ht's environment-delimiter paragraphs are pure formatting.
        pstyle = p.find(f'.//{{{W}}}pStyle')
        is_env = (pstyle is not None and
                  pstyle.get(f'{{{W}}}val') in ('begin-env-p', 'end-env-p'))
        if (not has_text and not has_break and not has_sect) or is_env:
            p.getparent().remove(p)

    # Table cleanup: tex4ht renders each LaTeX \hline as an empty filler row.
    # Strip them so the table has only header + data rows, and add vMerge on
    # the first column header so "Operation" spans the two-row header block
    # like the source \multirow{2}{*}{...} declares.
    for tbl in body.iter(f'{{{W}}}tbl'):
        rows = tbl.findall(f'{{{W}}}tr')
        for tr in list(rows):
            cells = tr.findall(f'{{{W}}}tc')
            all_empty = all(
                not any((t.text or '').strip()
                        for t in tc.iter(f'{{{W}}}t'))
                for tc in cells
            )
            if all_empty:
                tr.getparent().remove(tr)

        # Don't try to vMerge the "Operation" header across the two header
        # rows: LibreOffice's renderer drops the cell *immediately right* of
        # the merge anchor (the first "Fmax") when the column widths match.
        # The two-row header reads fine with the first column simply blank
        # in the second header row, so leave the cells alone after filler
        # removal.

    return etree.tostring(root, xml_declaration=True,
                          encoding='UTF-8', standalone=True)

def clean_styles_xml(data):
    """Make Normal style ragged-right."""
    xml = data.decode('utf-8')
    # Strip existing justify alignment.
    xml = re.sub(r'<w:jc\s+w:val="(both|justify)"\s*/>', '', xml)
    # Add explicit left alignment to Normal.
    xml = re.sub(
        r'(<w:style\s+w:type="paragraph"[^>]*w:styleId="Normal"[^>]*>)',
        r'\1<w:pPr><w:jc w:val="left"/></w:pPr>',
        xml, count=1)
    return xml.encode('utf-8')

with zipfile.ZipFile(src) as zin, zipfile.ZipFile(dst, 'w', zipfile.ZIP_DEFLATED) as zout:
    for item in zin.namelist():
        data = zin.read(item)
        if item == 'word/styles.xml':
            data = clean_styles_xml(data)
        elif item == 'word/document.xml':
            data = clean_document_xml(data)
        zout.writestr(item, data)
shutil.move(dst, src)
print("cleanup: ragged-right + empty-paragraph + trailing-space removal applied")
PYEOF

cp /tmp/patent_clip.docx "$OUT_DOCX"
echo ""
echo "Built: $(pwd)/$OUT_DOCX ($(du -h "$OUT_DOCX" | cut -f1))"
