//! Ideas for when a search finds nothing.
//!
//! An operator who types *"worried about money"* or *"feeling hopeless"* is
//! usually not looking for a phrase — they are looking for **where to look**.
//! Full-text search can only answer the first question, so when it comes back
//! empty this module offers passages by theme instead.
//!
//! Only a **reference** is stored here (book, chapter, verse) plus the topic
//! words. Selah ships no Bible text, so nothing in this table is Scripture
//! itself: the words still come from whatever translation the operator has
//! installed.

use serde::Serialize;

use crate::scripture::books::book_by_id;

/// One "you might be looking for…" entry.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicSuggestion {
    /// Short theme name shown as the button label, e.g. `Anxiety`.
    pub topic: String,
    /// Why this passage fits, shown under the label.
    pub why: String,
    pub book_id: i32,
    pub chapter: u16,
    pub verse: u16,
    /// Human-readable reference, e.g. `Philippians 4:6`.
    pub reference: String,
}

/// A theme and the passage that answers it.
struct Topic {
    topic: &'static str,
    why: &'static str,
    /// Words (and word stems) that make this topic relevant.
    keywords: &'static [&'static str],
    book_id: i32,
    chapter: u16,
    verse: u16,
}

/// The theme table.
///
/// Kept deliberately short and pastoral: these are the subjects that actually
/// come up in a service. References are canonical and stable, so a suggestion
/// always resolves against any installed translation.
static TOPICS: &[Topic] = &[
    Topic {
        topic: "Anxiety",
        why: "for a worried heart",
        keywords: &["anxiety", "anxious", "worry", "worried", "stress", "panic"],
        book_id: 50, // Philippians
        chapter: 4,
        verse: 6,
    },
    Topic {
        topic: "Fear",
        why: "when you are afraid",
        keywords: &["fear", "afraid", "scared", "terror", "danger"],
        book_id: 23, // Isaiah
        chapter: 41,
        verse: 10,
    },
    Topic {
        topic: "Comfort",
        why: "comfort in trouble",
        keywords: &[
            "comfort", "grief", "grieving", "sad", "sorrow", "mourning", "loss", "death", "died",
        ],
        book_id: 19, // Psalms
        chapter: 23,
        verse: 1,
    },
    Topic {
        topic: "Hope",
        why: "when hope is hard to find",
        keywords: &[
            "hope",
            "hopeless",
            "despair",
            "discouraged",
            "discouragement",
            "weary",
        ],
        book_id: 45, // Romans
        chapter: 15,
        verse: 13,
    },
    Topic {
        topic: "Forgiveness",
        why: "being forgiven, and forgiving others",
        keywords: &[
            "forgive",
            "forgiveness",
            "guilt",
            "guilty",
            "shame",
            "ashamed",
            "sin",
            "sinned",
        ],
        book_id: 62, // 1 John
        chapter: 1,
        verse: 9,
    },
    Topic {
        topic: "Salvation",
        why: "how someone is saved",
        keywords: &[
            "salvation",
            "saved",
            "save",
            "eternal",
            "heaven",
            "hell",
            "gospel",
        ],
        book_id: 43, // John
        chapter: 3,
        verse: 16,
    },
    Topic {
        topic: "Grace",
        why: "grace rather than effort",
        keywords: &["grace", "mercy", "undeserved", "kindness"],
        book_id: 49, // Ephesians
        chapter: 2,
        verse: 8,
    },
    Topic {
        topic: "Healing",
        why: "for the sick",
        keywords: &[
            "heal", "healing", "sick", "sickness", "illness", "hospital", "pain", "disease",
        ],
        book_id: 59, // James
        chapter: 5,
        verse: 15,
    },
    Topic {
        topic: "Strength",
        why: "when you have nothing left",
        keywords: &[
            "strength",
            "strong",
            "weak",
            "weakness",
            "exhausted",
            "burnout",
        ],
        book_id: 50, // Philippians
        chapter: 4,
        verse: 13,
    },
    Topic {
        topic: "Guidance",
        why: "which way to go",
        keywords: &[
            "guidance",
            "guide",
            "direction",
            "decision",
            "decide",
            "choice",
            "wisdom",
            "confused",
        ],
        book_id: 20, // Proverbs
        chapter: 3,
        verse: 5,
    },
    Topic {
        topic: "Prayer",
        why: "how to pray",
        keywords: &["pray", "prayer", "praying", "intercede", "intercession"],
        book_id: 40, // Matthew
        chapter: 6,
        verse: 9,
    },
    Topic {
        topic: "Peace",
        why: "rest for a restless mind",
        keywords: &["peace", "peaceful", "rest", "calm", "turmoil"],
        book_id: 43, // John
        chapter: 14,
        verse: 27,
    },
    Topic {
        topic: "Joy",
        why: "joy that does not depend on circumstances",
        keywords: &[
            "joy",
            "joyful",
            "happy",
            "happiness",
            "rejoice",
            "celebrate",
        ],
        book_id: 19, // Psalms
        chapter: 16,
        verse: 11,
    },
    Topic {
        topic: "Love",
        why: "what love actually is",
        keywords: &["love", "loving", "charity", "beloved"],
        book_id: 46, // 1 Corinthians
        chapter: 13,
        verse: 4,
    },
    Topic {
        topic: "Marriage",
        why: "for husbands, wives and weddings",
        keywords: &[
            "marriage", "married", "wedding", "husband", "wife", "spouse",
        ],
        book_id: 49, // Ephesians
        chapter: 5,
        verse: 25,
    },
    Topic {
        topic: "Family",
        why: "for parents and children",
        keywords: &[
            "family", "children", "child", "parent", "parents", "mother", "father", "son",
            "daughter",
        ],
        book_id: 20, // Proverbs
        chapter: 22,
        verse: 6,
    },
];

