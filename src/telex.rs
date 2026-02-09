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
        if ch.is_whitespace() || ch.is_ascii_punctuation() {
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

    let original_chars: Vec<char> = word.chars().collect();
    let mut i = 0;

    // Process each character, looking ahead for two-character combinations
    while i < original_chars.len() {
        let current = original_chars[i];
        let current_lower = current.to_ascii_lowercase();
        let next = original_chars.get(i + 1).cloned();
        let next_lower = next.map(|c| c.to_ascii_lowercase());

        // Pattern matching for two-character vowel mark combinations and special cases
        match (current_lower, next_lower) {
            // Circumflex (^) vowels
            ('a', Some('a')) => {
                // aa → â (a with circumflex)
                out_chars.push('â');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('e', Some('e')) => {
                // ee → ê (e with circumflex)
                out_chars.push('ê');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('o', Some('o')) => {
                // oo → ô (o with circumflex)
                out_chars.push('ô');
                out_upper.push(current.is_uppercase());
                i += 2;
            }

            // Breve (˘) and horn (ˆ) vowels using 'w'
            ('a', Some('w')) => {
                // aw → ă (a with breve)
                out_chars.push('ă');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('o', Some('w')) => {
                // ow → ơ (o with horn)
                out_chars.push('ơ');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('u', Some('w')) => {
                // uw → ư (u with horn)
                out_chars.push('ư');
                out_upper.push(current.is_uppercase());
                i += 2;
            }

            // Đ with stroke
            ('d', Some('d')) => {
                // dd → đ (d with stroke)
                out_chars.push('đ');
                out_upper.push(current.is_uppercase());
                i += 2;
            }

            // Standalone 'w' shortcut for ư or ơ
            ('w', _) => {
                // 'w' can transform the previous vowel or become 'ư' on its own
                if let Some(&last) = out_chars.last() {
                    let last_lower = last.to_ascii_lowercase();
                    if last_lower == 'u' {
                        // Replace 'u' with 'ư'
                        out_chars.pop();
                        out_chars.push('ư');
                    } else if last_lower == 'o' {
                        // Replace 'o' with 'ơ'
                        out_chars.pop();
                        out_chars.push('ơ');
                    } else {
                        // No preceding u/o, 'w' becomes 'ư'
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

            // Tone mark characters (only apply if we have a vowel to mark)
            ('s', _) | ('f', _) | ('r', _) | ('x', _) | ('j', _) => {
                let vowels = "aeiouyâăêôơư";
                let has_vowel = out_chars
                    .iter()
                    .any(|c| vowels.contains(c.to_ascii_lowercase()));

                if has_vowel {
                    // Toggle behavior: typing the same tone twice removes it
                    if tone == Some(current_lower) {
                        tone = None;
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

            // Regular character, just pass it through unchanged
            (c, _) => {
                out_chars.push(c);
                out_upper.push(current.is_uppercase());
                i += 1;
            }
        }
    }

    // Convert character vector to string for further processing
    let mut out_str: String = out_chars.into_iter().collect();

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
    if word.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
        result.to_uppercase()
    } else {
        result
    }
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

    // Find all vowel positions in the word
    let mut vowel_indices = Vec::new();
    for (i, &c) in chars.iter().enumerate() {
        if vowels.contains(c.to_ascii_lowercase()) {
            vowel_indices.push(i);
        }
    }

    // If no vowels found, just append the tone character literally (edge case)
    if vowel_indices.is_empty() {
        let mut s = word.to_string();
        s.push(tone_char);
        return s;
    }

    // Handle "qu" and "gi" clusters where the second letter is a semi-consonant
    // In these cases, skip the first vowel (u in qu, i in gi)
    if vowel_indices.len() > 1 {
        let first_char = chars[0].to_ascii_lowercase();
        let second_char = chars[1].to_ascii_lowercase();
        if (first_char == 'q' && second_char == 'u') || (first_char == 'g' && second_char == 'i') {
            vowel_indices.remove(0);
        }
    }

    // Strategy 1: Prefer placing tone on a vowel that already has a diacritic
    // This follows Vietnamese orthographic conventions
    let marked_vowel_idx = vowel_indices
        .iter()
        .rfind(|&&idx| marked_vowels.contains(chars[idx].to_ascii_lowercase()));

    let target_idx = if let Some(&idx) = marked_vowel_idx {
        // Found a marked vowel, use it
        idx
    } else {
        // No marked vowel, apply syllable-based rules
        let last_char = chars.last().cloned().unwrap_or(' ').to_ascii_lowercase();
        let ends_with_consonant = !vowels.contains(last_char);

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
    let ch_lower = ch.to_ascii_lowercase();

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
    }

    /// Test tone toggling (typing same tone twice removes it)
    #[test]
    fn test_tone_toggling() {
        assert_eq!(transform_buffer("ass"), "a"); // s + s cancels
        assert_eq!(transform_buffer("asf"), "à"); // s then f = f wins
        assert_eq!(transform_buffer("aass"), "â"); // aa = â, ss cancels
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
}
