//! build_docx.sh step 6: tidy the DOCX that soffice made from tex4ht's ODT.
//!
//! Usage: docx_tidy DIR   (DIR = the unzipped .docx; word/document.xml and
//! word/styles.xml are rewritten in place, build_docx.sh zips DIR back up)
//!
//! - styles.xml: justified alignment stripped, Normal made explicitly left
//!   (ragged-right).
//! - document.xml: Word field framing removed (tex4ht wraps the bibliography in
//!   a BIBLIOGRAPHY field that Word shades grey), internal hyperlinks unwrapped
//!   to plain text, page breaks in text-less paragraphs dropped, trailing
//!   whitespace runs and trailing spaces removed, justified paragraphs made
//!   left, empty and tex4ht environment-delimiter paragraphs dropped (except the
//!   only paragraph of a table cell, which OOXML requires), and the empty filler
//!   rows tex4ht emits for each \hline removed.
//!
//! document.xml is parsed into a small element tree and written back the way
//! libxml2 serializes it, so the output is byte-for-byte what the lxml version
//! of this step produced.

use std::path::Path;
use std::process::exit;

const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

struct Attr {
    qname: String,
    ns: String,
    local: String,
    value: String,
}

enum Kind {
    Element {
        qname: String,
        ns: String,
        local: String,
        ns_decls: Vec<(String, String)>,
        attrs: Vec<Attr>,
    },
    Comment(String),
    Pi(String),
}

/// One node in the tree. As in lxml, `text` is the character data before the
/// first child and `tail` the character data after this node's end tag, so a
/// node removed from its parent takes its tail with it.
struct Node {
    kind: Kind,
    parent: Option<usize>,
    children: Vec<usize>,
    text: Option<String>,
    tail: Option<String>,
}

struct Doc {
    nodes: Vec<Node>,
    root: usize,
}

fn fail(msg: String) -> ! {
    eprintln!("docx_tidy: {msg}");
    exit(1)
}

// ── parsing ─────────────────────────────────────────────────────────────────

fn is_xml_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

/// Character data or an attribute value with its references resolved. Raw tabs
/// and newlines in an attribute value become spaces (XML attribute-value
/// normalization); references are resolved after, so &#10; survives.
fn decode(raw: &str, attr: bool) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(amp) = rest.find('&') {
        push_raw(&mut out, &rest[..amp], attr);
        let after = &rest[amp + 1..];
        let semi = after.find(';').unwrap_or_else(|| fail(format!("unterminated reference in {raw:?}")));
        let name = &after[..semi];
        let ch = match name {
            "lt" => '<',
            "gt" => '>',
            "amp" => '&',
            "quot" => '"',
            "apos" => '\'',
            _ => {
                let code = if let Some(hex) = name.strip_prefix("#x") {
                    u32::from_str_radix(hex, 16).ok()
                } else if let Some(dec) = name.strip_prefix('#') {
                    dec.parse().ok()
                } else {
                    None
                };
                code.and_then(char::from_u32)
                    .unwrap_or_else(|| fail(format!("unknown reference &{name};")))
            }
        };
        out.push(ch);
        rest = &after[semi + 1..];
    }
    push_raw(&mut out, rest, attr);
    out
}

fn push_raw(out: &mut String, raw: &str, attr: bool) {
    if attr {
        out.extend(raw.chars().map(|c| if c == '\t' || c == '\n' { ' ' } else { c }));
    } else {
        out.push_str(raw);
    }
}

fn split_qname(qname: &str) -> (&str, &str) {
    match qname.split_once(':') {
        Some((prefix, local)) => (prefix, local),
        None => ("", qname),
    }
}

fn resolve(scopes: &[Vec<(String, String)>], prefix: &str) -> String {
    if prefix == "xml" {
        return XML_NS.to_string();
    }
    for scope in scopes.iter().rev() {
        if let Some((_, uri)) = scope.iter().rev().find(|(p, _)| p == prefix) {
            return uri.clone();
        }
    }
    if prefix.is_empty() {
        return String::new();
    }
    fail(format!("unbound namespace prefix {prefix:?}"))
}

