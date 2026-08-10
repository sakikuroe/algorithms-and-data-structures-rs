#!/usr/bin/env python3
"""
anmitsu を path 依存で参照している cargo プロジェクトの解答を単一ファイルへ
バンドルし、続けて不要コードを枝刈りするスクリプトである。

使い方:
    bundler                     # 対象を全件バンドル
    bundler a                   # ファイル名で指定
    bundler src/bin/a.rs        # 実行位置からの相対パスで指定
    bundler --strip-docs        # ドキュメントコメントも取り除く

対象の指定には、src/bin からの相対パス、拡張子を除いたファイル名、実行位置からの
相対パス、絶対パスのいずれも使える。

基準となるのは、このスクリプトの置き場所ではなく実行時のカレントディレクトリーで
ある。そのため、このライブラリーのリポジトリーでも、anmitsu を参照している
コンテスト用のプロジェクトでも、同じスクリプトをそのまま使うことができる。

展開する anmitsu の所在は、実行位置の Cargo.toml の [dependencies] に書かれた
path から解決する。実行位置自身が anmitsu である場合は、実行位置をそのまま用いる。

バンドルの対象は、cargo が認識しているバイナリーターゲット、すなわち自動認識される
src/bin/*.rs と src/bin/*/main.rs、および [[bin]] で登録されたものである。出力は
src/bin/bundled/ の下へ、src/bin からの相対位置を保って書き出す。たとえば
src/bin/library_checker/cycle_detection.rs は
src/bin/bundled/library_checker/cycle_detection.rs となる。src/bin の外を指す
[[bin]] は bundled/ の下での置き場所を決められないため対象から除き、その旨を表示する。

ソースが anmitsu を参照していない場合は、ライブラリーを展開せずそのまま複写する。
展開しても枝刈りですべて削られるだけであり、提出用のファイルが常に bundled/ の下に
揃っていればよいためである。この場合は注記も付けず、元のファイルと同じ内容にする。

展開したライブラリーはファイルの末尾に置き、その直前に出所とライセンスの注記を
添える。読み手が数百行のライブラリーをまたがずに解答へ辿り着けるようにするため
であり、Rust ではアイテムの記述順が問われないため本文の use はそのまま解決できる。
ソースの側には手を加えないため、生成物の先頭は元のファイルと同じ内容になる。

バンドルは、まず src/lib.rs のモジュールツリーを `mod anmitsu { ... }` として単一
ファイルへ展開し、そのうえでコンパイラの診断を頼りに到達しないコードを削る、という
手順で行う。必要なモジュールの判断をコンパイラに委ねているため、`log` が `inverse`
を呼ぶといった use 文に現れない依存関係や、`*` 演算子から Mul の実装に到達する
依存関係も取りこぼさない。

`mod anmitsu` にまとめるのは、ソースの `use anmitsu::...` をそのまま残すためである。
同名のローカルなモジュールと外部クレートがある場合、その名前で始まるパスはローカル
のほうを指すと Rust の仕様で定められているため、利用側の Cargo.toml に anmitsu への
依存が残っていても競合しない。

枝刈りが働く前提として、この `mod anmitsu` には `pub` を付けずに出力している。
`pub mod` にすると rustc が全アイテムを外部から到達可能とみなし、dead_code を
ひとつも報告しなくなるためである。

コンパイラの診断を得るには生成物をビルドする必要があるが、そのために利用側の
Cargo.toml へ [[bin]] を登録させるのは煩わしい。そこで一時ディレクトリーに枝刈り
専用の cargo パッケージを用意し、生成物をその src/main.rs としてビルドする。この
パッケージには実行位置の edition と、anmitsu を除いた [dependencies] を写す。
利用側の Cargo.toml は読むだけで、書き換えない。

main() 本体はこのスクリプトに複製せず、実行の都度ソースから読み込むため、実装を
変更しても再バンドルするだけで常に最新の内容が反映される。
"""

import argparse
import dataclasses
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

# バンドルの基準となる実行位置、展開する anmitsu の所在、枝刈り用の作業場、および
# 実行位置の edition。いずれも実行時にしか決まらないため、configure() が設定する。
PROJECT = None
LIBRARY = None
WORKDIR = None
EDITION = None

# 出力をまとめるディレクトリーの名前。src/bin からの相対位置をこの下に再現する。
BUNDLED_DIR_NAME = "bundled"

