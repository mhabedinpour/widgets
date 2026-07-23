// jsonlite — minimal hand-rolled JSON field extraction. No libraries.
//
// Instead of building a parse tree, these helpers scan the raw response
// string for '"key":' patterns and read the value in place. Callers scope
// a search to a sub-object by passing the [from, to) window obtained from
// valueStart() + valueEnd(), which avoids matching keys that also occur in
// sibling/nested objects (e.g. Spotify's album.name vs item.name).

// ─── char codes ───────────────────────────────────────────────────────────────
const QUOTE: i32     = 0x22; // "
const BACKSLASH: i32 = 0x5C; // \
const COLON: i32     = 0x3A; // :
const COMMA: i32     = 0x2C; // ,
const LBRACE: i32    = 0x7B; // {
const RBRACE: i32    = 0x7D; // }
const LBRACKET: i32  = 0x5B; // [
const RBRACKET: i32  = 0x5D; // ]

function isSpace(c: i32): bool {
  return c == 0x20 || c == 0x09 || c == 0x0A || c == 0x0D;
}

/**
 * Index of the first character of the value belonging to `"key":`,
 * searching the window [from, to). Returns -1 if not found.
 * Only matches complete keys (quote-key-quote followed by a colon), so
 * "current" never matches inside "current_units".
 */
export function valueStart(src: string, key: string, from: i32, to: i32): i32 {
  const pat = '"' + key + '"';
  const n   = src.length;
  let i = src.indexOf(pat, from);
  while (i >= 0 && i < to) {
    let j = i + pat.length;
    while (j < n && isSpace(src.charCodeAt(j))) j++;
    if (j < n && src.charCodeAt(j) == COLON) {
      j++;
      while (j < n && isSpace(src.charCodeAt(j))) j++;
      return j < to ? j : -1;
    }
    i = src.indexOf(pat, i + 1);
  }
  return -1;
}

/**
 * End index (exclusive) of the value starting at `start`.
 * Handles strings (with escapes), nested objects/arrays (brace matching,
 * skipping braces inside strings), and bare literals (numbers/true/false/null).
 */
export function valueEnd(src: string, start: i32): i32 {
  const n = src.length;
  const c = src.charCodeAt(start);

  if (c == QUOTE) {
    let i = start + 1;
    while (i < n) {
      const ch = src.charCodeAt(i);
      if (ch == BACKSLASH) { i += 2; continue; }
      if (ch == QUOTE) return i + 1;
      i++;
    }
    return n;
  }

  if (c == LBRACE || c == LBRACKET) {
    let depth = 0;
    let i = start;
    while (i < n) {
      const ch = src.charCodeAt(i);
      if (ch == QUOTE) {
        i++;
        while (i < n) {
          const s = src.charCodeAt(i);
          if (s == BACKSLASH) { i += 2; continue; }
          if (s == QUOTE) break;
          i++;
        }
      } else if (ch == LBRACE || ch == LBRACKET) {
        depth++;
      } else if (ch == RBRACE || ch == RBRACKET) {
        depth--;
        if (depth == 0) return i + 1;
      }
      i++;
    }
    return n;
  }

  // Bare literal: number / true / false / null
  let i = start;
  while (i < n) {
    const ch = src.charCodeAt(i);
    if (ch == COMMA || ch == RBRACE || ch == RBRACKET || isSpace(ch)) break;
    i++;
  }
  return i;
}

/** True if the value at `start` is the literal null. */
export function isNullValue(src: string, start: i32): bool {
  return start >= 0 && start < src.length && src.charCodeAt(start) == 0x6E; // 'n'
}

/**
 * String value for `key` in [from, to), unescaped. Null if the key is
 * missing or its value is not a string.
 */
export function getString(src: string, key: string, from: i32, to: i32): string | null {
  const vs = valueStart(src, key, from, to);
  if (vs < 0 || src.charCodeAt(vs) != QUOTE) return null;

  const n   = src.length;
  let out   = "";
  let i     = vs + 1;
  let plain = i; // start of the current run of unescaped chars

  while (i < n) {
    const ch = src.charCodeAt(i);
    if (ch == QUOTE) break;
    if (ch != BACKSLASH) { i++; continue; }

    out += src.substring(plain, i);
    i++; // at escape char
    if (i >= n) break;
    const esc = src.charCodeAt(i);
    if      (esc == 0x6E) out += "\n";       // \n
    else if (esc == 0x74) out += "\t";       // \t
    else if (esc == 0x72) out += "\r";       // \r
    else if (esc == 0x75 && i + 4 < n) {     // \uXXXX
      const code = <i32>parseInt(src.substring(i + 1, i + 5), 16);
      out += String.fromCharCode(code);
      i += 4;
    } else {
      out += src.charAt(i);                  // \" \\ \/ and anything else
    }
    i++;
    plain = i;
  }
  return out + src.substring(plain, i);
}

/** Numeric value for `key` in [from, to). NaN if missing or non-numeric. */
export function getNumber(src: string, key: string, from: i32, to: i32): f64 {
  const vs = valueStart(src, key, from, to);
  if (vs < 0) return NaN;
  const c = src.charCodeAt(vs);
  if (!(c == 0x2D || (c >= 0x30 && c <= 0x39))) return NaN; // '-' or digit
  return parseFloat(src.substring(vs, valueEnd(src, vs)));
}

/** Boolean value for `key` in [from, to), or `def` if missing. */
export function getBool(src: string, key: string, def: bool, from: i32, to: i32): bool {
  const vs = valueStart(src, key, from, to);
  if (vs < 0) return def;
  const c = src.charCodeAt(vs);
  if (c == 0x74) return true;  // 't'
  if (c == 0x66) return false; // 'f'
  return def;
}
