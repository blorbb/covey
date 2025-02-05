import type { Hotkey, KeyCode } from "./bindings";

export const symbolToKeyCode = (symbol: string): KeyCode | undefined => {
  const key = symbol.toLowerCase();

  // Alphabetical
  if (/^[a-z]$/.test(key)) {
    return key as KeyCode;
  }

  // Numerical
  if (/^[0-9]$/.test(key)) {
    return ("digit" + key) as KeyCode;
  }

  // make sure this starts from shift+0, shift+1, ... shift+9
  // not shift+1, ..., shift+0
  const digitShiftIndex = ")!@#$%^&*(".indexOf(key);
  if (digitShiftIndex !== -1) {
    return ("digit" + digitShiftIndex.toString()) as KeyCode;
  }

  // f* keys
  if (key.startsWith("f")) {
    const fNum = Number.parseInt(key.slice(1), 10);
    if (1 <= fNum && fNum <= 24) {
      return key as KeyCode;
    }
  }

  switch (key) {
    case "`":
    case "~":
      return "backtick";
    case "-":
    case "_":
      return "hyphen";
    case "=":
    case "+":
      return "equal";
    case "tab":
      return "tab";
    case "[":
    case "{":
      return "left-bracket";
    case "]":
    case "}":
      return "right-bracket";
    case "\\":
    case "|":
      return "backslash";
    case ";":
    case ":":
      return "semicolon";
    case "'":
    case '"':
      return "apostrophe";
    case "enter":
      return "enter";
    case ",":
    case "<":
      return "comma";
    case ".":
    case ">":
      return "period";
    case "/":
    case "?":
      return "slash";
    default:
      return;
  }
};

export const nameToSymbol = (name: KeyCode): string => {
  // numbers
  if (name.startsWith("digit")) {
    return name.slice("digit".length);
  }

  // letters
  if (/^[a-z]$/.test(name)) {
    return name.toLocaleUpperCase();
  }

  // fn keys
  // just "f" is caught by above
  if (/^f\d+$/.test(name)) {
    return name.toLocaleUpperCase();
  }

  switch (name) {
    case "backtick":
      return "`";
    case "hyphen":
      return "-";
    case "equal":
      return "=";
    case "tab":
      return "⇥";
    case "left-bracket":
      return "[";
    case "right-bracket":
      return "]";
    case "backslash":
      return "\\";
    case "semicolon":
      return ";";
    case "apostrophe":
      return "'";
    case "enter":
      return "↵";
    case "comma":
      return ",";
    case "period":
      return ".";
    case "slash":
      return "/";
    default:
      throw new Error("all key names should be matched");
  }
};

export const hotkeysEqual = (
  a: Readonly<Hotkey>,
  b: Readonly<Hotkey>,
): boolean =>
  a.key === b.key &&
  a.ctrl === b.ctrl &&
  a.shift === b.shift &&
  a.alt === b.alt &&
  a.meta === b.meta;