# 枝刈り用の作業場で、生成物を置く位置。cargo が自動認識するため登録は要らない。
PROBE_REL_PATH = "src/main.rs"

# 枝刈りを繰り返す上限。トレイト実装の除去が新たな dead_code を生むため、
# 何も削れなくなるまで数回の往復が必要になる。
MAX_PRUNE_ITERATIONS = 30

# 展開したライブラリーの直前に置く注記。ジャッジで人が読む可能性を考えて英語とし、
# 出所とライセンスを示す。直後の `mod anmitsu` だけを指す位置に置いているため、
# どこからがライブラリーなのかを別途断る必要がない。複写しただけのファイルには
# バンドルするものがないため、何も付けない。
BUNDLE_NOTICE = (
    "// The following is anmitsu (CC0-1.0), a Rust library for competitive\n"
    "// programming, bundled into this file with unused items removed.\n"
    "// https://github.com/sakikuroe/algorithms-and-data-structures-rs"
)


# =============================================================================
# 実行位置の解決と枝刈り用の作業場
# =============================================================================


def read_manifest(directory):
    path = directory / "Cargo.toml"
    if not path.exists():
        raise RuntimeError(
            f"{directory} に Cargo.toml がない。cargo プロジェクトの直下で実行すること"
        )
    return tomllib.loads(path.read_text(encoding="utf-8"))


def resolve_library(project_dir, manifest):
    """展開する anmitsu の所在を、実行位置の Cargo.toml から解決する。

    実行位置が anmitsu 自身である場合、自分自身への依存は書かれていないため、
    パッケージ名を手がかりに実行位置そのものを所在とみなす。
    """
    if manifest.get("package", {}).get("name") == "anmitsu":
        return project_dir
    spec = manifest.get("dependencies", {}).get("anmitsu")
    if not isinstance(spec, dict) or "path" not in spec:
        raise RuntimeError(
            "Cargo.toml の [dependencies] に anmitsu の path 指定が見つからない。"
            'anmitsu = { path = "..." } を追加すること'
        )
    # path は相対で書かれることがあるため、Cargo.toml のある位置を起点に解決する。
    return (project_dir / spec["path"]).resolve()


def resolve_edition(manifest):
    edition = manifest.get("package", {}).get("edition")
    if isinstance(edition, str):
        return edition
    # cargo 自身の既定は 2015 だが、それを仮定すると現代的なコードはまず通らない。
    # 黙って選ぶと原因が分かりにくいため、仮定したことを表示する。
    print("note: Cargo.toml に edition の指定がないため 2024 として扱う")
    return "2024"


def render_toml_value(value):
    """依存の指定を、作業場の Cargo.toml へ書き戻せる形の TOML 値にする。"""
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return str(value)
    if isinstance(value, str):
        # TOML の基本文字列は JSON の文字列と同じ書式であるため、そのまま使える。
        return json.dumps(value)
    if isinstance(value, list):
        return "[" + ", ".join(render_toml_value(item) for item in value) + "]"
    if isinstance(value, dict):
        body = ", ".join(f"{key} = {render_toml_value(item)}" for key, item in value.items())
        return "{ " + body + " }"
    raise RuntimeError(f"依存の指定に扱えない値が含まれている: {value!r}")


def render_dependencies(project_dir, manifest):
    """実行位置の [dependencies] を、anmitsu を除いて書き写す。

    anmitsu を除くのは、その中身が生成物へ展開済みだからである。他の path 依存は
    作業場から見た位置が変わってしまうため、絶対パスへ直したうえで写す。
    """
    lines = ["[dependencies]"]
    for name, spec in manifest.get("dependencies", {}).items():
        if name == "anmitsu":
            continue
        if isinstance(spec, dict) and "path" in spec:
            spec = dict(spec)
            spec["path"] = str((project_dir / spec["path"]).resolve())
        lines.append(f"{name} = {render_toml_value(spec)}")
    return "\n".join(lines) + "\n"


def workspace_for(project_dir):
    """作業場の位置を、実行位置ごとに分けて決める。

    1 箇所に固定すると、依存の異なるプロジェクトを行き来するたびに Cargo.toml が
    書き換わり、そのつど依存を再コンパイルすることになる。パスから作った短い
    ハッシュを添えることで、名前が同じ別のプロジェクトどうしも衝突しない。
    """
    digest = hashlib.sha1(str(project_dir).encode("utf-8")).hexdigest()[:12]
    return Path(tempfile.gettempdir()) / "bundle-py" / f"{project_dir.name}-{digest}"


