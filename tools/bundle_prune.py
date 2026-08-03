import json
import re
import subprocess

PATH = "src/bin/library_checker/bundled/convolution_mod.rs"

# 単相化 (monomorphization) が実際に行われた <Type as Trait>::method を拾う正規表現。
MONO_IMPL_RE = re.compile(r"<(?P<ty>.+?) as (?P<trait>[\w:]+)>::")

# 安全のため、削除候補の探索から常に除外する trait。Drop は明示的な呼び出しがなくても
# スコープを抜けるときに暗黙に呼ばれるため、"呼ばれていないように見える" だけで
# 消してしまうと (flush 漏れなどの) 実害のあるバグになる。
NEVER_PRUNE_TRAITS = {"Drop"}


def build_dead_code_diagnostics():
    proc = subprocess.run(
        ["cargo", "build", "--bin", "lc-convolution-mod-bundled", "--message-format=json"],
        capture_output=True, text=True,
    )
    diags = []
    for line in proc.stdout.splitlines():
        try:
            msg = json.loads(line)
        except Exception:
            continue
        if msg.get("reason") != "compiler-message":
            continue
        m = msg.get("message", {})
        code = (m.get("code") or {}).get("code")
        if code != "dead_code":
            continue
        for sp in m.get("spans", []):
            if sp.get("is_primary") and sp.get("file_name") == PATH:
                diags.append(sp["line_start"])
    return sorted(set(diags))


def build_mono_items():
    # nightly の -Z print-mono-items=yes は、実際に単相化された (=生成される) アイテムを
    # 標準出力へ書き出す。これにより、dead_code lint では検出できない
    # 「trait 実装はあるが、その型では一度も呼ばれていない」ケースを 1 回のビルドで判定できる。
    #
    # ファイルが (この関数を呼ぶ前の段階で) コンパイルエラーを起こしている場合、
    # 単相化の収集が最後まで走らず標準出力が不完全になる。その不完全な結果を
    # 「使われていない」と誤判定して usable な impl まで消してしまうと危険なので、
    # コンパイルが失敗した場合は None を返し、呼び出し側で今回の枝刈りを見送る。
    proc = subprocess.run(
        [
            "rustc", "+nightly", "--edition", "2021", "-O",
            "-Z", "print-mono-items=yes", "--crate-type", "bin",
            "-o", "/tmp/_mono_probe_out", PATH,
        ],
        capture_output=True, text=True,
    )
    if proc.returncode != 0:
        print("  (mono-items probe failed to compile; skipping this round)")
        return None
    pairs = set()
    for line in proc.stdout.splitlines():
        if not line.startswith("MONO_ITEM"):
            continue
        for match in MONO_IMPL_RE.finditer(line):
            pairs.add((match.group("ty"), match.group("trait")))
    return pairs


def type_matches(mono_ty, short_ty):
    return mono_ty == short_ty or mono_ty.endswith("::" + short_ty)


def trait_matches(mono_trait, short_trait):
    return mono_trait == short_trait or mono_trait.endswith("::" + short_trait)


def is_comment_or_attr(line):
    s = line.strip()
    return s.startswith("///") or s.startswith("//!") or s.startswith("//") or s.startswith("#[")


def find_item_range(lines, sig_line_1indexed):
    # lines: 0-indexed list of file lines (no trailing \n)
    i = sig_line_1indexed - 1
    start = i
    # extend upward over contiguous doc/attr/plain-comment lines
    j = start - 1
    while j >= 0 and is_comment_or_attr(lines[j]):
        start = j
        j -= 1
    # determine terminator kind by scanning tokens on/after the signature line
    text = "\n".join(lines[i:])
    depth = 0
    in_string = False
    in_char = False
    k = 0
    n = len(text)
    started_brace = False
    while k < n:
        c = text[k]
        if in_string:
            if c == "\\":
                k += 2
                continue
            if c == '"':
                in_string = False
            k += 1
            continue
        if in_char:
            if c == "\\":
                k += 2
                continue
            if c == "'":
                in_char = False
            k += 1
            continue
        if c == "/" and k + 1 < n and text[k + 1] == "/":
            nl = text.find("\n", k)
            k = nl if nl != -1 else n
            continue
        if c == "/" and k + 1 < n and text[k + 1] == "*":
            end = text.find("*/", k + 2)
            k = end + 2 if end != -1 else n
            continue
        if c == '"':
            in_string = True
            k += 1
            continue
        if c == "'":
            if k + 2 < n and text[k + 2] == "'":
                in_char = True
                k += 1
                continue
            k += 1
            continue
        if c in "([{":
            depth += 1
            started_brace = started_brace or c == "{"
            k += 1
            continue
        if c in ")]}":
            depth -= 1
            k += 1
            if depth == 0 and started_brace:
                end_offset = k
                consumed = text[:end_offset]
                end_line_rel = consumed.count("\n")
                return start, i + end_line_rel
            continue
        if c == ";" and depth == 0:
            end_offset = k + 1
            consumed = text[:end_offset]
            end_line_rel = consumed.count("\n")
            return start, i + end_line_rel
        k += 1
    raise RuntimeError(f"could not find end of item starting at line {sig_line_1indexed}")


