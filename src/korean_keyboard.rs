const HANGUL_BASE: u32 = 0xAC00;
const HANGUL_END: u32 = 0xD7A3;
const JUNGSEONG_COUNT: u32 = 21;
const JONGSEONG_COUNT: u32 = 28;

pub fn normalize_password_input(value: &mut String) {
    let Some(normalized) = to_qwerty_if_korean(value) else {
        return;
    };
    *value = normalized;
}

pub fn to_qwerty_if_korean(value: &str) -> Option<String> {
    let mut changed = false;
    let mut normalized = String::with_capacity(value.len());

    for ch in value.chars() {
        if let Some(keys) = hangul_syllable_to_keys(ch) {
            normalized.push_str(&keys);
            changed = true;
        } else if let Some(keys) = jamo_to_keys(ch) {
            normalized.push_str(keys);
            changed = true;
        } else {
            normalized.push(ch);
        }
    }

    changed.then_some(normalized)
}

fn hangul_syllable_to_keys(ch: char) -> Option<String> {
    let code = ch as u32;
    if !(HANGUL_BASE..=HANGUL_END).contains(&code) {
        return None;
    }

    let syllable_index = code - HANGUL_BASE;
    let leading = syllable_index / (JUNGSEONG_COUNT * JONGSEONG_COUNT);
    let vowel = (syllable_index % (JUNGSEONG_COUNT * JONGSEONG_COUNT)) / JONGSEONG_COUNT;
    let trailing = syllable_index % JONGSEONG_COUNT;

    let leading = leading_to_keys(leading)?;
    let vowel = vowel_to_keys(vowel)?;
    let trailing = trailing_to_keys(trailing)?;
    Some(format!("{leading}{vowel}{trailing}"))
}

fn leading_to_keys(index: u32) -> Option<&'static str> {
    Some(match index {
        0 => "r",
        1 => "R",
        2 => "s",
        3 => "e",
        4 => "E",
        5 => "f",
        6 => "a",
        7 => "q",
        8 => "Q",
        9 => "t",
        10 => "T",
        11 => "d",
        12 => "w",
        13 => "W",
        14 => "c",
        15 => "z",
        16 => "x",
        17 => "v",
        18 => "g",
        _ => return None,
    })
}

fn vowel_to_keys(index: u32) -> Option<&'static str> {
    Some(match index {
        0 => "k",
        1 => "o",
        2 => "i",
        3 => "O",
        4 => "j",
        5 => "p",
        6 => "u",
        7 => "P",
        8 => "h",
        9 => "hk",
        10 => "ho",
        11 => "hl",
        12 => "y",
        13 => "n",
        14 => "nj",
        15 => "np",
        16 => "nl",
        17 => "b",
        18 => "m",
        19 => "ml",
        20 => "l",
        _ => return None,
    })
}

fn trailing_to_keys(index: u32) -> Option<&'static str> {
    Some(match index {
        0 => "",
        1 => "r",
        2 => "R",
        3 => "rt",
        4 => "s",
        5 => "sw",
        6 => "sg",
        7 => "e",
        8 => "f",
        9 => "fr",
        10 => "fa",
        11 => "fq",
        12 => "ft",
        13 => "fx",
        14 => "fv",
        15 => "fg",
        16 => "a",
        17 => "q",
        18 => "qt",
        19 => "t",
        20 => "T",
        21 => "d",
        22 => "w",
        23 => "c",
        24 => "z",
        25 => "x",
        26 => "v",
        27 => "g",
        _ => return None,
    })
}

