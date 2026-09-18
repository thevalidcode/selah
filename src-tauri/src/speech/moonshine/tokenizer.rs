//! Moonshine tokenizer: turns generated token ids back into readable text.
//!
//! Moonshine ships a SentencePiece-style vocabulary inside `tokenizer.json`.
//! The decoder chain that file describes is:
//!
//!   1. `Replace("\u{2581}", " ")` — the word-boundary marker becomes a space
//!   2. `ByteFallback`            — `<0xNN>` pieces stand for a single byte
//!   3. `Fuse` / `Strip`          — join the pieces, drop the leading space
//!
//! Reproducing those steps is cheaper (and far smaller) than pulling in a
//! full tokenizer runtime for a vocabulary that is only ever decoded.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::errors::AppError;

/// The SentencePiece word-boundary marker used by Moonshine.
const WORD_BOUNDARY: char = '\u{2581}'; // ▁

/// Decodes Moonshine token ids back into text.
pub struct MoonshineTokenizer {
    /// token id -> sub-word piece
    vocab: HashMap<u32, String>,
    /// Ids that must never reach the output: `<s>`, `</s>`, `<<ST_n>>`, ...
    special_ids: Vec<u32>,
}

impl MoonshineTokenizer {
    /// Reads `tokenizer.json` from a Moonshine model directory.
    pub fn load(model_dir: &Path) -> Result<Self, AppError> {
        let path = model_dir.join("tokenizer.json");
        if !path.is_file() {
            return Err(AppError::SpeechModelNotFound(path.display().to_string()));
        }

        let file = File::open(&path).map_err(|e| {
            AppError::SpeechModelNotFound(format!("cannot read {}: {e}", path.display()))
        })?;
        let json: serde_json::Value = serde_json::from_reader(BufReader::new(file))
            .map_err(|e| AppError::InvalidConfiguration(format!("invalid tokenizer.json: {e}")))?;

        let vocab = read_vocab(&json)?;
        if vocab.is_empty() {
            return Err(AppError::InvalidConfiguration(
                "tokenizer.json contains no vocabulary".to_string(),
            ));
        }

        Ok(Self {
            vocab,
            special_ids: read_special_ids(&json),
        })
    }

    /// Number of entries in the vocabulary (used for sanity checks).
    pub fn len(&self) -> usize {
        self.vocab.len()
    }

    /// Whether the vocabulary is empty. Always false for a loaded tokenizer.
    pub fn is_empty(&self) -> bool {
        self.vocab.is_empty()
    }

    /// Decodes token ids into a cleaned-up string.
    ///
    /// Special tokens are dropped and the leading word-boundary space is
    /// trimmed, so `"▁the▁lord"` becomes `"the lord"` not `" the lord"`.
    pub fn decode(&self, ids: &[i64]) -> String {
        let mut bytes: Vec<u8> = Vec::new();

        for &id in ids {
            let id = id as u32;
            if self.special_ids.contains(&id) {
                continue;
            }
            let Some(piece) = self.vocab.get(&id) else {
                continue;
            };

            // ByteFallback pieces encode one raw byte directly.
            if let Some(byte) = parse_byte_piece(piece) {
                bytes.push(byte);
                continue;
            }

            for ch in piece.chars() {
                if ch == WORD_BOUNDARY {
                    bytes.push(b' ');
                } else {
                    let mut buf = [0u8; 4];
                    bytes.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                }
            }
        }

        let text = String::from_utf8_lossy(&bytes);
        text.strip_prefix(' ').unwrap_or(&text).to_string()
    }
}

/// Extracts `model.vocab` (`token -> id`, inverted to `id -> token`).
fn read_vocab(json: &serde_json::Value) -> Result<HashMap<u32, String>, AppError> {
    let Some(entries) = json
        .get("model")
        .and_then(|model| model.get("vocab"))
        .and_then(|vocab| vocab.as_object())
    else {
        return Err(AppError::InvalidConfiguration(
            "tokenizer.json is missing model.vocab".to_string(),
        ));
    };

    Ok(entries
        .iter()
        .filter_map(|(token, id)| id.as_u64().map(|id| (id as u32, token.clone())))
        .collect())
}

/// Collects ids flagged `"special": true` in `added_tokens`.
fn read_special_ids(json: &serde_json::Value) -> Vec<u32> {
    json.get("added_tokens")
        .and_then(|added| added.as_array())
        .map(|added| {
            added
                .iter()
                .filter(|token| {
                    token
                        .get("special")
                        .and_then(|special| special.as_bool())
                        .unwrap_or(false)
                })
                .filter_map(|token| token.get("id").and_then(|id| id.as_u64()))
                .map(|id| id as u32)
                .collect()
        })
        .unwrap_or_default()
}

/// Parses a `<0xNN>` byte-fallback piece into the byte it represents.
fn parse_byte_piece(piece: &str) -> Option<u8> {
    let inner = piece.strip_prefix("<0x")?.strip_suffix('>')?;
    u8::from_str_radix(inner, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenizer(vocab: &[(u32, &str)], specials: &[u32]) -> MoonshineTokenizer {
        MoonshineTokenizer {
            vocab: vocab
                .iter()
                .map(|(id, token)| (*id, (*token).to_string()))
                .collect(),
            special_ids: specials.to_vec(),
        }
    }

    #[test]
    fn decodes_word_boundaries_as_spaces() {
        let t = tokenizer(&[(10, "\u{2581}the"), (11, "\u{2581}lord")], &[1, 2]);
        assert_eq!(t.decode(&[10, 11]), "the lord");
    }

    #[test]
    fn drops_special_tokens() {
        let t = tokenizer(&[(1, "<s>"), (10, "hello"), (2, "</s>")], &[1, 2]);
        assert_eq!(t.decode(&[1, 10, 2]), "hello");
    }

    #[test]
    fn reassembles_byte_fallback_pieces() {
        // "é" is absent from this toy vocabulary, so it arrives as two bytes.
        let t = tokenizer(&[(10, "\u{2581}caf"), (11, "<0xC3>"), (12, "<0xA9>")], &[]);
        assert_eq!(t.decode(&[10, 11, 12]), "café");
    }

    #[test]
    fn ignores_unknown_ids() {
        let t = tokenizer(&[(10, "ok")], &[]);
        assert_eq!(t.decode(&[9999, 10]), "ok");
    }

    #[test]
    fn parses_byte_pieces() {
        assert_eq!(parse_byte_piece("<0x41>"), Some(0x41));
        assert_eq!(parse_byte_piece("<0xff>"), Some(0xff));
        assert_eq!(parse_byte_piece("\u{2581}word"), None);
        assert_eq!(parse_byte_piece("<0xZZ>"), None);
    }
}
