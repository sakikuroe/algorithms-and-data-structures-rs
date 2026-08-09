"""
Library Checker / AtCoder / CodeChef / AOJ / Baekjoon / POJ / yukicoder /
Codeforces / SPOJ 提出用に、
src/bin/{library_checker,atcoder,codechef,aoj,baekjoon,poj,yukicoder,codeforces,spoj}/
<slug>.rs を単一ファイルへバンドルし、続けて不要コードを枝刈りするスクリプトである。

使い方:
    python3 tools/bundle.py <slug> [<slug> ...]    # 指定した問題をバンドル + 枝刈り
    python3 tools/bundle.py --all                  # src/bin 以下の全問題
    python3 tools/bundle.py --prune <path> <bin>   # 既存ファイルの枝刈りのみ行う

新しい問題を追加する場合は、src/bin/library_checker/<slug>.rs、
src/bin/atcoder/<slug>.rs、src/bin/codechef/<slug>.rs、src/bin/aoj/<slug>.rs、
src/bin/baekjoon/<slug>.rs、src/bin/poj/<slug>.rs、src/bin/yukicoder/<slug>.rs、
src/bin/codeforces/<slug>.rs、または src/bin/spoj/<slug>.rs を作成し (先頭 2 行が
`// Library Checker: <題名>` / `// AtCoder: <題名>` / `// CodeChef: <題名>` /
`// AOJ: <題名>` / `// Baekjoon: <題名>` / `// POJ: <題名>` / `// yukicoder: <題名>` /
`// Codeforces: <題名>` / `// SPOJ: <題名>` と `// <URL>` になっている前提)、
Cargo.toml にバンドル後のバイナリーを登録したうえで
`python3 tools/bundle.py <slug>` を実行するだけでよい。
どのモジュールを取り込むかを手で指定する必要はない。

バンドル後のバイナリーは生成物であってリポジトリーには含めないため、Cargo.toml へは
`required-features = ["bundled"]` を添えて登録する。こうしておかないと、生成前の
クリーンチェックアウトで cargo がパスを解決できず、`cargo test` 自体が失敗する。

バンドルは、まず src/lib.rs のモジュールツリーをそのまま単一ファイルへ展開し、
そのうえでコンパイラの診断を頼りに到達しないコードを削る、という手順で行う。
必要なモジュールの判断をコンパイラに委ねているため、`log` が `inverse` を呼ぶ
といった use 文に現れない依存関係や、`*` 演算子から Mul の実装に到達する依存関係も
取りこぼさない。

枝刈りが働く前提として、展開後のトップレベルのモジュールには `pub` を付けずに
出力している。`pub mod` にすると rustc が全アイテムを外部から到達可能とみなし、
dead_code をひとつも報告しなくなるためである。

main() 本体はこのスクリプトに複製せず、実行の都度 src/bin/**/<slug>.rs から
読み込むため、実装を変更しても再バンドルするだけで常に最新の内容が反映される。
"""

import argparse
import dataclasses
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

SRC_DIRS = {
    "Library Checker": REPO / "src/bin/library_checker",
    "AtCoder": REPO / "src/bin/atcoder",
    "CodeChef": REPO / "src/bin/codechef",
    "AOJ": REPO / "src/bin/aoj",
    "Baekjoon": REPO / "src/bin/baekjoon",
    "POJ": REPO / "src/bin/poj",
    "yukicoder": REPO / "src/bin/yukicoder",
    "Codeforces": REPO / "src/bin/codeforces",
    "SPOJ": REPO / "src/bin/spoj",
}
BIN_PREFIX = {
    "Library Checker": "lc",
    "AtCoder": "ac",
    "CodeChef": "cc",
    "AOJ": "aoj",
    "Baekjoon": "bj",
    "POJ": "poj",
    "yukicoder": "yuki",
    "Codeforces": "cf",
    "SPOJ": "spoj",
}

# 枝刈りを繰り返す上限。トレイト実装の除去が新たな dead_code を生むため、
# 何も削れなくなるまで数回の往復が必要になる。
MAX_PRUNE_ITERATIONS = 30


# =============================================================================
# モジュールツリーの探索
# =============================================================================

