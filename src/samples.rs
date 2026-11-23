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

#[cfg(test)]
mod tests {
    use super::*;

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