fn parse(src: &str) -> Doc {
    // XML line-end normalization: \r\n and a lone \r both read as \n.
    let normalized;
    let src = if src.contains('\r') {
        normalized = src.replace("\r\n", "\n").replace('\r', "\n");
        normalized.as_str()
    } else {
        src
    };
    let b = src.as_bytes();
    let mut pos = 0;
    let skip_space = |pos: &mut usize| {
        while *pos < b.len() && is_xml_space(b[*pos]) {
            *pos += 1;
        }
    };

    skip_space(&mut pos);
    if src[pos..].starts_with("<?xml") {
        pos += src[pos..].find("?>").unwrap_or_else(|| fail("unterminated XML declaration".into())) + 2;
        skip_space(&mut pos);
    }
    if src[pos..].starts_with("<!") || src[pos..].starts_with("<?") || !src[pos..].starts_with('<') {
        fail("expected the root element right after the XML declaration".into());
    }

    let mut doc = Doc { nodes: Vec::new(), root: 0 };
    let mut stack: Vec<usize> = Vec::new();
    let mut scopes: Vec<Vec<(String, String)>> = Vec::new();
    let mut seen_root = false;

    loop {
        if stack.is_empty() && seen_root {
            if !src[pos..].bytes().all(is_xml_space) {
                fail("content after the root element".into());
            }
            break;
        }
        let rest = &src[pos..];
        if !rest.starts_with('<') {
            let end = rest.find('<').unwrap_or_else(|| fail("document ends inside an element".into()));
            append_text(&mut doc, &stack, decode(&rest[..end], false));
            pos += end;
        } else if let Some(body) = rest.strip_prefix("<!--") {
            let end = body.find("-->").unwrap_or_else(|| fail("unterminated comment".into()));
            add_node(&mut doc, &stack, Kind::Comment(body[..end].to_string()));
            pos += "<!--".len() + end + "-->".len();
        } else if let Some(body) = rest.strip_prefix("<![CDATA[") {
            let end = body.find("]]>").unwrap_or_else(|| fail("unterminated CDATA".into()));
            append_text(&mut doc, &stack, body[..end].to_string());
            pos += "<![CDATA[".len() + end + "]]>".len();
        } else if let Some(body) = rest.strip_prefix("<?") {
            let end = body.find("?>").unwrap_or_else(|| fail("unterminated processing instruction".into()));
            add_node(&mut doc, &stack, Kind::Pi(body[..end].to_string()));
            pos += "<?".len() + end + "?>".len();
        } else if let Some(body) = rest.strip_prefix("</") {
            let end = body.find('>').unwrap_or_else(|| fail("unterminated end tag".into()));
            let name = body[..end].trim_end_matches(|c: char| c.is_ascii_whitespace());
            let open = stack.pop().unwrap_or_else(|| fail(format!("stray </{name}>")));
            scopes.pop();
            match &doc.nodes[open].kind {
                Kind::Element { qname, .. } if qname == name => {}
                _ => fail(format!("</{name}> does not close the open element")),
            }
            pos += "</".len() + end + 1;
        } else {
            // Start tag.
            pos += 1;
            let name_start = pos;
            while !is_xml_space(b[pos]) && b[pos] != b'/' && b[pos] != b'>' {
                pos += 1;
            }
            let qname = src[name_start..pos].to_string();
            let mut raw_attrs: Vec<(String, String)> = Vec::new();
            let self_closing;
            loop {
                skip_space(&mut pos);
                if b[pos] == b'/' {
                    if b[pos + 1] != b'>' {
                        fail(format!("malformed <{qname}"));
                    }
                    pos += 2;
                    self_closing = true;
                    break;
                }
                if b[pos] == b'>' {
                    pos += 1;
                    self_closing = false;
                    break;
                }
                let attr_start = pos;
                while !is_xml_space(b[pos]) && b[pos] != b'=' {
                    pos += 1;
                }
                let attr_name = src[attr_start..pos].to_string();
                skip_space(&mut pos);
                if b[pos] != b'=' {
                    fail(format!("attribute {attr_name} of <{qname} has no value"));
                }
                pos += 1;
                skip_space(&mut pos);
                let quote = b[pos];
                if quote != b'"' && quote != b'\'' {
                    fail(format!("attribute {attr_name} of <{qname} is not quoted"));
                }
                pos += 1;
                let value_end = pos + src[pos..].find(quote as char).unwrap_or_else(|| fail("unterminated attribute".into()));
                raw_attrs.push((attr_name, decode(&src[pos..value_end], true)));
                pos = value_end + 1;
            }

            let mut scope = Vec::new();
            let mut ns_decls = Vec::new();
            let mut plain = Vec::new();
            for (name, value) in raw_attrs {
                if name == "xmlns" {
                    scope.push((String::new(), value.clone()));
                    ns_decls.push((name, value));
                } else if let Some(prefix) = name.strip_prefix("xmlns:") {
                    scope.push((prefix.to_string(), value.clone()));
                    ns_decls.push((name, value));
                } else {
                    plain.push((name, value));
                }
            }
            scopes.push(scope);
            let (prefix, local) = split_qname(&qname);
            let ns = resolve(&scopes, prefix);
            let local = local.to_string();
            let attrs = plain
                .into_iter()
                .map(|(qname, value)| {
                    let (prefix, local) = split_qname(&qname);
                    // An unprefixed attribute is in no namespace, whatever the default is.
                    let ns = if prefix.is_empty() { String::new() } else { resolve(&scopes, prefix) };
                    let local = local.to_string();
                    Attr { qname, ns, local, value }
                })
                .collect();
            let id = add_node(&mut doc, &stack, Kind::Element { qname, ns, local, ns_decls, attrs });
            if !seen_root {
                doc.root = id;
                seen_root = true;
            }
            if self_closing {
                scopes.pop();
            } else {
                stack.push(id);
            }
        }
    }
    doc
}