# `mod X;` 形式のファイル参照と、`mod X { ... }` 形式のインライン宣言。
# 可視性は `pub`、`pub(crate)` のいずれも受け付ける。
MOD_DECL_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)\s*;")
MOD_OPEN_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)\s*\{")


@dataclasses.dataclass
class Module:
    """バンドル対象のモジュール 1 つ分を表す。"""

    name: str

    # 対応する .rs ファイル。src/lib.rs 内で `mod X { ... }` と直接書かれている
    # 階層 (algebra や ds など) には実体のファイルがないため、その場合は None である。
    path: "Path | None"

    children: "list[Module]"


def discover_children(path):
    """モジュールファイルが宣言する子モジュールを再帰的に探索する。

    `src/modulo998244353/fps.rs` の `mod add;` から
    `src/modulo998244353/fps/add.rs` を辿る、という対応関係を利用する。
    """
    text = strip_cfg_test_mod(path.read_text(encoding="utf-8"))
    directory = path.parent / path.stem
    children = []
    for line in text.split("\n"):
        match = MOD_DECL_RE.match(line)
        if not match:
            continue
        child_path = directory / f"{match.group(1)}.rs"
        if not child_path.exists():
            raise FileNotFoundError(f"{path}: 宣言された {child_path} が存在しない")
        children.append(Module(match.group(1), child_path, discover_children(child_path)))
    return children


def discover_module_tree():
    """src/lib.rs を読み取り、バンドルすべきモジュールツリーを組み立てる。

    lib.rs のモジュール宣言だけを対象とし、`pub fn add` のような lib.rs 自身の
    アイテムは展開しない。波括弧の深さを数えることで、`mod X { ... }` の入れ子と
    関数本体の波括弧を区別している。
    """
    lines = strip_cfg_test_mod((REPO / "src/lib.rs").read_text(encoding="utf-8")).split("\n")

    root = []
    children = root
    directory = REPO / "src"
    depth = 0
    # 開いている `mod X { ... }` を、抜けたときに復帰するための情報とともに積む。
    stack = []

    for line in lines:
        match = MOD_OPEN_RE.match(line)
        if match:
            node = Module(match.group(1), None, [])
            children.append(node)
            stack.append((children, directory, depth))
            children = node.children
            directory = directory / match.group(1)
            depth += line.count("{") - line.count("}")
            continue

        match = MOD_DECL_RE.match(line)
        if match:
            path = directory / f"{match.group(1)}.rs"
            if not path.exists():
                raise FileNotFoundError(f"src/lib.rs: 宣言された {path} が存在しない")
            children.append(Module(match.group(1), path, discover_children(path)))
            continue

        depth += line.count("{") - line.count("}")
        while stack and depth <= stack[-1][2]:
            children, directory, _ = stack.pop()

    return root


# =============================================================================
# バンドル生成
# =============================================================================


def find_cfg_test_mod_marker(text):
    """`#[cfg(test)]` に直接続けて `mod tests` が現れる位置 (文字オフセット) を探す。

    単純な部分文字列検索では、この属性をコメント中で言及している行
    (`segment_tree_dense.rs` の `` `#[cfg(test)]` で分離する。`` など) に
    誤って反応してしまう。そこから実際の `mod tests` ブロックの終わりまでを
    除去範囲とみなすと、その間にある本体コードを丸ごと削除してしまう
    (実際に、この誤判定でファイルの大部分が消える不具合が過去にあった)。
    そこで、属性行そのもの (前後の空白を除いた行全体が完全に一致する行) で
    あり、かつ直後の非空行が `mod tests` から始まっている場合に限って
    マーカーとみなす。同じファイルに `#[cfg(test)] use ...;` のような他の
    属性があっても、直後が `mod tests` でなければ読み飛ばす。
    """
    lines = text.split("\n")
    for i, line in enumerate(lines):
        if line.strip() != "#[cfg(test)]":
            continue
        j = i + 1
        while j < len(lines) and not lines[j].strip():
            j += 1
        if j < len(lines) and lines[j].lstrip().startswith("mod tests"):
            return sum(len(prev) + 1 for prev in lines[:i])
    return -1


def strip_cfg_test_mod(text):
    # `#[cfg(test)]` に続く `mod tests { ... }` ブロックを、波括弧の対応を数えながら除去する。
    idx = find_cfg_test_mod_marker(text)
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


def load(path, strip_mods=None):
    text = path.read_text(encoding="utf-8")
    text = strip_cfg_test_mod(text)
    if strip_mods:
        text = strip_submodule_decls(text, strip_mods)
    return text.strip("\n")


def indent(text, level):
    pad = "    " * level
    return "\n".join((pad + line if line.strip() else "") for line in text.split("\n"))


def render_module(module, level):
    """モジュールを入れ子の `mod` ブロックとして出力する。

    トップレベルだけ `pub` を付けないのは、`pub mod` にすると rustc が
    全アイテムを外部到達可能とみなして dead_code を報告しなくなるためである。
    内側は `pub` のままでよく、外側が private であれば実効可視性は抑えられる。
    """
    keyword = "mod" if level == 0 else "pub mod"
    body_parts = []
    if module.path is not None:
        content = load(module.path, strip_mods=[child.name for child in module.children])
        if content.strip():
            body_parts.append(indent(content, level + 1))
    for child in module.children:
        body_parts.append(render_module(child, level + 1))
    body = "\n\n".join(part for part in body_parts if part.strip())
    pad = "    " * level
    return f"{pad}{keyword} {module.name} {{\n{body}\n{pad}}}"


def build_core():
    return "\n\n".join(render_module(module, 0) for module in discover_module_tree())


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

    title_match = re.match(
        r"^// (Library Checker|AtCoder|CodeChef|AOJ|Baekjoon|POJ|yukicoder|Codeforces|SPOJ): (.+)$",
        lines[0],
    )
    if not title_match:
        raise RuntimeError(
            f"{src_path}: 1 行目が '// Library Checker: ...' / '// AtCoder: ...' / "
            "'// CodeChef: ...' / '// AOJ: ...' / '// Baekjoon: ...' / '// POJ: ...' / "
            "'// yukicoder: ...' / '// Codeforces: ...' / '// SPOJ: ...' 形式ではない"
        )
    source, title = title_match.groups()

    url_match = re.match(r"^// (https?://\S+)$", lines[1])
    if not url_match:
        raise RuntimeError(f"{src_path}: 2 行目が URL のコメント行ではない")
    url = url_match.group(1)

    # 本文は、1・2 行目のヘッダーコメントに続く空行の直後から始まる。
    # 以前は最初に現れる `use anmitsu::` の行を本文の開始位置としていたが、
    # `use std::collections::{HashSet, VecDeque};` のように anmitsu 以外の
    # use 文を先に書いている問題では、その行が本文から丸ごと欠落してしまう
    # 不具合があった。
    body_start = 2
    while body_start < len(lines) and not lines[body_start].strip():
        body_start += 1
    body = "\n".join(lines[body_start:]).replace("anmitsu::", "")
    return source, title, url, body.strip("\n") + "\n"


def bin_name_for(source, slug):
    return f"{BIN_PREFIX[source]}-{slug.replace('_', '-')}-bundled"


def out_path_for(source, slug):
    return SRC_DIRS[source] / "bundled" / f"{slug}.rs"


def ensure_bin_registered(bin_name, out_path):
    """バンドル後のバイナリーが Cargo.toml に登録されているかを確認する。

    枝刈りは `cargo build` の診断を利用するため、登録がないと何も削れない。
    cargo の分かりにくいエラーになる前に、追記すべき内容を示して中断する。
    """
    manifest = (REPO / "Cargo.toml").read_text(encoding="utf-8")
    if f'name = "{bin_name}"' in manifest:
        return
    snippet = (
        f'[[bin]]\nname = "{bin_name}"\npath = "{out_path.relative_to(REPO)}"\n'
        'required-features = ["bundled"]'
    )
    raise RuntimeError(
        f"Cargo.toml に {bin_name} が登録されていない。"
        "枝刈りは cargo の診断を用いるため、以下を Cargo.toml へ追記してから再実行すること。\n\n"
        f"{snippet}\n"
    )


def generate(slug, core):
    source, src_path = find_source_file(slug)
    title, url, body = extract_source(src_path)[1:]

    out_path = out_path_for(source, slug)
    bin_name = bin_name_for(source, slug)
    ensure_bin_registered(bin_name, out_path)

    header = (
        f"// {source}: {title}\n"
        f"// {url}\n"
        "//\n"
        "// anmitsu クレートを単一ファイルへ展開したうえで、この問題から到達しない\n"
        "// コードを tools/bundle.py が自動で枝刈りしたものである。ジャッジは外部\n"
        "// クレートへの依存を解決できないため、このファイル単体で完結させている。\n\n"
    )
    content = header + core + "\n\n" + body

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(content, encoding="utf-8")
    print(f"generated {out_path.relative_to(REPO)} ({len(content.splitlines())} lines)")
    return out_path, bin_name


# =============================================================================
# 枝刈り (dead code / 未使用の trait 実装 / doc コメント除去)
# =============================================================================

# mono-items が実装を書き出す形式は 2 通りある。ジェネリックな実装や標準ライブラリー
# の型に対する実装は `<&str as std::convert::AsRef<Path>>::as_ref` のように書かれるが、
# このクレート自身の型に対する実装は
# `modulo998244353::fps::mul::<impl std::ops::Mul for modulo998244353::fps::FPS>::mul`
# という形になる。後者を取りこぼすと、実際には使われている `*` や `-` の実装を
# 未使用と誤判定してしまうため、両方を拾う。
MONO_IMPL_RE = re.compile(r"<(?P<ty>.+?) as (?P<trait>[\w:]+)>::")
MONO_IMPL_FOR_RE = re.compile(r"<impl\s+(?P<trait>[\w:]+)(?:<[^>]*>)?\s+for\s+(?P<ty>[\w:]+)")
# Drop は明示的な呼び出しがなくてもスコープを抜けるときに暗黙に呼ばれるため、
# "呼ばれていないように見える" だけで消してしまうと (flush 漏れなどの) 実害の
# あるバグになる。安全のため、削除候補の探索から常に除外する。
NEVER_PRUNE_TRAITS = {"Drop"}

# トレイト名はパス修飾されることがある (`impl ops::Add for FPS`) ため、`::` を許す。
IMPL_FOR_RE = re.compile(
    r"^\s*impl\s+([A-Za-z_][\w:]*)\s+for\s+([A-Za-z_][\w:<>\[\], ]*?)\s*\{"
)

# 孤立した impl と use を見つけるための定義。impl のヘッダーは `where` 節が続いて
# `{` が次行以降に来ることがあるため、`{` の存在を前提にしない。
# ジェネリクスパラメータ一覧の除去は正規表現ではなく `strip_leading_generic_params`
# に委ね、ここでは `impl` の直後の文字列をそのまま捕捉するだけにとどめる。
IMPL_LINE_RE = re.compile(r"^\s*impl\b(.*)$")

# `macro_rules!` の本体は展開前のテンプレートであり、通常の Rust アイテムでは
# ない。テンプレート内の `impl Trait for $t { ... }` を通常の impl と誤認して
# 削除すると、`$( ... )*` の中身が空になり、対応するメタ変数を持たない不正な
# 繰り返し展開としてコンパイルエラーになる。そのため、この本体の行範囲は
# dead_code・孤立 impl・未使用 impl のいずれの削除対象からも除外する。
MACRO_RULES_OPEN_RE = re.compile(r"^\s*macro_rules!\s+([A-Za-z_]\w*)\s*\{\s*$")


def find_macro_rules_ranges(lines):
    """`macro_rules! name { ... }` 定義本体の (開始行, 終了行, マクロ名) を、
    行番号は 0-indexed・両端を含む形で列挙する。
    """
    ranges = []
    for idx, line in enumerate(lines):
        match = MACRO_RULES_OPEN_RE.match(line)
        if match:
            _, end = find_item_range(lines, idx + 1)
            ranges.append((idx, end, match.group(1)))
    return ranges


def is_within_ranges(idx, ranges):
    return any(start <= idx <= end for start, end, *_ in ranges)


def find_macro_invocation_lines(lines, name):
    """`name!( ... );` という単純な呼び出し文の行番号 (1-indexed) を列挙する。"""
    pattern = re.compile(r"^\s*" + re.escape(name) + r"!\s*\(.*\)\s*;\s*$")
    return [idx + 1 for idx, line in enumerate(lines) if pattern.match(line)]


def find_orphaned_macro_ranges(lines, removed_names):
    """本体のテンプレートが参照するトレイトが既に削除された `macro_rules!` を、
    定義本体と呼び出し文ごと (0-indexed, inclusive の行範囲として) 列挙する。

    `macro_rules!` の本体内にある `impl Trait for $t { ... }` は、通常の枝刈り
    対象からは除外している ([`find_macro_rules_ranges`] を参照) ため、その
    Trait 自体が削除された場合はここでマクロ定義・呼び出しをまとめて取り除く。
    """
    ranges = []
    for start, end, name in find_macro_rules_ranges(lines):
        body_refs_removed = False
        for line in lines[start : end + 1]:
            match = IMPL_LINE_RE.match(line)
            if not match:
                continue
            head = strip_leading_generic_params(match.group(1))
            trait_name, _ = impl_targets(head)
            if trait_name and trait_name in removed_names:
                body_refs_removed = True
                break
        if not body_refs_removed:
            continue
        ranges.append((start, end))
        for ln in find_macro_invocation_lines(lines, name):
            ranges.append((ln - 1, ln - 1))
    return ranges


TYPE_DEF_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|union)\s+([A-Za-z_]\w*)"
)
TRAIT_DEF_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?trait\s+([A-Za-z_]\w*)"
)
# バンドル後のファイルでは、モジュールは render_module により常に
# `mod X { ... }` というブロック形式で出力される (`mod X;` のファイル参照形式には
# ならない) ため、ブロック形式のみを対象とすればよい。
MODULE_DEF_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)\s*\{\s*$"
)
USE_SINGLE_RE = re.compile(r"^\s*use\s+[\w:]+::([A-Za-z_]\w*)\s*;")
USE_GROUP_RE = re.compile(r"^(\s*use\s+[\w:]+::)\{([^{}]*)\}(\s*;\s*)$")