static MORE_TOPICS: &[Topic] = &[
    Topic {
        topic: "Money",
        why: "for giving, debt and worry about provision",
        keywords: &[
            "money",
            "finances",
            "financial",
            "debt",
            "give",
            "giving",
            "offering",
            "tithe",
            "poor",
        ],
        book_id: 40, // Matthew
        chapter: 6,
        verse: 33,
    },
    Topic {
        topic: "Work",
        why: "for work and effort",
        keywords: &[
            "work", "job", "labour", "labor", "business", "study", "students",
        ],
        book_id: 51, // Colossians
        chapter: 3,
        verse: 23,
    },
    Topic {
        topic: "Patience",
        why: "for waiting well",
        keywords: &["patience", "patient", "wait", "waiting", "delay"],
        book_id: 23, // Isaiah
        chapter: 40,
        verse: 31,
    },
    Topic {
        topic: "Temptation",
        why: "when you are being pulled away",
        keywords: &[
            "temptation",
            "tempted",
            "tempt",
            "trial",
            "trials",
            "addiction",
        ],
        book_id: 46, // 1 Corinthians
        chapter: 10,
        verse: 13,
    },
    Topic {
        topic: "Baptism",
        why: "for a baptism service",
        keywords: &[
            "baptism",
            "baptised",
            "baptized",
            "baptise",
            "baptize",
            "immersion",
        ],
        book_id: 44, // Acts
        chapter: 2,
        verse: 38,
    },
    Topic {
        topic: "Communion",
        why: "for the Lord's supper",
        keywords: &[
            "communion",
            "supper",
            "bread",
            "wine",
            "covenant",
            "sacrament",
        ],
        book_id: 46, // 1 Corinthians
        chapter: 11,
        verse: 23,
    },
    Topic {
        topic: "Christmas",
        why: "for the nativity",
        keywords: &[
            "christmas",
            "nativity",
            "manger",
            "bethlehem",
            "incarnation",
        ],
        book_id: 42, // Luke
        chapter: 2,
        verse: 11,
    },
    Topic {
        topic: "Easter",
        why: "for the resurrection",
        keywords: &["easter", "resurrection", "resurrected", "risen", "tomb"],
        book_id: 40, // Matthew
        chapter: 28,
        verse: 6,
    },
    Topic {
        topic: "Praise",
        why: "for a song or a call to worship",
        keywords: &["praise", "worship", "hymn", "sing", "glory", "glorify"],
        book_id: 19, // Psalms
        chapter: 100,
        verse: 1,
    },
    Topic {
        topic: "Thanksgiving",
        why: "for gratitude and harvest",
        keywords: &[
            "thanks",
            "thanksgiving",
            "grateful",
            "gratitude",
            "thankful",
            "harvest",
        ],
        book_id: 52, // 1 Thessalonians
        chapter: 5,
        verse: 18,
    },
    Topic {
        topic: "Protection",
        why: "for safety and shelter",
        keywords: &[
            "protection",
            "protect",
            "safe",
            "safety",
            "shelter",
            "refuge",
            "rescue",
        ],
        book_id: 19, // Psalms
        chapter: 91,
        verse: 1,
    },
    Topic {
        topic: "New beginnings",
        why: "for a new year or a new start",
        keywords: &["beginning", "beginnings", "fresh"],
        book_id: 23, // Isaiah
        chapter: 43,
        verse: 19,
    },
    Topic {
        topic: "The lost",
        why: "for outreach and invitation",
        keywords: &[
            "lost",
            "outreach",
            "invite",
            "invitation",
            "mission",
            "missionary",
            "seeking",
        ],
        book_id: 42, // Luke
        chapter: 19,
        verse: 10,
    },
    Topic {
        topic: "Doubting",
        why: "when faith feels thin",
        keywords: &[
            "doubt", "doubts", "doubting", "faith", "believe", "belief", "unbelief",
        ],
        book_id: 41, // Mark
        chapter: 9,
        verse: 24,
    },
    Topic {
        topic: "Second coming",
        why: "for the return of Christ",
        keywords: &["revelation", "judgement", "judgment", "last"],
        book_id: 66, // Revelation
        chapter: 21,
        verse: 4,
    },
];

