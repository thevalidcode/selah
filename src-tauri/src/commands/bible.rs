//! Bible commands: translations, books, passages, search, import.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::bible::models::{Passage, Translation, Verse};
use crate::errors::{AppError, CommandResult};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassageRequest {
    pub translation_id: String,
    pub book_id: i32,
    pub chapter: u16,
    pub start_verse: Option<u16>,
    pub end_verse: Option<u16>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationStatus {
    pub translation: Translation,
    pub verse_count: i64,
}

/// What a search found — and what to try when it found nothing.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BibleSearchResult {
    pub verses: Vec<Verse>,
    /// True when the exact words did not match and these are the closest.
    pub related: bool,
    /// Themes to offer when no verse matched at all. Carries references only —
    /// Selah ships no Bible text.
    pub suggestions: Vec<crate::bible::topics::TopicSuggestion>,
}

/// Ideas for an operator who is not sure what to search for.
#[tauri::command]
pub fn suggest_bible_topics(
    query: String,
    limit: Option<usize>,
) -> CommandResult<Vec<crate::bible::topics::TopicSuggestion>> {
    Ok(crate::bible::topics::suggest(
        &query,
        limit.unwrap_or(6).clamp(1, 12),
    ))
}

fn list_books_impl(
    conn: &rusqlite::Connection,
) -> Result<Vec<crate::bible::models::BibleBook>, crate::errors::AppError> {
    crate::bible::repository::BibleRepository::new(conn).list_books()
}

#[tauri::command]
pub fn list_bible_translations(
    state: State<'_, AppState>,
) -> CommandResult<Vec<TranslationStatus>> {
    state.with_conn(|conn| {
        let repo = crate::bible::repository::BibleRepository::new(conn);
        let mut out = Vec::new();
        for translation in repo.list_translations()? {
            let verse_count = repo.verse_count(&translation.id)?;
            out.push(TranslationStatus {
                translation,
                verse_count,
            });
        }
        Ok(out)
    })
}

#[tauri::command]
pub fn list_books(
    state: State<'_, AppState>,
) -> CommandResult<Vec<crate::bible::models::BibleBook>> {
    state.with_conn(list_books_impl)
}

#[tauri::command]
pub fn get_passage(request: PassageRequest, state: State<'_, AppState>) -> CommandResult<Passage> {
    let reference = crate::scripture::ScriptureReference {
        book_id: request.book_id,
        chapter: request.chapter,
        start_verse: request.start_verse,
        end_verse: request.end_verse,
    };
    state.with_conn(|conn| {
        let repo = crate::bible::repository::BibleRepository::new(conn);
        crate::scripture::resolver::resolve_passage(&repo, &request.translation_id, &reference)
    })
}

/// Searches verse text, optionally across every installed translation.
///
/// `translation_id` may be:
///   * an id, e.g. `kjv` — search that translation only;
///   * `None`, `""` or `"*"` — search **every** translation, so a phrase can be
///     found without knowing which version it came from;
///   * `@all` — the same as `*`, kept for readability in logs.
#[tauri::command]
pub fn search_bible(
    translation_id: Option<String>,
    query: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> CommandResult<BibleSearchResult> {
    let limit = limit.unwrap_or(25);
    let query = query.trim().to_string();

    // A one-character query would match most of the Bible; treating it as "no
    // search yet" keeps the results meaningful.
    if query.chars().count() < 2 {
        return Ok(BibleSearchResult {
            verses: Vec::new(),
            related: false,
            suggestions: crate::bible::topics::suggest(&query, 6),
        });
    }

    let scope = translation_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty() && *id != "*" && *id != "@all");

    state.with_conn(|conn| {
        let repo = crate::bible::repository::BibleRepository::new(conn);

        // The operator's words first, exactly as typed. FTS5 treats several
        // characters as syntax, so the words are quoted before use.
        if let Some(terms) = search_terms(&query) {
            let exact = repo.search(scope, &terms, limit)?;
            if !exact.is_empty() {
                return Ok(BibleSearchResult {
                    verses: exact,
                    related: false,
                    suggestions: Vec::new(),
                });
            }
        }

        // Nothing matched. Try the words with a prefix so `shep` still finds
        // `shepherd`, and report those as "related" so the interface can say so.
        let widened = match prefix_terms(&query) {
            Some(terms) => repo.search(scope, &terms, limit)?,
            None => Vec::new(),
        };

        let suggestions = if widened.is_empty() {
            crate::bible::topics::suggest(&query, 6)
        } else {
            Vec::new()
        };

        Ok(BibleSearchResult {
            verses: widened,
            related: true,
            suggestions,
        })
    })
}

