//! Default sample sentence per writing system, checked in this order: the first one whose
//! test letter the font has wins (same rule and order as FontBase's talker.js).
pub const SCRIPTS: [(char, &str); 16] = [
    ('A', "The quick brown fox jumps over the lazy dog"),
    ('我', "我能吞下玻璃而不伤身体"),
    ('私', "私はガラスを食べられます。それは私を傷つけません"),
    ('अ', "कः खगौघाङचिच्छौजा झाञ्ज्ञोऽटौठीडडण्ढणः।"),
    ('ب', "أنا قادر على أكل الزجاج و هذا لا يؤلمني"),
    ('и', "Я можу їсти скло, і воно мені не зашкодить"),
    ('ξ', "ξεσκεπάζω την ψυχοφθόρα βδελυγμία"),
    ('ក', "ខ្ញុំអាចញុំកញ្ចក់បាន ដោយគ្មានបញ្ហារ"),
    ('ກ', "ຂອ້ຍກິນແກ້ວໄດ້ໂດຍທີ່ມັນບໍ່ໄດ້ເຮັດໃຫ້ຂອ້ຍເຈັບ"),
    ('ข', "ฉันกินกระจกได้ แต่มันไม่ทำให้ฉันเจ็บิ"),
    ('அ', "நான் கண்ணாடி சாப்பிடுவேன்"),
    ('က', "ငါဖန်စားနိုင်ကငါ့ကိုထိခိုက်စေပါဘူး။"),
    ('অ', "আমি কাঁচ খেতে পারি, তাতে আমার কোনো ক্ষতি হয় না।"),
    ('א', "אני יכול לאכול זכוכית וזה לא מזיק לי"),
    ('Ա', "Կրնամ ապակի ուտել և ինծի անհանգիստ չըներ։"),
    ('Ⴀ', "მინას ვჭამ და არა მტკივა"),
];

/// One glyph per script in [`SCRIPTS`] order, for the grid cards: "Aa" for Latin, a single
/// fitting letter elsewhere (the Arabic card uses ain rather than the test letter).
const GLYPHS: [&str; 16] = ["Aa", "我", "私", "अ", "ع", "и", "ξ", "ក", "ກ", "ข", "அ", "က", "অ", "א", "Ա", "Ⴀ"];

/// The glyph pair shown on a grid card for a script index; out of range falls back to Latin.
pub fn card_glyph(script: u8) -> &'static str {
    GLYPHS.get(script as usize).copied().unwrap_or(GLYPHS[0])
}

/// Index into [`SCRIPTS`] for a font, given a "does it have this letter" check. Latin if none match.
pub fn script_of(has: impl Fn(char) -> bool) -> u8 {
    SCRIPTS.iter().position(|(c, _)| has(*c)).unwrap_or(0) as u8
}

/// Bit of the Latin script ("English") and of the Arabic script in a script mask.
#[allow(dead_code)] // used by the filters in the next task
pub const LATIN: u16 = 1 << 0;
#[allow(dead_code)] // used by the filters in the next task
pub const ARABIC: u16 = 1 << 4;

/// What people call each script, in [`SCRIPTS`] order (bit 0 is Latin, shown as "English").
#[allow(dead_code)] // used by the filters in the next task
const NAMES: [&str; 16] = [
    "English", "Chinese", "Japanese", "Devanagari", "Arabic", "Cyrillic", "Greek", "Khmer", "Lao", "Thai", "Tamil", "Burmese",
    "Bengali", "Hebrew", "Armenian", "Georgian",
];

/// Every script a font supports: bit `i` is set when it has the test letter of `SCRIPTS[i]`.
#[allow(dead_code)] // used by the filters in the next task
pub fn script_mask(has: impl Fn(char) -> bool) -> u16 {
    SCRIPTS.iter().enumerate().filter(|(_, (c, _))| has(*c)).fold(0, |mask, (i, _)| mask | 1 << i)
}

/// Index into [`SCRIPTS`] of the script a letter belongs to, by Unicode block.
#[allow(dead_code)] // used by the filters in the next task
fn script_index(c: char) -> Option<usize> {
    Some(match c as u32 {
        _ if c.is_ascii_alphabetic() => 0,
        0x00C0..=0x024F | 0x1E00..=0x1EFF => 0,
        0x0370..=0x03FF => 6,
        0x0400..=0x04FF => 5,
        0x0530..=0x058F => 14,
        0x0590..=0x05FF => 13,
        0x0600..=0x06FF | 0x0750..=0x077F | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF => 4,
        0x0900..=0x097F => 3,
        0x0980..=0x09FF => 12,
        0x0B80..=0x0BFF => 10,
        0x0E00..=0x0E7F => 9,
        0x0E80..=0x0EFF => 8,
        0x1000..=0x109F => 11,
        0x10A0..=0x10FF => 15,
        0x1780..=0x17FF => 7,
        0x3040..=0x30FF => 2,
        0x4E00..=0x9FFF => 1,
        _ => return None,
    })
}

/// The scripts of the letters in `text`. Digits, spaces, punctuation and symbols add nothing.
#[allow(dead_code)] // used by the filters in the next task
pub fn scripts_in(text: &str) -> u16 {
    text.chars().filter(|c| c.is_alphabetic()).filter_map(script_index).fold(0, |mask, i| mask | 1 << i)
}

/// "Arabic", "Arabic and English", "Arabic, English and Hebrew"; empty for no script.
#[allow(dead_code)] // used by the filters in the next task
pub fn script_names(mask: u16) -> String {
    let names: Vec<&str> = NAMES.iter().enumerate().filter(|(i, _)| mask & (1 << i) != 0).map(|(_, n)| *n).collect();
    match names.as_slice() {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_has_a_bit_per_supported_script() {
        assert_eq!(script_mask(|c| c == 'A' || c == 'ب'), LATIN | ARABIC);
        assert_eq!(script_mask(|_| false), 0);
        assert_eq!(script_mask(|_| true), 0xFFFF);
    }

    #[test]
    fn text_scripts_follow_the_letters_only() {
        assert_eq!(scripts_in("Hello"), LATIN);
        assert_eq!(scripts_in("عبد الرحمن"), ARABIC);
        assert_eq!(scripts_in("Hello عبد"), LATIN | ARABIC);
        assert_eq!(scripts_in("Café"), LATIN);
        for s in ["", "   ", "123 -- !?", "😀", "١٢٣"] {
            assert_eq!(scripts_in(s), 0, "{s:?}");
        }
    }

    #[test]
    fn names_read_naturally() {
        assert_eq!(script_names(ARABIC), "Arabic");
        assert_eq!(script_names(LATIN | ARABIC), "English and Arabic"); // names follow bit order
        assert_eq!(script_names(LATIN | ARABIC | 1 << 13), "English, Arabic and Hebrew");
        assert_eq!(script_names(0), "");
    }

    #[test]
    fn card_glyph_per_script() {
        assert_eq!(SCRIPTS.len(), 16);
        for i in 0..16u8 {
            assert!(!card_glyph(i).is_empty());
        }
        assert_eq!(card_glyph(0), "Aa");
        assert_eq!(card_glyph(4), "\u{639}"); // Arabic ain
        assert_eq!(card_glyph(200), "Aa"); // out of range falls back to Latin
    }

    #[test]
    fn first_supported_script_wins() {
        assert_eq!(script_of(|_| true), 0);
        assert_eq!(script_of(|c| c == 'ب' || c == 'и'), 4);
        assert_eq!(script_of(|_| false), 0);
    }
}