def ensure_workspace(project_dir, manifest, edition):
    """枝刈り用の cargo パッケージを用意し、その位置を返す。"""
    workdir = workspace_for(project_dir)
    (workdir / "src").mkdir(parents=True, exist_ok=True)

    content = (
        "# tools/bundle.py が枝刈り用に生成した作業場である。手で編集しても、\n"
        "# 次回の実行で上書きされる。\n"
        "[package]\n"
        'name = "bundle-probe"\n'
        'version = "0.1.0"\n'
        f'edition = "{edition}"\n'
        "\n" + render_dependencies(project_dir, manifest)
    )
    manifest_path = workdir / "Cargo.toml"
    # 内容が同じときに書き換えると、mtime の変化だけで cargo が依存を再コンパイル
    # してしまう。変わったときにだけ書く。
    if not manifest_path.exists() or manifest_path.read_text(encoding="utf-8") != content:
        manifest_path.write_text(content, encoding="utf-8")

    # 依存のバージョンを実行位置に揃えるため、初回だけ Cargo.lock を写す。以降は
    # 作業場のものを cargo が保守するので、上書きすると解決をやり直させてしまう。
    # なお、作業場の依存は実行位置から anmitsu を除いたものであり、写した
    # Cargo.lock がそのまま使えるとは限らない。cargo が不足分を取得できるよう、
    # ビルドには --offline を付けていない。
    lock = project_dir / "Cargo.lock"
    if lock.exists() and not (workdir / "Cargo.lock").exists():
        shutil.copyfile(lock, workdir / "Cargo.lock")

    return workdir


def sync_to_workspace(path, force=False):
    """生成物を作業場へ写す。

    force を指定すると、内容が同じでも書き直して mtime を更新する。mono-items の
    採取は cargo が実際に再コンパイルしたときにしか出力されないため、そちらの
    呼び出しでは常に再コンパイルさせる必要がある。
    """
    target = WORKDIR / PROBE_REL_PATH
    content = path.read_text(encoding="utf-8")
    if force or not target.exists() or target.read_text(encoding="utf-8") != content:
        target.write_text(content, encoding="utf-8")


def configure(project_dir):
    global PROJECT, LIBRARY, WORKDIR, EDITION

    PROJECT = project_dir.resolve()
    manifest = read_manifest(PROJECT)
    LIBRARY = resolve_library(PROJECT, manifest)
    if not (LIBRARY / "src" / "lib.rs").exists():
        raise RuntimeError(f"anmitsu の所在として解決した {LIBRARY} に src/lib.rs がない")
    EDITION = resolve_edition(manifest)
    WORKDIR = ensure_workspace(PROJECT, manifest, EDITION)

    print(f"project:   {PROJECT}")
    print(f"library:   {LIBRARY}")
    print(f"workspace: {WORKDIR}")
    print()


# =============================================================================
# 対象の探索
# =============================================================================


def cargo_metadata(project_dir):
    proc = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        capture_output=True, text=True, cwd=project_dir,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"cargo metadata に失敗した:\n{proc.stderr.strip()}")
    return json.loads(proc.stdout)


def discover_targets(project_dir):
    """バンドルすべきバイナリーターゲットと、対象外にしたものを返す。

    対象は cargo が認識しているバイナリーであり、自動認識される src/bin/*.rs と
    src/bin/*/main.rs に加え、[[bin]] で登録されたものも含まれる。ただし出力先で
    ある bundled/ の下は、生成物を再びバンドルしてしまうため除く。
    """
    metadata = cargo_metadata(project_dir)
    manifest_path = (project_dir / "Cargo.toml").resolve()
    bin_dir = project_dir / "src" / "bin"

    targets = []
    skipped = []
    for package in metadata["packages"]:
        if Path(package["manifest_path"]).resolve() != manifest_path:
            continue
        for target in package["targets"]:
            if "bin" not in target["kind"]:
                continue
            src_path = Path(target["src_path"]).resolve()
            try:
                relative = src_path.relative_to(bin_dir)
            except ValueError:
                # src/bin の外にあるものは、bundled/ の下での置き場所を決められない。
                skipped.append((target["name"], src_path))
                continue
            if relative.parts[0] == BUNDLED_DIR_NAME:
                continue
            targets.append((relative, src_path))
    return sorted(targets), skipped