/// Turns free text into an FTS5 query, quoting each word so punctuation the
/// operator typed (`"`, `*`, `-`, `:`) cannot be read as search syntax.
///
/// `None` means there was nothing searchable to begin with.
fn search_terms(query: &str) -> Option<String> {
    let words: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|word| !word.is_empty())
        .map(|word| format!("\"{}\"", word.to_lowercase()))
        .collect();

    (!words.is_empty()).then(|| words.join(" AND "))
}

/// Like [`search_terms`], but every word may also match as a prefix, so a
/// half-remembered word still finds something.
fn prefix_terms(query: &str) -> Option<String> {
    let words: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|word| word.chars().count() >= 3)
        .map(|word| format!("\"{}\"*", word.to_lowercase()))
        .collect();

    (!words.is_empty()).then(|| words.join(" AND "))
}

#[tauri::command]
pub fn set_default_translation(
    translation_id: String,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    state.with_conn(|conn| {
        crate::bible::repository::BibleRepository::new(conn)
            .set_default_translation(&translation_id)
    })?;
    // Keep settings in sync so the UI has a single source of truth.
    if let Ok(mut settings) = state.settings.lock() {
        settings.general.default_translation_id = Some(translation_id);
    }
    state.save_settings()
}

/// Result of importing a translation document.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub translation_id: String,
    pub verses_imported: usize,
}

/// Request to import one of the common SQLite Bible downloads.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqliteImportRequest {
    /// Absolute path to the `.sqlite` file the operator chose.
    pub path: String,
    /// Short id used everywhere afterwards, e.g. `kjv`.
    pub translation_id: String,
    /// Full name shown in the interface, e.g. `King James Version`.
    pub name: String,
    pub abbreviation: Option<String>,
    /// Whether this should become the translation Selah starts with.
    #[serde(default)]
    pub make_default: bool,
}

/// Imports a translation from a SQLite file that uses the widely published
/// `verses(book_id, chapter, number, text)` layout.
///
/// Selah ships no Bible text; the operator points at a file they are entitled
/// to use. The file is attached read-only and copied in one transaction.
#[tauri::command]
pub fn import_sqlite_bible_translation(
    request: SqliteImportRequest,
    state: State<'_, AppState>,
) -> CommandResult<ImportResult> {
    let path = std::path::PathBuf::from(request.path.trim());
    if !path.is_absolute() {
        return Err(AppError::InvalidConfiguration(
            "the Bible file path must be absolute".to_string(),
        ));
    }
    if !path.is_file() {
        return Err(AppError::Media(format!(
            "Bible file does not exist: {}",
            path.display()
        )));
    }

    let translation_id = request.translation_id.trim().to_lowercase();
    if translation_id.is_empty() || request.name.trim().is_empty() {
        return Err(AppError::InvalidConfiguration(
            "a short id and a name are both required".to_string(),
        ));
    }

    let info = crate::bible::models::TranslationInfo {
        id: translation_id,
        name: request.name.trim().to_string(),
        language: "en".to_string(),
        abbreviation: request.abbreviation.filter(|a| !a.trim().is_empty()),
        is_default: request.make_default,
    };

    let outcome = state
        .with_conn(|conn| crate::bible::import::import_sqlite_translation(conn, &path, &info))?;

    if request.make_default {
        if let Ok(mut settings) = state.settings.lock() {
            settings.general.default_translation_id = Some(outcome.translation_id.clone());
        }
        state.save_settings()?;
    }

    Ok(ImportResult {
        translation_id: outcome.translation_id,
        verses_imported: outcome.verses_imported,
    })
}