# 丸ごと削除してよい dead_code 診断だけを選ぶ。rustc は enum のバリアントや
# 構造体のフィールドについても dead_code を報告するが、それらの primary span は
# アイテムの先頭ではなくバリアント行やフィールド行を指す。そこを起点に
# find_item_range を走らせると、次に現れる `;` までを巻き込んで enum 定義を
# 破壊してしまうため、アイテム単位の診断のみを受け付ける方針とする。
REMOVABLE_DEAD_CODE_RE = re.compile(
    r"^(?:multiple\s+)?"
    r"(?:function|method|associated function|associated item|struct|enum|union"
    r"|trait|type alias|constant|static|module|macro)s?\b"
)


def build_diagnostics(rel_path, bin_name):
    """バンドル結果をビルドし、コンパイルの可否と dead_code の行番号を返す。

    cargo が JSON で返す `file_name` はワークスペース相対パスであるため、
    突き合わせる側も相対パスでなければならない。絶対パスを渡すと一致せず、
    dead_code による枝刈りが黙って無効になる。

    バンドル後のバイナリーは `bundled` feature の下に置かれているため、
    ここで明示的に有効化する。有効化を忘れると cargo がターゲットを
    解決できず、枝刈りが一切行われないまま終わる。
    """
    proc = subprocess.run(
        [
            "cargo", "build", "--bin", bin_name,
            "--features", "bundled", "--message-format=json",
        ],
        capture_output=True, text=True, cwd=REPO,
    )
    dead_lines = []
    for line in proc.stdout.splitlines():
        try:
            record = json.loads(line)
        except ValueError:
            continue
        if record.get("reason") != "compiler-message":
            continue
        message = record.get("message", {})
        if (message.get("code") or {}).get("code") != "dead_code":
            continue
        if not REMOVABLE_DEAD_CODE_RE.match(message.get("message", "")):
            continue
        for span in message.get("spans", []):
            if span.get("is_primary") and span.get("file_name") == rel_path:
                dead_lines.append(span["line_start"])
    return proc.returncode == 0, sorted(set(dead_lines))