def output_path_for(relative):
    return PROJECT / "src" / "bin" / BUNDLED_DIR_NAME / relative


def matches_pattern(relative, src_path, pattern):
    """対象の指定として使える書き方を、まとめて受け付ける。

    src/bin からの相対パス (`library_checker/cycle_detection.rs`)、拡張子を除いた
    同じ形、ファイル名のみ (`cycle_detection`) に加えて、シェルの補完でそのまま
    入力できる実行位置からの相対パス (`src/bin/a.rs`) と絶対パスも認める。
    """
    posix = relative.as_posix()
    from_project = src_path.relative_to(PROJECT).as_posix()
    candidates = {
        posix,
        posix.removesuffix(".rs"),
        relative.stem,
        from_project,
        from_project.removesuffix(".rs"),
    }
    if pattern in candidates:
        return True
    # `./src/bin/a.rs` のような表記や絶対パスは、解決してから突き合わせる。
    if "/" in pattern:
        try:
            return Path(pattern).resolve() == src_path
        except OSError:
            return False
    return False


def select_targets(targets, patterns):
    if not patterns:
        return targets
    selected = []
    for pattern in patterns:
        matches = [entry for entry in targets if matches_pattern(*entry, pattern)]
        if not matches:
            raise RuntimeError(f"{pattern}: 対象が見つからない")
        if len(matches) > 1:
            # ファイル名だけでは絞り込めない場合、黙って 1 つ選ぶと意図しない
            # ファイルを処理しかねないため、候補を示して止まる。
            candidates = "\n".join(f"  {relative.as_posix()}" for relative, _ in matches)
            raise RuntimeError(
                f"{pattern}: 複数の対象に一致する。相対パスで指定すること\n{candidates}"
            )
        selected.append(matches[0])
    return selected


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
    lines = strip_cfg_test_mod((LIBRARY / "src/lib.rs").read_text(encoding="utf-8")).split("\n")

    root = []
    children = root
    directory = LIBRARY / "src"
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
    """モジュールを入れ子の `pub mod` ブロックとして出力する。

    どの階層にも `pub` を付けてよいのは、全体を包む `mod anmitsu` 自体が private
    だからである ([`build_core`] を参照)。
    """
    keyword = "pub mod"
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
    """展開したライブラリー全体を `mod anmitsu { ... }` にまとめる。

    こうしておくと、ソースの `use anmitsu::ds::union_find::UnionFind;` をそのまま
    残せる。同名のローカルなモジュールと外部クレートがある場合、その名前で始まる
    パスはローカルのほうを指すと Rust の仕様で定められているため、利用側の
    Cargo.toml に anmitsu への依存が残っていても競合しない。

    包む側に `pub` を付けないのは、`pub mod` にすると rustc が全アイテムを外部から
    到達可能とみなし、dead_code をひとつも報告しなくなるためである。内側は `pub`
    のままでよく、外側が private であれば実効可視性は抑えられる。
    """
    body = "\n\n".join(render_module(module, 1) for module in discover_module_tree())
    # 1 段深くなるぶん、ライブラリー内の `crate::` 起点のパスがずれる。展開した
    # 部分にしか手を入れないため、解答側が書いた `crate::` には影響しない。
    body = body.replace("crate::", "crate::anmitsu::")
    return "mod anmitsu {\n" + body + "\n}"


def compose(source, core):
    """バンドルした生成物の中身を組み立てる。

    ソースをそのまま置き、その後ろに注記と展開したライブラリーを続ける。
    ライブラリーを末尾へ回すのは、ファイルを開いた読み手が数百行の `mod anmitsu`
    をまたがずに解答へ辿り着けるようにするためである。Rust ではアイテムの記述順は
    問われないため、本文の `use anmitsu::...` は後ろにあるモジュールを問題なく
    参照できる。

    先頭には何も足さない。注記は `mod anmitsu` の直前にあれば足り、そこへ置く
    ことで指す対象が直後のブロックに限られるためである。先頭に置くと、続く解答
    そのものがライブラリーであるかのようにも読めてしまう。
    """
    return source.strip("\n") + "\n\n" + BUNDLE_NOTICE + "\n" + core + "\n"


def write_output(out_path, content):
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(content, encoding="utf-8")
    print(
        f"generated {out_path.relative_to(PROJECT)} ({len(content.splitlines())} lines)"
    )


