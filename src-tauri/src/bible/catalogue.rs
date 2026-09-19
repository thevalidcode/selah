//! The published Bible translations Selah knows how to hold.
//!
//! Selah ships no Bible text, so this list is **metadata only**: it is the
//! answer to "which translations can I add, and what is each one called?".
//! Nothing here is downloaded, and the words still have to arrive from
//! somewhere the operator is entitled to use — a published file they own, or
//! verses they supply (see the Bible screen).
//!
//! Adding one of these creates the translation in the database, ready to hold
//! verses. Its name and abbreviation then come from here rather than being
//! typed out again, which is what stops the picker filling up with `MSG` and
//! `The Message` and `message bible` as three different entries.

use serde::Serialize;

/// One translation an operator can add.
pub struct Published {
    /// Short id used everywhere afterwards, e.g. `msg`.
    id: &'static str,
    /// Full name, e.g. `The Message`.
    name: &'static str,
    /// Usual abbreviation, e.g. `MSG`.
    abbreviation: &'static str,
    language: &'static str,
    /// Which family of translations this is, for grouping in the interface.
    group: &'static str,
    /// Whether the translation text is freely redistributable. Informational
    /// only: Selah never downloads or ships Bible text either way.
    public_domain: bool,
}

/// One entry in the catalogue, as the interface sees it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogueEntry {
    pub id: String,
    pub name: String,
    pub abbreviation: String,
    pub language: String,
    pub group: String,
    /// True for the translations Selah ships, which cannot be removed.
    pub builtin: bool,
    /// Whether the translation is already installed.
    pub installed: bool,
    /// How many verses are stored for it, when it is installed.
    pub verse_count: i64,
    /// Informational: whether the translation text is public domain.
    pub public_domain: bool,
}

/// The three translations that ship with Selah.
pub const BUILTIN_IDS: [&str; 3] = ["web", "kjv", "asv"];

/// Whether an id belongs to one of the bundled translations.
pub fn is_builtin(id: &str) -> bool {
    BUILTIN_IDS
        .iter()
        .any(|builtin| builtin.eq_ignore_ascii_case(id.trim()))
}

/// Looks a published translation up by id, case-insensitively.
pub fn find(id: &str) -> Option<&'static Published> {
    PUBLISHED
        .iter()
        .find(|published| published.id.eq_ignore_ascii_case(id.trim()))
}

/// Every published translation, as catalogue entries.
pub fn all() -> Vec<CatalogueEntry> {
    PUBLISHED
        .iter()
        .map(CatalogueEntry::from_published)
        .collect()
}

impl Published {
    /// The id used in the database.
    pub fn id(&self) -> &'static str {
        self.id
    }

    /// The full name, e.g. `The Message`.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The usual abbreviation, e.g. `MSG`.
    pub fn abbreviation(&self) -> &'static str {
        self.abbreviation
    }

    /// The language code, e.g. `en`.
    pub fn language(&self) -> &'static str {
        self.language
    }
}

impl CatalogueEntry {
    /// The entry as it appears before anything is installed.
    pub fn from_published(published: &Published) -> Self {
        Self {
            id: published.id.to_string(),
            name: published.name.to_string(),
            abbreviation: published.abbreviation.to_string(),
            language: published.language.to_string(),
            group: published.group.to_string(),
            builtin: is_builtin(published.id),
            installed: false,
            verse_count: 0,
            public_domain: published.public_domain,
        }
    }
}

