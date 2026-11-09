#!/usr/bin/env python3
"""Generate src/check/std_inherent.txt: the items std gives each std type.

Rule zero needs it (src/check/std_inherent.rs, L0643): rustc picks a std
type's inherent item before a trait item of the same name at the same
probe, and a std trait's item next to a user trait's item of the same name
is ambiguous. So lust must never pick a user trait's item where std has an
item of that name that wins or ties.

Input is the rust-docs component of the pinned toolchain (rustdoc HTML of
std). Usage:

    curl -O https://static.rust-lang.org/dist/rust-docs-1.99.0-x86_64-unknown-linux-gnu.tar.xz
    tar -xJf rust-docs-1.99.0-x86_64-unknown-linux-gnu.tar.xz --wildcards '*/html/std/*'
    python3 scripts/gen-std-inherent.py rust-docs-1.99.0-*/rust-docs/share/doc/rust/html/std > src/check/std_inherent.txt

Each output line is `<type name> <item> <item> ...`: the simple name of a
std struct, enum, or union (`Vec`, `HashMap`, `Iter`), or a primitive page
name (`u8`, `str`, `slice`, `array`, `pointer`, `reference`), then the
items std gives it, sorted. An item is one form letter and the item name.
Lowercase letters are inherent items, uppercase ones are items of a std
trait the page lists an impl of (trait, auto, and blanket impls; the
trait's provided methods too). For `reference` and `pointer`, the impls
come from the implementor lists of the trait pages (`Ord for &A`). The letter is the receiver: `v`/`V` self,
`r`/`R` &self, `m`/`M` &mut self, `t`/`T` a typed self (`self: Box<Self>`),
`-`/`_` none (an associated fn or const, reachable only by a path). One
name can have several items. Uppercase items are the 2024 prelude traits'
only; a type's other std traits are `@std::io::Read` tokens, and each of
those traits has a line `#std::io::Read <item> ...` at the end, since
rustc only probes a trait that is in scope.

Every set is a union: types that share a simple name (the many `Iter`
structs) share one line, which only rejects more, never less. Traits are
matched by full path (`String` gets `std::fmt::Write`, not `std::io::Write`).
Unstable items are left out: rustc's probe skips an unstable candidate when
a stable one (a user trait item) applies. Items marked doc(hidden) are not
in the docs and so are not in the table.
"""
import html as htmllib
import os
import re
import sys

ITEM = re.compile(r'<section id="(tymethod|method|associatedconstant)\.([A-Za-z0-9_]+)"')
# Sections that end the inherent `Implementations` list of a type page.
TYPE_END = re.compile(r'<h2 id="(deref-methods|trait-implementations|synthetic-implementations|blanket-implementations)')
# Sections that end the item list of a trait page.
TRAIT_END = re.compile(r'<h2 id="(dyn-compatibility|implementors|foreign-impls|object-safety)')
# A trait impl's section id (`impl-From%3C%26str%3E-for-String` is `From`)
# and its header, whose trait links carry full paths.
IMPL = re.compile(r'<section id="impl-([A-Za-z0-9_]+)(?:%3C[^"]*?)?-for-[^"]*"[^>]*>.*?<h3 class="code-header">(.*?)</h3>', re.S)
TRAIT_LINK = re.compile(r'class="trait" href="[^"]*" title="trait ([A-Za-z0-9_:]+)"')
# An implementor on a trait page whose Self is a reference or a raw pointer
# (`impl-Ord-for-%26A`): rustdoc lists those there, not on the primitive page.
PTR_IMPLEMENTOR = re.compile(r'<section id="impl-[^"]*?-for-(%26|%2A)')
TAG = re.compile(r'<[^>]*>')
# The traits of the 2024 prelude, always in scope; any other std trait
# counts only where the program may import it (`@path` tokens).
PRELUDE = {'std::' + p for p in (
    'marker::Copy marker::Send marker::Sized marker::Sync marker::Unpin ops::Drop ops::Fn ops::FnMut ops::FnOnce '
    'ops::AsyncFn ops::AsyncFnMut ops::AsyncFnOnce borrow::ToOwned clone::Clone cmp::PartialEq cmp::PartialOrd cmp::Eq '
    'cmp::Ord convert::AsRef convert::AsMut convert::Into convert::From convert::TryFrom convert::TryInto '
    'default::Default iter::Iterator iter::Extend iter::IntoIterator iter::DoubleEndedIterator '
    'iter::ExactSizeIterator iter::FromIterator string::ToString future::Future future::IntoFuture').split()}