fn add_node(doc: &mut Doc, stack: &[usize], kind: Kind) -> usize {
    let id = doc.nodes.len();
    let parent = stack.last().copied();
    doc.nodes.push(Node { kind, parent, children: Vec::new(), text: None, tail: None });
    if let Some(p) = parent {
        doc.nodes[p].children.push(id);
    }
    id
}

fn append_text(doc: &mut Doc, stack: &[usize], s: String) {
    if s.is_empty() {
        return;
    }
    let top = *stack.last().unwrap_or_else(|| fail("character data outside the root element".into()));
    let slot = match doc.nodes[top].children.last() {
        Some(&last) => &mut doc.nodes[last].tail,
        None => &mut doc.nodes[top].text,
    };
    match slot {
        Some(existing) => existing.push_str(&s),
        None => *slot = Some(s),
    }
}

// ── serializing (libxml2's escaping, as lxml's tostring writes it) ──────────

fn escape_text(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '\r' => out.push_str("&#13;"),
            _ => out.push(c),
        }
    }
}

fn escape_attr(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            '\t' => out.push_str("&#9;"),
            _ => out.push(c),
        }
    }
}

fn write_node(doc: &Doc, id: usize, out: &mut String) {
    let node = &doc.nodes[id];
    match &node.kind {
        Kind::Element { qname, ns_decls, attrs, .. } => {
            out.push('<');
            out.push_str(qname);
            // libxml2 writes namespace declarations ahead of the attributes.
            for (name, value) in ns_decls {
                out.push(' ');
                out.push_str(name);
                out.push_str("=\"");
                escape_attr(out, value);
                out.push('"');
            }
            for a in attrs {
                out.push(' ');
                out.push_str(&a.qname);
                out.push_str("=\"");
                escape_attr(out, &a.value);
                out.push('"');
            }
            if node.children.is_empty() && node.text.is_none() {
                out.push_str("/>");
            } else {
                out.push('>');
                if let Some(t) = &node.text {
                    escape_text(out, t);
                }
                for &c in &node.children {
                    write_node(doc, c, out);
                }
                out.push_str("</");
                out.push_str(qname);
                out.push('>');
            }
        }
        Kind::Comment(c) => {
            out.push_str("<!--");
            out.push_str(c);
            out.push_str("-->");
        }
        Kind::Pi(p) => {
            out.push_str("<?");
            out.push_str(p);
            out.push_str("?>");
        }
    }
    if let Some(t) = &node.tail {
        escape_text(out, t);
    }
}