/// Suggestions returned when a query gives no clue at all.
///
/// A blank or unrecognised query is the "I do not know where to start" case, so
/// rather than an error the operator gets the passages most often used in a
/// service.
static STARTERS: &[&str] = &["Salvation", "Comfort", "Prayer", "Hope", "Love", "Praise"];

/// Every theme, in table order.
fn all_topics() -> impl Iterator<Item = &'static Topic> {
    TOPICS.iter().chain(MORE_TOPICS.iter())
}

/// Finds the themes closest to `query`, most relevant first.
///
/// Matching is deterministic: a keyword counts when a query word equals it or
/// starts with it (so `praying` finds `pray`), and multi-word keywords are
/// matched against the whole query. Nothing here is fuzzy scoring or guessing —
/// the same query always gives the same list.
pub fn suggest(query: &str, limit: usize) -> Vec<TopicSuggestion> {
    if limit == 0 {
        return Vec::new();
    }
    let normalised = query.trim().to_lowercase();
    let words: Vec<&str> = normalised
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .collect();

    let mut scored: Vec<(usize, &'static Topic)> = Vec::new();
    for topic in all_topics() {
        let score: usize = topic
            .keywords
            .iter()
            .map(|keyword| {
                let by_word = words
                    .iter()
                    .filter(|word| matches_word(word, keyword))
                    .count();
                // Phrase keywords (`born again`) are matched against the query
                // as typed, because the words may not be adjacent.
                let by_phrase =
                    usize::from(keyword.contains(' ') && normalised.contains(keyword)) * 2;
                by_word + by_phrase
            })
            .sum();
        if score > 0 {
            scored.push((score, topic));
        }
    }

    // A query with no clue (or nothing recognised) gets the starters instead of
    // an empty list: the operator is never left staring at "no results".
    if scored.is_empty() {
        return starters(limit);
    }

    // `sort_by` is stable, so equal scores keep the table's own order and the
    // list does not reshuffle between identical searches.
    scored.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, topic)| suggestion(topic))
        .collect()
}

