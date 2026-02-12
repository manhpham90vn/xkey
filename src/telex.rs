//! Telex Transformation Engine for Vietnamese Input
//!
//! This module implements the Vietnamese Telex input method, which allows users to type
//! Vietnamese characters using only ASCII keys. Telex is one of the most popular input
//! methods for Vietnamese, alongside VNI and VIQR.
//!
//! # Telex Rules
//!
//! ## Vowel Diacritics (Marks)
//! - `aa` → `â` (a with circumflex)
//! - `aw` → `ă` (a with breve)
//! - `ee` → `ê` (e with circumflex)
//! - `oo` → `ô` (o with circumflex)
//! - `ow` → `ơ` (o with horn)
//! - `uw` → `ư` (u with horn)
//! - `dd` → `đ` (d with stroke)
//! - `w` alone → `ư` (shortcut)
//!
//! ## Tone Marks
//! - `s` → acute accent (sắc): á, é, í, ó, ú, ý
//! - `f` → grave accent (huyền): à, è, ì, ò, ù, ỳ
//! - `r` → hook above (hỏi): ả, ẻ, ỉ, ỏ, ủ, ỷ
//! - `x` → tilde (ngã): ã, ẽ, ĩ, õ, ũ, ỹ
//! - `j` → dot below (nặng): ạ, ẹ, ị, ọ, ụ, ỵ
//!
//! ## Tone Placement Rules
//! Vietnamese has specific rules for where tone marks should be placed:
//! 1. If there's a vowel with a diacritic (â, ă, ê, ô, ơ, ư), place tone on it
//! 2. Otherwise, if the word ends with a consonant, place tone on the last vowel
//! 3. Otherwise, place tone on the second-to-last vowel (if multiple vowels)
//! 4. Special handling for 'qu' and 'gi' where u/i are semi-consonants
//!
//! # Examples
//! ```
//! use xkey::telex::transform_buffer;
//! assert_eq!(transform_buffer("vieetj"), "việt");
//! assert_eq!(transform_buffer("chaof"), "chào");
//! assert_eq!(transform_buffer("Vieejt Nam"), "Việt Nam");
//! ```

/// Processes an entire buffer (potentially containing multiple words separated by
/// whitespace or punctuation) and transforms it into Vietnamese text.
///
/// This is the main entry point for Telex transformation. It handles:
/// - Multiple words separated by spaces
/// - Punctuation preservation
/// - Individual word processing via `process_word`
///
/// # Arguments
/// * `buffer` - Raw Telex input string
///
/// # Returns
/// Transformed Vietnamese string with proper diacritics and tone marks
///
/// # Examples
/// ```
/// assert_eq!(transform_buffer("xin chaof"), "xin chào");
/// assert_eq!(transform_buffer("vieetj,nam"), "việt,nam");
/// ```
pub fn transform_buffer(buffer: &str) -> String {
    let mut result = String::new();
    let mut current_word = String::new();

    // Iterate through characters to separate words by whitespace or punctuation
    for ch in buffer.chars() {
        if ch.is_whitespace() || (ch.is_ascii_punctuation() && ch != '[' && ch != ']') {
            // Process the accumulated word before adding the separator
            result.push_str(&process_word(&current_word));
            result.push(ch);
            current_word.clear();
        } else {
            current_word.push(ch);
        }
    }
    // Process the final word in the buffer (no trailing separator)
    result.push_str(&process_word(&current_word));
    result
}

/// Unicode-aware lowercase conversion for single chars.
///
/// `to_ascii_lowercase` is insufficient for Vietnamese uppercase letters
/// like `Ư`, `Ơ`, `Â`, etc.
fn lower_char(ch: char) -> char {
    ch.to_lowercase().next().unwrap_or(ch)
}