def generate(source, out_path, core):
    write_output(out_path, compose(source, core))


def copy_source(source, out_path):
    """anmitsu を使っていないソースを、手を加えずそのまま出力する。

    バンドルするものが何もないため、注記も付けない。元のファイルと内容が一致して
    いるほうが、提出前に見比べたときに分かりやすい。
    """
    write_output(out_path, source)


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


def build_diagnostics(path):
    """バンドル結果をビルドし、コンパイルの可否と dead_code の行番号を返す。

    ビルドは、生成物を写した枝刈り用の作業場で行う。利用側の Cargo.toml へ
    [[bin]] を登録しなくても診断を得られるようにするための仕組みである。

    cargo が JSON で返す `file_name` はパッケージ相対パスであるため、突き合わせる
    側も相対パスでなければならない。絶対パスを渡すと一致せず、dead_code による
    枝刈りが黙って無効になる。
    """
    sync_to_workspace(path)
    proc = subprocess.run(
        ["cargo", "build", "--message-format=json"],
        capture_output=True, text=True, cwd=WORKDIR,
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
            if span.get("is_primary") and span.get("file_name") == PROBE_REL_PATH:
                dead_lines.append(span["line_start"])
    return proc.returncode == 0, sorted(set(dead_lines))


def build_mono_items(path):
    # nightly の -Z print-mono-items=yes は、実際に単相化された (=生成される) アイテムを
    # 標準出力へ書き出す。これにより、dead_code lint では検出できない「trait 実装は
    # あるが、その型では一度も呼ばれていない」ケースを 1 回のビルドで判定できる。
    #
    # 素の rustc ではなく作業場の cargo を通すのは、外部クレートを解決させるため
    # である。proconio のようなジャッジ側に用意されているクレートを使う解答では、
    # rustc を直接呼ぶとその解決に失敗し、トレイト実装の枝刈りが常に見送られる。
    #
    # ファイルが (この関数を呼ぶ前の段階で) コンパイルエラーを起こしている場合、
    # 単相化の収集が最後まで走らず標準出力が不完全になる。その不完全な結果を
    # 「使われていない」と誤判定して usable な impl まで消してしまうと危険なので、
    # コンパイルが失敗した場合は None を返し、呼び出し側で今回の枝刈りを見送る。
    sync_to_workspace(path, force=True)
    proc = subprocess.run(
        [
            "cargo", "+nightly", "rustc", "--release", "--",
            "-Z", "print-mono-items=yes",
        ],
        capture_output=True, text=True, cwd=WORKDIR,
    )
    if proc.returncode != 0:
        print("  (mono-items probe failed to compile; skipping this round)")
        return None
    if "MONO_ITEM" not in proc.stdout:
        # cargo が再コンパイルを省略すると、ビルドは成功したまま出力だけが空になる。
        # 空の結果は「どの実装も使われていない」と読めてしまい、そのまま進めると
        # 使用中の実装まで消してしまうため、収穫のなかった回として見送る。
        print("  (mono-items probe produced no output; skipping this round)")
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
        ["rustfmt", "--edition", EDITION, str(path)],
        capture_output=True, text=True, cwd=PROJECT, check=True,
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


def prune_unused_impls(path):
    mono_pairs = build_mono_items(path)
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


def prune_unused_impls_safely(path):
    """トレイト実装の除去を試み、コンパイルが通らなくなった場合は取り消す。

    単相化されているかどうかの判定は、型名の末尾どうしを比べる発見的なもので
    あるため、まだ必要な実装を消してしまうことがある。ラウンド全体の巻き戻しに
    任せると、同じラウンドで得られた dead_code の枝刈りまで道連れになるので、
    この手順だけを個別に検証して切り分ける。
    """
    snapshot = path.read_text(encoding="utf-8")
    removed = prune_unused_impls(path)
    removed += strip_empty_blocks(path)
    if removed == 0:
        return 0
    compiled, _ = build_diagnostics(path)
    if not compiled:
        path.write_text(snapshot, encoding="utf-8")
        print("    unused-impl pruning broke the build; reverted this step")
        return 0
    return removed


def prune(path, strip_docs=False):
    """バンドル結果から到達しないコードを、コンパイルが通らなくなるまで削る。

    枝刈りは診断を頼りにした発見的な処理であるため、削りすぎてコンパイルが
    通らなくなる可能性がある。そこで各ラウンドの開始時にビルドの成否を確認し、
    失敗していれば直前の正常な状態へ巻き戻して打ち切る。

    strip_docs を指定すると、仕上げにドキュメントコメントも取り除く。既定で
    残しているのは、提出物を読み返すときに各アイテムの説明があるほうが助かる
    ためである。取り除くと分量はおよそ半分になる。
    """
    label = path.relative_to(PROJECT)

    # 何が消えたのかを判定する基準として、枝刈り前の定義一覧を控えておく。
    original_names = collect_definitions(path.read_text(encoding="utf-8").split("\n"))

    total_removed = 0
    snapshot = None
    for iteration in range(MAX_PRUNE_ITERATIONS):
        compiled, dead_lines = build_diagnostics(path)
        if not compiled:
            if snapshot is None:
                raise RuntimeError(f"{label}: 生成した直後の時点でコンパイルが通らない")
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

        # トレイト実装の除去へ進むのは、dead_code による枝刈りが収束してからに
        # する。単相化されていない実装であっても、それを参照する関数がまだ残って
        # いる間に消すと、参照先を失ってコンパイルが通らなくなるためである
        # (`pow` が未使用のまま残っている状態で `*=` の実装を消す、など)。
        # 実装を除去すると新たな dead_code が生じるため、次のラウンドで再び
        # dead_code の除去に戻り、双方が何も削れなくなるまで往復する。
        if removed == 0:
            removed += prune_unused_impls_safely(path)
        total_removed += removed
        print(f"  iteration {iteration}: removed {removed} items")
        if removed == 0:
            break
    else:
        print("  stopped after max iterations")

    doc_lines_removed = strip_doc_comments(path) if strip_docs else 0
    run_rustfmt(path)
    leading_blank_lines_removed = strip_leading_blank_lines_in_blocks(path)
    compiled, _ = build_diagnostics(path)
    if not compiled:
        raise RuntimeError(f"{label}: 枝刈り後のファイルがコンパイルできない")

    if strip_docs:
        print(f"  stripped {doc_lines_removed} doc-comment lines (///, //!)")
    print(f"  removed {leading_blank_lines_removed} leading blank lines left by pruning")
    print(f"  total items removed: {total_removed}")
    print(f"  {len(path.read_text(encoding='utf-8').splitlines())} lines remain")


# =============================================================================
# CLI
# =============================================================================


def main():
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "targets", nargs="*",
        help="バンドルする対象。src/bin からの相対パス、または拡張子を除いた"
             "ファイル名で指定する。省略すると対象を全件バンドルする",
    )
    parser.add_argument(
        "--strip-docs", action="store_true",
        help="生成物からドキュメントコメント (///, //!) を取り除く。分量はおよそ"
             "半分になるが、各アイテムの説明は失われる",
    )
    args = parser.parse_args()

    try:
        configure(Path.cwd())
        targets, skipped = discover_targets(PROJECT)
        for name, src_path in skipped:
            print(f"skipped {name}: {src_path} は src/bin の外にあるため対象にできない")
        if skipped:
            print()
        selected = select_targets(targets, args.targets)
    except (RuntimeError, FileNotFoundError, json.JSONDecodeError) as error:
        print(f"error: {error}")
        sys.exit(1)

    if not selected:
        print("バンドルする対象がない")
        return

    # モジュールツリーの展開結果は対象によらず共通であるため、実際に展開が必要に
    # なった時点で一度だけ構築する。複写だけで済む場合は構築しない。
    core = None

    failures = []
    for relative, src_path in selected:
        print(f"### {relative.as_posix()} ###")
        try:
            source = src_path.read_text(encoding="utf-8")
            out_path = output_path_for(relative)
            if "anmitsu" not in source:
                # ライブラリーを参照していないため、展開しても枝刈りですべて
                # 削られるだけである。提出用のファイルが bundled/ の下に揃うよう、
                # 複写だけを行う。
                copy_source(source, out_path)
            else:
                if core is None:
                    core = build_core()
                generate(source, out_path, core)
                prune(out_path, strip_docs=args.strip_docs)
        except (RuntimeError, FileNotFoundError, subprocess.CalledProcessError) as error:
            print(f"  failed: {error}")
            failures.append(relative.as_posix())
        print()

    if failures:
        print(f"failed: {', '.join(failures)}")
        sys.exit(1)


if __name__ == "__main__":
    main()