/// The starter list, used when nothing in the query was recognised.
fn starters(limit: usize) -> Vec<TopicSuggestion> {
    STARTERS
        .iter()
        .filter_map(|name| all_topics().find(|t| t.topic == *name))
        .take(limit)
        .map(suggestion)
        .collect()
}

/// Whether a query word refers to `keyword`.
fn matches_word(word: &str, keyword: &str) -> bool {
    if word == keyword {
        return true;
    }
    // Stems only: `praying` → `pray`, `worries` → `worry`. Short keywords are
    // matched exactly so `in` does not match `invite`.
    keyword.len() >= 4 && word.len() > keyword.len() && word.starts_with(keyword)
}

/// Builds the serializable form of a table entry.
fn suggestion(topic: &Topic) -> TopicSuggestion {
    let reference = match book_by_id(topic.book_id) {
        Some(book) => format!("{} {}:{}", book.name, topic.chapter, topic.verse),
        None => format!("Book {} {}:{}", topic.book_id, topic.chapter, topic.verse),
    };

    TopicSuggestion {
        topic: topic.topic.to_string(),
        why: topic.why.to_string(),
        book_id: topic.book_id,
        chapter: topic.chapter,
        verse: topic.verse,
        reference,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripture::books::book_by_id;

    #[test]
    fn a_theme_word_finds_its_passage() {
        let found = suggest("anxiety", 3);
        assert_eq!(found[0].topic, "Anxiety");
        assert_eq!(found[0].reference, "Philippians 4:6");
    }

    #[test]
    fn stems_and_sentences_both_match() {
        // A whole sentence, not just a keyword.
        let found = suggest("I am worried about money", 3);
        let topics: Vec<&str> = found.iter().map(|t| t.topic.as_str()).collect();
        assert!(topics.contains(&"Money"), "got: {topics:?}");
        assert!(topics.contains(&"Anxiety"), "got: {topics:?}");

        // `praying` is not a keyword; `pray` is.
        assert_eq!(suggest("praying", 1)[0].topic, "Prayer");
    }

    #[test]
    fn a_hopeless_query_returns_starters_rather_than_nothing() {
        // The operator is lost: nothing is recognised, so they get somewhere to
        // start instead of an empty screen.
        for query in ["", "   ", "qwertyuiop", "???"] {
            let found = suggest(query, 4);
            assert_eq!(found.len(), 4, "query: {query:?}");
            assert_eq!(found[0].topic, "Salvation");
        }
    }

    #[test]
    fn results_are_ordered_and_repeatable() {
        let first = suggest("fear", 5);
        let second = suggest("fear", 5);
        assert_eq!(first, second, "same query must give the same list");
        assert!(first.len() <= 5);
        assert_eq!(first[0].topic, "Fear");
    }

    #[test]
    fn every_suggestion_points_at_a_real_book() {
        // A suggestion carries only a reference; a wrong book id would send the
        // operator nowhere.
        for topic in all_topics() {
            let book = book_by_id(topic.book_id).expect("book id must exist");
            assert!(
                topic.chapter <= book.chapters,
                "{} chapter {} is beyond {}",
                topic.topic,
                topic.chapter,
                book.name
            );
            assert!(topic.verse >= 1, "{} has no verse", topic.topic);
        }
    }

    #[test]
    fn the_limit_is_respected() {
        assert_eq!(suggest("fear", 1).len(), 1);
        assert!(suggest("fear", 0).is_empty());
        // A limit larger than the matches returns the matches, not more.
        let all = suggest("fear", 100);
        assert!(!all.is_empty());
        assert_eq!(all.len(), suggest("fear", 1000).len());
    }
}