/// Transforms a single word based on Telex rules.
///
/// This function processes a single word (no whitespace or punctuation) and applies:
/// 1. Vowel mark transformations (aa→â, aw→ă, etc.)
/// 2. Tone mark collection and application
/// 3. Case preservation
///
/// # Processing Steps
/// 1. Scan the word character by character
/// 2. Detect and apply vowel mark combinations (aa, aw, ee, oo, ow, uw, dd)
/// 3. Collect tone marks (s, f, r, x, j) - only applied if there's a vowel
/// 4. Handle 'w' shortcut for ư and ơ
/// 5. Apply tone marks to the appropriate vowel based on Vietnamese rules
/// 6. Preserve original case (uppercase/lowercase)
///
/// # Arguments
/// * `word` - A single word without whitespace or punctuation
///
/// # Returns
/// Transformed Vietnamese word with diacritics and tone marks
fn process_word(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    let mut out_chars = Vec::new(); // Accumulated output characters
    let mut out_upper = Vec::new(); // Tracks case for each character (true = uppercase)
    let mut tone: Option<char> = None; // Current tone mark (s, f, r, x, j) or None
    let mut bypass = false; // When true, skip Telex rules (word is English/raw)

    let original_chars: Vec<char> = word.chars().collect();
    let mut i = 0;

    // Process each character, looking ahead for two-character combinations
    while i < original_chars.len() {
        let current = original_chars[i];
        let current_lower = current.to_ascii_lowercase();

        // If bypass mode is active, push raw characters without any transformation
        if bypass {
            out_chars.push(current_lower);
            out_upper.push(current.is_uppercase());
            i += 1;
            continue;
        }

        let next = original_chars.get(i + 1).cloned();
        let next_lower = next.map(|c| c.to_ascii_lowercase());
        let third_lower = original_chars.get(i + 2).map(|c| c.to_ascii_lowercase());

        // Pattern matching for two-character vowel mark combinations and special cases
        match (current_lower, next_lower, third_lower) {
            // Triple-key undo: aaa → aa (undo â, activate bypass)
            ('a', Some('a'), Some('a')) => {
                out_chars.push('a');
                out_upper.push(original_chars[i].is_uppercase());
                out_chars.push('a');
                out_upper.push(original_chars[i + 1].is_uppercase());
                bypass = true;
                i += 3;
            }
            // Triple-key undo: eee → ee (undo ê, activate bypass)
            ('e', Some('e'), Some('e')) => {
                out_chars.push('e');
                out_upper.push(original_chars[i].is_uppercase());
                out_chars.push('e');
                out_upper.push(original_chars[i + 1].is_uppercase());
                bypass = true;
                i += 3;
            }
            // Triple-key undo: ooo → oo (undo ô, activate bypass)
            ('o', Some('o'), Some('o')) => {
                out_chars.push('o');
                out_upper.push(original_chars[i].is_uppercase());
                out_chars.push('o');
                out_upper.push(original_chars[i + 1].is_uppercase());
                bypass = true;
                i += 3;
            }

            // Circumflex (^) vowels
            ('a', Some('a'), _) => {
                // aa → â (a with circumflex)
                out_chars.push('â');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('e', Some('e'), _) => {
                // ee → ê (e with circumflex)
                out_chars.push('ê');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('o', Some('o'), _) => {
                // oo → ô (o with circumflex)
                out_chars.push('ô');
                out_upper.push(current.is_uppercase());
                i += 2;
            }

            // Breve (˘) and horn (ˆ) vowels using 'w'
            ('a', Some('w'), _) => {
                // aw → ă (a with breve)
                out_chars.push('ă');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('o', Some('w'), _) => {
                // ow → ơ (o with horn)
                out_chars.push('ơ');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('u', Some('w'), _) => {
                // uw → ư (u with horn)
                out_chars.push('ư');
                out_upper.push(current.is_uppercase());
                i += 2;
            }

            // Triple-key undo: ddd → dd (undo đ, activate bypass)
            ('d', Some('d'), Some('d')) => {
                out_chars.push('d');
                out_upper.push(original_chars[i].is_uppercase());
                out_chars.push('d');
                out_upper.push(original_chars[i + 1].is_uppercase());
                bypass = true;
                i += 3;
            }
            // Đ with stroke
            ('d', Some('d'), _) => {
                // dd → đ (d with stroke)
                out_chars.push('đ');
                out_upper.push(current.is_uppercase());
                i += 2;
            }

            // Double-w undo: 'ww' after a vowel that was transformed → undo and output raw 'w'
            // e.g., "uww" → "uw", "oww" → "ow", "ww" → "w"
            ('w', Some('w'), _) => {
                // Check if previous char was transformed to ư or ơ by a prior 'w'
                if let Some(&last) = out_chars.last() {
                    let last_lower = lower_char(last);
                    if last_lower == 'ư' {
                        // Undo ư → u, then add literal 'w'
                        let was_upper = *out_upper.last().unwrap_or(&false);
                        out_chars.pop();
                        out_upper.pop();
                        out_chars.push('u');
                        out_upper.push(was_upper);
                        out_chars.push('w');
                        out_upper.push(original_chars[i].is_uppercase());
                        bypass = true;
                    } else if last_lower == 'ơ' {
                        // Undo ơ → o, then add literal 'w'
                        let was_upper = *out_upper.last().unwrap_or(&false);
                        out_chars.pop();
                        out_upper.pop();
                        out_chars.push('o');
                        out_upper.push(was_upper);
                        out_chars.push('w');
                        out_upper.push(original_chars[i].is_uppercase());
                        bypass = true;
                    } else {
                        // Previous char wasn't transformed, just output 'w'
                        out_chars.push('w');
                        out_upper.push(original_chars[i].is_uppercase());
                        bypass = true;
                    }
                } else {
                    // 'ww' at start of word → just 'w'
                    out_chars.push('w');
                    out_upper.push(original_chars[i].is_uppercase());
                    bypass = true;
                }
                i += 2;
            }

            // Standalone 'w' shortcut for ư or ơ
            ('w', _, _) => {
                // 'w' can transform the previous vowel or become 'ư' on its own
                if let Some(&last) = out_chars.last() {
                    let last_lower = lower_char(last);
                    if last_lower == 'u' {
                        // Replace 'u' with 'ư'
                        out_chars.pop();
                        out_chars.push('ư');
                    } else if last_lower == 'o' {
                        // Replace 'o' with 'ơ'
                        out_chars.pop();
                        out_chars.push('ơ');
                    } else if last_lower == 'ă' {
                        // Undo ă → a, then add literal 'w'
                        // This handles the "awws" -> "aws" case
                        let was_upper = *out_upper.last().unwrap_or(&false);
                        out_chars.pop();
                        out_upper.pop();
                        out_chars.push('a');
                        out_upper.push(was_upper);
                        out_chars.push('w');
                        out_upper.push(current.is_uppercase());
                        bypass = true;
                    } else {
                        // No preceding u/o/ă, 'w' becomes 'ư'
                        out_chars.push('ư');
                        out_upper.push(current.is_uppercase());
                    }
                } else {
                    // 'w' at start of word becomes 'ư'
                    out_chars.push('ư');
                    out_upper.push(current.is_uppercase());
                }
                i += 1;
            }

            // Bracket shortcuts: '[' → ư, ']' → ơ
            ('[', _, _) => {
                out_chars.push('ư');
                out_upper.push(false);
                i += 1;
            }
            (']', _, _) => {
                out_chars.push('ơ');
                out_upper.push(false);
                i += 1;
            }

            // Tone mark characters (only apply if we have a vowel to mark)
            ('s', _, _) | ('f', _, _) | ('r', _, _) | ('x', _, _) | ('j', _, _) => {
                let vowels = "aeiouyâăêôơư";
                let has_vowel = out_chars.iter().any(|c| vowels.contains(lower_char(*c)));

                if has_vowel {
                    // Double-tap behavior: typing the same tone twice
                    // outputs the literal character (e.g., "ass" → "as")
                    // Also activates bypass mode (word is likely English)
                    if tone == Some(current_lower) {
                        tone = None;
                        out_chars.push(current_lower);
                        out_upper.push(current.is_uppercase());
                        bypass = true;
                    } else {
                        tone = Some(current_lower);
                    }
                } else {
                    // No vowel yet, treat as a regular consonant character
                    // (e.g., "s" at the start of a word)
                    out_chars.push(current);
                    out_upper.push(current.is_uppercase());
                }
                i += 1;
            }

            // 'z' key: remove any active tone mark
            // Also activates bypass mode (word is likely English)
            ('z', _, _) => {
                let vowels = "aeiouyâăêôơư";
                let has_vowel = out_chars.iter().any(|c| vowels.contains(lower_char(*c)));

                if has_vowel && tone.is_some() {
                    // Remove the tone mark and activate bypass
                    tone = None;
                    bypass = true;
                } else {
                    // No tone to remove or no vowel, treat as regular character
                    out_chars.push(current);
                    out_upper.push(current.is_uppercase());
                }
                i += 1;
            }

            // Regular character, just pass it through unchanged
            (c, _, _) => {
                out_chars.push(c);
                out_upper.push(current.is_uppercase());
                i += 1;
            }
        }
    }

    // Convert character vector to string for further processing
    let mut out_str: String = out_chars.into_iter().collect();

    // If bypass mode was triggered, skip all Vietnamese-specific post-processing.
    // Just restore case and return raw output.
    if bypass {
        let mut result = String::new();
        for (i, ch) in out_str.chars().enumerate() {
            if *out_upper.get(i).unwrap_or(&false) {
                result.push(ch.to_uppercase().next().unwrap_or(ch));
            } else {
                result.push(ch);
            }
        }
        return result;
    }

    // Special rule: "uơ" should become "ươ" for words like "hươu" (deer)
    // This handles the case where 'ư' and 'ơ' appear together
    if out_str.contains("uơ") {
        out_str = out_str.replace("uơ", "ươ");
    }

    // Apply the collected tone mark to the appropriate vowel
    if let Some(t) = tone {
        out_str = apply_tone(&out_str, t);
    }

    // Restore original case for each character
    let mut result = String::new();
    for (i, ch) in out_str.chars().enumerate() {
        if *out_upper.get(i).unwrap_or(&false) {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
        } else {
            result.push(ch);
        }
    }

    // If the original input was ALL UPPERCASE, make the result all uppercase
    // This handles cases like "CHAOF" → "CHÀO"
    // Require at least one actual uppercase letter to avoid false positives
    // with non-alphabetic-only input (e.g., "[" brackets)
    let has_uppercase = word.chars().any(|c| c.is_uppercase());
    let result = if has_uppercase && word.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
        result.to_uppercase()
    } else {
        result
    };

    // Auto-correct tone position: if the tone was placed on the wrong vowel
    // (e.g., "chưong" → "chương"), relocate it to the correct position
    relocate_tone(&result)
}

/// Determines which vowel should receive the tone mark based on Vietnamese grammar rules.
///
/// Vietnamese has specific rules for tone mark placement:
/// 1. **Marked vowels first**: If there's a vowel with a diacritic (â, ă, ê, ô, ơ, ư),
///    the tone mark goes on that vowel.
/// 2. **Closed syllable rule**: If the word ends with a consonant, the tone goes on
///    the last vowel.
/// 3. **Open syllable rule**: If the word ends with a vowel and has multiple vowels,
///    the tone goes on the second-to-last vowel.
/// 4. **Qu/Gi exception**: In "qu" and "gi" clusters, the 'u' and 'i' are treated as
///    semi-consonants, so the tone goes on the following vowel.
///
/// # Arguments
/// * `word` - The word with vowel marks applied but no tone yet
/// * `tone_char` - The tone mark character (s, f, r, x, j)
///
/// # Returns
/// The word with the tone mark applied to the correct vowel
fn apply_tone(word: &str, tone_char: char) -> String {
    let vowels = "aeiouyâăêôơư";
    let marked_vowels = "âăêôơư"; // Vowels that already have diacritics

    let chars: Vec<char> = word.chars().collect();

    // Check if word ends with consonant (closed syllable)
    let last_char = lower_char(chars.last().cloned().unwrap_or(' '));
    let ends_with_consonant = !vowels.contains(last_char);

    // Special handling for ưo cluster in closed syllables
    // In Vietnamese, "ưo" should often become "ươ" when a tone is applied
    // For words like "chương", "hương", the tone goes on ơ, not ư
    let word_lower: String = chars
        .iter()
        .map(|c| c.to_lowercase().next().unwrap_or(*c))
        .collect();

    // Check for ưo pattern (user typed uw + o but not ow for ơ)
    let has_uo_pattern = word_lower.contains("ưo") && !word_lower.contains("ươ");

    // If we have ưo pattern in closed syllable, convert o to ơ
    let mut chars = chars; // Make mutable
    if has_uo_pattern && ends_with_consonant {
        // Find the position of 'o' after 'ư' and convert it to 'ơ'
        for i in 0..chars.len().saturating_sub(1) {
            let curr = chars[i].to_lowercase().next().unwrap_or(' ');
            let next = chars[i + 1].to_lowercase().next().unwrap_or(' ');
            if curr == 'ư' && next == 'o' {
                // Preserve case when converting o to ơ
                chars[i + 1] = if chars[i + 1].is_uppercase() {
                    'Ơ'
                } else {
                    'ơ'
                };
                break;
            }
        }
    }

    // Calculate vowel indices from the (potentially modified) chars
    let mut vowel_indices = Vec::new();
    for (i, &c) in chars.iter().enumerate() {
        if vowels.contains(lower_char(c)) {
            vowel_indices.push(i);
        }
    }

    // If no vowels found, just append the tone character literally (edge case)
    if vowel_indices.is_empty() {
        let mut s: String = chars.into_iter().collect();
        s.push(tone_char);
        return s;
    }

    // Handle "qu" and "gi" clusters where the second letter is a semi-consonant
    // In these cases, skip the first vowel (u in qu, i in gi)
    if vowel_indices.len() > 1 && chars.len() >= 2 {
        let first_char = lower_char(chars[0]);
        let second_char = lower_char(chars[1]);
        if (first_char == 'q' && second_char == 'u') || (first_char == 'g' && second_char == 'i') {
            vowel_indices.remove(0);
        }
    }

    // Now find the target vowel for tone placement
    let has_uo_cluster = {
        let word_lower: String = chars
            .iter()
            .map(|c| c.to_lowercase().next().unwrap_or(*c))
            .collect();
        word_lower.contains("ươ")
    };

    let target_idx = if has_uo_cluster && ends_with_consonant && vowel_indices.len() >= 2 {
        // For ươ cluster in closed syllable: tone on the second vowel (ơ)
        // Find which vowel is ơ or o with diacritic potential
        let uo_pair: Vec<usize> = vowel_indices
            .iter()
            .cloned()
            .filter(|&idx| {
                let ch = chars[idx].to_lowercase().next().unwrap_or(' ');
                ch == 'ư' || ch == 'ơ'
            })
            .collect();

        if uo_pair.len() >= 2 {
            // Return the second one (ơ position)
            uo_pair[1]
        } else {
            // Fallback to last vowel for closed syllable
            *vowel_indices.last().unwrap()
        }
    } else {
        // Standard logic: prefer marked vowels first
        let marked_vowel_idx = vowel_indices
            .iter()
            .rfind(|&&idx| marked_vowels.contains(lower_char(chars[idx])));

        if let Some(&idx) = marked_vowel_idx {
            // Found a marked vowel, use it
            idx
        } else {
            // No marked vowel, apply syllable-based rules
            if ends_with_consonant {
                // Closed syllable: tone on the last vowel
                *vowel_indices.last().unwrap()
            } else {
                // Open syllable: tone on the second-to-last vowel (if available)
                if vowel_indices.len() >= 2 {
                    vowel_indices[vowel_indices.len() - 2]
                } else {
                    vowel_indices[0]
                }
            }
        }
    };

    // Build the result string with the tone applied to the target vowel
    let mut result = String::new();
    for (i, ch) in chars.iter().enumerate() {
        if i == target_idx {
            result.push(add_mark(*ch, tone_char));
        } else {
            result.push(*ch);
        }
    }
    result
}

/// Maps a vowel and a tone mark to the corresponding combined Vietnamese character.
///
/// This function handles both:
/// - Base vowels: a, e, i, o, u, y
/// - Vowels with diacritics: â, ă, ê, ô, ơ, ư
///
/// And combines them with tone marks:
/// - s (sắc/acute): á, é, í, ó, ú, ý, ấ, ắ, ế, ố, ớ, ứ
/// - f (huyền/grave): à, è, ì, ò, ù, ỳ, ầ, ằ, ề, ồ, ờ, ừ
/// - r (hỏi/hook): ả, ẻ, ỉ, ỏ, ủ, ỷ, ẩ, ẳ, ể, ổ, ở, ử
/// - x (ngã/tilde): ã, ẽ, ĩ, õ, ũ, ỹ, ẫ, ẵ, ễ, ỗ, ỡ, ữ
/// - j (nặng/dot): ạ, ẹ, ị, ọ, ụ, ỵ, ậ, ặ, ệ, ộ, ợ, ự
///
/// # Arguments
/// * `ch` - The base vowel character (may have diacritic)
/// * `tone` - The tone mark character (s, f, r, x, j)
///
/// # Returns
/// The combined character with both diacritic and tone mark
fn add_mark(ch: char, tone: char) -> char {
    let is_upper = ch.is_uppercase();
    let ch_lower = lower_char(ch);

    // Map (vowel, tone) pairs to pre-composed Unicode characters
    let res = match (ch_lower, tone) {
        // Base 'a' with tones
        ('a', 's') => 'á',
        ('a', 'f') => 'à',
        ('a', 'r') => 'ả',
        ('a', 'x') => 'ã',
        ('a', 'j') => 'ạ',
        // â (a circumflex) with tones
        ('â', 's') => 'ấ',
        ('â', 'f') => 'ầ',
        ('â', 'r') => 'ẩ',
        ('â', 'x') => 'ẫ',
        ('â', 'j') => 'ậ',
        // ă (a breve) with tones
        ('ă', 's') => 'ắ',
        ('ă', 'f') => 'ằ',
        ('ă', 'r') => 'ẳ',
        ('ă', 'x') => 'ẵ',
        ('ă', 'j') => 'ặ',
        // Base 'e' with tones
        ('e', 's') => 'é',
        ('e', 'f') => 'è',
        ('e', 'r') => 'ẻ',
        ('e', 'x') => 'ẽ',
        ('e', 'j') => 'ẹ',
        // ê (e circumflex) with tones
        ('ê', 's') => 'ế',
        ('ê', 'f') => 'ề',
        ('ê', 'r') => 'ể',
        ('ê', 'x') => 'ễ',
        ('ê', 'j') => 'ệ',
        // Base 'o' with tones
        ('o', 's') => 'ó',
        ('o', 'f') => 'ò',
        ('o', 'r') => 'ỏ',
        ('o', 'x') => 'õ',
        ('o', 'j') => 'ọ',
        // ô (o circumflex) with tones
        ('ô', 's') => 'ố',
        ('ô', 'f') => 'ồ',
        ('ô', 'r') => 'ổ',
        ('ô', 'x') => 'ỗ',
        ('ô', 'j') => 'ộ',
        // ơ (o horn) with tones
        ('ơ', 's') => 'ớ',
        ('ơ', 'f') => 'ờ',
        ('ơ', 'r') => 'ở',
        ('ơ', 'x') => 'ỡ',
        ('ơ', 'j') => 'ợ',
        // Base 'u' with tones
        ('u', 's') => 'ú',
        ('u', 'f') => 'ù',
        ('u', 'r') => 'ủ',
        ('u', 'x') => 'ũ',
        ('u', 'j') => 'ụ',
        // ư (u horn) with tones
        ('ư', 's') => 'ứ',
        ('ư', 'f') => 'ừ',
        ('ư', 'r') => 'ử',
        ('ư', 'x') => 'ữ',
        ('ư', 'j') => 'ự',
        // Base 'i' with tones
        ('i', 's') => 'í',
        ('i', 'f') => 'ì',
        ('i', 'r') => 'ỉ',
        ('i', 'x') => 'ĩ',
        ('i', 'j') => 'ị',
        // Base 'y' with tones
        ('y', 's') => 'ý',
        ('y', 'f') => 'ỳ',
        ('y', 'r') => 'ỷ',
        ('y', 'x') => 'ỹ',
        ('y', 'j') => 'ỵ',
        // Unknown combination, return original character
        _ => ch_lower,
    };

    // Restore original case
    if is_upper {
        res.to_uppercase().next().unwrap_or(res)
    } else {
        res
    }
}

/// Extracts the tone mark from a Vietnamese vowel character.
///
/// Given a vowel with a tone mark, this function returns the base vowel
/// (with diacritic preserved but tone removed) and the tone mark character.
///
/// # Arguments
/// * `ch` - A Vietnamese character (may have tone mark)
///
/// # Returns
/// A tuple of (base_char, Option<tone_char>) where:
/// - `base_char` is the vowel without tone (but keeps diacritic like ư, ơ, â)
/// - `tone_char` is 's', 'f', 'r', 'x', 'j' or None if no tone
///
/// # Examples
/// - 'ừ' → ('ư', Some('f'))
/// - 'ự' → ('ư', Some('j'))
/// - 'ư' → ('ư', None)
/// - 'a' → ('a', None)
fn extract_tone(ch: char) -> (char, Option<char>) {
    let is_upper = ch.is_uppercase();
    let ch_lower = ch.to_lowercase().next().unwrap_or(ch);

    let (base, tone) = match ch_lower {
        // a with tones
        'á' => ('a', Some('s')),
        'à' => ('a', Some('f')),
        'ả' => ('a', Some('r')),
        'ã' => ('a', Some('x')),
        'ạ' => ('a', Some('j')),
        // â with tones
        'ấ' => ('â', Some('s')),
        'ầ' => ('â', Some('f')),
        'ẩ' => ('â', Some('r')),
        'ẫ' => ('â', Some('x')),
        'ậ' => ('â', Some('j')),
        // ă with tones
        'ắ' => ('ă', Some('s')),
        'ằ' => ('ă', Some('f')),
        'ẳ' => ('ă', Some('r')),
        'ẵ' => ('ă', Some('x')),
        'ặ' => ('ă', Some('j')),
        // e with tones
        'é' => ('e', Some('s')),
        'è' => ('e', Some('f')),
        'ẻ' => ('e', Some('r')),
        'ẽ' => ('e', Some('x')),
        'ẹ' => ('e', Some('j')),
        // ê with tones
        'ế' => ('ê', Some('s')),
        'ề' => ('ê', Some('f')),
        'ể' => ('ê', Some('r')),
        'ễ' => ('ê', Some('x')),
        'ệ' => ('ê', Some('j')),
        // o with tones
        'ó' => ('o', Some('s')),
        'ò' => ('o', Some('f')),
        'ỏ' => ('o', Some('r')),
        'õ' => ('o', Some('x')),
        'ọ' => ('o', Some('j')),
        // ô with tones
        'ố' => ('ô', Some('s')),
        'ồ' => ('ô', Some('f')),
        'ổ' => ('ô', Some('r')),
        'ỗ' => ('ô', Some('x')),
        'ộ' => ('ô', Some('j')),
        // ơ with tones
        'ớ' => ('ơ', Some('s')),
        'ờ' => ('ơ', Some('f')),
        'ở' => ('ơ', Some('r')),
        'ỡ' => ('ơ', Some('x')),
        'ợ' => ('ơ', Some('j')),
        // u with tones
        'ú' => ('u', Some('s')),
        'ù' => ('u', Some('f')),
        'ủ' => ('u', Some('r')),
        'ũ' => ('u', Some('x')),
        'ụ' => ('u', Some('j')),
        // ư with tones
        'ứ' => ('ư', Some('s')),
        'ừ' => ('ư', Some('f')),
        'ử' => ('ư', Some('r')),
        'ữ' => ('ư', Some('x')),
        'ự' => ('ư', Some('j')),
        // i with tones
        'í' => ('i', Some('s')),
        'ì' => ('i', Some('f')),
        'ỉ' => ('i', Some('r')),
        'ĩ' => ('i', Some('x')),
        'ị' => ('i', Some('j')),
        // y with tones
        'ý' => ('y', Some('s')),
        'ỳ' => ('y', Some('f')),
        'ỷ' => ('y', Some('r')),
        'ỹ' => ('y', Some('x')),
        'ỵ' => ('y', Some('j')),
        // No tone
        _ => (ch_lower, None),
    };

    // Restore original case
    let base_with_case = if is_upper {
        base.to_uppercase().next().unwrap_or(base)
    } else {
        base
    };

    (base_with_case, tone)
}

/// Relocates the tone mark to the correct vowel position in a Vietnamese word.
///
/// This function handles cases where the user typed the tone mark before
/// completing the word, causing it to land on the wrong vowel. For example:
/// - "chưong" typo: tone on ư, should be on ơ → "chương"
/// - "đựoc" typo: tone on ự, should be on ọ → "được"
///
/// The function extracts any existing tone marks, removes them, and then
/// reapplies the tone to the correct vowel using Vietnamese grammar rules.
///
/// # Arguments
/// * `word` - A Vietnamese word that may have incorrectly placed tone marks
///
/// # Returns
/// The word with tone mark relocated to the correct position
fn relocate_tone(word: &str) -> String {
    // First pass: find any tone mark in the word
    let mut found_tone: Option<char> = None;
    let mut chars_without_tone = String::new();

    for ch in word.chars() {
        let (base, tone) = extract_tone(ch);
        if tone.is_some() && found_tone.is_none() {
            found_tone = tone;
        }
        chars_without_tone.push(base);
    }

    // If no tone found, return original word
    let Some(tone) = found_tone else {
        return word.to_string();
    };

    // Apply tone to correct position using existing apply_tone logic
    apply_tone(&chars_without_tone, tone)
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test basic vowel mark transformations
    #[test]
    fn test_vowel_marks() {
        assert_eq!(transform_buffer("aa"), "â");
        assert_eq!(transform_buffer("aw"), "ă");
        assert_eq!(transform_buffer("ee"), "ê");
        assert_eq!(transform_buffer("oo"), "ô");
        assert_eq!(transform_buffer("ow"), "ơ");
        assert_eq!(transform_buffer("uw"), "ư");
        assert_eq!(transform_buffer("dd"), "đ");
    }

    /// Test tone mark application on base vowels
    #[test]
    fn test_tones() {
        assert_eq!(transform_buffer("as"), "á"); // sắc (acute)
        assert_eq!(transform_buffer("af"), "à"); // huyền (grave)
        assert_eq!(transform_buffer("ar"), "ả"); // hỏi (hook)
        assert_eq!(transform_buffer("ax"), "ã"); // ngã (tilde)
        assert_eq!(transform_buffer("aj"), "ạ"); // nặng (dot below)
    }

    /// Test combined vowel marks and tones
    #[test]
    fn test_combined() {
        assert_eq!(transform_buffer("tieengs"), "tiếng");
        assert_eq!(transform_buffer("vieetj"), "việt");
        assert_eq!(transform_buffer("chao"), "chao"); // no tone
        assert_eq!(transform_buffer("chaof"), "chào"); // with tone
    }

    /// Test special handling for 'qu' and 'gi' clusters
    #[test]
    fn test_qu_gi() {
        assert_eq!(transform_buffer("quas"), "quá"); // tone on 'a', not 'u'
        assert_eq!(transform_buffer("giaf"), "già"); // tone on 'a', not 'i'
        assert_eq!(transform_buffer("gif"), "gì"); // single vowel case
    }

    /// Test case preservation in transformations
    #[test]
    fn test_case_preservation() {
        assert_eq!(transform_buffer("Aa"), "Â"); // First letter uppercase
        assert_eq!(transform_buffer("AA"), "Â"); // All uppercase
        assert_eq!(transform_buffer("vIeetj"), "vIệt"); // Mixed case
        assert_eq!(transform_buffer("CHAOF"), "CHÀO"); // All caps
        assert_eq!(transform_buffer("CHUOWFNG"), "CHƯỜNG"); // All caps with ươ
        assert_eq!(transform_buffer("DDUWOWCJ"), "ĐƯỢC"); // All caps with ươ + nặng
    }

    /// Test double-tap tone keys: typing same tone twice outputs literal character
    #[test]
    fn test_double_tap_tone() {
        assert_eq!(transform_buffer("ass"), "as"); // s + s = literal 's'
        assert_eq!(transform_buffer("asf"), "à"); // s then f = f wins
        assert_eq!(transform_buffer("aass"), "âs"); // aa = â, ss = literal 's'
    }

    /// Test 'w' shortcut for ư and ơ
    #[test]
    fn test_w_shortcuts() {
        assert_eq!(transform_buffer("w"), "ư"); // standalone w
        assert_eq!(transform_buffer("uow"), "ươ"); // u + ow = ươ
        assert_eq!(transform_buffer("uows"), "ướ"); // with tone
    }

    /// Test complex Vietnamese words
    #[test]
    fn test_complex_words() {
        assert_eq!(transform_buffer("nghiax"), "nghĩa");
        assert_eq!(transform_buffer("khuyeen"), "khuyên");
        assert_eq!(transform_buffer("huwowu"), "hươu");
        assert_eq!(transform_buffer("nguyeenx"), "nguyễn");
        assert_eq!(transform_buffer("khueechs"), "khuếch");
    }

    /// Test additional tone placement scenarios
    #[test]
    fn test_tone_placement_more() {
        assert_eq!(transform_buffer("hoaf"), "hòa"); // open syllable
        assert_eq!(transform_buffer("thuys"), "thúy"); // y vowel
        assert_eq!(transform_buffer("quas"), "quá"); // qu cluster
    }

    /// Test that numbers and punctuation pass through unchanged
    #[test]
    fn test_punctuation_ascii() {
        assert_eq!(transform_buffer("viet1"), "viet1");
        assert_eq!(transform_buffer("viet!"), "viet!");
        assert_eq!(transform_buffer("chaof?"), "chào?");
    }

    /// Test auto-correction of tone placement for ươ cluster in closed syllables
    /// When user types tone before completing ươ cluster, it should auto-correct
    #[test]
    fn test_auto_correct_tone_position() {
        // Case: User types chuw (chư) + f (tone) + ong → should become chường (not chừong)
        // The o should auto-convert to ơ and tone should apply to ơ
        assert_eq!(transform_buffer("chuwfong"), "chường"); // huyền tone
        assert_eq!(transform_buffer("chuwsong"), "chướng"); // sắc tone

        // Case: User types dduw (đư) + j (nặng) + oc → should become được (not đựoc)
        assert_eq!(transform_buffer("dduwjoc"), "được");

        // Already correct sequences should stay the same
        assert_eq!(transform_buffer("chuowfng"), "chường"); // formal way to type
        assert_eq!(transform_buffer("dduwowcj"), "được"); // formal way to type
    }

    /// Test 'z' key removes tone mark and activates bypass
    #[test]
    fn test_z_remove_tone() {
        assert_eq!(transform_buffer("asz"), "a"); // s then z removes tone, bypass active
        assert_eq!(transform_buffer("aafz"), "â"); // â + f → ầ, z removes tone → â
        assert_eq!(transform_buffer("az"), "az"); // no tone to remove, literal z
        assert_eq!(transform_buffer("z"), "z"); // standalone z
        // After z-remove, bypass is active: subsequent chars are raw
        assert_eq!(transform_buffer("aszdd"), "add"); // bypass → 'dd' stays literal
    }

    /// Test bracket shortcuts for ư and ơ
    #[test]
    fn test_bracket_shortcuts() {
        assert_eq!(transform_buffer("["), "ư"); // [ → ư
        assert_eq!(transform_buffer("]"), "ơ"); // ] → ơ
        assert_eq!(transform_buffer("t[s"), "tứ"); // with tone
        assert_eq!(transform_buffer("h]s"), "hớ"); // with tone
    }

    /// Test bypass mode: triple-d undoes đ and activates raw mode
    #[test]
    fn test_bypass_triple_d() {
        assert_eq!(transform_buffer("addd"), "add"); // ddd → bypass → raw "add"
        assert_eq!(transform_buffer("addds"), "adds"); // bypass → 's' stays literal
        assert_eq!(transform_buffer("addde"), "adde"); // bypass → 'e' stays literal
        assert_eq!(transform_buffer("ddd"), "dd"); // standalone triple-d
    }

    /// Test bypass mode: triple vowel undoes diacritic and activates raw mode
    #[test]
    fn test_bypass_triple_vowel() {
        assert_eq!(transform_buffer("aaa"), "aa"); // aaa → bypass → raw "aa"
        assert_eq!(transform_buffer("aaas"), "aas"); // bypass → 's' stays literal
        assert_eq!(transform_buffer("eee"), "ee"); // eee → bypass → raw "ee"
        assert_eq!(transform_buffer("ooo"), "oo"); // ooo → bypass → raw "oo"
    }

    /// Test bypass mode: double-tap tone activates bypass
    #[test]
    fn test_bypass_double_tap_tone() {
        assert_eq!(transform_buffer("ass"), "as"); // existing behavior preserved
        assert_eq!(transform_buffer("assd"), "asd"); // bypass → 'd' stays literal
        assert_eq!(transform_buffer("assdd"), "asdd"); // bypass → 'dd' stays literal
    }

    /// Test bypass mode: z-remove activates bypass
    #[test]
    fn test_bypass_z_remove() {
        assert_eq!(transform_buffer("aszd"), "ad"); // z removes tone + bypass, d is raw
        assert_eq!(transform_buffer("aszdd"), "add"); // bypass → 'dd' stays literal
    }

    /// Test that bypass does not affect normal transformations
    #[test]
    fn test_no_bypass_normal() {
        assert_eq!(transform_buffer("dd"), "đ"); // existing behavior
        assert_eq!(transform_buffer("vieetj"), "việt"); // existing behavior
        assert_eq!(transform_buffer("chaof"), "chào"); // existing behavior
        assert_eq!(transform_buffer("aa"), "â"); // existing behavior
        assert_eq!(transform_buffer("ee"), "ê"); // existing behavior
        assert_eq!(transform_buffer("oo"), "ô"); // existing behavior
    }

    // ========================================================================
    // SUPPLEMENTARY TESTS – covering identified gaps
    // ========================================================================

    /// Test tone marks on all base vowels (e, i, o, u, y)
    /// Original tests only covered 'a'.
    #[test]
    fn test_tones_all_vowels() {
        // e
        assert_eq!(transform_buffer("es"), "é");
        assert_eq!(transform_buffer("ef"), "è");
        assert_eq!(transform_buffer("er"), "ẻ");
        assert_eq!(transform_buffer("ex"), "ẽ");
        assert_eq!(transform_buffer("ej"), "ẹ");
        // i
        assert_eq!(transform_buffer("is"), "í");
        assert_eq!(transform_buffer("if"), "ì");
        assert_eq!(transform_buffer("ir"), "ỉ");
        assert_eq!(transform_buffer("ix"), "ĩ");
        assert_eq!(transform_buffer("ij"), "ị");
        // o
        assert_eq!(transform_buffer("os"), "ó");
        assert_eq!(transform_buffer("of"), "ò");
        assert_eq!(transform_buffer("or"), "ỏ");
        assert_eq!(transform_buffer("ox"), "õ");
        assert_eq!(transform_buffer("oj"), "ọ");
        // u
        assert_eq!(transform_buffer("us"), "ú");
        assert_eq!(transform_buffer("uf"), "ù");
        assert_eq!(transform_buffer("ur"), "ủ");
        assert_eq!(transform_buffer("ux"), "ũ");
        assert_eq!(transform_buffer("uj"), "ụ");
        // y
        assert_eq!(transform_buffer("ys"), "ý");
        assert_eq!(transform_buffer("yf"), "ỳ");
        assert_eq!(transform_buffer("yr"), "ỷ");
        assert_eq!(transform_buffer("yx"), "ỹ");
        assert_eq!(transform_buffer("yj"), "ỵ");
    }

    /// Test tone marks on vowels with diacritics (â, ă, ê, ô, ơ, ư)
    #[test]
    fn test_tones_on_diacritic_vowels() {
        // â (aa + tone)
        assert_eq!(transform_buffer("aas"), "ấ");
        assert_eq!(transform_buffer("aaf"), "ầ");
        assert_eq!(transform_buffer("aar"), "ẩ");
        assert_eq!(transform_buffer("aax"), "ẫ");
        assert_eq!(transform_buffer("aaj"), "ậ");
        // ă (aw + tone)
        assert_eq!(transform_buffer("aws"), "ắ");
        assert_eq!(transform_buffer("awf"), "ằ");
        assert_eq!(transform_buffer("awr"), "ẳ");
        assert_eq!(transform_buffer("awx"), "ẵ");
        assert_eq!(transform_buffer("awj"), "ặ");
        // ê (ee + tone)
        assert_eq!(transform_buffer("ees"), "ế");
        assert_eq!(transform_buffer("eef"), "ề");
        assert_eq!(transform_buffer("eer"), "ể");
        assert_eq!(transform_buffer("eex"), "ễ");
        assert_eq!(transform_buffer("eej"), "ệ");
        // ô (oo + tone)
        assert_eq!(transform_buffer("oos"), "ố");
        assert_eq!(transform_buffer("oof"), "ồ");
        assert_eq!(transform_buffer("oor"), "ổ");
        assert_eq!(transform_buffer("oox"), "ỗ");
        assert_eq!(transform_buffer("ooj"), "ộ");
        // ơ (ow + tone)
        assert_eq!(transform_buffer("ows"), "ớ");
        assert_eq!(transform_buffer("owf"), "ờ");
        assert_eq!(transform_buffer("owr"), "ở");
        assert_eq!(transform_buffer("owx"), "ỡ");
        assert_eq!(transform_buffer("owj"), "ợ");
        // ư (uw + tone)
        assert_eq!(transform_buffer("uws"), "ứ");
        assert_eq!(transform_buffer("uwf"), "ừ");
        assert_eq!(transform_buffer("uwr"), "ử");
        assert_eq!(transform_buffer("uwx"), "ữ");
        assert_eq!(transform_buffer("uwj"), "ự");
    }

    /// Test 'ww' undo behavior
    /// Note: 'uw' is consumed as ư first, then standalone 'w' becomes ư again.
    /// The ww undo only triggers when 'ww' appears after a vowel that was
    /// already transformed by a prior 'w' — but 'uw' matches the uw→ư rule
    /// at higher priority, so uww = ư + w(standalone) = ưư.
    /// True ww undo: when previous output char is ư/ơ from a w transform.
    #[test]
    fn test_ww_undo() {
        assert_eq!(transform_buffer("ww"), "w"); // standalone ww → w, bypass
        assert_eq!(transform_buffer("wwa"), "wa"); // bypass active after ww
        // After uw → ư, the next w is standalone (becomes ư)
        assert_eq!(transform_buffer("uww"), "ưư");
        assert_eq!(transform_buffer("oww"), "ơư"); // ow → ơ, then w → ư
    }

    /// Test multi-word buffer with spaces and punctuation
    #[test]
    fn test_multi_word_buffer() {
        assert_eq!(transform_buffer("xin chaof"), "xin chào");
        assert_eq!(transform_buffer("vieetj nam"), "việt nam");
        assert_eq!(transform_buffer("tooi laf"), "tôi là");
        assert_eq!(transform_buffer("vieetj,nam"), "việt,nam");
        assert_eq!(transform_buffer("ddaats nuwowcs"), "đất nước");
        assert_eq!(transform_buffer("chaof banj"), "chào bạn");
    }

    /// Test uơ → ươ auto-conversion
    #[test]
    fn test_uo_to_uow_conversion() {
        assert_eq!(transform_buffer("huwowu"), "hươu"); // already tested
        assert_eq!(transform_buffer("luowng"), "lương");
        assert_eq!(transform_buffer("cuowng"), "cương");
        assert_eq!(transform_buffer("tuowng"), "tương");
        assert_eq!(transform_buffer("duowng"), "dương");
    }

    /// Test tone replacement (typing a different tone replaces the previous one)
    #[test]
    fn test_tone_replacement() {
        assert_eq!(transform_buffer("asf"), "à"); // s → f: grave replaces acute
        assert_eq!(transform_buffer("afr"), "ả"); // f → r: hook replaces grave
        assert_eq!(transform_buffer("arx"), "ã"); // r → x: tilde replaces hook
        assert_eq!(transform_buffer("axj"), "ạ"); // x → j: dot replaces tilde
        assert_eq!(transform_buffer("ajs"), "á"); // j → s: acute replaces dot
    }

    /// Test edge cases
    #[test]
    fn test_edge_cases() {
        assert_eq!(transform_buffer(""), ""); // empty input
        assert_eq!(transform_buffer("b"), "b"); // single consonant
        assert_eq!(transform_buffer("bcd"), "bcd"); // only consonants
        assert_eq!(transform_buffer("a"), "a"); // single vowel
        assert_eq!(transform_buffer("1"), "1"); // single digit (passthrough)
    }

    /// Test extract_tone function directly
    #[test]
    fn test_extract_tone() {
        // Toned vowels
        assert_eq!(extract_tone('á'), ('a', Some('s')));
        assert_eq!(extract_tone('à'), ('a', Some('f')));
        assert_eq!(extract_tone('ả'), ('a', Some('r')));
        assert_eq!(extract_tone('ã'), ('a', Some('x')));
        assert_eq!(extract_tone('ạ'), ('a', Some('j')));
        // Diacritic vowels with tones
        assert_eq!(extract_tone('ấ'), ('â', Some('s')));
        assert_eq!(extract_tone('ừ'), ('ư', Some('f')));
        assert_eq!(extract_tone('ợ'), ('ơ', Some('j')));
        assert_eq!(extract_tone('ệ'), ('ê', Some('j')));
        // No tone
        assert_eq!(extract_tone('a'), ('a', None));
        assert_eq!(extract_tone('ư'), ('ư', None));
        assert_eq!(extract_tone('ơ'), ('ơ', None));
        assert_eq!(extract_tone('b'), ('b', None));
        // Uppercase
        assert_eq!(extract_tone('Á'), ('A', Some('s')));
        assert_eq!(extract_tone('Ừ'), ('Ư', Some('f')));
    }

    /// Test add_mark function directly
    #[test]
    fn test_add_mark() {
        // Base vowels
        assert_eq!(add_mark('a', 's'), 'á');
        assert_eq!(add_mark('e', 'f'), 'è');
        assert_eq!(add_mark('i', 'r'), 'ỉ');
        assert_eq!(add_mark('o', 'x'), 'õ');
        assert_eq!(add_mark('u', 'j'), 'ụ');
        assert_eq!(add_mark('y', 's'), 'ý');
        // Diacritic vowels
        assert_eq!(add_mark('â', 's'), 'ấ');
        assert_eq!(add_mark('ă', 'f'), 'ằ');
        assert_eq!(add_mark('ê', 'r'), 'ể');
        assert_eq!(add_mark('ô', 'x'), 'ỗ');
        assert_eq!(add_mark('ơ', 'j'), 'ợ');
        assert_eq!(add_mark('ư', 's'), 'ứ');
        // Uppercase preservation
        assert_eq!(add_mark('A', 's'), 'Á');
        assert_eq!(add_mark('Ê', 'f'), 'Ề');
        assert_eq!(add_mark('Ư', 'j'), 'Ự');
        // Unknown combination returns original
        assert_eq!(add_mark('b', 's'), 'b');
    }

    /// Test relocate_tone function directly
    #[test]
    fn test_relocate_tone() {
        // Word with no tone → unchanged
        assert_eq!(relocate_tone("chung"), "chung");
        // Word with tone already in correct position → unchanged
        assert_eq!(relocate_tone("chào"), "chào");
        // Word where tone needs relocation
        assert_eq!(relocate_tone("chừong"), "chường");
        assert_eq!(relocate_tone("đựoc"), "được");
    }

    /// Test 'gi' edge cases
    #[test]
    fn test_gi_edge_cases() {
        assert_eq!(transform_buffer("gis"), "gí"); // single vowel 'i'
        assert_eq!(transform_buffer("giax"), "giã"); // gi semi-consonant, tone on 'a'
        assert_eq!(transform_buffer("giangs"), "giáng"); // gi cluster in longer word
    }

    /// Test 'w' after various characters
    #[test]
    fn test_w_after_various() {
        assert_eq!(transform_buffer("tw"), "tư"); // w after consonant → ư
        assert_eq!(transform_buffer("lw"), "lư"); // w after consonant → ư
        assert_eq!(transform_buffer("nw"), "nư"); // w after consonant → ư
        assert_eq!(transform_buffer("tws"), "tứ"); // w + tone
        assert_eq!(transform_buffer("ngw"), "ngư"); // ng + w → ngư
    }

    /// Test uppercase with bypass modes
    #[test]
    fn test_uppercase_bypass() {
        assert_eq!(transform_buffer("ADDD"), "ADD"); // uppercase triple-d bypass
        assert_eq!(transform_buffer("AAA"), "AA"); // uppercase triple-a bypass
        assert_eq!(transform_buffer("ASS"), "AS"); // uppercase double-tap tone bypass
    }

    /// Test real-world Vietnamese sentences
    #[test]
    fn test_real_world_sentences() {
        assert_eq!(transform_buffer("Xin chaof cacs banj"), "Xin chào các bạn");
        assert_eq!(
            transform_buffer("Tooi laf nguwowfi Vieetj Nam"),
            "Tôi là người Việt Nam"
        );
        assert_eq!(transform_buffer("Haf Nooji"), "Hà Nội");
        assert_eq!(
            transform_buffer("thafs"),
            "thá" // f then s: sắc replaces huyền
        );
    }

    /// Test reproduction case for 'awws' -> 'aws'
    #[test]
    fn test_awws_repro() {
        assert_eq!(transform_buffer("awws"), "aws");
        assert_eq!(transform_buffer("aww"), "aw");
    }
}
