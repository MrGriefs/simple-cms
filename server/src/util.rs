use core::iter;
use std::str::Chars;

fn advance_while<P: FnMut(char) -> bool>(iter: &mut iter::Peekable<Chars>, mut predicate: P) -> usize {
  let mut counter = 0;
  while let Some(&w) = iter.peek() {
    if !predicate(w) {
      break;
    }
    counter += 1;
    iter.next();
  }
  counter
}

pub fn parse_lp_cmd_line<'a>(code_units: &String) -> Vec<String> {
  const BACKSLASH: char = b'\\' as char;
  const QUOTE: char = b'"' as char;
  const TAB: char = b'\t' as char;
  const SPACE: char = b' ' as char;

  let mut ret_val: Vec<String> = Vec::from([String::new()]);
  let mut code_units = code_units.chars().peekable();

  advance_while(&mut code_units, |w| w == SPACE || w == TAB);

  // Parse the arguments according to these rules:
  // * All code units are taken literally except space, tab, quote and backslash.
  // * When not `in_quotes`, space and tab separate arguments. Consecutive spaces and tabs are
  // treated as a single separator.
  // * A space or tab `in_quotes` is taken literally.
  // * A quote toggles `in_quotes` mode unless it's escaped. An escaped quote is taken literally.
  // * A quote can be escaped if preceded by an odd number of backslashes.
  // * If any number of backslashes is immediately followed by a quote then the number of
  // backslashes is halved (rounding down).
  // * Backslashes not followed by a quote are all taken literally.
  // * If `in_quotes` then a quote can also be escaped using another quote
  // (i.e. two consecutive quotes become one literal quote).
  let mut cur = Vec::new();
  let mut in_quotes = false;
  while let Some(w) = code_units.next() {
    match w {
      // If not `in_quotes`, a space or tab ends the argument.
      SPACE | TAB if !in_quotes => {
        ret_val.push(cur.iter().collect());
        cur.truncate(0);

        // Skip whitespace.
        advance_while(&mut code_units, |w| w == SPACE || w == TAB);
      }
      // Backslashes can escape quotes or backslashes but only if consecutive backslashes are followed by a quote.
      BACKSLASH => {
        let backslash_count = advance_while(&mut code_units, |w| w == BACKSLASH) + 1;
        if code_units.peek() == Some(&QUOTE) {
          cur.extend(iter::repeat(BACKSLASH).take(backslash_count / 2));
          // The quote is escaped if there are an odd number of backslashes.
          if backslash_count % 2 == 1 {
            code_units.next();
            cur.push(QUOTE);
          }
        } else {
          // If there is no quote on the end then there is no escaping.
          cur.extend(iter::repeat(BACKSLASH).take(backslash_count));
        }
      }
      // If `in_quotes` and not backslash escaped (see above) then a quote either
      // unsets `in_quote` or is escaped by another quote.
      QUOTE if in_quotes => match code_units.peek() {
        // Two consecutive quotes when `in_quotes` produces one literal quote.
        Some(&QUOTE) => {
          cur.push(QUOTE);
          code_units.next();
        }
        // Otherwise set `in_quotes`.
        Some(_) => in_quotes = false,
        // The end of the command line.
        // Push `cur` even if empty, which we do by breaking while `in_quotes` is still set.
        None => break,
      },
      // If not `in_quotes` and not BACKSLASH escaped (see above) then a quote sets `in_quote`.
      QUOTE => in_quotes = true,
      // Everything else is always taken literally.
      _ => cur.push(w),
    }
  }
  // Push the final argument, if any.
  if !cur.is_empty() || in_quotes {
    ret_val.push(cur.iter().collect());
  }
  ret_val
}