/// Imports a translation from a local JSON document.
///
/// Selah ships no Bible text: the operator points at a file they are licensed
/// to use. The path is read directly (no shell, no globbing) and the whole
/// import runs inside a single transaction.
#[tauri::command]
pub fn import_bible_translation(
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<ImportResult> {
    let path = std::path::PathBuf::from(path.trim());
    if !path.is_absolute() {
        return Err(AppError::InvalidConfiguration(
            "the import path must be absolute".to_string(),
        ));
    }
    if !path.is_file() {
        return Err(AppError::Media(format!(
            "import file does not exist: {}",
            path.display()
        )));
    }

    let document = crate::bible::import::parse_import(&path)?;
    let (verses_imported, translation_id) =
        state.with_conn(|conn| crate::bible::import::apply_import(conn, &document))?;

    tracing::info!(
        translation = %translation_id,
        verses = verses_imported,
        "bible translation imported"
    );

    Ok(ImportResult {
        translation_id,
        verses_imported,
    })
}

// ---------------------------------------------------------
// Translations: what is available, and managing what is installed
// ---------------------------------------------------------

/// Every published translation Selah knows about, merged with what is
/// installed.
///
/// This is what the "Add a Translation" screen lists: nothing is downloaded,
/// and an entry that is not installed yet is there so the operator can add it
/// in one press rather than typing a name and hoping it matches.
#[tauri::command]
pub fn list_translation_catalogue(
    state: State<'_, AppState>,
) -> CommandResult<Vec<crate::bible::catalogue::CatalogueEntry>> {
    state.with_conn(|conn| {
        let repo = crate::bible::repository::BibleRepository::new(conn);
        let installed = repo.list_translations()?;

        let mut entries = crate::bible::catalogue::all();
        for entry in &mut entries {
            if let Some(found) = installed.iter().find(|t| t.id == entry.id) {
                entry.installed = true;
                entry.builtin = found.builtin;
                entry.verse_count = repo.verse_count(&found.id)?;
            }
        }

        // Anything installed that is not in the catalogue (imported from a
        // file, or added by hand) still belongs in the list.
        for translation in &installed {
            if entries.iter().any(|entry| entry.id == translation.id) {
                continue;
            }
            entries.push(crate::bible::catalogue::CatalogueEntry {
                id: translation.id.clone(),
                name: translation.name.clone(),
                abbreviation: translation
                    .abbreviation
                    .clone()
                    .unwrap_or_else(|| translation.name.clone()),
                language: translation.language.clone(),
                group: match translation.origin.as_str() {
                    "bundled" => "Selah's own".to_string(),
                    _ => "Added by you".to_string(),
                },
                builtin: translation.builtin,
                installed: true,
                verse_count: repo.verse_count(&translation.id)?,
                public_domain: false,
            });
        }

        Ok(entries)
    })
}

/// Turns a typed id into something the database and the picker can live with.
fn normalise_translation_id(input: &str) -> Result<String, AppError> {
    let cleaned: String = input
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let cleaned = cleaned.trim_matches('-').to_string();

    if cleaned.is_empty() {
        return Err(AppError::InvalidConfiguration(
            "give the translation a short id, such as msg or my-readings".to_string(),
        ));
    }
    Ok(cleaned.chars().take(MAX_CUSTOM_ID_CHARS).collect())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTranslationRequest {
    /// A published id (`msg`), or a short id the operator chooses.
    pub id: String,
    /// Only needed for a translation that is not in the published list.
    pub name: Option<String>,
    pub abbreviation: Option<String>,
    pub language: Option<String>,
}

/// Adds a translation so verses can be stored under it.
///
/// Adding one of the published translations takes its name, abbreviation and
/// language from the catalogue, so the picker stays consistent.
#[tauri::command]
pub fn add_translation(
    request: AddTranslationRequest,
    state: State<'_, AppState>,
) -> CommandResult<TranslationStatus> {
    let id = normalise_translation_id(&request.id)?;
    if crate::bible::catalogue::is_builtin(&id) {
        return Err(AppError::InvalidConfiguration(format!(
            "{id} is one of the Bibles that come with Selah, so it is already here"
        )));
    }

    let published = crate::bible::catalogue::find(&id);
    let name = published
        .map(|p| p.name().to_string())
        .or_else(|| {
            request
                .name
                .as_deref()
                .map(str::trim)
                .filter(|n| !n.is_empty())
                .map(str::to_string)
        })
        .ok_or_else(|| {
            AppError::InvalidConfiguration(
                "give the translation a name — that is what appears in the Bible list".to_string(),
            )
        })?;

    let abbreviation = published.map(|p| p.abbreviation().to_string()).or_else(|| {
        request
            .abbreviation
            .as_deref()
            .map(str::trim)
            .filter(|a| !a.is_empty())
            .map(str::to_string)
    });

    let language = published
        .map(|p| p.language().to_string())
        .unwrap_or_else(|| {
            request
                .language
                .as_deref()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .unwrap_or("en")
                .to_string()
        });

    let info = crate::bible::models::TranslationInfo {
        id: id.clone(),
        name,
        language,
        abbreviation,
        // Adding a Bible never silently changes what people preach from.
        is_default: false,
    };

    state.with_conn(|conn| {
        let repo = crate::bible::repository::BibleRepository::new(conn);
        if published.is_some() {
            repo.add_catalogue_translation(&info)?;
        } else {
            repo.upsert_translation(&info)?;
        }
        let translation = repo
            .get_translation(&id)?
            .ok_or_else(|| AppError::Internal("the translation was not stored".to_string()))?;
        Ok(TranslationStatus {
            verse_count: repo.verse_count(&id)?,
            translation,
        })
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTranslationRequest {
    pub id: String,
    pub name: String,
    pub abbreviation: Option<String>,
    pub language: Option<String>,
}

/// Renames a translation the operator added. The bundled three are refused.
#[tauri::command]
pub fn update_translation(
    request: UpdateTranslationRequest,
    state: State<'_, AppState>,
) -> CommandResult<Translation> {
    let name = request.name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidConfiguration(
            "a translation needs a name".to_string(),
        ));
    }

    let language = request
        .language
        .as_deref()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .unwrap_or("en");

    let abbreviation = request
        .abbreviation
        .as_deref()
        .map(str::trim)
        .filter(|a| !a.is_empty());

    state.with_conn(|conn| {
        let repo = crate::bible::repository::BibleRepository::new(conn);
        repo.rename_translation(&request.id, name, abbreviation, language)?;
        repo.get_translation(&request.id)?
            .ok_or_else(|| AppError::BibleTranslationNotFound(request.id.clone()))
    })
}

/// Removes a translation and its verses. The bundled three are refused.
#[tauri::command]
pub fn delete_translation(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    let removed_default = state.with_conn(|conn| {
        let repo = crate::bible::repository::BibleRepository::new(conn);
        let was_default = repo
            .get_translation(&id)?
            .map(|translation| translation.is_default)
            .unwrap_or(false);
        repo.delete_translation(&id)?;
        Ok(was_default)
    })?;

    // Settings hold the translation the Bible screen opens on: clear it if it
    // pointed at the one that just went away, so nothing dangles.
    if removed_default {
        if let Ok(mut settings) = state.settings.lock() {
            if settings.general.default_translation_id.as_deref() == Some(id.as_str()) {
                settings.general.default_translation_id = None;
            }
        }
        state.save_settings()?;
    }

    Ok(())
}

// ---------------------------------------------------------
// Verses for a translation (typed, or pasted from elsewhere)
// ---------------------------------------------------------

/// One verse the operator typed or pasted.
///
/// The book is given as text (`"John"`, `"1 Corinthians"`, `"Jn"`) and resolved
/// against Selah's canonical registry, so a typo is reported instead of being
/// written into the database.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomVerseInput {
    pub book: String,
    pub chapter: u16,
    pub verse: u16,
    pub text: String,
}

/// A whole chapter (or a single verse) of operator-supplied text.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCustomVersesRequest {
    /// Short id for the translation the verses belong to, e.g. `my-notes`.
    /// Defaults to `custom`.
    pub translation_id: Option<String>,
    /// Name shown in the Bible picker, e.g. `My notes`.
    pub name: Option<String>,
    pub abbreviation: Option<String>,
    pub verses: Vec<CustomVerseInput>,
}

/// What was stored, and anything the operator should know about it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomVerseImportResult {
    pub translation_id: String,
    pub reference: String,
    pub book_id: i32,
    pub chapter: u16,
    pub verses_saved: usize,
    /// Verse numbers inside the saved range that were not supplied.
    pub missing_verses: Vec<u16>,
}