fn serialize(doc: &Doc) -> String {
    let mut out = String::from("<?xml version='1.0' encoding='UTF-8' standalone='yes'?>\n");
    write_node(doc, doc.root, &mut out);
    out
}

// ── tree queries and edits (lxml semantics) ─────────────────────────────────

impl Doc {
    fn is_w(&self, id: usize, name: &str) -> bool {
        matches!(&self.nodes[id].kind, Kind::Element { ns, local, .. } if ns == W && local == name)
    }

    fn attr(&self, id: usize, ns_uri: &str, name: &str) -> Option<&str> {
        match &self.nodes[id].kind {
            Kind::Element { attrs, .. } => attrs
                .iter()
                .find(|a| a.ns == ns_uri && a.local == name)
                .map(|a| a.value.as_str()),
            _ => None,
        }
    }

    fn w_val(&self, id: usize) -> Option<&str> {
        self.attr(id, W, "val")
    }

    fn text(&self, id: usize) -> &str {
        self.nodes[id].text.as_deref().unwrap_or("")
    }

    /// `el.iter('{W}name')`: this node and its descendants, document order.
    fn iter(&self, id: usize, name: &str) -> Vec<usize> {
        let mut found = Vec::new();
        let mut pending = vec![id];
        while let Some(n) = pending.pop() {
            if self.is_w(n, name) {
                found.push(n);
            }
            pending.extend(self.nodes[n].children.iter().rev());
        }
        found
    }

    /// `el.findall('.//{W}name')`: descendants only.
    fn descendants(&self, id: usize, name: &str) -> Vec<usize> {
        let mut found = self.iter(id, name);
        if self.is_w(id, name) {
            found.remove(0);
        }
        found
    }

    /// `el.findall('{W}name')`: direct children only.
    fn children(&self, id: usize, name: &str) -> Vec<usize> {
        self.nodes[id].children.iter().copied().filter(|&c| self.is_w(c, name)).collect()
    }

    fn has_visible_text(&self, id: usize) -> bool {
        self.iter(id, "t").iter().any(|&t| !self.text(t).trim().is_empty())
    }

    /// `el.getparent().remove(el)`: detaches the node, tail and all.
    fn remove(&mut self, id: usize) {
        let parent = self.nodes[id].parent.unwrap_or_else(|| fail("removing a detached node".into()));
        self.nodes[parent].children.retain(|&c| c != id);
        self.nodes[id].parent = None;
    }

    /// `parent.insert(idx, child)`: moves the child (and its tail) from wherever it is.
    fn insert(&mut self, parent: usize, idx: usize, child: usize) {
        if let Some(old) = self.nodes[child].parent {
            self.nodes[old].children.retain(|&c| c != child);
        }
        self.nodes[parent].children.insert(idx, child);
        self.nodes[child].parent = Some(parent);
    }

    fn position(&self, parent: usize, child: usize) -> usize {
        self.nodes[parent].children.iter().position(|&c| c == child).unwrap()
    }

    /// The node after `id` in a depth-first walk of `top`'s subtree, as lxml's
    /// iterator steps: into the children, else the next sibling of the nearest
    /// ancestor that has one; a detached ancestor ends the walk.
    fn next_in_walk(&self, id: usize, top: usize) -> Option<usize> {
        if let Some(&first) = self.nodes[id].children.first() {
            return Some(first);
        }
        let mut n = id;
        while n != top {
            let parent = self.nodes[n].parent?;
            let siblings = &self.nodes[parent].children;
            let at = siblings.iter().position(|&c| c == n).unwrap();
            if at + 1 < siblings.len() {
                return Some(siblings[at + 1]);
            }
            n = parent;
        }
        None
    }

    fn next_w_in_walk(&self, mut id: usize, top: usize, name: &str) -> Option<usize> {
        loop {
            id = self.next_in_walk(id, top)?;
            if self.is_w(id, name) {
                return Some(id);
            }
        }
    }
}

