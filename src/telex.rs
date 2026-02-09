/// Processes an entire buffer (potentially containing multiple words)
/// and transforms it into Vietnamese text using the Telex input method.
pub fn transform_buffer(buffer: &str) -> String {
    let mut result = String::new();
    let mut current_word = String::new();

    // Iterate through characters to separate words by whitespace or punctuation
    for ch in buffer.chars() {
        if ch.is_whitespace() || ch.is_ascii_punctuation() {
            result.push_str(&process_word(&current_word));
            result.push(ch);
            current_word.clear();
        } else {
            current_word.push(ch);
        }
    }
    // Process the final word in the buffer
    result.push_str(&process_word(&current_word));
    result
}

/// Transforms a single word based on Telex rules (vowel marks and tones).
fn process_word(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    let mut out_chars = Vec::new(); // Final sequence of characters
    let mut out_upper = Vec::new(); // Tracks case preservation for each character
    let mut tone: Option<char> = None; // Stores the current tone mark (s, f, r, x, j)

    let original_chars: Vec<char> = word.chars().collect();
    let mut i = 0;
    while i < original_chars.len() {
        let current = original_chars[i];
        let current_lower = current.to_ascii_lowercase();
        let next = original_chars.get(i + 1).cloned();
        let next_lower = next.map(|c| c.to_ascii_lowercase());

        // Handle double characters for vowel marks and 'dd' for 'đ'
        match (current_lower, next_lower) {
            ('a', Some('a')) => {
                out_chars.push('â');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('a', Some('w')) => {
                out_chars.push('ă');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('e', Some('e')) => {
                out_chars.push('ê');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('o', Some('o')) => {
                out_chars.push('ô');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('o', Some('w')) => {
                out_chars.push('ơ');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('u', Some('w')) => {
                out_chars.push('ư');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('d', Some('d')) => {
                out_chars.push('đ');
                out_upper.push(current.is_uppercase());
                i += 2;
            }
            ('w', _) => {
                // Short-cut 'w' for 'ư' or 'ơ' based on previous character
                if let Some(&last) = out_chars.last() {
                    let last_lower = last.to_ascii_lowercase();
                    if last_lower == 'u' {
                        out_chars.pop();
                        out_chars.push('ư');
                    } else if last_lower == 'o' {
                        out_chars.pop();
                        out_chars.push('ơ');
                    } else {
                        out_chars.push('ư');
                        out_upper.push(current.is_uppercase());
                    }
                } else {
                    out_chars.push('ư');
                    out_upper.push(current.is_uppercase());
                }
                i += 1;
            }
            // Handle tone suffixes (s, f, r, x, j)
            ('s', _) | ('f', _) | ('r', _) | ('x', _) | ('j', _) => {
                // Toggle tone if the same mark is typed again
                if tone == Some(current_lower) {
                    tone = None;
                } else {
                    tone = Some(current_lower);
                }
                i += 1;
            }
            (c, _) => {
                // Regular character, just pass it through
                out_chars.push(c);
                out_upper.push(current.is_uppercase());
                i += 1;
            }
        }
    }

    let mut out_str: String = out_chars.into_iter().collect();

    // Special rule: "uơ" should be "ươ" (e.g., "hươu")
    if out_str.contains("uơ") {
        out_str = out_str.replace("uơ", "ươ");
    }

    // Apply the collected tone to the appropriate vowel
    if let Some(t) = tone {
        out_str = apply_tone(&out_str, t);
    }

    // Final case adjustment for each character
    let mut result = String::new();
    for (i, ch) in out_str.chars().enumerate() {
        if *out_upper.get(i).unwrap_or(&false) {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
        } else {
            result.push(ch);
        }
    }

    // If the original input was all uppercase, make the result all uppercase
    if word.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
        result.to_uppercase()
    } else {
        result
    }
}

/// Determines which vowel should receive the tone mark based on Vietnamese grammar rules.
fn apply_tone(word: &str, tone_char: char) -> String {
    let vowels = "aeiouyâăêôơư";
    let marked_vowels = "âăêôơư";

    let chars: Vec<char> = word.chars().collect();
    let mut vowel_indices = Vec::new();
    for (i, &c) in chars.iter().enumerate() {
        if vowels.contains(c.to_ascii_lowercase()) {
            vowel_indices.push(i);
        }
    }

    // If no vowels, append the tone char as literal (highly unlikely for real words)
    if vowel_indices.is_empty() {
        let mut s = word.to_string();
        s.push(tone_char);
        return s;
    }

    // Special handling for 'qu' and 'gi' clusters where 'u' and 'i' are semi-consonants
    if vowel_indices.len() > 1 {
        let first_char = chars[0].to_ascii_lowercase();
        let second_char = chars[1].to_ascii_lowercase();
        if (first_char == 'q' && second_char == 'u') || (first_char == 'g' && second_char == 'i') {
            vowel_indices.remove(0);
        }
    }

    // Prefer marking a vowel that already has a diacritic (â, ă, ê, ô, ơ, ư)
    let marked_vowel_idx = vowel_indices
        .iter()
        .rfind(|&&idx| marked_vowels.contains(chars[idx].to_ascii_lowercase()));

    let target_idx = if let Some(&idx) = marked_vowel_idx {
        idx
    } else {
        let last_char = chars.last().cloned().unwrap_or(' ').to_ascii_lowercase();
        let is_ends_with_consonant = !vowels.contains(last_char);

        // If it ends with a consonant, mark the last vowel
        if is_ends_with_consonant {
            *vowel_indices.last().unwrap()
        } else {
            // If it ends with a vowel, mark the second to last vowel if available
            if vowel_indices.len() >= 2 {
                vowel_indices[vowel_indices.len() - 2]
            } else {
                vowel_indices[0]
            }
        }
    };

    let mut result = String::new();
    for (i, ch) in chars.iter().enumerate() {
        if i == target_idx {
            // Apply the actual mark to the target vowel character
            result.push(add_mark(*ch, tone_char));
        } else {
            result.push(*ch);
        }
    }
    result
}

/// Maps a vowel and a tone mark to the corresponding combined Vietnamese character.
/// It correctly handles both base vowels (a, e, o, u, i, y) and vowels
/// with diacritics (â, ă, ê, ô, ơ, ư).
fn add_mark(ch: char, tone: char) -> char {
    let is_upper = ch.is_uppercase();
    let ch_lower = ch.to_ascii_lowercase();
    let res = match (ch_lower, tone) {
        ('a', 's') => 'á',
        ('a', 'f') => 'à',
        ('a', 'r') => 'ả',
        ('a', 'x') => 'ã',
        ('a', 'j') => 'ạ',
        ('â', 's') => 'ấ',
        ('â', 'f') => 'ầ',
        ('â', 'r') => 'ẩ',
        ('â', 'x') => 'ẫ',
        ('â', 'j') => 'ậ',
        ('ă', 's') => 'ắ',
        ('ă', 'f') => 'ằ',
        ('ă', 'r') => 'ẳ',
        ('ă', 'x') => 'ẵ',
        ('ă', 'j') => 'ặ',
        ('e', 's') => 'é',
        ('e', 'f') => 'è',
        ('e', 'r') => 'ẻ',
        ('e', 'x') => 'ẽ',
        ('e', 'j') => 'ẹ',
        ('ê', 's') => 'ế',
        ('ê', 'f') => 'ề',
        ('ê', 'r') => 'ể',
        ('ê', 'x') => 'ễ',
        ('ê', 'j') => 'ệ',
        ('o', 's') => 'ó',
        ('o', 'f') => 'ò',
        ('o', 'r') => 'ỏ',
        ('o', 'x') => 'õ',
        ('o', 'j') => 'ọ',
        ('ô', 's') => 'ố',
        ('ô', 'f') => 'ồ',
        ('ô', 'r') => 'ổ',
        ('ô', 'x') => 'ỗ',
        ('ô', 'j') => 'ộ',
        ('ơ', 's') => 'ớ',
        ('ơ', 'f') => 'ờ',
        ('ơ', 'r') => 'ở',
        ('ơ', 'x') => 'ỡ',
        ('ơ', 'j') => 'ợ',
        ('u', 's') => 'ú',
        ('u', 'f') => 'ù',
        ('u', 'r') => 'ủ',
        ('u', 'x') => 'ũ',
        ('u', 'j') => 'ụ',
        ('ư', 's') => 'ứ',
        ('ư', 'f') => 'ừ',
        ('ư', 'r') => 'ử',
        ('ư', 'x') => 'ữ',
        ('ư', 'j') => 'ự',
        ('i', 's') => 'í',
        ('i', 'f') => 'ì',
        ('i', 'r') => 'ỉ',
        ('i', 'x') => 'ĩ',
        ('i', 'j') => 'ị',
        ('y', 's') => 'ý',
        ('y', 'f') => 'ỳ',
        ('y', 'r') => 'ỷ',
        ('y', 'x') => 'ỹ',
        ('y', 'j') => 'ỵ',
        _ => ch_lower,
    };
    if is_upper {
        res.to_uppercase().next().unwrap_or(res)
    } else {
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_tones() {
        assert_eq!(transform_buffer("as"), "á");
        assert_eq!(transform_buffer("af"), "à");
        assert_eq!(transform_buffer("ar"), "ả");
        assert_eq!(transform_buffer("ax"), "ã");
        assert_eq!(transform_buffer("aj"), "ạ");
    }

    #[test]
    fn test_combined() {
        assert_eq!(transform_buffer("tieengs"), "tiếng");
        assert_eq!(transform_buffer("vieetj"), "việt");
        assert_eq!(transform_buffer("chao"), "chao");
        assert_eq!(transform_buffer("chaof"), "chào");
    }

    #[test]
    fn test_qu_gi() {
        assert_eq!(transform_buffer("quas"), "quá");
        assert_eq!(transform_buffer("giaf"), "già");
        assert_eq!(transform_buffer("gif"), "gì");
    }

    #[test]
    fn test_case_preservation() {
        assert_eq!(transform_buffer("Aa"), "Â");
        assert_eq!(transform_buffer("AA"), "Â");
        assert_eq!(transform_buffer("vIeetj"), "vIệt");
        assert_eq!(transform_buffer("CHAOF"), "CHÀO");
    }

    #[test]
    fn test_tone_toggling() {
        assert_eq!(transform_buffer("ass"), "a");
        assert_eq!(transform_buffer("asf"), "à");
        assert_eq!(transform_buffer("aass"), "â");
    }

    #[test]
    fn test_w_shortcuts() {
        assert_eq!(transform_buffer("w"), "ư");
        assert_eq!(transform_buffer("uow"), "ươ");
        assert_eq!(transform_buffer("uows"), "ướ");
    }

    #[test]
    fn test_complex_words() {
        assert_eq!(transform_buffer("nghiax"), "nghĩa");
        assert_eq!(transform_buffer("khuyeen"), "khuyên");
        assert_eq!(transform_buffer("huwowu"), "hươu");
        assert_eq!(transform_buffer("nguyeenx"), "nguyễn");
        assert_eq!(transform_buffer("khueechs"), "khuếch");
    }

    #[test]
    fn test_tone_placement_more() {
        // Old style (as implemented in the current code)
        assert_eq!(transform_buffer("hoaf"), "hòa");
        assert_eq!(transform_buffer("thuys"), "thúy");
        assert_eq!(transform_buffer("quas"), "quá");
    }

    #[test]
    fn test_punctuation_ascii() {
        assert_eq!(transform_buffer("viet1"), "viet1");
        assert_eq!(transform_buffer("viet!"), "viet!");
        assert_eq!(transform_buffer("chaof?"), "chào?");
    }
}