/// Most verses Selah will accept in one paste. The longest chapter in the Bible
/// has 176 verses; the ceiling is a typo guard, not a limit on chapter size.
const MAX_CUSTOM_VERSES: usize = 400;
/// Longest single verse Selah will accept (the longest verse in the Bible is
/// well under this).
const MAX_CUSTOM_VERSE_CHARS: usize = 2000;
/// Highest verse number accepted, used only as a sanity bound.
const MAX_CUSTOM_VERSE_NUMBER: u16 = 200;
/// Shortest name a custom translation may have.
const MAX_CUSTOM_ID_CHARS: usize = 40;

/// Saves verses the operator supplied, under the translation they chose.
///
/// This is the manual-entry path used by the "Add a Translation" screen: the
/// words are the operator's responsibility, the *shape* is Selah's. The checks
/// are deliberately strict, because the text is coming from a person rather
/// than a published file:
///
///   1. every verse must name a **real book**, resolved through the canonical
///      registry (names, abbreviations and spoken aliases all work);
///   2. the chapter and verse numbers must exist within that book;
///   3. all verses must be from **one book and one chapter** — the paste is a
///      chapter, not a mixed list;
///   4. verse numbers must be unique, and their text non-empty;
///   5. the text must be a verse, not markup: a JSON blob or a whole paragraph
///      is rejected with an explanation rather than quietly stored.
///
/// The three Bibles Selah ships are read-only, so verses may not be written
/// into them; the operator adds a translation and stores their words there.
#[tauri::command]
pub fn save_custom_verses(
    request: SaveCustomVersesRequest,
    state: State<'_, AppState>,
) -> CommandResult<CustomVerseImportResult> {
    let translation_id = normalise_custom_id(request.translation_id.as_deref());

    let builtin = state.with_conn(|conn| {
        Ok(crate::bible::repository::BibleRepository::new(conn)
            .get_translation(&translation_id)?
            .map(|translation| (translation.builtin, translation.name)))
    })?;

    if let Some((true, name)) = builtin {
        return Err(AppError::InvalidConfiguration(format!(
            "{name} is one of the Bibles that come with Selah, so it cannot be changed. \
             Add a translation of your own and put the verses there."
        )));
    }

    let checked = validate_custom_verses(&request)?;

    let name = request
        .name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .unwrap_or("My verses")
        .to_string();

    let info = crate::bible::models::TranslationInfo {
        id: translation_id.clone(),
        name,
        language: "en".to_string(),
        abbreviation: request
            .abbreviation
            .as_deref()
            .map(str::trim)
            .filter(|a| !a.is_empty())
            .map(str::to_string),
        // Never silently replaces the translation the operator preaches from.
        is_default: false,
    };

    let saved = state.with_conn(|conn| {
        crate::bible::repository::BibleRepository::new(conn).save_verses(&info, &checked.verses)
    })?;

    tracing::info!(
        translation = %translation_id,
        reference = %checked.reference,
        verses = saved,
        "custom bible verses saved"
    );

    Ok(CustomVerseImportResult {
        translation_id,
        reference: checked.reference,
        book_id: checked.book_id,
        chapter: checked.chapter,
        verses_saved: saved,
        missing_verses: checked.missing_verses,
    })
}