// ── the cleanup ─────────────────────────────────────────────────────────────

fn clean_document(doc: &mut Doc) -> bool {
    let root = doc.root;
    let Some(&body) = doc.children(root, "body").first() else {
        return false;
    };

    // Word field framing (fldChar / instrText runs): tex4ht wraps the
    // bibliography in a BIBLIOGRAPHY field, which Word shades grey.
    for r in doc.iter(body, "r") {
        if !doc.children(r, "fldChar").is_empty() || !doc.children(r, "instrText").is_empty() {
            doc.remove(r);
        }
    }

    // Internal hyperlinks (\ref cross-references) unwrapped to plain runs, their
    // Hyperlink character style dropped.
    for hl in doc.iter(body, "hyperlink") {
        let parent = doc.nodes[hl].parent.unwrap_or_else(|| fail("detached hyperlink".into()));
        let mut idx = doc.position(parent, hl);
        for child in doc.nodes[hl].children.clone() {
            for rstyle in doc.descendants(child, "rStyle") {
                if doc.w_val(rstyle) == Some("Hyperlink") {
                    doc.remove(rstyle);
                }
            }
            doc.insert(parent, idx, child);
            idx += 1;
        }
        doc.remove(hl);
    }

    for p in doc.iter(body, "p") {
        // Page breaks in a paragraph with no text only make blank pages.
        if !doc.has_visible_text(p) {
            for br in doc.descendants(p, "br") {
                if doc.attr(br, W, "type") == Some("page") {
                    doc.remove(br);
                }
            }
        }

        // Trailing runs that hold only whitespace (a run with a break stops the walk).
        for r in doc.children(p, "r").into_iter().rev() {
            if !doc.children(r, "br").is_empty() {
                break;
            }
            if doc.children(r, "t").iter().all(|&t| doc.text(t).trim().is_empty()) {
                doc.remove(r);
            } else {
                break;
            }
        }

        // tex4ht ends paragraphs with ". ": trim the last non-empty <w:t>.
        for t in doc.iter(p, "t").into_iter().rev() {
            let txt = doc.text(t);
            if txt.is_empty() {
                continue;
            }
            let trimmed = txt.trim_end();
            if trimmed.len() != txt.len() {
                if trimmed.is_empty() {
                    doc.nodes[t].text = None;
                    // No orphan xml:space="preserve" on an element left empty.
                    if let Kind::Element { attrs, .. } = &mut doc.nodes[t].kind {
                        attrs.retain(|a| !(a.ns == XML_NS && a.local == "space"));
                    }
                } else {
                    doc.nodes[t].text = Some(trimmed.to_string());
                }
            }
            break;
        }

        // Justified → left, inline.
        for jc in doc.iter(p, "jc") {
            if matches!(doc.w_val(jc), Some("both" | "justify")) {
                if let Kind::Element { attrs, .. } = &mut doc.nodes[jc].kind {
                    for a in attrs.iter_mut().filter(|a| a.ns == W && a.local == "val") {
                        a.value = "left".to_string();
                    }
                }
            }
        }

        // Drop paragraphs with nothing visible (no text, break or section
        // properties) and tex4ht's environment-delimiter paragraphs, but never
        // the only paragraph of a table cell: OOXML requires one, and without it
        // Word and LibreOffice collapse the cell and shift the row left.
        let has_text = doc.has_visible_text(p);
        let has_break = !doc.descendants(p, "br").is_empty();
        let has_sect = !doc.descendants(p, "sectPr").is_empty();
        let is_env = doc
            .descendants(p, "pStyle")
            .first()
            .is_some_and(|&s| matches!(doc.w_val(s), Some("begin-env-p" | "end-env-p")));
        let parent = doc.nodes[p].parent;
        let only_p_in_cell = parent.is_some_and(|q| doc.is_w(q, "tc") && doc.children(q, "p").len() == 1);
        if ((!has_text && !has_break && !has_sect) || is_env) && !only_p_in_cell {
            doc.remove(p);
        }
    }

    // tex4ht renders each \hline as an empty filler row: drop rows whose cells
    // hold no text. The walk steps the way lxml's live iterator does (the next
    // table is found before this one's rows are removed).
    let mut next = if doc.is_w(body, "tbl") { Some(body) } else { doc.next_w_in_walk(body, body, "tbl") };
    while let Some(tbl) = next {
        next = doc.next_w_in_walk(tbl, body, "tbl");
        for tr in doc.children(tbl, "tr") {
            if doc.children(tr, "tc").iter().all(|&tc| !doc.has_visible_text(tc)) {
                doc.remove(tr);
            }
        }
    }
    true
}