def build_mono_items(rel_path):
    # nightly の -Z print-mono-items=yes は、実際に単相化された (=生成される) アイテムを
    # 標準出力へ書き出す。これにより、dead_code lint では検出できない「trait 実装は
    # あるが、その型では一度も呼ばれていない」ケースを 1 回のビルドで判定できる。
    #
    # ファイルが (この関数を呼ぶ前の段階で) コンパイルエラーを起こしている場合、
    # 単相化の収集が最後まで走らず標準出力が不完全になる。その不完全な結果を
    # 「使われていない」と誤判定して usable な impl まで消してしまうと危険なので、
    # コンパイルが失敗した場合は None を返し、呼び出し側で今回の枝刈りを見送る。
    #
    # edition は Cargo.toml と揃える必要がある。ずれているとプローブが必ず失敗し、
    # トレイト実装の枝刈りが常に見送られてしまう。
    with tempfile.TemporaryDirectory() as workdir:
        proc = subprocess.run(
            [
                "rustc", "+nightly", "--edition", "2024", "-O",
                "-Z", "print-mono-items=yes", "--crate-type", "bin",
                "-o", str(Path(workdir) / "probe"), rel_path,
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
        for pattern in (MONO_IMPL_RE, MONO_IMPL_FOR_RE):
            for match in pattern.finditer(line):
                pairs.add((match.group("ty"), match.group("trait")))
    return pairs


def path_matches(mono_name, source_name):
    """mono-items が出力するパスと、ソース上の表記が同じものを指すかを判定する。

    ソース側は `impl ops::Sub for super::FPS` のように相対パスで書かれるのに対し、
    mono-items 側は `<modulo998244353::fps::FPS as std::ops::Sub>` のように絶対パスで
    出力される。前方の修飾は一致しないため、末尾の識別子どうしを比べる。

    取り違えた場合に使用中の実装まで消してしまうことを避けたいので、判定は
    一致する側へ倒してある。別モジュールに同名の型があると使われていない実装を
    残すことがあるが、その場合の損失は出力が数行大きくなることだけである。
    """
    return base_ident(mono_name) == base_ident(source_name)


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


def collect_definitions(lines):
    """ファイル内で定義されている型・トレイト・モジュールの名前を集める。"""
    names = set()
    for line in lines:
        for pattern in (TYPE_DEF_RE, TRAIT_DEF_RE, MODULE_DEF_RE):
            match = pattern.match(line)
            if match:
                names.add(match.group(1))
    return names


def base_ident(expr):
    """`super::FPS` や `SegmentTreeDense<M>` から、基になる識別子を取り出す。"""
    return expr.split("<", 1)[0].strip().rsplit("::", 1)[-1].strip()


def strip_leading_generic_params(text):
    """`impl` の直後にある `<...>` のジェネリクスパラメータ一覧を、内部の
    ネストした `<...>` (`Add<Output = T>` のような関連型束縛など) を数えながら
    読み飛ばす。

    単純な正規表現 `<[^>]*>` は最初に現れた `>` で止まってしまうため、
    `impl<T: std::ops::Add<Output = T>> Foo<T>` のような行を正しく扱えない。
    このような束縛は本リポジトリの数値ジェネリクスで頻出するため、深さを
    数える形で正しく対応する。
    """
    text = text.lstrip()
    if not text.startswith("<"):
        return text
    depth = 0
    for i, c in enumerate(text):
        if c == "<":
            depth += 1
        elif c == ">":
            depth -= 1
            if depth == 0:
                return text[i + 1 :].lstrip()
    # 閉じ括弧が見つからない場合 (通常は起きない) は、安全側に倒して
    # 元の文字列をそのまま返す。
    return text


def impl_targets(head):
    """impl のヘッダーから、実装するトレイト名と自己型名を取り出す。

    トレイト実装でない場合、トレイト名は None である。呼び出し側で、先頭の
    ジェネリクスパラメータ一覧は [`strip_leading_generic_params`] により
    除去済みであることを前提とする。
    """
    head = head.split("{", 1)[0].split(" where ", 1)[0].strip()
    if " for " in head:
        trait_part, type_part = head.split(" for ", 1)
        return base_ident(trait_part), base_ident(type_part)
    return None, base_ident(head)


def find_orphaned_lines(lines, removed_names):
    """定義が消えた型やトレイトを参照している impl と use の行を探す。

    dead_code の枝刈りは `struct MinMonoid` や `trait Monoid` を単体で消すため、
    それらを対象とする `impl Monoid for MinMonoid` や、別モジュールからの
    `use ...::monoid::Monoid;` が参照先を失って取り残される。放置するとコンパイル
    エラーになるので、同じラウンドのうちに回収する。

    判定に用いるのは、バンドル生成時点では定義されていたのに今は存在しない名前の
    集合である。`impl FastWrite for u32` の `u32` のように、もともと定義がない名前を
    誤って対象にしてしまうことがない。

    `macro_rules!` の本体にある `impl Trait for $t { ... }` のようなテンプレートは、
    見た目上 `impl ... for ...` に一致してしまうが実際のアイテムではないため、
    対象から除外する ([`find_macro_rules_ranges`] を参照)。
    """
    macro_ranges = find_macro_rules_ranges(lines)
    orphaned = []
    for idx, line in enumerate(lines):
        if is_within_ranges(idx, macro_ranges):
            continue
        match = IMPL_LINE_RE.match(line)
        if match:
            head = strip_leading_generic_params(match.group(1))
            trait_name, type_name = impl_targets(head)
            if type_name in removed_names or (trait_name and trait_name in removed_names):
                orphaned.append(idx + 1)
                continue
        match = USE_SINGLE_RE.match(line)
        if match and match.group(1) in removed_names:
            orphaned.append(idx + 1)
    return orphaned


def heal_use_groups(path, removed_names):
    """`use path::{a, b};` のように複数名をまとめた import から、定義が消えた
    名前だけを取り除く。

    `find_orphaned_lines` が扱う `USE_SINGLE_RE` は `use path::Name;` という単数形の
    import にしか一致しない。複数形の import (例: `use super::{convolution, modulo};`)
    では、一部の名前だけが枝刈りで消えても、他の名前は依然として使われている
    可能性がある。行ごと削除すると、生き残っている名前まで道連れにしてコンパイル
    エラーを起こしかねないため、消えた名前だけを import 一覧から除く。全滅した
    場合に限り、行ごと削除する (`self` はここで定義される名前ではないため、
    誤って除かれることはない)。
    """
    lines = path.read_text(encoding="utf-8").split("\n")
    changed = 0
    result = []
    for line in lines:
        match = USE_GROUP_RE.match(line)
        if not match:
            result.append(line)
            continue
        prefix, names_part, suffix = match.groups()
        names = [name.strip() for name in names_part.split(",") if name.strip()]
        survivors = [name for name in names if name not in removed_names]
        if len(survivors) == len(names):
            result.append(line)
            continue
        changed += 1
        if not survivors:
            continue
        if len(survivors) == 1:
            # `self` は `use path::{self};` のように波括弧の中でのみ許される
            # 特殊な要素であり、`use path::self;` は構文エラーになる。生存者が
            # `self` 単体の場合は、末尾の `::` ごと落として `use path;` の形に
            # 戻す必要がある (prefix は正規表現の都合で必ず `::` で終わる)。
            if survivors[0] == "self":
                result.append(f"{prefix[:-2]}{suffix}")
            else:
                result.append(f"{prefix}{survivors[0]}{suffix}")
        else:
            result.append(f"{prefix}{{{', '.join(survivors)}}}{suffix}")
    if changed:
        path.write_text("\n".join(result), encoding="utf-8")
    return changed


def find_unused_impl_lines(lines, mono_pairs):
    macro_ranges = find_macro_rules_ranges(lines)
    unused = []
    for idx, line in enumerate(lines):
        if is_within_ranges(idx, macro_ranges):
            continue
        m = IMPL_FOR_RE.match(line)
        if not m:
            continue
        trait_name, type_name = m.group(1), m.group(2)
        if trait_name.rsplit("::", 1)[-1] in NEVER_PRUNE_TRAITS:
            continue
        used = any(
            path_matches(mono_trait, trait_name) and path_matches(mono_ty, type_name)
            for mono_ty, mono_trait in mono_pairs
        )
        if not used:
            unused.append(idx + 1)
    return unused


def remove_ranges(path, ranges):
    lines = path.read_text(encoding="utf-8").split("\n")
    ranges = sorted(set(ranges), key=lambda r: -r[0])
    removed_here = 0
    for s, e in ranges:
        overlap = any((s2, e2) != (s, e) and s2 <= s and e <= e2 for s2, e2 in ranges)
        if overlap:
            continue
        del lines[s:e + 1]
        removed_here += 1
    path.write_text("\n".join(lines), encoding="utf-8")
    return removed_here


def strip_doc_comments(path):
    lines = path.read_text(encoding="utf-8").split("\n")
    kept = [line for line in lines if line.strip()[:3] not in ("///", "//!")]
    removed = len(lines) - len(kept)
    path.write_text("\n".join(kept), encoding="utf-8")
    return removed


def run_rustfmt(path):
    """バンドル後のファイルを rustfmt で整形する。

    アイテムは find_item_range/remove_ranges によって1つずつ正確な行範囲で
    削除されるが、元のソースでそのアイテムを他のアイテムと区切っていた空行
    自体は削除対象に含まれない。そのため、同じモジュール内の複数のアイテムが
    連続して削除されると、区切りだった空行だけが隙間として積み重なって残る。
    これを自前の正規表現ヒューリスティックで検出・整形するのではなく、
    Rust の構文を正しく解釈する rustfmt に委ねる。
    """
    subprocess.run(
        ["rustfmt", "--edition", "2024", str(path)],
        capture_output=True, text=True, cwd=REPO, check=True,
    )


LEADING_BLANK_IN_BLOCK_RE = re.compile(r"\{\s*$")


def strip_leading_blank_lines_in_blocks(path):
    """`{` の直後にある空行を取り除く。

    rustfmt は `}` 直前の空行は取り除くが、`}` 直後の空行はブロック冒頭の
    意図的な区切りとみなして残す仕様である (rustfmt 自体の挙動であり、本
    ツールが検出すべき範囲は「モジュール doc コメントが strip_doc_comments で
    消えた跡に残る空行」のような枝刈り由来のものに限られる)。両者を区別する
    手段がないため、`{` 直後の空行は一律で取り除く。
    """
    lines = path.read_text(encoding="utf-8").split("\n")
    kept = []
    removed = 0
    for line in lines:
        if (
            line.strip() == ""
            and kept
            and LEADING_BLANK_IN_BLOCK_RE.search(kept[-1])
        ):
            removed += 1
            continue
        kept.append(line)
    if removed:
        path.write_text("\n".join(kept), encoding="utf-8")
    return removed


TRAIT_OPEN_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?trait\s+[A-Za-z_]\w*.*\{\s*$"
)


def is_inside_trait_definition_or_impl(lines, line_idx):
    """line_idx (0-indexed) を包む直近のブロックが、トレイト定義、または
    トレイトを実装する impl のいずれかであれば True を返す。

    現在の行よりインデントが浅い直近の行を、開いているブロックの見出しとみなす
    (バンドル後のファイルは `indent()` により深さに比例した空白で揃っている)。
    """
    indent = len(lines[line_idx]) - len(lines[line_idx].lstrip())
    for i in range(line_idx - 1, -1, -1):
        line = lines[i]
        if not line.strip():
            continue
        cur_indent = len(line) - len(line.lstrip())
        if cur_indent < indent:
            if TRAIT_OPEN_RE.match(line):
                return True
            match = IMPL_LINE_RE.match(line)
            if not match:
                return False
            head = strip_leading_generic_params(match.group(1))
            trait_name, _ = impl_targets(head)
            return trait_name is not None
    return False


def prune_dead_items(path, dead_lines):
    """dead_code 診断のあった行を削除する。

    ただし、トレイト定義そのもの、およびトレイトを実装する impl 内のメソッドは
    対象から除く。トレイトのメソッドに既定の本体が無い場合、そのメソッドだけを
    個別に消すと、他のメソッド (例えば `write_to`) がまだ使われていてトレイト
    定義や impl 自体は残るときに、トレイトへの適合が壊れてコンパイルが通らなく
    なる。トレイトや impl 全体が丸ごと不要かどうかは `prune_unused_impls` の
    役割とし、ここでは個々のメソッド単位の削除に留める。
    """
    if not dead_lines:
        return 0
    lines = path.read_text(encoding="utf-8").split("\n")
    macro_ranges = find_macro_rules_ranges(lines)
    safe_lines = [
        ln
        for ln in dead_lines
        if not is_inside_trait_definition_or_impl(lines, ln - 1)
        and not is_within_ranges(ln - 1, macro_ranges)
    ]
    if not safe_lines:
        return 0
    ranges = [find_item_range(lines, ln) for ln in safe_lines]
    return remove_ranges(path, ranges)


def prune_orphans(path, original_names):
    lines = path.read_text(encoding="utf-8").split("\n")
    removed_names = original_names - collect_definitions(lines)
    if not removed_names:
        return 0
    removed = 0

    # macro_rules! 本体はテンプレートであり、通常の impl とは別枠で扱う必要が
    # あるため、参照先のトレイトが消えたマクロ定義・呼び出しを先にまとめて
    # 取り除く。この後の find_orphaned_lines は、マクロ本体の行をそもそも
    # 対象から除外しているため、ここで処理しておかないと孤立したままになる。
    macro_ranges = find_orphaned_macro_ranges(lines, removed_names)
    if macro_ranges:
        removed += remove_ranges(path, macro_ranges)
        lines = path.read_text(encoding="utf-8").split("\n")

    orphaned_lines = find_orphaned_lines(lines, removed_names)
    if orphaned_lines:
        ranges = [find_item_range(lines, ln) for ln in orphaned_lines]
        removed += remove_ranges(path, ranges)
    removed += heal_use_groups(path, removed_names)
    return removed


def prune_unused_impls(path, rel_path):
    mono_pairs = build_mono_items(rel_path)
    if mono_pairs is None:
        return 0
    lines = path.read_text(encoding="utf-8").split("\n")
    unused_lines = find_unused_impl_lines(lines, mono_pairs)
    if not unused_lines:
        return 0
    ranges = [find_item_range(lines, ln) for ln in unused_lines]
    return remove_ranges(path, ranges)


EMPTY_BLOCK_RE = re.compile(r"^(\s*)(?:pub(?:\([^)]*\))?\s+)?(mod\s+\w+|impl\s+[^{]*?)\s*\{\s*$")


def is_vacuous_line(line):
    """ブロックの中身として意味を持たない行かどうかを判定する。

    空行とコメントに加えて、`use` の宣言も中身とはみなさない。使われていた型が
    すべて消えたモジュールには `use` だけが残ることがあり、それを中身と数えると
    空の殻を畳めなくなるためである。ただし `pub use` は再エクスポートとして
    外部から参照され得るので、中身として扱う。
    """
    stripped = line.strip()
    if not stripped or is_comment_or_attr(line):
        return True
    return stripped.startswith("use ") and stripped.endswith(";")


def strip_empty_blocks(path):
    """中身が空になった `mod` と素の `impl` のブロックを取り除く。

    不要なモジュールは、内部のアイテムがすべて dead_code として消えたあと、
    `pub mod exp { impl super::FPS { } }` のような殻だけが残る。これを畳むことで、
    使わないモジュールがファイルから丸ごと消える。トレイト実装は空であっても
    `impl Copy for X {}` のように意味を持つため、`for` を含む行は対象から除く。
    """
    lines = path.read_text(encoding="utf-8").split("\n")
    removed = 0
    changed = True
    while changed:
        changed = False
        for i, line in enumerate(lines):
            match = EMPTY_BLOCK_RE.match(line)
            if not match or " for " in match.group(2):
                continue
            pad = match.group(1)
            j = i + 1
            while j < len(lines) and is_vacuous_line(lines[j]):
                j += 1
            if j >= len(lines) or lines[j] != pad + "}":
                continue
            start = i
            while start > 0 and is_comment_or_attr(lines[start - 1]):
                start -= 1
            del lines[start:j + 1]
            removed += 1
            changed = True
            break
    path.write_text("\n".join(lines), encoding="utf-8")
    return removed


def prune(out_path, bin_name):
    """バンドル結果から到達しないコードを、コンパイルが通らなくなるまで削る。

    枝刈りは診断を頼りにした発見的な処理であるため、削りすぎてコンパイルが
    通らなくなる可能性がある。そこで各ラウンドの開始時にビルドの成否を確認し、
    失敗していれば直前の正常な状態へ巻き戻して打ち切る。
    """
    path = Path(out_path)
    if not path.is_absolute():
        path = REPO / path
    rel_path = str(path.relative_to(REPO))

    # 何が消えたのかを判定する基準として、枝刈り前の定義一覧を控えておく。
    original_names = collect_definitions(path.read_text(encoding="utf-8").split("\n"))

    total_removed = 0
    snapshot = None
    for iteration in range(MAX_PRUNE_ITERATIONS):
        compiled, dead_lines = build_diagnostics(rel_path, bin_name)
        if not compiled:
            if snapshot is None:
                raise RuntimeError(f"{rel_path}: 生成した直後の時点でコンパイルが通らない")
            path.write_text(snapshot, encoding="utf-8")
            print(f"  iteration {iteration}: build broke; rolled back to the previous round")
            break
        # 次のラウンドが壊した場合に備え、コンパイルが通る状態を控えておく。
        snapshot = path.read_text(encoding="utf-8")

        removed = prune_dead_items(path, dead_lines)
        # 空になった mod ブロックは、この時点ではまだ「モジュール名を持つ殻」として
        # 残っている。prune_orphans がモジュール名の消滅を検知できるよう、
        # 殻を畳んでから孤立 import の検出を行う。
        removed += strip_empty_blocks(path)
        removed += prune_orphans(path, original_names)
        removed += prune_unused_impls(path, rel_path)
        removed += strip_empty_blocks(path)
        total_removed += removed
        print(f"  iteration {iteration}: removed {removed} items")
        if removed == 0:
            break
    else:
        print("  stopped after max iterations")

    doc_lines_removed = strip_doc_comments(path)
    run_rustfmt(path)
    leading_blank_lines_removed = strip_leading_blank_lines_in_blocks(path)
    compiled, _ = build_diagnostics(rel_path, bin_name)
    if not compiled:
        raise RuntimeError(f"{rel_path}: 枝刈り後のファイルがコンパイルできない")

    print(f"  stripped {doc_lines_removed} doc-comment lines (///, //!)")
    print(f"  removed {leading_blank_lines_removed} leading blank lines left by pruning")
    print(f"  total items removed: {total_removed}")
    print(f"  {len(path.read_text(encoding='utf-8').splitlines())} lines remain")


# =============================================================================
# CLI
# =============================================================================


def discover_slugs():
    slugs = set()
    for directory in SRC_DIRS.values():
        for path in directory.glob("*.rs"):
            slugs.add(path.stem)
    return sorted(slugs)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("slugs", nargs="*", help="バンドルする問題のスラッグ")
    parser.add_argument("--all", action="store_true", help="src/bin 以下の全問題をバンドルする")
    parser.add_argument(
        "--prune", nargs=2, metavar=("PATH", "BIN_NAME"),
        help="既存ファイルの枝刈りのみ行う (手動でバンドルしたファイル向け)",
    )
    args = parser.parse_args()

    if args.prune:
        path, bin_name = args.prune
        print(f"### prune-only: {path} ({bin_name}) ###")
        prune(path, bin_name)
        return

    targets = discover_slugs() if args.all else args.slugs
    if not targets:
        parser.error("バンドルする問題のスラッグ、--all、--prune のいずれかを指定すること")

    # モジュールツリーの展開結果は問題によらず共通であるため、一度だけ構築する。
    core = build_core()

    failures = []
    for slug in targets:
        print(f"### {slug} ###")
        try:
            out_path, bin_name = generate(slug, core)
            prune(out_path, bin_name)
        except (RuntimeError, FileNotFoundError) as error:
            print(f"  failed: {error}")
            failures.append(slug)
        print()

    if failures:
        print(f"failed: {', '.join(failures)}")
        sys.exit(1)


if __name__ == "__main__":
    main()