/// Verses that survived validation, plus what was noticed along the way.
#[derive(Debug)]
struct CheckedVerses {
    verses: Vec<Verse>,
    book_id: i32,
    chapter: u16,
    reference: String,
    missing_verses: Vec<u16>,
}

/// Validates a paste and converts it into stored verses.
fn validate_custom_verses(request: &SaveCustomVersesRequest) -> Result<CheckedVerses, AppError> {
    if request.verses.is_empty() {
        return Err(AppError::InvalidConfiguration(
            "there are no verses to save".to_string(),
        ));
    }
    if request.verses.len() > MAX_CUSTOM_VERSES {
        return Err(AppError::InvalidConfiguration(format!(
            "{} verses is more than one chapter; paste a single chapter (up to {MAX_CUSTOM_VERSES} verses)",
            request.verses.len()
        )));
    }

    let translation_id = normalise_custom_id(request.translation_id.as_deref());

    // The first verse fixes the book and chapter; everything else must agree.
    let first_book = resolve_book_name(&request.verses[0].book)?;
    let chapter = request.verses[0].chapter;
    validate_chapter(first_book, chapter)?;

    let mut verses = Vec::with_capacity(request.verses.len());
    let mut seen: Vec<u16> = Vec::with_capacity(request.verses.len());

    for entry in &request.verses {
        let book = resolve_book_name(&entry.book)?;
        if book.id != first_book.id {
            return Err(AppError::InvalidConfiguration(format!(
                "this paste mixes {} with {}; save one chapter at a time",
                first_book.name, book.name
            )));
        }
        if entry.chapter != chapter {
            return Err(AppError::InvalidConfiguration(format!(
                "this paste mixes chapter {} with chapter {}; save one chapter at a time",
                chapter, entry.chapter
            )));
        }
        if entry.verse < 1 || entry.verse > MAX_CUSTOM_VERSE_NUMBER {
            return Err(AppError::InvalidConfiguration(format!(
                "verse number {} is not a verse number",
                entry.verse
            )));
        }
        if seen.contains(&entry.verse) {
            return Err(AppError::InvalidConfiguration(format!(
                "verse {} appears more than once",
                entry.verse
            )));
        }
        seen.push(entry.verse);

        let text = check_verse_text(&entry.text, entry.verse)?;
        verses.push(Verse {
            translation_id: translation_id.clone(),
            book_id: book.id,
            chapter: entry.chapter as i32,
            verse: entry.verse as i32,
            text,
        });
    }

    verses.sort_by_key(|verse| verse.verse);
    let missing_verses = gaps_between(&seen);

    Ok(CheckedVerses {
        verses,
        book_id: first_book.id,
        chapter,
        reference: format!("{} {} ({} verses)", first_book.name, chapter, seen.len()),
        missing_verses,
    })
}

