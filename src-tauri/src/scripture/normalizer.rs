//! Text normalization and number-word conversion.
//!
//! Spoken input ("chapter three verse sixteen") and typed input ("John 3:16")
//! are both reduced to a normalized token stream before parsing.

/// Collapses whitespace, lowercases, and normalizes curly quotes/apostrophes.
pub fn normalize_text(input: &str) -> String {
    input
        .replace(['\u{2018}', '\u{2019}', '\u{201B}'], "'")
        .replace(['\u{201C}', '\u{201D}', '\u{201F}'], "\"")
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Basic cardinal number words up to one hundred.
fn simple_number(word: &str) -> Option<u32> {
    Some(match word {
        "zero" => 0,
        "one" | "first" => 1,
        "two" | "second" => 2,
        "three" | "third" => 3,
        "four" | "fourth" => 4,
        "five" | "fifth" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" => 10,
        "eleven" => 11,
        "twelve" => 12,
        "thirteen" => 13,
        "fourteen" => 14,
        "fifteen" => 15,
        "sixteen" => 16,
        "seventeen" => 17,
        "eighteen" => 18,
        "nineteen" => 19,
        "twenty" => 20,
        "thirty" => 30,
        "forty" => 40,
        "fifty" => 50,
        "sixty" => 60,
        "seventy" => 70,
        "eighty" => 80,
        "ninety" => 90,
        "hundred" => 100,
        _ => return None,
    })
}

/// Parses a compound number word sequence starting at `tokens[idx]`.
///
/// Supports "twenty eight", "one hundred", "one hundred five", ... up to 999.
/// Consumption is minimal: the sequence stops as soon as the next word cannot
/// combine with the running value, so "three sixteen" parses as 3 followed by
/// 16 rather than one big number.
/// Returns the value and the index one past the consumed words.
pub fn words_to_number(tokens: &[&str], idx: usize) -> Option<(u32, usize)> {
    let mut value = 0u32;
    let mut current = 0u32;
    let mut cursor = idx;

    while cursor < tokens.len() {
        let Some(n) = simple_number(tokens[cursor]) else {
            break;
        };
        if n == 100 {
            if current == 0 {
                current = 100;
            } else if (1..=9).contains(&current) {
                current *= 100;
            } else {
                break;
            }
            cursor += 1;
            continue;
        }
        if current == 0 {
            current = n;
            cursor += 1;
            continue;
        }
        if current >= 20 && current % 10 == 0 && n < 10 {
            // tens + unit ("twenty eight")
            current += n;
            cursor += 1;
            continue;
        }
        if current >= 100 && current % 100 == 0 && n < 100 {
            // "one hundred five" — the hundreds part completes
            value += current;
            current = n;
            cursor += 1;
            continue;
        }
        break; // next word cannot combine — leave it for the caller
    }

    value += current;

    if cursor == idx {
        None
    } else {
        Some((value, cursor))
    }
}

/// True when the token is a cardinal/ordinal number word.
pub fn is_number_word(word: &str) -> bool {
    simple_number(word).is_some() || word.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Vec<&str> {
        s.split_whitespace().collect()
    }

    #[test]
    fn word_lookup() {
        assert_eq!(simple_number("twenty"), Some(20));
        assert_eq!(simple_number("eight"), Some(8));
        assert_eq!(simple_number("twenty-eight"), None);
    }

    #[test]
    fn compounds() {
        assert_eq!(words_to_number(&t("twenty eight 16"), 0), Some((28, 2)));
        assert_eq!(words_to_number(&t("one hundred five"), 0), Some((105, 3)));
        assert_eq!(words_to_number(&t("one hundred"), 0), Some((100, 2)));
        assert_eq!(words_to_number(&t("ninety nine"), 0), Some((99, 2)));
    }

    #[test]
    fn single_word() {
        assert_eq!(words_to_number(&t("four"), 0), Some((4, 1)));
        assert_eq!(words_to_number(&t("sixteen"), 0), Some((16, 1)));
    }

    #[test]
    fn non_number_stops() {
        assert_eq!(words_to_number(&t("twenty people"), 0), Some((20, 1)));
        assert_eq!(words_to_number(&t("people"), 0), None);
    }
}