/// The catalogue itself, ordered roughly as operators reach for them.
static PUBLISHED: &[Published] = &[
    Published {
        id: "kjv",
        name: "King James Version",
        abbreviation: "KJV",
        language: "en",
        group: "Classic",
        public_domain: true,
    },
    Published {
        id: "nkjv",
        name: "New King James Version",
        abbreviation: "NKJV",
        language: "en",
        group: "Classic",
        public_domain: false,
    },
    Published {
        id: "asv",
        name: "American Standard Version",
        abbreviation: "ASV",
        language: "en",
        group: "Classic",
        public_domain: true,
    },
    Published {
        id: "web",
        name: "World English Bible",
        abbreviation: "WEB",
        language: "en",
        group: "Classic",
        public_domain: true,
    },
    Published {
        id: "rsv",
        name: "Revised Standard Version",
        abbreviation: "RSV",
        language: "en",
        group: "Classic",
        public_domain: false,
    },
    Published {
        id: "nrsv",
        name: "New Revised Standard Version",
        abbreviation: "NRSV",
        language: "en",
        group: "Classic",
        public_domain: false,
    },
    Published {
        id: "kj21",
        name: "21st Century King James Version",
        abbreviation: "KJ21",
        language: "en",
        group: "Classic",
        public_domain: false,
    },
    Published {
        id: "niv",
        name: "New International Version",
        abbreviation: "NIV",
        language: "en",
        group: "Modern",
        public_domain: false,
    },
    Published {
        id: "esv",
        name: "English Standard Version",
        abbreviation: "ESV",
        language: "en",
        group: "Modern",
        public_domain: false,
    },
    Published {
        id: "nasb",
        name: "New American Standard Bible",
        abbreviation: "NASB",
        language: "en",
        group: "Modern",
        public_domain: false,
    },
    Published {
        id: "msg",
        name: "The Message",
        abbreviation: "MSG",
        language: "en",
        group: "Modern",
        public_domain: false,
    },
    Published {
        id: "amp",
        name: "Amplified Bible",
        abbreviation: "AMP",
        language: "en",
        group: "Modern",
        public_domain: false,
    },
    Published {
        id: "nlt",
        name: "New Living Translation",
        abbreviation: "NLT",
        language: "en",
        group: "Modern",
        public_domain: false,
    },
    Published {
        id: "csb",
        name: "Christian Standard Bible",
        abbreviation: "CSB",
        language: "en",
        group: "Modern",
        public_domain: false,
    },
    Published {
        id: "hcsb",
        name: "Holman Christian Standard Bible",
        abbreviation: "HCSB",
        language: "en",
        group: "Modern",
        public_domain: false,
    },
    Published {
        id: "net",
        name: "New English Translation",
        abbreviation: "NET",
        language: "en",
        group: "Modern",
        public_domain: false,
    },
    Published {
        id: "gnt",
        name: "Good News Translation",
        abbreviation: "GNT",
        language: "en",
        group: "Everyday",
        public_domain: false,
    },
    Published {
        id: "cev",
        name: "Contemporary English Version",
        abbreviation: "CEV",
        language: "en",
        group: "Everyday",
        public_domain: false,
    },
    Published {
        id: "tlb",
        name: "The Living Bible",
        abbreviation: "TLB",
        language: "en",
        group: "Everyday",
        public_domain: false,
    },
    Published {
        id: "ncv",
        name: "New Century Version",
        abbreviation: "NCV",
        language: "en",
        group: "Everyday",
        public_domain: false,
    },
    Published {
        id: "gnb",
        name: "Good News Bible",
        abbreviation: "GNB",
        language: "en",
        group: "Everyday",
        public_domain: false,
    },
    Published {
        id: "erv",
        name: "Easy-to-Read Version",
        abbreviation: "ERV",
        language: "en",
        group: "Everyday",
        public_domain: false,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn the_published_list_covers_the_translations_operators_ask_for() {
        let ids: Vec<&str> = PUBLISHED.iter().map(|p| p.id).collect();
        for expected in [
            "msg", "niv", "esv", "kjv", "nkjv", "nasb", "amp", "nlt", "csb", "hcsb", "net", "rsv",
            "nrsv", "gnt", "cev", "tlb",
        ] {
            assert!(ids.contains(&expected), "{expected} should be published");
        }
    }

    #[test]
    fn every_entry_is_unique_and_well_formed() {
        let mut seen = HashSet::new();
        for published in PUBLISHED {
            assert!(seen.insert(published.id), "{} twice", published.id);
            assert!(!published.name.trim().is_empty(), "{} name", published.id);
            assert!(
                !published.abbreviation.trim().is_empty(),
                "{} abbreviation",
                published.id
            );
            assert!(!published.group.trim().is_empty(), "{} group", published.id);
            // Ids are used as database keys and in `invoke` payloads, so they
            // must be plain lowercase slugs.
            assert!(
                published
                    .id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
                "{} is not a slug",
                published.id
            );
        }
    }

    #[test]
    fn the_bundled_three_are_marked_as_selahs_own() {
        for id in BUILTIN_IDS {
            assert!(is_builtin(id), "{id} should be built in");
            let entry = all()
                .into_iter()
                .find(|entry| entry.id == id)
                .unwrap_or_else(|| panic!("{id} should be in the catalogue"));
            assert!(entry.builtin, "{id} should be flagged built in");
        }

        // …and nothing else is.
        assert!(!is_builtin("msg"));
        assert!(!is_builtin(""));
        assert!(!is_builtin("  "));
    }

    #[test]
    fn lookup_ignores_case_and_surrounding_space() {
        assert_eq!(find("MSG").map(Published::name), Some("The Message"));
        assert_eq!(
            find("  niv  ").map(Published::name),
            Some("New International Version")
        );
        assert!(find("nope").is_none());
    }

    #[test]
    fn a_fresh_entry_starts_uninstalled() {
        let entry = all().into_iter().find(|e| e.id == "msg").expect("msg");
        assert!(!entry.installed);
        assert_eq!(entry.verse_count, 0);
        assert_eq!(entry.abbreviation, "MSG");
        assert_eq!(entry.language, "en");
    }
}