/// Resolves a book written in any recognised form: full name, abbreviation or
/// spoken alias. Numeric ids are accepted too, so a pasted array generated by
/// an AI agent with `"bookId": 43` still works.
fn resolve_book_name(book: &str) -> Result<&'static crate::scripture::books::BibleBook, AppError> {
    let trimmed = book.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidConfiguration(
            "every verse needs a book name".to_string(),
        ));
    }

    if let Ok(id) = trimmed.parse::<i32>() {
        return crate::scripture::books::book_by_id(id).ok_or_else(|| {
            AppError::InvalidConfiguration(format!("there is no book number {id}"))
        });
    }

    if let Some(found) = crate::scripture::books::book_by_name(trimmed) {
        return Ok(found);
    }

    // Aliases can be more than one word (`1 corinthians`, `song of songs`), so
    // they are matched against every token of the name.
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    match crate::scripture::books::find_book(&tokens, 0) {
        Some((found, consumed)) if consumed == tokens.len() => Ok(found),
        _ => Err(AppError::InvalidConfiguration(format!(
            "\"{trimmed}\" is not a book of the Bible Selah knows"
        ))),
    }
}

/// Rejects chapters that cannot exist in that book.
fn validate_chapter(
    book: &crate::scripture::books::BibleBook,
    chapter: u16,
) -> Result<(), AppError> {
    if chapter < 1 || chapter > book.chapters {
        return Err(AppError::InvalidConfiguration(format!(
            "{} has {} chapters, so there is no chapter {chapter}",
            book.name, book.chapters
        )));
    }
    Ok(())
}

/// Checks a verse's words and returns them trimmed.
///
/// The point of this function is to catch the two mistakes that actually
/// happen when text is pasted in: markup (a JSON object or a whole chapter
/// dropped in by mistake) and emptiness.
fn check_verse_text(text: &str, verse: u16) -> Result<String, AppError> {
    let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");

    if trimmed.is_empty() {
        return Err(AppError::InvalidConfiguration(format!(
            "verse {verse} has no words"
        )));
    }
    if trimmed.chars().count() > MAX_CUSTOM_VERSE_CHARS {
        return Err(AppError::InvalidConfiguration(format!(
            "verse {verse} has {} characters, which is longer than a verse",
            trimmed.chars().count()
        )));
    }
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return Err(AppError::InvalidConfiguration(format!(
            "verse {verse} looks like code rather than verse text — only the words belong here"
        )));
    }
    if !trimmed.chars().any(|c| c.is_alphabetic()) {
        return Err(AppError::InvalidConfiguration(format!(
            "verse {verse} has no words, only numbers or punctuation"
        )));
    }
    if looks_like_metadata(&trimmed) {
        return Err(AppError::InvalidConfiguration(format!(
            "verse {verse} still has its label on it — remove the book, chapter and verse number, \
             and keep only the words"
        )));
    }

    Ok(trimmed)
}

