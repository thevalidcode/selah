//! Deterministic Bible book registry.
//!
//! A book is identified by a stable numeric id shared with the SQLite `books`
//! table. Each entry carries canonical name, common written abbreviations and
//! spoken aliases ("first corinthians", "i corinthians", "psalms", ...).
//! Lookups are exact, lowercase, case-insensitive token matches — no fuzzy
//! scoring, so results are reproducible.

/// A book of the bible with all written/spoken reference variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BibleBook {
    pub id: i32,
    pub name: &'static str,
    pub testament: Testament,
    pub abbreviation: &'static str,
    /// Chapter count for the canonical protestant arrangement. Used for
    /// structural validation of parsed references.
    pub chapters: u16,
    /// Lowercase spoken/written variants, multi-word allowed.
    pub aliases: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Testament {
    Old,
    New,
}

// Numbered books include digit ("1 corinthians"), cardinal ("first
// corinthians"), ordinal ("1st corinthians") and roman ("i corinthians")
// forms; the most common scribal abbreviations are included. Alternate and
// plural spoken forms ("psalms"/"psalm", "revelations", "song of songs") are
// present where they are common in speech.
pub static BOOKS: &[BibleBook] = &[
    BibleBook {
        id: 1,
        name: "Genesis",
        testament: Testament::Old,
        abbreviation: "Gen",
        chapters: 50,
        aliases: &["genesis", "gen", "gn"],
    },
    BibleBook {
        id: 2,
        name: "Exodus",
        testament: Testament::Old,
        abbreviation: "Exo",
        chapters: 40,
        aliases: &["exodus", "exo", "ex", "exod"],
    },
    BibleBook {
        id: 3,
        name: "Leviticus",
        testament: Testament::Old,
        abbreviation: "Lev",
        chapters: 27,
        aliases: &["leviticus", "lev", "lv"],
    },
    BibleBook {
        id: 4,
        name: "Numbers",
        testament: Testament::Old,
        abbreviation: "Num",
        chapters: 36,
        aliases: &["numbers", "num", "nm", "nu"],
    },
    BibleBook {
        id: 5,
        name: "Deuteronomy",
        testament: Testament::Old,
        abbreviation: "Deu",
        chapters: 34,
        aliases: &["deuteronomy", "deut", "deu", "dt"],
    },
    BibleBook {
        id: 6,
        name: "Joshua",
        testament: Testament::Old,
        abbreviation: "Jos",
        chapters: 24,
        aliases: &["joshua", "josh", "jos", "jsh"],
    },
    BibleBook {
        id: 7,
        name: "Judges",
        testament: Testament::Old,
        abbreviation: "Jdg",
        chapters: 21,
        aliases: &["judges", "judg", "jdg", "jd"],
    },
    BibleBook {
        id: 8,
        name: "Ruth",
        testament: Testament::Old,
        abbreviation: "Rut",
        chapters: 4,
        aliases: &["ruth", "rut"],
    },
    BibleBook {
        id: 9,
        name: "1 Samuel",
        testament: Testament::Old,
        abbreviation: "1Sa",
        chapters: 31,
        aliases: &[
            "1 samuel",
            "first samuel",
            "i samuel",
            "1st samuel",
            "one samuel",
            "1sa",
            "1 sam",
            "i sam",
            "1sm",
        ],
    },
    BibleBook {
        id: 10,
        name: "2 Samuel",
        testament: Testament::Old,
        abbreviation: "2Sa",
        chapters: 24,
        aliases: &[
            "2 samuel",
            "second samuel",
            "ii samuel",
            "2nd samuel",
            "2sa",
            "2 sam",
            "2sm",
        ],
    },
    BibleBook {
        id: 11,
        name: "1 Kings",
        testament: Testament::Old,
        abbreviation: "1Ki",
        chapters: 22,
        aliases: &[
            "1 kings",
            "first kings",
            "i kings",
            "1st kings",
            "1ki",
            "1 kgs",
        ],
    },
    BibleBook {
        id: 12,
        name: "2 Kings",
        testament: Testament::Old,
        abbreviation: "2Ki",
        chapters: 25,
        aliases: &[
            "2 kings",
            "second kings",
            "ii kings",
            "2nd kings",
            "2ki",
            "2 kgs",
        ],
    },
    BibleBook {
        id: 13,
        name: "1 Chronicles",
        testament: Testament::Old,
        abbreviation: "1Ch",
        chapters: 29,
        aliases: &[
            "1 chronicles",
            "first chronicles",
            "i chronicles",
            "1st chronicles",
            "1ch",
            "1 chron",
        ],
    },
    BibleBook {
        id: 14,
        name: "2 Chronicles",
        testament: Testament::Old,
        abbreviation: "2Ch",
        chapters: 36,
        aliases: &[
            "2 chronicles",
            "second chronicles",
            "ii chronicles",
            "2nd chronicles",
            "2ch",
            "2 chron",
        ],
    },
    BibleBook {
        id: 15,
        name: "Ezra",
        testament: Testament::Old,
        abbreviation: "Ezr",
        chapters: 10,
        aliases: &["ezra", "ezr"],
    },
    BibleBook {
        id: 16,
        name: "Nehemiah",
        testament: Testament::Old,
        abbreviation: "Neh",
        chapters: 13,
        aliases: &["nehemiah", "neh"],
    },
    BibleBook {
        id: 17,
        name: "Esther",
        testament: Testament::Old,
        abbreviation: "Est",
        chapters: 10,
        aliases: &["esther", "est"],
    },
    BibleBook {
        id: 18,
        name: "Job",
        testament: Testament::Old,
        abbreviation: "Job",
        chapters: 42,
        aliases: &["job"],
    },
    BibleBook {
        id: 19,
        name: "Psalms",
        testament: Testament::Old,
        abbreviation: "Psa",
        chapters: 150,
        aliases: &["psalms", "psalm", "psa", "ps", "pslm"],
    },
    BibleBook {
        id: 20,
        name: "Proverbs",
        testament: Testament::Old,
        abbreviation: "Pro",
        chapters: 31,
        aliases: &["proverbs", "proverb", "prov", "pro"],
    },
    BibleBook {
        id: 21,
        name: "Ecclesiastes",
        testament: Testament::Old,
        abbreviation: "Ecc",
        chapters: 12,
        aliases: &["ecclesiastes", "eccles", "ecc", "eccl", "qc"],
    },
    BibleBook {
        id: 22,
        name: "Song of Solomon",
        testament: Testament::Old,
        abbreviation: "Sng",
        chapters: 8,
        aliases: &[
            "song of solomon",
            "song of songs",
            "song of song",
            "songs",
            "canticles",
            "sng",
            "sos",
            "so",
        ],
    },
    BibleBook {
        id: 23,
        name: "Isaiah",
        testament: Testament::Old,
        abbreviation: "Isa",
        chapters: 66,
        aliases: &["isaiah", "isa", "is"],
    },
    BibleBook {
        id: 24,
        name: "Jeremiah",
        testament: Testament::Old,
        abbreviation: "Jer",
        chapters: 52,
        aliases: &["jeremiah", "jeremias", "jer", "jr"],
    },
    BibleBook {
        id: 25,
        name: "Lamentations",
        testament: Testament::Old,
        abbreviation: "Lam",
        chapters: 5,
        aliases: &["lamentations", "lamentation", "lam"],
    },
    BibleBook {
        id: 26,
        name: "Ezekiel",
        testament: Testament::Old,
        abbreviation: "Ezk",
        chapters: 48,
        aliases: &["ezekiel", "ezek", "ezk", "eze"],
    },
    BibleBook {
        id: 27,
        name: "Daniel",
        testament: Testament::Old,
        abbreviation: "Dan",
        chapters: 12,
        aliases: &["daniel", "dan"],
    },
    BibleBook {
        id: 28,
        name: "Hosea",
        testament: Testament::Old,
        abbreviation: "Hos",
        chapters: 14,
        aliases: &["hosea", "hos"],
    },
    BibleBook {
        id: 29,
        name: "Joel",
        testament: Testament::Old,
        abbreviation: "Jol",
        chapters: 3,
        aliases: &["joel", "jol"],
    },
    BibleBook {
        id: 30,
        name: "Amos",
        testament: Testament::Old,
        abbreviation: "Amo",
        chapters: 9,
        aliases: &["amos", "amo"],
    },
    BibleBook {
        id: 31,
        name: "Obadiah",
        testament: Testament::Old,
        abbreviation: "Oba",
        chapters: 1,
        aliases: &["obadiah", "oba", "abdias"],
    },
    BibleBook {
        id: 32,
        name: "Jonah",
        testament: Testament::Old,
        abbreviation: "Jon",
        chapters: 4,
        aliases: &["jonah", "jon"],
    },
    BibleBook {
        id: 33,
        name: "Micah",
        testament: Testament::Old,
        abbreviation: "Mic",
        chapters: 7,
        aliases: &["micah", "mic"],
    },
    BibleBook {
        id: 34,
        name: "Nahum",
        testament: Testament::Old,
        abbreviation: "Nam",
        chapters: 3,
        aliases: &["nahum", "nam"],
    },
    BibleBook {
        id: 35,
        name: "Habakkuk",
        testament: Testament::Old,
        abbreviation: "Hab",
        chapters: 3,
        aliases: &["habakkuk", "hab"],
    },
    BibleBook {
        id: 36,
        name: "Zephaniah",
        testament: Testament::Old,
        abbreviation: "Zep",
        chapters: 3,
        aliases: &["zephaniah", "zeph", "zep"],
    },
    BibleBook {
        id: 37,
        name: "Haggai",
        testament: Testament::Old,
        abbreviation: "Hag",
        chapters: 2,
        aliases: &["haggai", "hag"],
    },
    BibleBook {
        id: 38,
        name: "Zechariah",
        testament: Testament::Old,
        abbreviation: "Zec",
        chapters: 14,
        aliases: &["zechariah", "zechar", "zec"],
    },
    BibleBook {
        id: 39,
        name: "Malachi",
        testament: Testament::Old,
        abbreviation: "Mal",
        chapters: 4,
        aliases: &["malachi", "mal"],
    },
    BibleBook {
        id: 40,
        name: "Matthew",
        testament: Testament::New,
        abbreviation: "Mat",
        chapters: 28,
        aliases: &["matthew", "matt", "mat", "mt"],
    },
    BibleBook {
        id: 41,
        name: "Mark",
        testament: Testament::New,
        abbreviation: "Mar",
        chapters: 16,
        aliases: &["mark", "mar", "mk"],
    },
    BibleBook {
        id: 42,
        name: "Luke",
        testament: Testament::New,
        abbreviation: "Luk",
        chapters: 24,
        aliases: &["luke", "luk", "lk"],
    },
    BibleBook {
        id: 43,
        name: "John",
        testament: Testament::New,
        abbreviation: "Jhn",
        chapters: 21,
        aliases: &["john", "jhn", "joh", "jn"],
    },
    BibleBook {
        id: 44,
        name: "Acts",
        testament: Testament::New,
        abbreviation: "Act",
        chapters: 28,
        aliases: &["acts", "act", "ac"],
    },
    BibleBook {
        id: 45,
        name: "Romans",
        testament: Testament::New,
        abbreviation: "Rom",
        chapters: 16,
        aliases: &["romans", "rom", "rm"],
    },
    BibleBook {
        id: 46,
        name: "1 Corinthians",
        testament: Testament::New,
        abbreviation: "1Co",
        chapters: 16,
        aliases: &[
            "1 corinthians",
            "first corinthians",
            "i corinthians",
            "1st corinthians",
            "one corinthians",
            "1co",
            "1 cor",
            "i cor",
        ],
    },
    BibleBook {
        id: 47,
        name: "2 Corinthians",
        testament: Testament::New,
        abbreviation: "2Co",
        chapters: 13,
        aliases: &[
            "2 corinthians",
            "second corinthians",
            "ii corinthians",
            "2nd corinthians",
            "2co",
            "2 cor",
        ],
    },
    BibleBook {
        id: 48,
        name: "Galatians",
        testament: Testament::New,
        abbreviation: "Gal",
        chapters: 6,
        aliases: &["galatians", "gal"],
    },
    BibleBook {
        id: 49,
        name: "Ephesians",
        testament: Testament::New,
        abbreviation: "Eph",
        chapters: 6,
        aliases: &["ephesians", "eph"],
    },
    BibleBook {
        id: 50,
        name: "Philippians",
        testament: Testament::New,
        abbreviation: "Php",
        chapters: 4,
        aliases: &["philippians", "phil", "php", "pp"],
    },
    BibleBook {
        id: 51,
        name: "Colossians",
        testament: Testament::New,
        abbreviation: "Col",
        chapters: 4,
        aliases: &["colossians", "col"],
    },
    BibleBook {
        id: 52,
        name: "1 Thessalonians",
        testament: Testament::New,
        abbreviation: "1Th",
        chapters: 5,
        aliases: &[
            "1 thessalonians",
            "first thessalonians",
            "i thessalonians",
            "1st thessalonians",
            "1th",
            "1 thess",
        ],
    },
    BibleBook {
        id: 53,
        name: "2 Thessalonians",
        testament: Testament::New,
        abbreviation: "2Th",
        chapters: 3,
        aliases: &[
            "2 thessalonians",
            "second thessalonians",
            "ii thessalonians",
            "2nd thessalonians",
            "2th",
            "2 thess",
        ],
    },
    BibleBook {
        id: 54,
        name: "1 Timothy",
        testament: Testament::New,
        abbreviation: "1Ti",
        chapters: 6,
        aliases: &[
            "1 timothy",
            "first timothy",
            "i timothy",
            "1st timothy",
            "1ti",
            "1 tim",
        ],
    },
    BibleBook {
        id: 55,
        name: "2 Timothy",
        testament: Testament::New,
        abbreviation: "2Ti",
        chapters: 4,
        aliases: &[
            "2 timothy",
            "second timothy",
            "ii timothy",
            "2nd timothy",
            "2ti",
            "2 tim",
        ],
    },
    BibleBook {
        id: 56,
        name: "Titus",
        testament: Testament::New,
        abbreviation: "Tit",
        chapters: 3,
        aliases: &["titus", "tit"],
    },
    BibleBook {
        id: 57,
        name: "Philemon",
        testament: Testament::New,
        abbreviation: "Phm",
        chapters: 1,
        aliases: &["philemon", "philomon", "phm", "phlm"],
    },
    BibleBook {
        id: 58,
        name: "Hebrews",
        testament: Testament::New,
        abbreviation: "Heb",
        chapters: 13,
        aliases: &["hebrews", "heb"],
    },
    BibleBook {
        id: 59,
        name: "James",
        testament: Testament::New,
        abbreviation: "Jas",
        chapters: 5,
        aliases: &["james", "jas", "jm"],
    },
    BibleBook {
        id: 60,
        name: "1 Peter",
        testament: Testament::New,
        abbreviation: "1Pe",
        chapters: 5,
        aliases: &[
            "1 peter",
            "first peter",
            "i peter",
            "1st peter",
            "1pe",
            "1 pet",
        ],
    },
    BibleBook {
        id: 61,
        name: "2 Peter",
        testament: Testament::New,
        abbreviation: "2Pe",
        chapters: 3,
        aliases: &[
            "2 peter",
            "second peter",
            "ii peter",
            "2nd peter",
            "2pe",
            "2 pet",
        ],
    },
    BibleBook {
        id: 62,
        name: "1 John",
        testament: Testament::New,
        abbreviation: "1Jn",
        chapters: 5,
        aliases: &["1 john", "first john", "i john", "1st john", "1jn", "1 jn"],
    },
    BibleBook {
        id: 63,
        name: "2 John",
        testament: Testament::New,
        abbreviation: "2Jn",
        chapters: 1,
        aliases: &[
            "2 john",
            "second john",
            "ii john",
            "2nd john",
            "2jn",
            "2 jn",
        ],
    },
    BibleBook {
        id: 64,
        name: "3 John",
        testament: Testament::New,
        abbreviation: "3Jn",
        chapters: 1,
        aliases: &[
            "3 john",
            "third john",
            "iii john",
            "3rd john",
            "3jn",
            "3 jn",
        ],
    },
    BibleBook {
        id: 65,
        name: "Jude",
        testament: Testament::New,
        abbreviation: "Jud",
        chapters: 1,
        aliases: &["jude", "jud"],
    },
    BibleBook {
        id: 66,
        name: "Revelation",
        testament: Testament::New,
        abbreviation: "Rev",
        chapters: 22,
        aliases: &[
            "revelation",
            "revelations",
            "rev",
            "revelation of john",
            "apocalypse",
        ],
    },
];

