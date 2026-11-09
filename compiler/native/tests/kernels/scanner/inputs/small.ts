// Small scanner fixture: every literal form, comments, directives.
// @ts-ignore: deliberate
/* block
   comment */
export const nums = [0, 42, 3.14, 1e10, 1.5e-7, 0xDEADbeef, 0o777, 0b1010, 1_000_000, 9007199254740993n, 0xFFn, .5, 5.];
export const strs = ["double \"esc\" \n \u{1F600}", 'single \'q\'', `tpl ${nums[0]} mid ${"x"} end`, `no-sub`];
export const re = /ab+c/gi;
export const re2 = /(?<year>\d{4})-\d{2}/u;
const café = "unicode id";
class C<T extends object> { #priv = 1; static { } m() { return this.#priv?.toString() ?? "x"; } }
function* gen() { yield* [1, 2]; }
async function f() { await Promise.resolve(); }
const jsxLike = a < b > c;
const arrow = (x: number): number => x ** 2;
let v = 1; v &&= 2; v ||= 3; v ??= 4; v++; --v; v >>>= 1;
const tagged = String.raw`raw\path`;