/// Whether a line still carries an AI/reference label such as
/// `John 3:16 — For God so loved`, or a `verse:`/`text:` field name.
fn looks_like_metadata(text: &str) -> bool {
    let lowered = text.to_lowercase();
    if lowered.starts_with("verse:") || lowered.starts_with("text:") {
        return true;
    }
    // `Something 3:16` at the very start of the line.
    let head: String = lowered.chars().take(40).collect();
    let mut digits = 0;
    let mut colon = false;
    for ch in head.chars() {
        if ch.is_ascii_digit() {
            digits += 1;
        } else if ch == ':' {
            colon = true;
        }
    }
    colon
        && digits > 0
        && head.split(':').next().is_some_and(|left| {
            left.split_whitespace()
                .last()
                .is_some_and(|word| word.chars().all(|c| c.is_ascii_digit()))
        })
}

/// Verse numbers between the supplied ones that were not supplied.
fn gaps_between(verses: &[u16]) -> Vec<u16> {
    let mut sorted = verses.to_vec();
    sorted.sort_unstable();
    sorted.dedup();

    let (Some(first), Some(last)) = (sorted.first(), sorted.last()) else {
        return Vec::new();
    };
    (*first..=*last)
        .filter(|number| !sorted.contains(number))
        .collect()
}

/// Turns whatever the interface sent into a safe, stable translation id.
fn normalise_custom_id(id: Option<&str>) -> String {
    id.map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| normalise_translation_id(value).ok())
        .unwrap_or_else(|| "custom".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verse(book: &str, chapter: u16, verse: u16, text: &str) -> CustomVerseInput {
        CustomVerseInput {
            book: book.to_string(),
            chapter,
            verse,
            text: text.to_string(),
        }
    }

    fn request(verses: Vec<CustomVerseInput>) -> SaveCustomVersesRequest {
        SaveCustomVersesRequest {
            translation_id: None,
            name: None,
            abbreviation: None,
            verses,
        }
    }

    #[test]
    fn a_pasted_chapter_is_checked_and_ordered() {
        let checked = validate_custom_verses(&request(vec![
            verse("Genesis", 1, 2, "And the earth was without form"),
            verse("Genesis", 1, 1, "In the beginning God created"),
        ]))
        .expect("should be accepted");

        assert_eq!(checked.book_id, 1);
        assert_eq!(checked.chapter, 1);
        assert_eq!(checked.verses.len(), 2);
        // Sorted, so the stored chapter reads in order.
        assert_eq!(checked.verses[0].verse, 1);
        assert_eq!(checked.reference, "Genesis 1 (2 verses)");
        assert!(checked.missing_verses.is_empty());
    }

    #[test]
    fn abbreviations_and_book_numbers_both_resolve() {
        // The format an AI is asked for uses the full name, but an operator
        // typing by hand may use either.
        for name in ["Jn", "John", "43", "jhn"] {
            let checked =
                validate_custom_verses(&request(vec![verse(name, 3, 16, "For God so loved")]))
                    .unwrap_or_else(|e| panic!("{name} should resolve: {e}"));
            assert_eq!(checked.book_id, 43, "book given as {name}");
        }
    }

    #[test]
    fn a_book_selah_does_not_know_is_reported_by_name() {
        let error = validate_custom_verses(&request(vec![verse("Hezekiah", 1, 1, "words")]))
            .expect_err("should be refused");
        assert!(error.to_string().contains("Hezekiah"));
    }

    #[test]
    fn a_paste_that_mixes_books_or_chapters_is_refused() {
        let mixed_books = validate_custom_verses(&request(vec![
            verse("John", 3, 16, "one"),
            verse("Luke", 3, 17, "two"),
        ]))
        .expect_err("should be refused");
        assert!(mixed_books.to_string().contains("one chapter at a time"));

        let mixed_chapters = validate_custom_verses(&request(vec![
            verse("John", 3, 16, "one"),
            verse("John", 4, 1, "two"),
        ]))
        .expect_err("should be refused");
        assert!(mixed_chapters.to_string().contains("one chapter at a time"));
    }

    #[test]
    fn a_chapter_that_does_not_exist_is_refused() {
        // Jude has one chapter; chapter two is a typo, not a request.
        let error = validate_custom_verses(&request(vec![verse("Jude", 2, 1, "words")]))
            .expect_err("should be refused");
        assert!(error.to_string().contains("Jude has 1 chapters"));
    }

    #[test]
    fn repeated_verse_numbers_are_refused() {
        let error = validate_custom_verses(&request(vec![
            verse("John", 3, 16, "one"),
            verse("John", 3, 16, "again"),
        ]))
        .expect_err("should be refused");
        assert!(error.to_string().contains("appears more than once"));
    }

    #[test]
    fn gaps_are_reported_but_not_treated_as_a_failure() {
        let checked = validate_custom_verses(&request(vec![
            verse("John", 3, 16, "one"),
            verse("John", 3, 19, "four"),
        ]))
        .expect("gaps are fine");
        assert_eq!(checked.missing_verses, vec![17, 18]);
    }

    #[test]
    fn an_empty_paste_is_refused() {
        let error = validate_custom_verses(&request(vec![])).expect_err("should be refused");
        assert!(error.to_string().contains("no verses"));
    }
}