/// Returns the canonical book for a numeric id.
pub fn book_by_id(id: i32) -> Option<&'static BibleBook> {
    BOOKS.iter().find(|b| b.id == id)
}

/// Returns the canonical book whose canonical name matches.
pub fn book_by_name(name: &str) -> Option<&'static BibleBook> {
    let n = name.trim().to_lowercase();
    BOOKS.iter().find(|b| b.name.to_lowercase() == n)
}

/// Resolves a book given a slice of tokens starting at `start`. Prefers the
/// longest alias match. Returns the book and the number of tokens consumed.
pub fn find_book(tokens: &[&str], start: usize) -> Option<(&'static BibleBook, usize)> {
    let mut best: Option<(&'static BibleBook, usize)> = None;
    for book in BOOKS {
        for alias in book.aliases {
            let words: Vec<&str> = alias.split_whitespace().collect();
            if words.is_empty() || start + words.len() > tokens.len() {
                continue;
            }
            if words
                .iter()
                .enumerate()
                .all(|(i, w)| tokens[start + i].eq_ignore_ascii_case(w))
            {
                let len = words.len();
                if best.map_or(true, |(_, best_len)| len > best_len) {
                    best = Some((book, len));
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_first_corinthians() {
        let tokens = ["first", "corinthians"];
        let (book, len) = find_book(&tokens, 0).expect("book resolved");
        assert_eq!(book.id, 46);
        assert_eq!(len, 2);
        assert_eq!(book.name, "1 Corinthians");
    }

    #[test]
    fn resolves_digit_and_roman_variants() {
        for alias in [
            "1 corinthians",
            "i corinthians",
            "1st corinthians",
            "one corinthians",
        ] {
            let tokens: Vec<&str> = alias.split_whitespace().collect();
            let (book, _) = find_book(&tokens, 0).expect("book resolved");
            assert_eq!(book.id, 46, "alias {alias}");
        }
    }

    #[test]
    fn resolves_psalm_and_psalms() {
        for alias in ["psalm", "psalms"] {
            let (book, _) = find_book(&[alias], 0).expect("book resolved");
            assert_eq!(book.id, 19);
        }
    }

    #[test]
    fn resolves_revelations_plural() {
        let (book, _) = find_book(&["revelations"], 0).expect("book resolved");
        assert_eq!(book.id, 66);
    }

    #[test]
    fn prefers_longest_alias() {
        let tokens = ["song", "of", "solomon", "1"];
        let (book, len) = find_book(&tokens, 0).expect("book resolved");
        assert_eq!(book.id, 22);
        assert_eq!(len, 3);
    }

    #[test]
    fn no_match_without_book() {
        assert!(find_book(&["chapter", "three"], 0).is_none());
        assert!(find_book(&["the"], 0).is_none());
    }

    #[test]
    fn book_by_id_roundtrip() {
        assert_eq!(book_by_id(1).map(|b| b.name), Some("Genesis"));
        assert_eq!(book_by_id(66).map(|b| b.name), Some("Revelation"));
        assert!(book_by_id(0).is_none());
    }
}