RECEIVER = re.compile(r"\(\s*(?:mut\s+)?(&\s*(?:'\w+\s+)?(mut\s+)?)?self\b(\s*:)?")


def form(sig):
    """The receiver letter of one item header (inherent case)."""
    m = RECEIVER.search(sig) if 'fn ' in sig else None
    if m is None:
        return '-'
    if m.group(3):
        return 't'
    if m.group(1):
        return 'm' if m.group(2) else 'r'
    return 'v'


def items(body, trait):
    """The stable items in one page region, as form letter + name."""
    out = set()
    matches = list(ITEM.finditer(body))
    for i, m in enumerate(matches):
        # The item's header and stability badge come before its docblock.
        stop = matches[i + 1].start() if i + 1 < len(matches) else len(body)
        doc = body.find('class="docblock"', m.end(), stop)
        head = body[m.end() : doc if doc >= 0 else stop]
        if 'class="stab unstable"' in head:
            continue
        f = form(htmllib.unescape(TAG.sub('', head)))
        if trait:
            f = '_' if f == '-' else f.upper()
        out.add(f + m.group(2))
    return out


def impl_traits(html):
    """Full paths of the traits a type page lists impls of. A header can
    also link bound traits; only those named like the impl's trait count."""
    out = set()
    for m in IMPL.finditer(html):
        out |= {t for t in TRAIT_LINK.findall(m.group(2)) if t.rsplit('::', 1)[-1] == m.group(1)}
    return out


def main():
    root = sys.argv[1].rstrip('/')
    inherent = {}
    impls = {}
    traits = {}
    for dirpath, _, files in os.walk(root):
        for f in files:
            m = re.fullmatch(r'(struct|enum|union|primitive|trait)\.([A-Za-z0-9_]+)\.html', f)
            if not m:
                continue
            html = open(os.path.join(dirpath, f), encoding='utf-8').read()
            if 'http-equiv="refresh"' in html[:2000]:
                continue
            name = m.group(2)
            if m.group(1) == 'trait':
                body = html[html.find('<section id="main-content"') :]
                end = TRAIT_END.search(body)
                path = os.path.relpath(dirpath, os.path.dirname(root)).replace(os.sep, '::') + '::' + name
                traits.setdefault(path, set()).update(items(body[: end.start() if end else len(body)], True))
                for m2 in PTR_IMPLEMENTOR.finditer(body):
                    impls.setdefault('reference' if m2.group(1) == '%26' else 'pointer', set()).add(path)
            else:
                start = html.find('<h2 id="implementations"')
                body = ''
                if start >= 0:
                    end = TYPE_END.search(html, start)
                    body = html[start : end.start() if end else len(html)]
                inherent.setdefault(name, set()).update(items(body, False))
                impls.setdefault(name, set()).update(impl_traits(html))
    key = lambda s: (s[1:], s[0])
    used = set()
    for ty in sorted(inherent):
        names = set(inherent[ty])
        others = set()
        for tr in impls.get(ty, ()):
            if tr in PRELUDE:
                names |= traits.get(tr, set())
            elif traits.get(tr):
                others.add('@' + tr)
        used |= others
        if names or others:
            print(ty, ' '.join(sorted(names, key=key) + sorted(others)))
    for tr in sorted(used):
        print('#' + tr[1:], ' '.join(sorted(traits[tr[1:]], key=key)))


if __name__ == '__main__':
    main()