#[cfg(test)]
mod text_and_search_tests {
    use super::*;

    #[test]
    fn markup_and_labels_are_refused_as_verse_text() {
        // The two mistakes that actually happen when text is pasted in.
        let code = validate_custom_verses(&request(vec![CustomVerseInput {
            book: "John".to_string(),
            chapter: 3,
            verse: 16,
            text: "{\"book\":\"John\"}".to_string(),
        }]))
        .expect_err("should be refused");
        assert!(code.to_string().contains("looks like code"));

        let labelled =
            check_verse_text("John 3:16 — For God so loved", 16).expect_err("should be refused");
        assert!(labelled.to_string().contains("remove the book, chapter"));

        let empty = check_verse_text("   ", 16).expect_err("should be refused");
        assert!(empty.to_string().contains("no words"));
    }

    #[test]
    fn words_are_tidied_before_they_are_stored() {
        assert_eq!(
            check_verse_text("  For   God\nso loved  ", 16).unwrap(),
            "For God so loved"
        );
    }

    #[test]
    fn a_custom_translation_id_cannot_break_the_database() {
        assert_eq!(normalise_custom_id(None), "custom");
        assert_eq!(normalise_custom_id(Some("   ")), "custom");
        assert_eq!(normalise_custom_id(Some("My Notes!")), "my-notes");
        assert_eq!(normalise_custom_id(Some("--weird--")), "weird");
        // Long ids are cut to a length the schema is happy with.
        let long = "a".repeat(200);
        assert_eq!(
            normalise_custom_id(Some(&long)).chars().count(),
            MAX_CUSTOM_ID_CHARS
        );
    }

    #[test]
    fn search_words_are_quoted_so_punctuation_cannot_break_the_query() {
        // FTS5 reads `"` and `*` as syntax; an operator typing them must not
        // produce an SQL error.
        assert_eq!(
            search_terms("God so loved").as_deref(),
            Some("\"god\" AND \"so\" AND \"loved\"")
        );
        assert_eq!(
            search_terms("shep*herd").as_deref(),
            Some("\"shep\" AND \"herd\"")
        );
        // A query with nothing searchable in it is not run at all.
        assert_eq!(search_terms("!!! ???"), None);
        assert_eq!(prefix_terms("!!"), None);
        assert_eq!(prefix_terms("shepherd").as_deref(), Some("\"shepherd\"*"));
        // Very short words are left out of the prefix pass: `"he"*` would match
        // most of the Bible, which is not a helpful "closest match".
        assert_eq!(prefix_terms("he is good").as_deref(), Some("\"good\"*"));
    }

    fn request(verses: Vec<CustomVerseInput>) -> SaveCustomVersesRequest {
        SaveCustomVersesRequest {
            translation_id: None,
            name: None,
            abbreviation: None,
            verses,
        }
    }
}