IMPL_FOR_RE = re.compile(r"^\s*impl\s+([A-Za-z_]\w*)\s+for\s+([A-Za-z_][\w:<>\[\], ]*?)\s*\{")
IMPL_INHERENT_RE = re.compile(r"^\s*impl\s+([A-Za-z_]\w*)\s*\{")
TYPE_DEF_RE = re.compile(r"^\s*(?:pub\s+)?(?:struct|enum)\s+([A-Za-z_]\w*)")


def find_orphaned_inherent_impl_lines(lines):
    # dead_code 枝刈りで struct/enum を消すと、それを対象にした "impl Type { ... }"
    # (trait 実装ではない、素の impl ブロック) が、参照先を失ったまま取り残されることがある。
    # コンパイルエラーになる前に、対応する定義が残っているかをここで確認して一緒に消す。
    defined = set()
    for line in lines:
        m = TYPE_DEF_RE.match(line)
        if m:
            defined.add(m.group(1))
    orphaned = []
    for idx, line in enumerate(lines):
        m = IMPL_INHERENT_RE.match(line)
        if not m:
            continue
        if m.group(1) not in defined:
            orphaned.append(idx + 1)
    return orphaned


def find_unused_impl_lines(lines, mono_pairs):
    # 「impl Trait for Type」のうち、mono_pairs に一致するものが 1 つもない行番号 (1-indexed) を返す。
    unused = []
    for idx, line in enumerate(lines):
        m = IMPL_FOR_RE.match(line)
        if not m:
            continue
        trait_name, type_name = m.group(1), m.group(2)
        if trait_name in NEVER_PRUNE_TRAITS:
            continue
        used = any(
            trait_matches(mono_trait, trait_name) and type_matches(mono_ty, type_name)
            for mono_ty, mono_trait in mono_pairs
        )
        if not used:
            unused.append(idx + 1)
    return unused


def remove_ranges(ranges):
    with open(PATH, encoding="utf-8") as f:
        lines = f.read().split("\n")
    ranges = sorted(set(ranges), key=lambda r: -r[0])
    removed_here = 0
    for s, e in ranges:
        overlap = any((s2, e2) != (s, e) and s2 <= s and e <= e2 for s2, e2 in ranges)
        if overlap:
            continue
        del lines[s:e + 1]
        removed_here += 1
    with open(PATH, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    return removed_here


def strip_doc_comments():
    with open(PATH, encoding="utf-8") as f:
        lines = f.read().split("\n")
    kept = [line for line in lines if line.strip()[:3] not in ("///", "//!")]
    removed = len(lines) - len(kept)
    with open(PATH, "w", encoding="utf-8") as f:
        f.write("\n".join(kept))
    return removed


def prune_dead_code_once():
    dead_lines = build_dead_code_diagnostics()
    with open(PATH, encoding="utf-8") as f:
        lines = f.read().split("\n")
    # 構造体・enum を消した結果として孤立した素の impl ブロックも、同じ回で一緒に検出する。
    # そうしないと、次のビルド (特に mono-items の探索用ビルド) がコンパイルエラーになり、
    # そのエラーによって不完全になった単相化リストを誤って信用してしまう。
    orphaned_lines = find_orphaned_inherent_impl_lines(lines)
    target_lines = sorted(set(dead_lines) | set(orphaned_lines))
    if not target_lines:
        return 0
    ranges = [find_item_range(lines, ln) for ln in target_lines]
    return remove_ranges(ranges)


def prune_unused_impls_once():
    mono_pairs = build_mono_items()
    if mono_pairs is None:
        return 0
    with open(PATH, encoding="utf-8") as f:
        lines = f.read().split("\n")
    unused_lines = find_unused_impl_lines(lines, mono_pairs)
    if not unused_lines:
        return 0
    ranges = [find_item_range(lines, ln) for ln in unused_lines]
    return remove_ranges(ranges)


def main():
    total_removed = 0
    for iteration in range(30):
        removed = prune_dead_code_once()
        removed += prune_unused_impls_once()
        total_removed += removed
        print(f"iteration {iteration}: removed {removed} items")
        if removed == 0:
            break
    else:
        print("stopped after max iterations")
        return

    doc_lines_removed = strip_doc_comments()
    print(f"stripped {doc_lines_removed} doc-comment lines (///, //!)")
    print(f"total items removed: {total_removed}")


if __name__ == "__main__":
    main()
