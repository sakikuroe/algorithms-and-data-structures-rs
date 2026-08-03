"""
Library Checker / AtCoder 提出用に、src/bin/{library_checker,atcoder}/<slug>.rs を
単一ファイルへバンドルし、続けて不要コードを枝刈りするスクリプトである。

使い方:
    python3 tools/bundle.py <slug> [<slug> ...]   # 指定した問題をバンドル + 枝刈り
    python3 tools/bundle.py --all                 # PROBLEM_OPERATIONS 登録済みの全問題
    python3 tools/bundle.py --list                # 登録済み/未登録の問題一覧を表示
    python3 tools/bundle.py --prune <path> <bin>   # 既存ファイルの枝刈りのみ行う
                                                    # (convolution_mod など、手動でバンドル
                                                    # した非 fps 系ファイル向け)

新しい問題を追加する場合:
    1. src/bin/library_checker/<slug>.rs または src/bin/atcoder/<slug>.rs を作成する
       (先頭 2 行が `// Library Checker: <題名>` / `// AtCoder: <題名>` と
       `// <URL>` になっている前提)。
    2. 下記 PROBLEM_OPERATIONS に `"<slug>": [使用する fps 操作のリスト]` を追記する。
       使用する操作は main() の呼び出しから判断する:
         .inverse(...)         -> "inv"
         .log(...)             -> "log"
         .exp(...)             -> "exp"
         .pow(...)             -> "pow"
         FPS::product(...) や `*`/`*=` 演算子 -> "mul"
         bostan_mori::...      -> "bostan_mori"
         partition::...        -> "partition"
    3. `python3 tools/bundle.py <slug>` を実行する。

main() 本体はこのスクリプトに複製せず、実行の都度 src/bin/**/<slug>.rs から
読み込むため、実装を変更しても再バンドルするだけで常に最新の内容が反映される。
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

SRC_DIRS = {
    "Library Checker": REPO / "src/bin/library_checker",
    "AtCoder": REPO / "src/bin/atcoder",
}
BIN_PREFIX = {"Library Checker": "lc", "AtCoder": "ac"}


# =============================================================================
# 依存関係テーブル
# =============================================================================

# 各 fps 操作が (自分自身を含め) 追加で必要とする fps サブモジュール一覧。
# 呼び出し関係の根拠:
# - inv: 他の fps サブモジュールに依存しない。
# - log: log_dense が inverse_dense (inv) と `f * inv` (mul の Mul 実装) を使う。
# - exp: exp_dense_scalar が log_dense (log とその依存の inv/mul) と、
#   `truncated - res.log_dense(...)` (sub の Sub 実装)、`res *= delta`
#   (mul の MulAssign 実装) を使う。
# - pow: pow_dense が log_dense と exp_dense を呼ぶため、log と exp の依存を引き継ぐ。
# - mul: FPS::product / Mul 演算子は convolution を直接使うのみ。
# - bostan_mori: linear_recurrence_kth_term / bostan_mori は Mul (mul) と
#   inverse (inv) だけを使う。
# - partition: product_(inv_)one_minus_x_powers は内部で exp を使うため、
#   exp の依存一式を引き継ぐ。
OPERATION_DEPS = {
    "inv": ["inv"],
    "log": ["inv", "mul", "log"],
    "exp": ["inv", "mul", "sub", "log", "exp"],
    "pow": ["inv", "mul", "sub", "log", "exp", "pow"],
    "mul": ["mul"],
    "bostan_mori": ["inv", "mul", "bostan_mori"],
    "partition": ["inv", "mul", "sub", "log", "exp", "partition"],
}

# fps サブモジュールを列挙する際の正準順序 (可読性のためだけの並び順であり、
# コンパイル可否には影響しない)。
SUBMODULE_ORDER = [
    "add", "bostan_mori", "div", "exp", "inv", "log", "mul", "partition", "pow", "sub",
]

# 問題ごとに、main() が直接使用する fps 操作の一覧。
PROBLEM_OPERATIONS = {
    "inv_of_formal_power_series": ["inv"],
    "exp_of_formal_power_series": ["exp"],
    "log_of_formal_power_series": ["log"],
    "pow_of_formal_power_series": ["pow"],
    "inv_of_formal_power_series_sparse": ["inv"],
    "exp_of_formal_power_series_sparse": ["exp"],
    "log_of_formal_power_series_sparse": ["log"],
    "pow_of_formal_power_series_sparse": ["pow"],
    "kth_term_of_linearly_recurrent_sequence": ["bostan_mori"],
    "partition_function": ["partition"],
    "product_of_polynomial_sequence": ["mul"],
    "fps24_k_permutation": ["inv"],
    "fps24_l_permutation2": ["exp"],
    "fps24_m_connected_graph": ["log"],
    "fps24_c_sequence": ["pow"],
    "fps24_i_score": ["mul"],
    "fps24_a_snack": ["pow"],
    "fps24_b_tuple_of_integers": ["bostan_mori"],
    "fps24_d_sequence2": ["inv", "pow"],
    "fps24_e_sequence3": ["mul"],
    "fps24_n_coin2": ["partition"],
    "abc422_g_balls_and_boxes": ["partition", "mul"],
}


def needed_submodules(slug):
    operations = PROBLEM_OPERATIONS[slug]
    needed = set()
    for op in operations:
        needed.update(OPERATION_DEPS[op])
    return [name for name in SUBMODULE_ORDER if name in needed]


# =============================================================================
# バンドル生成
# =============================================================================


def strip_cfg_test_mod(text):
    # `#[cfg(test)]` に続く `mod tests { ... }` ブロックを、波括弧の対応を数えながら除去する。
    marker = "#[cfg(test)]"
    idx = text.find(marker)
    if idx == -1:
        return text
    mod_idx = text.find("mod tests", idx)
    brace_idx = text.find("{", mod_idx)
    depth = 0
    i = brace_idx
    while i < len(text):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                end = i + 1
                break
        i += 1
    else:
        raise RuntimeError("unbalanced braces while stripping mod tests")
    # マーカー直前の空行も含めて除去する。
    start = idx
    while start > 0 and text[start - 1] in " \t":
        start -= 1
    if start > 0 and text[start - 1] == "\n":
        start -= 1
    return text[:start] + text[end:]


def strip_submodule_decls(text, names):
    # `pub mod X;` / `mod X;` の宣言行と、直前に連続する `///` ドキュメントコメント行を除去する。
    lines = text.split("\n")
    out = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        is_target_decl = any(
            stripped == f"pub mod {name};" or stripped == f"mod {name};" for name in names
        )
        if is_target_decl:
            while out and out[-1].strip().startswith("///"):
                out.pop()
            i += 1
            continue
        out.append(line)
        i += 1
    return "\n".join(out)


def load(relative_path, strip_mods=None):
    text = (REPO / relative_path).read_text(encoding="utf-8")
    text = strip_cfg_test_mod(text)
    if strip_mods:
        text = strip_submodule_decls(text, strip_mods)
    return text.strip("\n")


def indent(text, level):
    pad = "    " * level
    return "\n".join((pad + line if line.strip() else "") for line in text.split("\n"))


def render_mod(name, content, children, level):
    body_parts = []
    if content:
        body_parts.append(indent(content, level + 1))
    for child_name, (child_content, child_children) in children.items():
        body_parts.append(render_mod(child_name, child_content, child_children, level + 1))
    body = "\n\n".join(p for p in body_parts if p.strip())
    pad = "    " * level
    return f"{pad}pub mod {name} {{\n{body}\n{pad}}}"


def build_core(needed_fps_submodules):
    fastio = load("src/io/fastio.rs")
    modulo = load("src/modulo998244353/modulo.rs")
    convolution = load("src/modulo998244353/convolution.rs")
    convolution_avx2 = load("src/modulo998244353/convolution_avx2.rs")
    convolution_mont = load("src/modulo998244353/convolution_mont.rs")

    io_mod = render_mod("io", "", {"fastio": (fastio, {})}, 0)

    modulo_children = {
        "modulo": (modulo, {}),
        "convolution": (convolution, {}),
        "convolution_avx2": (convolution_avx2, {}),
        "convolution_mont": (convolution_mont, {}),
    }
    if needed_fps_submodules:
        # fps.rs が宣言する全サブモジュール (SUBMODULE_ORDER) をいったん全て取り除いた上で、
        # 実際に必要なものだけを子モジュールとして再度組み込む。
        fps_root = load("src/modulo998244353/fps.rs", strip_mods=SUBMODULE_ORDER)
        fps_children = {
            name: (load(f"src/modulo998244353/fps/{name}.rs"), {})
            for name in needed_fps_submodules
        }
        modulo_children["fps"] = (fps_root, fps_children)

    modulo_mod = render_mod("modulo998244353", "", modulo_children, 0)
    return io_mod + "\n\n" + modulo_mod


def find_source_file(slug):
    for source, directory in SRC_DIRS.items():
        candidate = directory / f"{slug}.rs"
        if candidate.exists():
            return source, candidate
    known = ", ".join(SRC_DIRS)
    raise FileNotFoundError(
        f"{slug}.rs が見つからない ({known} のいずれの下にも存在しない)"
    )


def extract_source(src_path):
    text = src_path.read_text(encoding="utf-8")
    lines = text.split("\n")

    title_match = re.match(r"^// (Library Checker|AtCoder): (.+)$", lines[0])
    if not title_match:
        raise RuntimeError(f"{src_path}: 1 行目が '// Library Checker: ...' / '// AtCoder: ...' 形式ではない")
    source, title = title_match.groups()

    url_match = re.match(r"^// (https?://\S+)$", lines[1])
    if not url_match:
        raise RuntimeError(f"{src_path}: 2 行目が URL のコメント行ではない")
    url = url_match.group(1)

    body_start = next(i for i, line in enumerate(lines) if line.startswith("use anmitsu::"))
    body = "\n".join(lines[body_start:]).replace("anmitsu::", "")
    return source, title, url, body.strip("\n") + "\n"


def bin_name_for(source, slug):
    return f"{BIN_PREFIX[source]}-{slug.replace('_', '-')}-bundled"


def out_path_for(source, slug):
    return SRC_DIRS[source] / "bundled" / f"{slug}.rs"


def generate(slug):
    if slug not in PROBLEM_OPERATIONS:
        raise KeyError(
            f"'{slug}' は PROBLEM_OPERATIONS に未登録である。"
            "tools/bundle.py 冒頭のコメントに従って追記すること。"
        )
    source, src_path = find_source_file(slug)
    title, url, body = extract_source(src_path)[1:]

    submodules = needed_submodules(slug)
    core = build_core(submodules)
    fps_note = f"、および fps 配下で実際に呼び出す {{{', '.join(submodules)}}}" if submodules else ""
    header = (
        f"// {source}: {title}\n"
        f"// {url}\n"
        "//\n"
        "// anmitsu クレートの io::fastio と modulo998244353::{modulo, convolution,\n"
        f"// convolution_avx2, convolution_mont}}{fps_note} を手動でこのファイルへ\n"
        "// バンドルしたものである。ジャッジは外部クレートへの依存を解決できないため、\n"
        "// このファイル単体で完結させている。\n\n"
    )
    content = header + core + "\n\n" + body

    out_path = out_path_for(source, slug)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(content, encoding="utf-8")
    bin_name = bin_name_for(source, slug)
    print(f"generated {out_path.relative_to(REPO)} ({len(content.splitlines())} lines)")
    return out_path, bin_name


# =============================================================================
# 枝刈り (dead code / 未使用の trait 実装 / doc コメント除去)
# =============================================================================

MONO_IMPL_RE = re.compile(r"<(?P<ty>.+?) as (?P<trait>[\w:]+)>::")
# Drop は明示的な呼び出しがなくてもスコープを抜けるときに暗黙に呼ばれるため、
# "呼ばれていないように見える" だけで消してしまうと (flush 漏れなどの) 実害の
# あるバグになる。安全のため、削除候補の探索から常に除外する。
NEVER_PRUNE_TRAITS = {"Drop"}

IMPL_FOR_RE = re.compile(r"^\s*impl\s+([A-Za-z_]\w*)\s+for\s+([A-Za-z_][\w:<>\[\], ]*?)\s*\{")
IMPL_INHERENT_RE = re.compile(r"^\s*impl\s+([A-Za-z_]\w*)\s*\{")
TYPE_DEF_RE = re.compile(r"^\s*(?:pub\s+)?(?:struct|enum)\s+([A-Za-z_]\w*)")


def build_dead_code_diagnostics(path, bin_name):
    proc = subprocess.run(
        ["cargo", "build", "--bin", bin_name, "--message-format=json"],
        capture_output=True, text=True, cwd=REPO,
    )
    diags = []
    for line in proc.stdout.splitlines():
        try:
            msg = json.loads(line)
        except ValueError:
            continue
        if msg.get("reason") != "compiler-message":
            continue
        m = msg.get("message", {})
        code = (m.get("code") or {}).get("code")
        if code != "dead_code":
            continue
        for sp in m.get("spans", []):
            if sp.get("is_primary") and sp.get("file_name") == path:
                diags.append(sp["line_start"])
    return sorted(set(diags))


def build_mono_items(path):
    # nightly の -Z print-mono-items=yes は、実際に単相化された (=生成される) アイテムを
    # 標準出力へ書き出す。これにより、dead_code lint では検出できない「trait 実装は
    # あるが、その型では一度も呼ばれていない」ケースを 1 回のビルドで判定できる。
    #
    # ファイルが (この関数を呼ぶ前の段階で) コンパイルエラーを起こしている場合、
    # 単相化の収集が最後まで走らず標準出力が不完全になる。その不完全な結果を
    # 「使われていない」と誤判定して usable な impl まで消してしまうと危険なので、
    # コンパイルが失敗した場合は None を返し、呼び出し側で今回の枝刈りを見送る。
    proc = subprocess.run(
        [
            "rustc", "+nightly", "--edition", "2021", "-O",
            "-Z", "print-mono-items=yes", "--crate-type", "bin",
            "-o", "/tmp/_mono_probe_out", path,
        ],
        capture_output=True, text=True, cwd=REPO,
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
    i = sig_line_1indexed - 1
    start = i
    j = start - 1
    while j >= 0 and is_comment_or_attr(lines[j]):
        start = j
        j -= 1
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


def remove_ranges(path, ranges):
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    ranges = sorted(set(ranges), key=lambda r: -r[0])
    removed_here = 0
    for s, e in ranges:
        overlap = any((s2, e2) != (s, e) and s2 <= s and e <= e2 for s2, e2 in ranges)
        if overlap:
            continue
        del lines[s:e + 1]
        removed_here += 1
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    return removed_here


def strip_doc_comments(path):
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    kept = [line for line in lines if line.strip()[:3] not in ("///", "//!")]
    removed = len(lines) - len(kept)
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(kept))
    return removed


def prune_dead_code_once(path, bin_name):
    dead_lines = build_dead_code_diagnostics(path, bin_name)
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    orphaned_lines = find_orphaned_inherent_impl_lines(lines)
    target_lines = sorted(set(dead_lines) | set(orphaned_lines))
    if not target_lines:
        return 0
    ranges = [find_item_range(lines, ln) for ln in target_lines]
    return remove_ranges(path, ranges)


def prune_unused_impls_once(path):
    mono_pairs = build_mono_items(path)
    if mono_pairs is None:
        return 0
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    unused_lines = find_unused_impl_lines(lines, mono_pairs)
    if not unused_lines:
        return 0
    ranges = [find_item_range(lines, ln) for ln in unused_lines]
    return remove_ranges(path, ranges)


def prune(path, bin_name):
    path = str(path)
    total_removed = 0
    for iteration in range(30):
        removed = prune_dead_code_once(path, bin_name)
        removed += prune_unused_impls_once(path)
        total_removed += removed
        print(f"  iteration {iteration}: removed {removed} items")
        if removed == 0:
            break
    else:
        print("  stopped after max iterations")
        return

    doc_lines_removed = strip_doc_comments(path)
    print(f"  stripped {doc_lines_removed} doc-comment lines (///, //!)")
    print(f"  total items removed: {total_removed}")


# =============================================================================
# CLI
# =============================================================================


def list_problems():
    print("登録済み (generate + prune 可能):")
    for slug in sorted(PROBLEM_OPERATIONS):
        try:
            source, _ = find_source_file(slug)
        except FileNotFoundError:
            source = "(ソースファイルなし!)"
        print(f"  {slug}  [{source}]")

    discovered = set()
    for directory in SRC_DIRS.values():
        for path in directory.glob("*.rs"):
            discovered.add(path.stem)
    unregistered = sorted(discovered - set(PROBLEM_OPERATIONS))
    if unregistered:
        print("\n未登録 (src/bin/ には存在するが PROBLEM_OPERATIONS に未追記):")
        for slug in unregistered:
            print(f"  {slug}")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("slugs", nargs="*", help="バンドルする問題のスラッグ")
    parser.add_argument("--all", action="store_true", help="登録済みの全問題をバンドルする")
    parser.add_argument("--list", action="store_true", help="登録済み/未登録の問題一覧を表示する")
    parser.add_argument(
        "--prune", nargs=2, metavar=("PATH", "BIN_NAME"),
        help="既存ファイルの枝刈りのみ行う (convolution_mod など手動バンドル向け)",
    )
    args = parser.parse_args()

    if args.list:
        list_problems()
        return

    if args.prune:
        path, bin_name = args.prune
        print(f"### prune-only: {path} ({bin_name}) ###")
        prune(path, bin_name)
        return

    if args.all:
        targets = sorted(PROBLEM_OPERATIONS)
    else:
        targets = args.slugs

    if not targets:
        parser.error("バンドルする問題のスラッグ、--all、--list、--prune のいずれかを指定すること")

    for slug in targets:
        print(f"### {slug} ###")
        out_path, bin_name = generate(slug)
        prune(out_path, bin_name)
        print()


if __name__ == "__main__":
    main()
