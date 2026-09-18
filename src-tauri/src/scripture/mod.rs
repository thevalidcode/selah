//! Scripture domain: deterministic book registry, normalized parsing and
//! typed references.

pub mod books;
pub mod normalizer;
pub mod parser;
pub mod reference;
pub mod resolver;

pub use books::{book_by_id, find_book, BibleBook, Testament};
pub use normalizer::normalize_text;
pub use parser::{find_references, parse_reference, tokenize};
pub use reference::ScriptureReference;
pub use resolver::resolve_passage;