/// Byte-wise: everything the patterns match is ASCII, and XML puts only ASCII
/// whitespace inside a tag. styles.xml is an external file, so the scanners stop
/// at its end rather than read past it.
fn space_run(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && is_xml_space(b[i]) {
        i += 1;
    }
    i
}

/// Length of `<w:jc\s+w:val="(both|justify)"\s*/>` at `i`, if it is there.
fn justify_tag_at(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i + b"<w:jc".len();
    if !b[i..].starts_with(b"<w:jc") || j >= b.len() || !is_xml_space(b[j]) {
        return None;
    }
    j = space_run(b, j);
    if !b[j..].starts_with(b"w:val=\"") {
        return None;
    }
    j += b"w:val=\"".len();
    if b[j..].starts_with(b"both\"") {
        j += b"both\"".len();
    } else if b[j..].starts_with(b"justify\"") {
        j += b"justify\"".len();
    } else {
        return None;
    }
    j = space_run(b, j);
    b[j..].starts_with(b"/>").then(|| j + 2 - i)
}

/// End of the first `<w:style\s+w:type="paragraph"[^>]*w:styleId="Normal"[^>]*>`.
fn normal_style_tag_end(b: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i < b.len() {
        if b[i..].starts_with(b"<w:style") {
            let j = i + b"<w:style".len();
            if j < b.len() && is_xml_space(b[j]) {
                let k = space_run(b, j);
                if b[k..].starts_with(b"w:type=\"paragraph\"") {
                    let tag_rest = k + b"w:type=\"paragraph\"".len();
                    if let Some(close) = b[tag_rest..].iter().position(|&c| c == b'>') {
                        let inside = &b[tag_rest..tag_rest + close];
                        if inside.windows(b"w:styleId=\"Normal\"".len()).any(|w| w == b"w:styleId=\"Normal\"") {
                            return Some(tag_rest + close + 1);
                        }
                    }
                }
            }
        }
        i += 1;
    }
    None
}

fn clean_styles(xml: &str) -> String {
    let b = xml.as_bytes();
    let mut kept = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match justify_tag_at(b, i) {
            Some(len) => i += len,
            None => {
                kept.push(b[i]);
                i += 1;
            }
        }
    }
    if let Some(end) = normal_style_tag_end(&kept) {
        let insert = b"<w:pPr><w:jc w:val=\"left\"/></w:pPr>";
        kept.splice(end..end, insert.iter().copied());
    }
    // Only ASCII byte runs were cut or inserted, so the UTF-8 stays whole.
    String::from_utf8(kept).unwrap()
}

fn rewrite(path: &Path, f: impl FnOnce(&str) -> Option<String>) {
    if !path.exists() {
        return;
    }
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| fail(format!("read {}: {e}", path.display())));
    if let Some(out) = f(&src) {
        std::fs::write(path, out).unwrap_or_else(|e| fail(format!("write {}: {e}", path.display())));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [dir] = args.as_slice() else {
        eprintln!("usage: docx_tidy UNZIPPED_DOCX_DIR");
        exit(2)
    };
    let word = Path::new(dir).join("word");
    rewrite(&word.join("styles.xml"), |xml| Some(clean_styles(xml)));
    rewrite(&word.join("document.xml"), |xml| {
        let mut doc = parse(xml);
        clean_document(&mut doc).then(|| serialize(&doc))
    });
    println!("cleanup: ragged-right + empty-paragraph + trailing-space removal applied");
}