fn jamo_to_keys(ch: char) -> Option<&'static str> {
    Some(match ch {
        'ㄱ' | '\u{1100}' | '\u{11A8}' => "r",
        'ㄲ' | '\u{1101}' | '\u{11A9}' => "R",
        'ㄳ' | '\u{11AA}' => "rt",
        'ㄴ' | '\u{1102}' | '\u{11AB}' => "s",
        'ㄵ' | '\u{11AC}' => "sw",
        'ㄶ' | '\u{11AD}' => "sg",
        'ㄷ' | '\u{1103}' | '\u{11AE}' => "e",
        'ㄸ' | '\u{1104}' => "E",
        'ㄹ' | '\u{1105}' | '\u{11AF}' => "f",
        'ㄺ' | '\u{11B0}' => "fr",
        'ㄻ' | '\u{11B1}' => "fa",
        'ㄼ' | '\u{11B2}' => "fq",
        'ㄽ' | '\u{11B3}' => "ft",
        'ㄾ' | '\u{11B4}' => "fx",
        'ㄿ' | '\u{11B5}' => "fv",
        'ㅀ' | '\u{11B6}' => "fg",
        'ㅁ' | '\u{1106}' | '\u{11B7}' => "a",
        'ㅂ' | '\u{1107}' | '\u{11B8}' => "q",
        'ㅃ' | '\u{1108}' => "Q",
        'ㅄ' | '\u{11B9}' => "qt",
        'ㅅ' | '\u{1109}' | '\u{11BA}' => "t",
        'ㅆ' | '\u{110A}' | '\u{11BB}' => "T",
        'ㅇ' | '\u{110B}' | '\u{11BC}' => "d",
        'ㅈ' | '\u{110C}' | '\u{11BD}' => "w",
        'ㅉ' | '\u{110D}' => "W",
        'ㅊ' | '\u{110E}' | '\u{11BE}' => "c",
        'ㅋ' | '\u{110F}' | '\u{11BF}' => "z",
        'ㅌ' | '\u{1110}' | '\u{11C0}' => "x",
        'ㅍ' | '\u{1111}' | '\u{11C1}' => "v",
        'ㅎ' | '\u{1112}' | '\u{11C2}' => "g",
        'ㅏ' | '\u{1161}' => "k",
        'ㅐ' | '\u{1162}' => "o",
        'ㅑ' | '\u{1163}' => "i",
        'ㅒ' | '\u{1164}' => "O",
        'ㅓ' | '\u{1165}' => "j",
        'ㅔ' | '\u{1166}' => "p",
        'ㅕ' | '\u{1167}' => "u",
        'ㅖ' | '\u{1168}' => "P",
        'ㅗ' | '\u{1169}' => "h",
        'ㅘ' | '\u{116A}' => "hk",
        'ㅙ' | '\u{116B}' => "ho",
        'ㅚ' | '\u{116C}' => "hl",
        'ㅛ' | '\u{116D}' => "y",
        'ㅜ' | '\u{116E}' => "n",
        'ㅝ' | '\u{116F}' => "nj",
        'ㅞ' | '\u{1170}' => "np",
        'ㅟ' | '\u{1171}' => "nl",
        'ㅠ' | '\u{1172}' => "b",
        'ㅡ' | '\u{1173}' => "m",
        'ㅢ' | '\u{1174}' => "ml",
        'ㅣ' | '\u{1175}' => "l",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_compatibility_jamo_to_qwerty_keys() {
        assert_eq!(
            to_qwerty_if_korean("ㅔㅁㄴㄴㅈㅐㄱㅇ").as_deref(),
            Some("password")
        );
        assert_eq!(to_qwerty_if_korean("ㅖㅃ").as_deref(), Some("PQ"));
    }

    #[test]
    fn converts_composed_hangul_to_qwerty_keys() {
        assert_eq!(to_qwerty_if_korean("안녕").as_deref(), Some("dkssud"));
        assert_eq!(to_qwerty_if_korean("과").as_deref(), Some("rhk"));
        assert_eq!(
            to_qwerty_if_korean("비밀번호").as_deref(),
            Some("qlalfqjsgh")
        );
    }

    #[test]
    fn preserves_non_korean_text() {
        assert_eq!(to_qwerty_if_korean("password-123"), None);
        assert_eq!(to_qwerty_if_korean("abcㅁ123").as_deref(), Some("abca123"));
    }
}
