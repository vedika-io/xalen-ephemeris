//! Multi-language support for Vedic astrology names.
//!
//! Provides planet, rashi (zodiac sign), nakshatra, tithi, yoga, karana, and
//! vara names in 19 languages: English, Hindi, Sanskrit, Tamil, Telugu,
//! Kannada, Malayalam, Bengali, Gujarati, Marathi, Punjabi, Odia, Spanish,
//! Portuguese, French, German, Japanese, Thai, and Indonesian.
//!
//! Swiss Ephemeris outputs English only. This module fills the gap for the
//! Indian astrology market (and global markets) where native-script names
//! are standard.

use serde::{Deserialize, Serialize};
use xalen_coords::Planet;

use crate::nakshatra::Nakshatra;
use crate::panchang::{Karana, Tithi, Vara, Yoga};
use crate::rashi::Rashi;

// ---------------------------------------------------------------------------
// Language enum
// ---------------------------------------------------------------------------

/// Supported languages for Vedic astrology terminology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Hindi,
    Sanskrit,
    Tamil,
    Telugu,
    Kannada,
    Malayalam,
    Bengali,
    Gujarati,
    Marathi,
    Punjabi,
    Odia,
    Spanish,
    Portuguese,
    French,
    German,
    Japanese,
    Thai,
    Indonesian,
}

impl Language {
    /// All supported languages.
    pub const ALL: [Language; 19] = [
        Language::English,
        Language::Hindi,
        Language::Sanskrit,
        Language::Tamil,
        Language::Telugu,
        Language::Kannada,
        Language::Malayalam,
        Language::Bengali,
        Language::Gujarati,
        Language::Marathi,
        Language::Punjabi,
        Language::Odia,
        Language::Spanish,
        Language::Portuguese,
        Language::French,
        Language::German,
        Language::Japanese,
        Language::Thai,
        Language::Indonesian,
    ];
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Language::English => "English",
            Language::Hindi => "Hindi",
            Language::Sanskrit => "Sanskrit",
            Language::Tamil => "Tamil",
            Language::Telugu => "Telugu",
            Language::Kannada => "Kannada",
            Language::Malayalam => "Malayalam",
            Language::Bengali => "Bengali",
            Language::Gujarati => "Gujarati",
            Language::Marathi => "Marathi",
            Language::Punjabi => "Punjabi",
            Language::Odia => "Odia",
            Language::Spanish => "Spanish",
            Language::Portuguese => "Portuguese",
            Language::French => "French",
            Language::German => "German",
            Language::Japanese => "Japanese",
            Language::Thai => "Thai",
            Language::Indonesian => "Indonesian",
        })
    }
}

// ---------------------------------------------------------------------------
// Planet names
// ---------------------------------------------------------------------------

/// Return the name of a planet in the given language.
///
/// Covers all 14 `Planet` variants. For Vedic-only planets (Rahu, Ketu) the
/// English name is the IAST transliteration; Indic languages return the
/// native script form. Global languages use astronomical names where
/// applicable and transliterations for Vedic-only grahas.
pub fn planet_name(planet: Planet, lang: Language) -> &'static str {
    match lang {
        Language::English => planet_name_en(planet),
        Language::Hindi => planet_name_hi(planet),
        Language::Sanskrit => planet_name_sa(planet),
        Language::Tamil => planet_name_ta(planet),
        Language::Telugu => planet_name_te(planet),
        Language::Kannada => planet_name_kn(planet),
        Language::Malayalam => planet_name_ml(planet),
        Language::Bengali => planet_name_bn(planet),
        Language::Gujarati => planet_name_gu(planet),
        Language::Marathi => planet_name_mr(planet),
        Language::Punjabi => planet_name_pa(planet),
        Language::Odia => planet_name_or(planet),
        Language::Spanish => planet_name_es(planet),
        Language::Portuguese => planet_name_pt(planet),
        Language::French => planet_name_fr(planet),
        Language::German => planet_name_de(planet),
        Language::Japanese => planet_name_ja(planet),
        Language::Thai => planet_name_th(planet),
        Language::Indonesian => planet_name_id(planet),
    }
}

fn planet_name_en(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "Sun",
        Planet::Moon => "Moon",
        Planet::Mars => "Mars",
        Planet::Mercury => "Mercury",
        Planet::Jupiter => "Jupiter",
        Planet::Venus => "Venus",
        Planet::Saturn => "Saturn",
        Planet::Rahu => "Rahu",
        Planet::Ketu => "Ketu",
        Planet::NorthNode => "North Node",
        Planet::SouthNode => "South Node",
        Planet::Uranus => "Uranus",
        Planet::Neptune => "Neptune",
        Planet::Pluto => "Pluto",
    }
}

fn planet_name_hi(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "सूर्य",
        Planet::Moon => "चन्द्र",
        Planet::Mars => "मंगल",
        Planet::Mercury => "बुध",
        Planet::Jupiter => "गुरु",
        Planet::Venus => "शुक्र",
        Planet::Saturn => "शनि",
        Planet::Rahu => "राहु",
        Planet::Ketu => "केतु",
        Planet::NorthNode => "राहु",
        Planet::SouthNode => "केतु",
        Planet::Uranus => "अरुण",
        Planet::Neptune => "वरुण",
        Planet::Pluto => "यम",
    }
}

fn planet_name_sa(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "सूर्यः",
        Planet::Moon => "चन्द्रः",
        Planet::Mars => "कुजः",
        Planet::Mercury => "बुधः",
        Planet::Jupiter => "बृहस्पतिः",
        Planet::Venus => "शुक्रः",
        Planet::Saturn => "शनैश्चरः",
        Planet::Rahu => "राहुः",
        Planet::Ketu => "केतुः",
        Planet::NorthNode => "राहुः",
        Planet::SouthNode => "केतुः",
        Planet::Uranus => "अरुणः",
        Planet::Neptune => "वरुणः",
        Planet::Pluto => "यमः",
    }
}

fn planet_name_ta(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "சூரியன்",
        Planet::Moon => "சந்திரன்",
        Planet::Mars => "செவ்வாய்",
        Planet::Mercury => "புதன்",
        Planet::Jupiter => "குரு",
        Planet::Venus => "சுக்கிரன்",
        Planet::Saturn => "சனி",
        Planet::Rahu => "ராகு",
        Planet::Ketu => "கேது",
        Planet::NorthNode => "ராகு",
        Planet::SouthNode => "கேது",
        Planet::Uranus => "யுரேனஸ்",
        Planet::Neptune => "நெப்டியூன்",
        Planet::Pluto => "புளூட்டோ",
    }
}

fn planet_name_te(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "సూర్యుడు",
        Planet::Moon => "చంద్రుడు",
        Planet::Mars => "కుజుడు",
        Planet::Mercury => "బుధుడు",
        Planet::Jupiter => "గురుడు",
        Planet::Venus => "శుక్రుడు",
        Planet::Saturn => "శని",
        Planet::Rahu => "రాహువు",
        Planet::Ketu => "కేతువు",
        Planet::NorthNode => "రాహువు",
        Planet::SouthNode => "కేతువు",
        Planet::Uranus => "యురేనస్",
        Planet::Neptune => "నెప్ట్యూన్",
        Planet::Pluto => "ప్లూటో",
    }
}

fn planet_name_kn(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "ಸೂರ್ಯ",
        Planet::Moon => "ಚಂದ್ರ",
        Planet::Mars => "ಮಂಗಳ",
        Planet::Mercury => "ಬುಧ",
        Planet::Jupiter => "ಗುರು",
        Planet::Venus => "ಶುಕ್ರ",
        Planet::Saturn => "ಶನಿ",
        Planet::Rahu => "ರಾಹು",
        Planet::Ketu => "ಕೇತು",
        Planet::NorthNode => "ರಾಹು",
        Planet::SouthNode => "ಕೇತು",
        Planet::Uranus => "ಯುರೇನಸ್",
        Planet::Neptune => "ನೆಪ್ಚೂನ್",
        Planet::Pluto => "ಪ್ಲೂಟೊ",
    }
}

fn planet_name_ml(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "സൂര്യന്‍",
        Planet::Moon => "ചന്ദ്രന്‍",
        Planet::Mars => "ചൊവ്വ",
        Planet::Mercury => "ബുധന്‍",
        Planet::Jupiter => "വ്യാഴം",
        Planet::Venus => "ശുക്രന്‍",
        Planet::Saturn => "ശനി",
        Planet::Rahu => "രാഹു",
        Planet::Ketu => "കേതു",
        Planet::NorthNode => "രാഹു",
        Planet::SouthNode => "കേതു",
        Planet::Uranus => "യുറാനസ്",
        Planet::Neptune => "നെപ്റ്റ്യൂണ്‍",
        Planet::Pluto => "പ്ലൂട്ടോ",
    }
}

fn planet_name_bn(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "সূর্য",
        Planet::Moon => "চন্দ্র",
        Planet::Mars => "মঙ্গল",
        Planet::Mercury => "বুধ",
        Planet::Jupiter => "বৃহস্পতি",
        Planet::Venus => "শুক্র",
        Planet::Saturn => "শনি",
        Planet::Rahu => "রাহু",
        Planet::Ketu => "কেতু",
        Planet::NorthNode => "রাহু",
        Planet::SouthNode => "কেতু",
        Planet::Uranus => "ইউরেনাস",
        Planet::Neptune => "নেপচুন",
        Planet::Pluto => "প্লুটো",
    }
}

fn planet_name_gu(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "સૂર્ય",
        Planet::Moon => "ચંદ્ર",
        Planet::Mars => "મંગળ",
        Planet::Mercury => "બુધ",
        Planet::Jupiter => "ગુરુ",
        Planet::Venus => "શુક્ર",
        Planet::Saturn => "શનિ",
        Planet::Rahu => "રાહુ",
        Planet::Ketu => "કેતુ",
        Planet::NorthNode => "રાહુ",
        Planet::SouthNode => "કેતુ",
        Planet::Uranus => "યુરેનસ",
        Planet::Neptune => "નેપ્ચ્યુન",
        Planet::Pluto => "પ્લુટો",
    }
}

fn planet_name_mr(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "सूर्य",
        Planet::Moon => "चंद्र",
        Planet::Mars => "मंगळ",
        Planet::Mercury => "बुध",
        Planet::Jupiter => "गुरू",
        Planet::Venus => "शुक्र",
        Planet::Saturn => "शनी",
        Planet::Rahu => "राहू",
        Planet::Ketu => "केतू",
        Planet::NorthNode => "राहू",
        Planet::SouthNode => "केतू",
        Planet::Uranus => "युरेनस",
        Planet::Neptune => "नेपच्यून",
        Planet::Pluto => "प्लुटो",
    }
}

fn planet_name_pa(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "ਸੂਰਜ",
        Planet::Moon => "ਚੰਦਰ",
        Planet::Mars => "ਮੰਗਲ",
        Planet::Mercury => "ਬੁੱਧ",
        Planet::Jupiter => "ਗੁਰੂ",
        Planet::Venus => "ਸ਼ੁੱਕਰ",
        Planet::Saturn => "ਸ਼ਨੀ",
        Planet::Rahu => "ਰਾਹੂ",
        Planet::Ketu => "ਕੇਤੂ",
        Planet::NorthNode => "ਰਾਹੂ",
        Planet::SouthNode => "ਕੇਤੂ",
        Planet::Uranus => "ਯੂਰੇਨਸ",
        Planet::Neptune => "ਨੈਪਚੂਨ",
        Planet::Pluto => "ਪਲੂਟੋ",
    }
}

fn planet_name_or(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "ସୂର୍ଯ୍ୟ",
        Planet::Moon => "ଚନ୍ଦ୍ର",
        Planet::Mars => "ମଙ୍ଗଳ",
        Planet::Mercury => "ବୁଧ",
        Planet::Jupiter => "ଗୁରୁ",
        Planet::Venus => "ଶୁକ୍ର",
        Planet::Saturn => "ଶନି",
        Planet::Rahu => "ରାହୁ",
        Planet::Ketu => "କେତୁ",
        Planet::NorthNode => "ରାହୁ",
        Planet::SouthNode => "କେତୁ",
        Planet::Uranus => "ୟୁରେନସ୍",
        Planet::Neptune => "ନେପଚୁନ୍",
        Planet::Pluto => "ପ୍ଲୁଟୋ",
    }
}

fn planet_name_es(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "Sol",
        Planet::Moon => "Luna",
        Planet::Mars => "Marte",
        Planet::Mercury => "Mercurio",
        Planet::Jupiter => "Júpiter",
        Planet::Venus => "Venus",
        Planet::Saturn => "Saturno",
        Planet::Rahu => "Rahu",
        Planet::Ketu => "Ketu",
        Planet::NorthNode => "Nodo Norte",
        Planet::SouthNode => "Nodo Sur",
        Planet::Uranus => "Urano",
        Planet::Neptune => "Neptuno",
        Planet::Pluto => "Plutón",
    }
}

fn planet_name_pt(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "Sol",
        Planet::Moon => "Lua",
        Planet::Mars => "Marte",
        Planet::Mercury => "Mercúrio",
        Planet::Jupiter => "Júpiter",
        Planet::Venus => "Vênus",
        Planet::Saturn => "Saturno",
        Planet::Rahu => "Rahu",
        Planet::Ketu => "Ketu",
        Planet::NorthNode => "Nodo Norte",
        Planet::SouthNode => "Nodo Sul",
        Planet::Uranus => "Urano",
        Planet::Neptune => "Netuno",
        Planet::Pluto => "Plutão",
    }
}

fn planet_name_fr(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "Soleil",
        Planet::Moon => "Lune",
        Planet::Mars => "Mars",
        Planet::Mercury => "Mercure",
        Planet::Jupiter => "Jupiter",
        Planet::Venus => "Vénus",
        Planet::Saturn => "Saturne",
        Planet::Rahu => "Rahu",
        Planet::Ketu => "Ketu",
        Planet::NorthNode => "Noeud Nord",
        Planet::SouthNode => "Noeud Sud",
        Planet::Uranus => "Uranus",
        Planet::Neptune => "Neptune",
        Planet::Pluto => "Pluton",
    }
}

fn planet_name_de(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "Sonne",
        Planet::Moon => "Mond",
        Planet::Mars => "Mars",
        Planet::Mercury => "Merkur",
        Planet::Jupiter => "Jupiter",
        Planet::Venus => "Venus",
        Planet::Saturn => "Saturn",
        Planet::Rahu => "Rahu",
        Planet::Ketu => "Ketu",
        Planet::NorthNode => "Mondknoten Nord",
        Planet::SouthNode => "Mondknoten Süd",
        Planet::Uranus => "Uranus",
        Planet::Neptune => "Neptun",
        Planet::Pluto => "Pluto",
    }
}

fn planet_name_ja(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "太陽",
        Planet::Moon => "月",
        Planet::Mars => "火星",
        Planet::Mercury => "水星",
        Planet::Jupiter => "木星",
        Planet::Venus => "金星",
        Planet::Saturn => "土星",
        Planet::Rahu => "ラーフ",
        Planet::Ketu => "ケートゥ",
        Planet::NorthNode => "ドラゴンヘッド",
        Planet::SouthNode => "ドラゴンテイル",
        Planet::Uranus => "天王星",
        Planet::Neptune => "海王星",
        Planet::Pluto => "冥王星",
    }
}

fn planet_name_th(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "อาทิตย์",
        Planet::Moon => "จันทร์",
        Planet::Mars => "อังคาร",
        Planet::Mercury => "พุธ",
        Planet::Jupiter => "พฤหัสบดี",
        Planet::Venus => "ศุกร์",
        Planet::Saturn => "เสาร์",
        Planet::Rahu => "ราหู",
        Planet::Ketu => "เกตุ",
        Planet::NorthNode => "ราหู",
        Planet::SouthNode => "เกตุ",
        Planet::Uranus => "มฤตยู",
        Planet::Neptune => "วรุณ",
        Planet::Pluto => "ยม",
    }
}

fn planet_name_id(p: Planet) -> &'static str {
    match p {
        Planet::Sun => "Matahari",
        Planet::Moon => "Bulan",
        Planet::Mars => "Mars",
        Planet::Mercury => "Merkurius",
        Planet::Jupiter => "Jupiter",
        Planet::Venus => "Venus",
        Planet::Saturn => "Saturnus",
        Planet::Rahu => "Rahu",
        Planet::Ketu => "Ketu",
        Planet::NorthNode => "Nodus Utara",
        Planet::SouthNode => "Nodus Selatan",
        Planet::Uranus => "Uranus",
        Planet::Neptune => "Neptunus",
        Planet::Pluto => "Pluto",
    }
}

// ---------------------------------------------------------------------------
// Rashi names
// ---------------------------------------------------------------------------

/// Return the name of a rashi in the given language.
pub fn rashi_name(rashi: Rashi, lang: Language) -> &'static str {
    match lang {
        Language::English => rashi_name_en(rashi),
        Language::Hindi => rashi_name_hi(rashi),
        Language::Sanskrit => rashi_name_sa(rashi),
        Language::Tamil => rashi_name_ta(rashi),
        Language::Telugu => rashi_name_te(rashi),
        Language::Kannada => rashi_name_kn(rashi),
        Language::Malayalam => rashi_name_ml(rashi),
        Language::Bengali => rashi_name_bn(rashi),
        Language::Gujarati => rashi_name_gu(rashi),
        Language::Marathi => rashi_name_mr(rashi),
        Language::Punjabi => rashi_name_pa(rashi),
        Language::Odia => rashi_name_or(rashi),
        Language::Spanish => rashi_name_es(rashi),
        Language::Portuguese => rashi_name_pt(rashi),
        Language::French => rashi_name_fr(rashi),
        Language::German => rashi_name_de(rashi),
        Language::Japanese => rashi_name_ja(rashi),
        Language::Thai => rashi_name_th(rashi),
        Language::Indonesian => rashi_name_id(rashi),
    }
}

fn rashi_name_en(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "Aries",
        Rashi::Vrishabha => "Taurus",
        Rashi::Mithuna => "Gemini",
        Rashi::Karka => "Cancer",
        Rashi::Simha => "Leo",
        Rashi::Kanya => "Virgo",
        Rashi::Tula => "Libra",
        Rashi::Vrishchika => "Scorpio",
        Rashi::Dhanu => "Sagittarius",
        Rashi::Makara => "Capricorn",
        Rashi::Kumbha => "Aquarius",
        Rashi::Meena => "Pisces",
    }
}

fn rashi_name_hi(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "मेष",
        Rashi::Vrishabha => "वृषभ",
        Rashi::Mithuna => "मिथुन",
        Rashi::Karka => "कर्क",
        Rashi::Simha => "सिंह",
        Rashi::Kanya => "कन्या",
        Rashi::Tula => "तुला",
        Rashi::Vrishchika => "वृश्चिक",
        Rashi::Dhanu => "धनु",
        Rashi::Makara => "मकर",
        Rashi::Kumbha => "कुम्भ",
        Rashi::Meena => "मीन",
    }
}

fn rashi_name_sa(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "मेषः",
        Rashi::Vrishabha => "वृषभः",
        Rashi::Mithuna => "मिथुनम्",
        Rashi::Karka => "कर्कटः",
        Rashi::Simha => "सिंहः",
        Rashi::Kanya => "कन्या",
        Rashi::Tula => "तुला",
        Rashi::Vrishchika => "वृश्चिकः",
        Rashi::Dhanu => "धनुः",
        Rashi::Makara => "मकरः",
        Rashi::Kumbha => "कुम्भः",
        Rashi::Meena => "मीनः",
    }
}

fn rashi_name_ta(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "மேஷம்",
        Rashi::Vrishabha => "ரிஷபம்",
        Rashi::Mithuna => "மிதுனம்",
        Rashi::Karka => "கடகம்",
        Rashi::Simha => "சிம்மம்",
        Rashi::Kanya => "கன்னி",
        Rashi::Tula => "துலாம்",
        Rashi::Vrishchika => "விருச்சிகம்",
        Rashi::Dhanu => "தனுசு",
        Rashi::Makara => "மகரம்",
        Rashi::Kumbha => "கும்பம்",
        Rashi::Meena => "மீனம்",
    }
}

fn rashi_name_te(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "మేషం",
        Rashi::Vrishabha => "వృషభం",
        Rashi::Mithuna => "మిథునం",
        Rashi::Karka => "కర్కాటకం",
        Rashi::Simha => "సింహం",
        Rashi::Kanya => "కన్య",
        Rashi::Tula => "తుల",
        Rashi::Vrishchika => "వృశ్చికం",
        Rashi::Dhanu => "ధనుస్సు",
        Rashi::Makara => "మకరం",
        Rashi::Kumbha => "కుంభం",
        Rashi::Meena => "మీనం",
    }
}

fn rashi_name_kn(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "ಮೇಷ",
        Rashi::Vrishabha => "ವೃಷಭ",
        Rashi::Mithuna => "ಮಿಥುನ",
        Rashi::Karka => "ಕರ್ಕಾಟಕ",
        Rashi::Simha => "ಸಿಂಹ",
        Rashi::Kanya => "ಕನ್ಯಾ",
        Rashi::Tula => "ತುಲಾ",
        Rashi::Vrishchika => "ವೃಶ್ಚಿಕ",
        Rashi::Dhanu => "ಧನು",
        Rashi::Makara => "ಮಕರ",
        Rashi::Kumbha => "ಕುಂಭ",
        Rashi::Meena => "ಮೀನ",
    }
}

fn rashi_name_ml(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "മേടം",
        Rashi::Vrishabha => "ഇടവം",
        Rashi::Mithuna => "മിഥുനം",
        Rashi::Karka => "കര്‍ക്കടകം",
        Rashi::Simha => "ചിങ്ങം",
        Rashi::Kanya => "കന്നി",
        Rashi::Tula => "തുലാം",
        Rashi::Vrishchika => "വൃശ്ചികം",
        Rashi::Dhanu => "ധനു",
        Rashi::Makara => "മകരം",
        Rashi::Kumbha => "കുംഭം",
        Rashi::Meena => "മീനം",
    }
}

fn rashi_name_bn(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "মেষ",
        Rashi::Vrishabha => "বৃষ",
        Rashi::Mithuna => "মিথুন",
        Rashi::Karka => "কর্কট",
        Rashi::Simha => "সিংহ",
        Rashi::Kanya => "কন্যা",
        Rashi::Tula => "তুলা",
        Rashi::Vrishchika => "বৃশ্চিক",
        Rashi::Dhanu => "ধনু",
        Rashi::Makara => "মকর",
        Rashi::Kumbha => "কুম্ভ",
        Rashi::Meena => "মীন",
    }
}

fn rashi_name_gu(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "મેષ",
        Rashi::Vrishabha => "વૃષભ",
        Rashi::Mithuna => "મિથુન",
        Rashi::Karka => "કર્ક",
        Rashi::Simha => "સિંહ",
        Rashi::Kanya => "કન્યા",
        Rashi::Tula => "તુલા",
        Rashi::Vrishchika => "વૃશ્ચિક",
        Rashi::Dhanu => "ધન",
        Rashi::Makara => "મકર",
        Rashi::Kumbha => "કુંભ",
        Rashi::Meena => "મીન",
    }
}

fn rashi_name_mr(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "मेष",
        Rashi::Vrishabha => "वृषभ",
        Rashi::Mithuna => "मिथुन",
        Rashi::Karka => "कर्क",
        Rashi::Simha => "सिंह",
        Rashi::Kanya => "कन्या",
        Rashi::Tula => "तूळ",
        Rashi::Vrishchika => "वृश्चिक",
        Rashi::Dhanu => "धनू",
        Rashi::Makara => "मकर",
        Rashi::Kumbha => "कुंभ",
        Rashi::Meena => "मीन",
    }
}

fn rashi_name_pa(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "ਮੇਖ",
        Rashi::Vrishabha => "ਬ੍ਰਿਖ",
        Rashi::Mithuna => "ਮਿਥੁਨ",
        Rashi::Karka => "ਕਰਕ",
        Rashi::Simha => "ਸਿੰਘ",
        Rashi::Kanya => "ਕੰਨਿਆ",
        Rashi::Tula => "ਤੁਲਾ",
        Rashi::Vrishchika => "ਬ੍ਰਿਸ਼ਚਕ",
        Rashi::Dhanu => "ਧਨੁ",
        Rashi::Makara => "ਮਕਰ",
        Rashi::Kumbha => "ਕੁੰਭ",
        Rashi::Meena => "ਮੀਨ",
    }
}

fn rashi_name_or(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "ମେଷ",
        Rashi::Vrishabha => "ବୃଷ",
        Rashi::Mithuna => "ମିଥୁନ",
        Rashi::Karka => "କର୍କଟ",
        Rashi::Simha => "ସିଂହ",
        Rashi::Kanya => "କନ୍ୟା",
        Rashi::Tula => "ତୁଳା",
        Rashi::Vrishchika => "ବୃଶ୍ଚିକ",
        Rashi::Dhanu => "ଧନୁ",
        Rashi::Makara => "ମକର",
        Rashi::Kumbha => "କୁମ୍ଭ",
        Rashi::Meena => "ମୀନ",
    }
}

fn rashi_name_es(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "Aries",
        Rashi::Vrishabha => "Tauro",
        Rashi::Mithuna => "Géminis",
        Rashi::Karka => "Cáncer",
        Rashi::Simha => "Leo",
        Rashi::Kanya => "Virgo",
        Rashi::Tula => "Libra",
        Rashi::Vrishchika => "Escorpio",
        Rashi::Dhanu => "Sagitario",
        Rashi::Makara => "Capricornio",
        Rashi::Kumbha => "Acuario",
        Rashi::Meena => "Piscis",
    }
}

fn rashi_name_pt(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "Áries",
        Rashi::Vrishabha => "Touro",
        Rashi::Mithuna => "Gêmeos",
        Rashi::Karka => "Câncer",
        Rashi::Simha => "Leão",
        Rashi::Kanya => "Virgem",
        Rashi::Tula => "Libra",
        Rashi::Vrishchika => "Escorpião",
        Rashi::Dhanu => "Sagitário",
        Rashi::Makara => "Capricórnio",
        Rashi::Kumbha => "Aquário",
        Rashi::Meena => "Peixes",
    }
}

fn rashi_name_fr(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "Bélier",
        Rashi::Vrishabha => "Taureau",
        Rashi::Mithuna => "Gémeaux",
        Rashi::Karka => "Cancer",
        Rashi::Simha => "Lion",
        Rashi::Kanya => "Vierge",
        Rashi::Tula => "Balance",
        Rashi::Vrishchika => "Scorpion",
        Rashi::Dhanu => "Sagittaire",
        Rashi::Makara => "Capricorne",
        Rashi::Kumbha => "Verseau",
        Rashi::Meena => "Poissons",
    }
}

fn rashi_name_de(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "Widder",
        Rashi::Vrishabha => "Stier",
        Rashi::Mithuna => "Zwillinge",
        Rashi::Karka => "Krebs",
        Rashi::Simha => "Löwe",
        Rashi::Kanya => "Jungfrau",
        Rashi::Tula => "Waage",
        Rashi::Vrishchika => "Skorpion",
        Rashi::Dhanu => "Schütze",
        Rashi::Makara => "Steinbock",
        Rashi::Kumbha => "Wassermann",
        Rashi::Meena => "Fische",
    }
}

fn rashi_name_ja(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "牡羊座",
        Rashi::Vrishabha => "牡牛座",
        Rashi::Mithuna => "双子座",
        Rashi::Karka => "蟹座",
        Rashi::Simha => "獅子座",
        Rashi::Kanya => "乙女座",
        Rashi::Tula => "天秤座",
        Rashi::Vrishchika => "蠍座",
        Rashi::Dhanu => "射手座",
        Rashi::Makara => "山羊座",
        Rashi::Kumbha => "水瓶座",
        Rashi::Meena => "魚座",
    }
}

fn rashi_name_th(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "เมษ",
        Rashi::Vrishabha => "พฤษภ",
        Rashi::Mithuna => "เมถุน",
        Rashi::Karka => "กรกฎ",
        Rashi::Simha => "สิงห์",
        Rashi::Kanya => "กันย์",
        Rashi::Tula => "ตุลย์",
        Rashi::Vrishchika => "พิจิก",
        Rashi::Dhanu => "ธนู",
        Rashi::Makara => "มกร",
        Rashi::Kumbha => "กุมภ์",
        Rashi::Meena => "มีน",
    }
}

fn rashi_name_id(r: Rashi) -> &'static str {
    match r {
        Rashi::Mesha => "Aries",
        Rashi::Vrishabha => "Taurus",
        Rashi::Mithuna => "Gemini",
        Rashi::Karka => "Kanser",
        Rashi::Simha => "Leo",
        Rashi::Kanya => "Virgo",
        Rashi::Tula => "Libra",
        Rashi::Vrishchika => "Skorpio",
        Rashi::Dhanu => "Sagitarius",
        Rashi::Makara => "Kaprikornus",
        Rashi::Kumbha => "Akuarius",
        Rashi::Meena => "Pisces",
    }
}

// ---------------------------------------------------------------------------
// Nakshatra names
// ---------------------------------------------------------------------------

/// Return the name of a nakshatra in the given language.
///
/// For global languages (Spanish, Portuguese, French, German, Indonesian),
/// IAST transliterations are used since nakshatras are a Vedic concept
/// without native equivalents. Japanese and Thai use phonetic adaptations.
pub fn nakshatra_name(nak: Nakshatra, lang: Language) -> &'static str {
    match lang {
        Language::English => nakshatra_name_en(nak),
        Language::Hindi => nakshatra_name_hi(nak),
        Language::Sanskrit => nakshatra_name_sa(nak),
        Language::Tamil => nakshatra_name_ta(nak),
        Language::Telugu => nakshatra_name_te(nak),
        Language::Kannada => nakshatra_name_kn(nak),
        Language::Malayalam => nakshatra_name_ml(nak),
        Language::Bengali => nakshatra_name_bn(nak),
        Language::Gujarati => nakshatra_name_gu(nak),
        Language::Marathi => nakshatra_name_mr(nak),
        Language::Punjabi => nakshatra_name_pa(nak),
        Language::Odia => nakshatra_name_or(nak),
        Language::Japanese => nakshatra_name_ja(nak),
        Language::Thai => nakshatra_name_th(nak),
        // IAST transliteration for European/Indonesian languages
        Language::Spanish
        | Language::Portuguese
        | Language::French
        | Language::German
        | Language::Indonesian => nakshatra_name_en(nak),
    }
}

fn nakshatra_name_en(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "Ashwini",
        Nakshatra::Bharani => "Bharani",
        Nakshatra::Krittika => "Krittika",
        Nakshatra::Rohini => "Rohini",
        Nakshatra::Mrigashira => "Mrigashira",
        Nakshatra::Ardra => "Ardra",
        Nakshatra::Punarvasu => "Punarvasu",
        Nakshatra::Pushya => "Pushya",
        Nakshatra::Ashlesha => "Ashlesha",
        Nakshatra::Magha => "Magha",
        Nakshatra::PurvaPhalguni => "Purva Phalguni",
        Nakshatra::UttaraPhalguni => "Uttara Phalguni",
        Nakshatra::Hasta => "Hasta",
        Nakshatra::Chitra => "Chitra",
        Nakshatra::Swati => "Swati",
        Nakshatra::Vishakha => "Vishakha",
        Nakshatra::Anuradha => "Anuradha",
        Nakshatra::Jyeshtha => "Jyeshtha",
        Nakshatra::Mula => "Mula",
        Nakshatra::PurvaAshadha => "Purva Ashadha",
        Nakshatra::UttaraAshadha => "Uttara Ashadha",
        Nakshatra::Shravana => "Shravana",
        Nakshatra::Dhanishta => "Dhanishta",
        Nakshatra::Shatabhisha => "Shatabhisha",
        Nakshatra::PurvaBhadrapada => "Purva Bhadrapada",
        Nakshatra::UttaraBhadrapada => "Uttara Bhadrapada",
        Nakshatra::Revati => "Revati",
    }
}

fn nakshatra_name_hi(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "अश्विनी",
        Nakshatra::Bharani => "भरणी",
        Nakshatra::Krittika => "कृत्तिका",
        Nakshatra::Rohini => "रोहिणी",
        Nakshatra::Mrigashira => "मृगशिरा",
        Nakshatra::Ardra => "आर्द्रा",
        Nakshatra::Punarvasu => "पुनर्वसु",
        Nakshatra::Pushya => "पुष्य",
        Nakshatra::Ashlesha => "आश्लेषा",
        Nakshatra::Magha => "मघा",
        Nakshatra::PurvaPhalguni => "पूर्वा फाल्गुनी",
        Nakshatra::UttaraPhalguni => "उत्तरा फाल्गुनी",
        Nakshatra::Hasta => "हस्त",
        Nakshatra::Chitra => "चित्रा",
        Nakshatra::Swati => "स्वाति",
        Nakshatra::Vishakha => "विशाखा",
        Nakshatra::Anuradha => "अनुराधा",
        Nakshatra::Jyeshtha => "ज्येष्ठा",
        Nakshatra::Mula => "मूल",
        Nakshatra::PurvaAshadha => "पूर्वाषाढ़ा",
        Nakshatra::UttaraAshadha => "उत्तराषाढ़ा",
        Nakshatra::Shravana => "श्रवण",
        Nakshatra::Dhanishta => "धनिष्ठा",
        Nakshatra::Shatabhisha => "शतभिषा",
        Nakshatra::PurvaBhadrapada => "पूर्वाभाद्रपद",
        Nakshatra::UttaraBhadrapada => "उत्तराभाद्रपद",
        Nakshatra::Revati => "रेवती",
    }
}

fn nakshatra_name_sa(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "अश्विनी",
        Nakshatra::Bharani => "भरणी",
        Nakshatra::Krittika => "कृत्तिका",
        Nakshatra::Rohini => "रोहिणी",
        Nakshatra::Mrigashira => "मृगशीर्षा",
        Nakshatra::Ardra => "आर्द्रा",
        Nakshatra::Punarvasu => "पुनर्वसुः",
        Nakshatra::Pushya => "पुष्यः",
        Nakshatra::Ashlesha => "आश्लेषा",
        Nakshatra::Magha => "मघा",
        Nakshatra::PurvaPhalguni => "पूर्वफल्गुनी",
        Nakshatra::UttaraPhalguni => "उत्तरफल्गुनी",
        Nakshatra::Hasta => "हस्तः",
        Nakshatra::Chitra => "चित्रा",
        Nakshatra::Swati => "स्वाती",
        Nakshatra::Vishakha => "विशाखा",
        Nakshatra::Anuradha => "अनुराधा",
        Nakshatra::Jyeshtha => "ज्येष्ठा",
        Nakshatra::Mula => "मूलम्",
        Nakshatra::PurvaAshadha => "पूर्वाषाढा",
        Nakshatra::UttaraAshadha => "उत्तराषाढा",
        Nakshatra::Shravana => "श्रवणम्",
        Nakshatra::Dhanishta => "धनिष्ठा",
        Nakshatra::Shatabhisha => "शतभिषक्",
        Nakshatra::PurvaBhadrapada => "पूर्वभाद्रपदा",
        Nakshatra::UttaraBhadrapada => "उत्तरभाद्रपदा",
        Nakshatra::Revati => "रेवती",
    }
}

fn nakshatra_name_ta(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "அசுவினி",
        Nakshatra::Bharani => "பரணி",
        Nakshatra::Krittika => "கிருத்திகை",
        Nakshatra::Rohini => "ரோகிணி",
        Nakshatra::Mrigashira => "மிருகசீரிடம்",
        Nakshatra::Ardra => "திருவாதிரை",
        Nakshatra::Punarvasu => "புனர்பூசம்",
        Nakshatra::Pushya => "பூசம்",
        Nakshatra::Ashlesha => "ஆயில்யம்",
        Nakshatra::Magha => "மகம்",
        Nakshatra::PurvaPhalguni => "பூரம்",
        Nakshatra::UttaraPhalguni => "உத்திரம்",
        Nakshatra::Hasta => "அஸ்தம்",
        Nakshatra::Chitra => "சித்திரை",
        Nakshatra::Swati => "சுவாதி",
        Nakshatra::Vishakha => "விசாகம்",
        Nakshatra::Anuradha => "அனுஷம்",
        Nakshatra::Jyeshtha => "கேட்டை",
        Nakshatra::Mula => "மூலம்",
        Nakshatra::PurvaAshadha => "பூராடம்",
        Nakshatra::UttaraAshadha => "உத்திராடம்",
        Nakshatra::Shravana => "திருவோணம்",
        Nakshatra::Dhanishta => "அவிட்டம்",
        Nakshatra::Shatabhisha => "சதயம்",
        Nakshatra::PurvaBhadrapada => "பூரட்டாதி",
        Nakshatra::UttaraBhadrapada => "உத்திரட்டாதி",
        Nakshatra::Revati => "ரேவதி",
    }
}

fn nakshatra_name_te(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "అశ్వని",
        Nakshatra::Bharani => "భరణి",
        Nakshatra::Krittika => "కృత్తిక",
        Nakshatra::Rohini => "రోహిణి",
        Nakshatra::Mrigashira => "మృగశిర",
        Nakshatra::Ardra => "ఆరుద్ర",
        Nakshatra::Punarvasu => "పునర్వసు",
        Nakshatra::Pushya => "పుష్యమి",
        Nakshatra::Ashlesha => "ఆశ్లేష",
        Nakshatra::Magha => "మఘ",
        Nakshatra::PurvaPhalguni => "పూర్వఫల్గుణి",
        Nakshatra::UttaraPhalguni => "ఉత్తరఫల్గుణి",
        Nakshatra::Hasta => "హస్త",
        Nakshatra::Chitra => "చిత్త",
        Nakshatra::Swati => "స్వాతి",
        Nakshatra::Vishakha => "విశాఖ",
        Nakshatra::Anuradha => "అనూరాధ",
        Nakshatra::Jyeshtha => "జ్యేష్ఠ",
        Nakshatra::Mula => "మూల",
        Nakshatra::PurvaAshadha => "పూర్వాషాఢ",
        Nakshatra::UttaraAshadha => "ఉత్తరాషాఢ",
        Nakshatra::Shravana => "శ్రవణం",
        Nakshatra::Dhanishta => "ధనిష్ఠ",
        Nakshatra::Shatabhisha => "శతభిషం",
        Nakshatra::PurvaBhadrapada => "పూర్వాభాద్ర",
        Nakshatra::UttaraBhadrapada => "ఉత్తరాభాద్ర",
        Nakshatra::Revati => "రేవతి",
    }
}

fn nakshatra_name_kn(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "ಅಶ್ವಿನಿ",
        Nakshatra::Bharani => "ಭರಣಿ",
        Nakshatra::Krittika => "ಕೃತ್ತಿಕಾ",
        Nakshatra::Rohini => "ರೋಹಿಣಿ",
        Nakshatra::Mrigashira => "ಮೃಗಶಿರಾ",
        Nakshatra::Ardra => "ಆರ್ದ್ರಾ",
        Nakshatra::Punarvasu => "ಪುನರ್ವಸು",
        Nakshatra::Pushya => "ಪುಷ್ಯ",
        Nakshatra::Ashlesha => "ಆಶ್ಲೇಷಾ",
        Nakshatra::Magha => "ಮಘಾ",
        Nakshatra::PurvaPhalguni => "ಪೂರ್ವ ಫಲ್ಗುಣಿ",
        Nakshatra::UttaraPhalguni => "ಉತ್ತರ ಫಲ್ಗುಣಿ",
        Nakshatra::Hasta => "ಹಸ್ತ",
        Nakshatra::Chitra => "ಚಿತ್ರಾ",
        Nakshatra::Swati => "ಸ್ವಾತಿ",
        Nakshatra::Vishakha => "ವಿಶಾಖ",
        Nakshatra::Anuradha => "ಅನುರಾಧಾ",
        Nakshatra::Jyeshtha => "ಜ್ಯೇಷ್ಠಾ",
        Nakshatra::Mula => "ಮೂಲಾ",
        Nakshatra::PurvaAshadha => "ಪೂರ್ವಾಷಾಢ",
        Nakshatra::UttaraAshadha => "ಉತ್ತರಾಷಾಢ",
        Nakshatra::Shravana => "ಶ್ರವಣ",
        Nakshatra::Dhanishta => "ಧನಿಷ್ಠಾ",
        Nakshatra::Shatabhisha => "ಶತಭಿಷಾ",
        Nakshatra::PurvaBhadrapada => "ಪೂರ್ವಾಭಾದ್ರ",
        Nakshatra::UttaraBhadrapada => "ಉತ್ತರಾಭಾದ್ರ",
        Nakshatra::Revati => "ರೇವತಿ",
    }
}

fn nakshatra_name_ml(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "അശ്വതി",
        Nakshatra::Bharani => "ഭരണി",
        Nakshatra::Krittika => "കാര്‍ത്തിക",
        Nakshatra::Rohini => "രോഹിണി",
        Nakshatra::Mrigashira => "മകയിരം",
        Nakshatra::Ardra => "തിരുവാതിര",
        Nakshatra::Punarvasu => "പുണര്‍തം",
        Nakshatra::Pushya => "പൂയം",
        Nakshatra::Ashlesha => "ആയില്യം",
        Nakshatra::Magha => "മകം",
        Nakshatra::PurvaPhalguni => "പൂരം",
        Nakshatra::UttaraPhalguni => "ഉത്രം",
        Nakshatra::Hasta => "അത്തം",
        Nakshatra::Chitra => "ചിത്തിര",
        Nakshatra::Swati => "ചോതി",
        Nakshatra::Vishakha => "വിശാഖം",
        Nakshatra::Anuradha => "അനിഴം",
        Nakshatra::Jyeshtha => "തൃക്കേട്ട",
        Nakshatra::Mula => "മൂലം",
        Nakshatra::PurvaAshadha => "പൂരാടം",
        Nakshatra::UttaraAshadha => "ഉത്രാടം",
        Nakshatra::Shravana => "തിരുവോണം",
        Nakshatra::Dhanishta => "അവിട്ടം",
        Nakshatra::Shatabhisha => "ചതയം",
        Nakshatra::PurvaBhadrapada => "പൂരുരുട്ടാതി",
        Nakshatra::UttaraBhadrapada => "ഉത്തൃട്ടാതി",
        Nakshatra::Revati => "രേവതി",
    }
}

fn nakshatra_name_bn(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "অশ্বিনী",
        Nakshatra::Bharani => "ভরণী",
        Nakshatra::Krittika => "কৃত্তিকা",
        Nakshatra::Rohini => "রোহিণী",
        Nakshatra::Mrigashira => "মৃগশিরা",
        Nakshatra::Ardra => "আর্দ্রা",
        Nakshatra::Punarvasu => "পুনর্বসু",
        Nakshatra::Pushya => "পুষ্যা",
        Nakshatra::Ashlesha => "আশ্লেষা",
        Nakshatra::Magha => "মঘা",
        Nakshatra::PurvaPhalguni => "পূর্বফাল্গুনী",
        Nakshatra::UttaraPhalguni => "উত্তরফাল্গুনী",
        Nakshatra::Hasta => "হস্তা",
        Nakshatra::Chitra => "চিত্রা",
        Nakshatra::Swati => "স্বাতী",
        Nakshatra::Vishakha => "বিশাখা",
        Nakshatra::Anuradha => "অনুরাধা",
        Nakshatra::Jyeshtha => "জ্যেষ্ঠা",
        Nakshatra::Mula => "মূলা",
        Nakshatra::PurvaAshadha => "পূর্বাষাঢ়া",
        Nakshatra::UttaraAshadha => "উত্তরাষাঢ়া",
        Nakshatra::Shravana => "শ্রবণা",
        Nakshatra::Dhanishta => "ধনিষ্ঠা",
        Nakshatra::Shatabhisha => "শতভিষা",
        Nakshatra::PurvaBhadrapada => "পূর্বভাদ্রপদ",
        Nakshatra::UttaraBhadrapada => "উত্তরভাদ্রপদ",
        Nakshatra::Revati => "রেবতী",
    }
}

fn nakshatra_name_gu(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "અશ્વિની",
        Nakshatra::Bharani => "ભરણી",
        Nakshatra::Krittika => "કૃત્તિકા",
        Nakshatra::Rohini => "રોહિણી",
        Nakshatra::Mrigashira => "મૃગશીર્ષ",
        Nakshatra::Ardra => "આર્દ્રા",
        Nakshatra::Punarvasu => "પુનર્વસુ",
        Nakshatra::Pushya => "પુષ્ય",
        Nakshatra::Ashlesha => "આશ્લેષા",
        Nakshatra::Magha => "મઘા",
        Nakshatra::PurvaPhalguni => "પૂર્વા ફાલ્ગુની",
        Nakshatra::UttaraPhalguni => "ઉત્તરા ફાલ્ગુની",
        Nakshatra::Hasta => "હસ્ત",
        Nakshatra::Chitra => "ચિત્રા",
        Nakshatra::Swati => "સ્વાતિ",
        Nakshatra::Vishakha => "વિશાખા",
        Nakshatra::Anuradha => "અનુરાધા",
        Nakshatra::Jyeshtha => "જ્યેષ્ઠા",
        Nakshatra::Mula => "મૂળ",
        Nakshatra::PurvaAshadha => "પૂર્વાષાઢા",
        Nakshatra::UttaraAshadha => "ઉત્તરાષાઢા",
        Nakshatra::Shravana => "શ્રવણ",
        Nakshatra::Dhanishta => "ધનિષ્ઠા",
        Nakshatra::Shatabhisha => "શતભિષા",
        Nakshatra::PurvaBhadrapada => "પૂર્વાભાદ્રપદ",
        Nakshatra::UttaraBhadrapada => "ઉત્તરાભાદ્રપદ",
        Nakshatra::Revati => "રેવતી",
    }
}

fn nakshatra_name_mr(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "अश्विनी",
        Nakshatra::Bharani => "भरणी",
        Nakshatra::Krittika => "कृत्तिका",
        Nakshatra::Rohini => "रोहिणी",
        Nakshatra::Mrigashira => "मृगशीर्ष",
        Nakshatra::Ardra => "आर्द्रा",
        Nakshatra::Punarvasu => "पुनर्वसू",
        Nakshatra::Pushya => "पुष्य",
        Nakshatra::Ashlesha => "आश्लेषा",
        Nakshatra::Magha => "मघा",
        Nakshatra::PurvaPhalguni => "पूर्वा फाल्गुनी",
        Nakshatra::UttaraPhalguni => "उत्तरा फाल्गुनी",
        Nakshatra::Hasta => "हस्त",
        Nakshatra::Chitra => "चित्रा",
        Nakshatra::Swati => "स्वाती",
        Nakshatra::Vishakha => "विशाखा",
        Nakshatra::Anuradha => "अनुराधा",
        Nakshatra::Jyeshtha => "ज्येष्ठा",
        Nakshatra::Mula => "मूळ",
        Nakshatra::PurvaAshadha => "पूर्वाषाढा",
        Nakshatra::UttaraAshadha => "उत्तराषाढा",
        Nakshatra::Shravana => "श्रवण",
        Nakshatra::Dhanishta => "धनिष्ठा",
        Nakshatra::Shatabhisha => "शतभिषा",
        Nakshatra::PurvaBhadrapada => "पूर्वाभाद्रपदा",
        Nakshatra::UttaraBhadrapada => "उत्तराभाद्रपदा",
        Nakshatra::Revati => "रेवती",
    }
}

fn nakshatra_name_pa(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "ਅਸ਼ਵਿਨੀ",
        Nakshatra::Bharani => "ਭਰਨੀ",
        Nakshatra::Krittika => "ਕ੍ਰਿਤਿਕਾ",
        Nakshatra::Rohini => "ਰੋਹਿਣੀ",
        Nakshatra::Mrigashira => "ਮ੍ਰਿਗਸ਼ਿਰਾ",
        Nakshatra::Ardra => "ਆਰਦਰਾ",
        Nakshatra::Punarvasu => "ਪੁਨਰਵਸੂ",
        Nakshatra::Pushya => "ਪੁਸ਼ਯ",
        Nakshatra::Ashlesha => "ਅਸ਼ਲੇਸ਼ਾ",
        Nakshatra::Magha => "ਮਘਾ",
        Nakshatra::PurvaPhalguni => "ਪੂਰਵਾ ਫਾਲਗੁਣੀ",
        Nakshatra::UttaraPhalguni => "ਉੱਤਰਾ ਫਾਲਗੁਣੀ",
        Nakshatra::Hasta => "ਹਸਤ",
        Nakshatra::Chitra => "ਚਿੱਤਰਾ",
        Nakshatra::Swati => "ਸਵਾਤੀ",
        Nakshatra::Vishakha => "ਵਿਸ਼ਾਖਾ",
        Nakshatra::Anuradha => "ਅਨੁਰਾਧਾ",
        Nakshatra::Jyeshtha => "ਜਯੇਸ਼ਠਾ",
        Nakshatra::Mula => "ਮੂਲ",
        Nakshatra::PurvaAshadha => "ਪੂਰਵਾਸ਼ਾਢਾ",
        Nakshatra::UttaraAshadha => "ਉੱਤਰਾਸ਼ਾਢਾ",
        Nakshatra::Shravana => "ਸ਼੍ਰਵਣ",
        Nakshatra::Dhanishta => "ਧਨਿਸ਼ਠਾ",
        Nakshatra::Shatabhisha => "ਸ਼ਤਭਿਸ਼ਾ",
        Nakshatra::PurvaBhadrapada => "ਪੂਰਵਾਭਾਦ੍ਰਪਦ",
        Nakshatra::UttaraBhadrapada => "ਉੱਤਰਾਭਾਦ੍ਰਪਦ",
        Nakshatra::Revati => "ਰੇਵਤੀ",
    }
}

fn nakshatra_name_or(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "ଅଶ୍ୱିନୀ",
        Nakshatra::Bharani => "ଭରଣୀ",
        Nakshatra::Krittika => "କୃତ୍ତିକା",
        Nakshatra::Rohini => "ରୋହିଣୀ",
        Nakshatra::Mrigashira => "ମୃଗଶିରା",
        Nakshatra::Ardra => "ଆର୍ଦ୍ରା",
        Nakshatra::Punarvasu => "ପୁନର୍ବସୁ",
        Nakshatra::Pushya => "ପୁଷ୍ୟ",
        Nakshatra::Ashlesha => "ଆଶ୍ଳେଷା",
        Nakshatra::Magha => "ମଘା",
        Nakshatra::PurvaPhalguni => "ପୂର୍ବଫାଲ୍ଗୁନୀ",
        Nakshatra::UttaraPhalguni => "ଉତ୍ତରଫାଲ୍ଗୁନୀ",
        Nakshatra::Hasta => "ହସ୍ତ",
        Nakshatra::Chitra => "ଚିତ୍ରା",
        Nakshatra::Swati => "ସ୍ୱାତୀ",
        Nakshatra::Vishakha => "ବିଶାଖା",
        Nakshatra::Anuradha => "ଅନୁରାଧା",
        Nakshatra::Jyeshtha => "ଜ୍ୟେଷ୍ଠା",
        Nakshatra::Mula => "ମୂଳ",
        Nakshatra::PurvaAshadha => "ପୂର୍ବାଷାଢ଼ା",
        Nakshatra::UttaraAshadha => "ଉତ୍ତରାଷାଢ଼ା",
        Nakshatra::Shravana => "ଶ୍ରବଣ",
        Nakshatra::Dhanishta => "ଧନିଷ୍ଠା",
        Nakshatra::Shatabhisha => "ଶତଭିଷା",
        Nakshatra::PurvaBhadrapada => "ପୂର୍ବଭାଦ୍ରପଦ",
        Nakshatra::UttaraBhadrapada => "ଉତ୍ତରଭାଦ୍ରପଦ",
        Nakshatra::Revati => "ରେବତୀ",
    }
}

fn nakshatra_name_ja(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "アシュヴィニー",
        Nakshatra::Bharani => "バラニー",
        Nakshatra::Krittika => "クリッティカー",
        Nakshatra::Rohini => "ローヒニー",
        Nakshatra::Mrigashira => "ムリガシラー",
        Nakshatra::Ardra => "アールドラー",
        Nakshatra::Punarvasu => "プナルヴァス",
        Nakshatra::Pushya => "プシュヤ",
        Nakshatra::Ashlesha => "アーシュレーシャー",
        Nakshatra::Magha => "マガー",
        Nakshatra::PurvaPhalguni => "プールヴァ・パルグニー",
        Nakshatra::UttaraPhalguni => "ウッタラ・パルグニー",
        Nakshatra::Hasta => "ハスタ",
        Nakshatra::Chitra => "チトラー",
        Nakshatra::Swati => "スヴァーティ",
        Nakshatra::Vishakha => "ヴィシャーカー",
        Nakshatra::Anuradha => "アヌラーダー",
        Nakshatra::Jyeshtha => "ジェーシュター",
        Nakshatra::Mula => "ムーラ",
        Nakshatra::PurvaAshadha => "プールヴァーシャーダー",
        Nakshatra::UttaraAshadha => "ウッタラーシャーダー",
        Nakshatra::Shravana => "シュラヴァナ",
        Nakshatra::Dhanishta => "ダニシュター",
        Nakshatra::Shatabhisha => "シャタビシャー",
        Nakshatra::PurvaBhadrapada => "プールヴァバードラパダー",
        Nakshatra::UttaraBhadrapada => "ウッタラバードラパダー",
        Nakshatra::Revati => "レーヴァティ",
    }
}

fn nakshatra_name_th(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "อัศวินี",
        Nakshatra::Bharani => "ภรณี",
        Nakshatra::Krittika => "กฤตติกา",
        Nakshatra::Rohini => "โรหิณี",
        Nakshatra::Mrigashira => "มฤคศิระ",
        Nakshatra::Ardra => "อารทรา",
        Nakshatra::Punarvasu => "ปุนรวสุ",
        Nakshatra::Pushya => "ปุษยะ",
        Nakshatra::Ashlesha => "อาศเลษา",
        Nakshatra::Magha => "มฆา",
        Nakshatra::PurvaPhalguni => "ปุรวผลคุนี",
        Nakshatra::UttaraPhalguni => "อุตตรผลคุนี",
        Nakshatra::Hasta => "หัสตะ",
        Nakshatra::Chitra => "จิตรา",
        Nakshatra::Swati => "สวาตี",
        Nakshatra::Vishakha => "วิศาขา",
        Nakshatra::Anuradha => "อนุราธา",
        Nakshatra::Jyeshtha => "เชษฐา",
        Nakshatra::Mula => "มูละ",
        Nakshatra::PurvaAshadha => "ปุรวาษาฒา",
        Nakshatra::UttaraAshadha => "อุตตราษาฒา",
        Nakshatra::Shravana => "ศรวณะ",
        Nakshatra::Dhanishta => "ธนิษฐา",
        Nakshatra::Shatabhisha => "ศตภิษช",
        Nakshatra::PurvaBhadrapada => "ปุรวภัทรปทา",
        Nakshatra::UttaraBhadrapada => "อุตตรภัทรปทา",
        Nakshatra::Revati => "เรวดี",
    }
}

// ---------------------------------------------------------------------------
// Nakshatra deity names
// ---------------------------------------------------------------------------

/// Return the presiding deity of a nakshatra in the given language.
///
/// For languages without dedicated deity translations, falls back to English.
pub fn nakshatra_deity(nak: Nakshatra, lang: Language) -> &'static str {
    match lang {
        Language::English => nak.deity(), // reuse existing method
        Language::Hindi | Language::Sanskrit | Language::Marathi => nakshatra_deity_hi(nak),
        Language::Tamil => nakshatra_deity_ta(nak),
        Language::Telugu => nakshatra_deity_te(nak),
        // Kannada, Malayalam, Bengali, Gujarati, Punjabi, Odia share Devanagari
        // deity names as the deities are pan-Indian
        Language::Kannada | Language::Malayalam | Language::Bengali | Language::Gujarati
        | Language::Punjabi | Language::Odia => nakshatra_deity_hi(nak),
        // Global languages fall back to English
        Language::Spanish
        | Language::Portuguese
        | Language::French
        | Language::German
        | Language::Japanese
        | Language::Thai
        | Language::Indonesian => nak.deity(),
    }
}

fn nakshatra_deity_hi(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "अश्विनी कुमार",
        Nakshatra::Bharani => "यम",
        Nakshatra::Krittika => "अग्नि",
        Nakshatra::Rohini => "ब्रह्मा",
        Nakshatra::Mrigashira => "सोम",
        Nakshatra::Ardra => "रुद्र",
        Nakshatra::Punarvasu => "अदिति",
        Nakshatra::Pushya => "बृहस्पति",
        Nakshatra::Ashlesha => "सर्प",
        Nakshatra::Magha => "पितर",
        Nakshatra::PurvaPhalguni => "भग",
        Nakshatra::UttaraPhalguni => "अर्यमा",
        Nakshatra::Hasta => "सवितृ",
        Nakshatra::Chitra => "त्वष्टा",
        Nakshatra::Swati => "वायु",
        Nakshatra::Vishakha => "इन्द्राग्नि",
        Nakshatra::Anuradha => "मित्र",
        Nakshatra::Jyeshtha => "इन्द्र",
        Nakshatra::Mula => "निऋति",
        Nakshatra::PurvaAshadha => "अपः",
        Nakshatra::UttaraAshadha => "विश्वेदेव",
        Nakshatra::Shravana => "विष्णु",
        Nakshatra::Dhanishta => "वसु",
        Nakshatra::Shatabhisha => "वरुण",
        Nakshatra::PurvaBhadrapada => "अज एकपाद",
        Nakshatra::UttaraBhadrapada => "अहिर्बुध्न्य",
        Nakshatra::Revati => "पूषन्",
    }
}

fn nakshatra_deity_ta(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "அசுவினி தேவர்கள்",
        Nakshatra::Bharani => "யமன்",
        Nakshatra::Krittika => "அக்னி",
        Nakshatra::Rohini => "பிரம்மா",
        Nakshatra::Mrigashira => "சோமன்",
        Nakshatra::Ardra => "ருத்ரன்",
        Nakshatra::Punarvasu => "அதிதி",
        Nakshatra::Pushya => "பிருகஸ்பதி",
        Nakshatra::Ashlesha => "சர்ப்பம்",
        Nakshatra::Magha => "பித்ருக்கள்",
        Nakshatra::PurvaPhalguni => "பகன்",
        Nakshatra::UttaraPhalguni => "அர்யமன்",
        Nakshatra::Hasta => "சவிதா",
        Nakshatra::Chitra => "துவஷ்டா",
        Nakshatra::Swati => "வாயு",
        Nakshatra::Vishakha => "இந்திராக்னி",
        Nakshatra::Anuradha => "மித்ரன்",
        Nakshatra::Jyeshtha => "இந்திரன்",
        Nakshatra::Mula => "நிருதி",
        Nakshatra::PurvaAshadha => "அபஸ்",
        Nakshatra::UttaraAshadha => "விஸ்வதேவர்",
        Nakshatra::Shravana => "விஷ்ணு",
        Nakshatra::Dhanishta => "வசுக்கள்",
        Nakshatra::Shatabhisha => "வருணன்",
        Nakshatra::PurvaBhadrapada => "அஜ ஏகபாதன்",
        Nakshatra::UttaraBhadrapada => "அஹிர்புத்னியன்",
        Nakshatra::Revati => "பூஷன்",
    }
}

fn nakshatra_deity_te(n: Nakshatra) -> &'static str {
    match n {
        Nakshatra::Ashwini => "అశ్వినీ దేవతలు",
        Nakshatra::Bharani => "యముడు",
        Nakshatra::Krittika => "అగ్ని",
        Nakshatra::Rohini => "బ్రహ్మ",
        Nakshatra::Mrigashira => "సోముడు",
        Nakshatra::Ardra => "రుద్రుడు",
        Nakshatra::Punarvasu => "అదితి",
        Nakshatra::Pushya => "బృహస్పతి",
        Nakshatra::Ashlesha => "సర్పములు",
        Nakshatra::Magha => "పితృదేవతలు",
        Nakshatra::PurvaPhalguni => "భగుడు",
        Nakshatra::UttaraPhalguni => "అర్యముడు",
        Nakshatra::Hasta => "సవితా",
        Nakshatra::Chitra => "త్వష్ట",
        Nakshatra::Swati => "వాయుదేవుడు",
        Nakshatra::Vishakha => "ఇంద్రాగ్ని",
        Nakshatra::Anuradha => "మిత్రుడు",
        Nakshatra::Jyeshtha => "ఇంద్రుడు",
        Nakshatra::Mula => "నిరృతి",
        Nakshatra::PurvaAshadha => "అపస్",
        Nakshatra::UttaraAshadha => "విశ్వదేవతలు",
        Nakshatra::Shravana => "విష్ణువు",
        Nakshatra::Dhanishta => "వసువులు",
        Nakshatra::Shatabhisha => "వరుణుడు",
        Nakshatra::PurvaBhadrapada => "అజ ఏకపాదుడు",
        Nakshatra::UttaraBhadrapada => "అహిర్బుధ్న్యుడు",
        Nakshatra::Revati => "పూషుడు",
    }
}

// ---------------------------------------------------------------------------
// Tithi names
// ---------------------------------------------------------------------------

/// Return the name of a tithi in the given language.
///
/// The tithi number (1-30) is used; names repeat across Shukla and Krishna
/// pakshas except for 15 (Purnima) and 30 (Amavasya).
/// For languages without dedicated tithi translations, falls back to English.
pub fn tithi_name(tithi: &Tithi, lang: Language) -> &'static str {
    match lang {
        Language::English => tithi.name(),
        Language::Hindi | Language::Sanskrit | Language::Marathi => tithi_name_hi(tithi),
        Language::Tamil => tithi_name_ta(tithi),
        Language::Telugu => tithi_name_te(tithi),
        // Other Indic + global languages fall back to English
        Language::Kannada | Language::Malayalam | Language::Bengali | Language::Gujarati
        | Language::Punjabi | Language::Odia => tithi_name_hi(tithi),
        Language::Spanish
        | Language::Portuguese
        | Language::French
        | Language::German
        | Language::Japanese
        | Language::Thai
        | Language::Indonesian => tithi.name(),
    }
}

fn tithi_name_hi(t: &Tithi) -> &'static str {
    const NAMES: [&str; 30] = [
        "प्रतिपदा",
        "द्वितीया",
        "तृतीया",
        "चतुर्थी",
        "पञ्चमी",
        "षष्ठी",
        "सप्तमी",
        "अष्टमी",
        "नवमी",
        "दशमी",
        "एकादशी",
        "द्वादशी",
        "त्रयोदशी",
        "चतुर्दशी",
        "पूर्णिमा",
        "प्रतिपदा",
        "द्वितीया",
        "तृतीया",
        "चतुर्थी",
        "पञ्चमी",
        "षष्ठी",
        "सप्तमी",
        "अष्टमी",
        "नवमी",
        "दशमी",
        "एकादशी",
        "द्वादशी",
        "त्रयोदशी",
        "चतुर्दशी",
        "अमावस्या",
    ];
    NAMES[(t.number - 1) as usize % 30]
}

fn tithi_name_ta(t: &Tithi) -> &'static str {
    const NAMES: [&str; 30] = [
        "பிரதமை",
        "துவிதியை",
        "திருதியை",
        "சதுர்த்தி",
        "பஞ்சமி",
        "சஷ்டி",
        "சப்தமி",
        "அஷ்டமி",
        "நவமி",
        "தசமி",
        "ஏகாதசி",
        "துவாதசி",
        "திரயோதசி",
        "சதுர்தசி",
        "பௌர்ணமி",
        "பிரதமை",
        "துவிதியை",
        "திருதியை",
        "சதுர்த்தி",
        "பஞ்சமி",
        "சஷ்டி",
        "சப்தமி",
        "அஷ்டமி",
        "நவமி",
        "தசமி",
        "ஏகாதசி",
        "துவாதசி",
        "திரயோதசி",
        "சதுர்தசி",
        "அமாவாசை",
    ];
    NAMES[(t.number - 1) as usize % 30]
}

fn tithi_name_te(t: &Tithi) -> &'static str {
    const NAMES: [&str; 30] = [
        "పాడ్యమి",
        "విదియ",
        "తదియ",
        "చవితి",
        "పంచమి",
        "షష్ఠి",
        "సప్తమి",
        "అష్టమి",
        "నవమి",
        "దశమి",
        "ఏకాదశి",
        "ద్వాదశి",
        "త్రయోదశి",
        "చతుర్దశి",
        "పూర్ణిమ",
        "పాడ్యమి",
        "విదియ",
        "తదియ",
        "చవితి",
        "పంచమి",
        "షష్ఠి",
        "సప్తమి",
        "అష్టమి",
        "నవమి",
        "దశమి",
        "ఏకాదశి",
        "ద్వాదశి",
        "త్రయోదశి",
        "చతుర్దశి",
        "అమావాస్య",
    ];
    NAMES[(t.number - 1) as usize % 30]
}

// ---------------------------------------------------------------------------
// Yoga names
// ---------------------------------------------------------------------------

/// Return the name of a nithya yoga in the given language.
/// For languages without dedicated yoga translations, falls back to English.
pub fn yoga_name(yoga: &Yoga, lang: Language) -> &'static str {
    match lang {
        Language::English => yoga.name(),
        Language::Hindi | Language::Sanskrit | Language::Marathi => yoga_name_hi(yoga),
        Language::Tamil => yoga_name_ta(yoga),
        Language::Telugu => yoga_name_te(yoga),
        Language::Kannada | Language::Malayalam | Language::Bengali | Language::Gujarati
        | Language::Punjabi | Language::Odia => yoga_name_hi(yoga),
        Language::Spanish
        | Language::Portuguese
        | Language::French
        | Language::German
        | Language::Japanese
        | Language::Thai
        | Language::Indonesian => yoga.name(),
    }
}

fn yoga_name_hi(y: &Yoga) -> &'static str {
    const NAMES: [&str; 27] = [
        "विष्कम्भ",
        "प्रीति",
        "आयुष्मान्",
        "सौभाग्य",
        "शोभन",
        "अतिगण्ड",
        "सुकर्मा",
        "धृति",
        "शूल",
        "गण्ड",
        "वृद्धि",
        "ध्रुव",
        "व्याघात",
        "हर्षण",
        "वज्र",
        "सिद्धि",
        "व्यतीपात",
        "वरीयान्",
        "परिघ",
        "शिव",
        "सिद्ध",
        "साध्य",
        "शुभ",
        "शुक्ल",
        "ब्रह्म",
        "इन्द्र",
        "वैधृति",
    ];
    NAMES[(y.number - 1) as usize % 27]
}

fn yoga_name_ta(y: &Yoga) -> &'static str {
    const NAMES: [&str; 27] = [
        "விஷ்கம்பம்",
        "ப்ரீதி",
        "ஆயுஷ்மான்",
        "சௌபாக்யம்",
        "சோபனம்",
        "அதிகண்டம்",
        "சுகர்மா",
        "திருதி",
        "சூலம்",
        "கண்டம்",
        "விருத்தி",
        "த்ருவம்",
        "வியாகாதம்",
        "ஹர்ஷணம்",
        "வஜ்ரம்",
        "சித்தி",
        "வியதீபாதம்",
        "வரீயான்",
        "பரிகம்",
        "சிவம்",
        "சித்தம்",
        "சாத்தியம்",
        "சுபம்",
        "சுக்லம்",
        "பிரம்மம்",
        "இந்திரம்",
        "வைதிருதி",
    ];
    NAMES[(y.number - 1) as usize % 27]
}

fn yoga_name_te(y: &Yoga) -> &'static str {
    const NAMES: [&str; 27] = [
        "విష్కంభం",
        "ప్రీతి",
        "ఆయుష్మాన్",
        "సౌభాగ్యం",
        "శోభనం",
        "అతిగండం",
        "సుకర్మ",
        "ధృతి",
        "శూలం",
        "గండం",
        "వృద్ధి",
        "ధ్రువం",
        "వ్యాఘాతం",
        "హర్షణం",
        "వజ్రం",
        "సిద్ధి",
        "వ్యతీపాతం",
        "వరీయాన్",
        "పరిఘం",
        "శివం",
        "సిద్ధం",
        "సాధ్యం",
        "శుభం",
        "శుక్లం",
        "బ్రహ్మం",
        "ఇంద్రం",
        "వైధృతి",
    ];
    NAMES[(y.number - 1) as usize % 27]
}

// ---------------------------------------------------------------------------
// Karana names
// ---------------------------------------------------------------------------

/// Return the name of a karana in the given language.
/// For languages without dedicated karana translations, falls back to English.
pub fn karana_name(karana: &Karana, lang: Language) -> &'static str {
    match lang {
        Language::English => karana.name(),
        Language::Hindi | Language::Sanskrit | Language::Marathi => karana_name_hi(karana),
        Language::Tamil => karana_name_ta(karana),
        Language::Telugu => karana_name_te(karana),
        Language::Kannada | Language::Malayalam | Language::Bengali | Language::Gujarati
        | Language::Punjabi | Language::Odia => karana_name_hi(karana),
        Language::Spanish
        | Language::Portuguese
        | Language::French
        | Language::German
        | Language::Japanese
        | Language::Thai
        | Language::Indonesian => karana.name(),
    }
}

fn karana_name_hi(k: &Karana) -> &'static str {
    const NAMES: [&str; 11] = [
        "बव",
        "बालव",
        "कौलव",
        "तैतिल",
        "गर",
        "वणिज",
        "विष्टि",
        "शकुनि",
        "चतुष्पद",
        "नाग",
        "किंस्तुघ्न",
    ];
    NAMES[k.name_index as usize % 11]
}

fn karana_name_ta(k: &Karana) -> &'static str {
    const NAMES: [&str; 11] = [
        "பவம்",
        "பாலவம்",
        "கௌலவம்",
        "தைதுலம்",
        "கரம்",
        "வணிஜம்",
        "விஷ்டி",
        "சகுனி",
        "சதுஷ்பதம்",
        "நாகம்",
        "கிம்ஸ்துக்னம்",
    ];
    NAMES[k.name_index as usize % 11]
}

fn karana_name_te(k: &Karana) -> &'static str {
    const NAMES: [&str; 11] = [
        "బవ",
        "బాలవ",
        "కౌలవ",
        "తైతిల",
        "గర",
        "వణిజ",
        "విష్టి",
        "శకుని",
        "చతుష్పద",
        "నాగ",
        "కింస్తుఘ్న",
    ];
    NAMES[k.name_index as usize % 11]
}

// ---------------------------------------------------------------------------
// Vara (weekday) names
// ---------------------------------------------------------------------------

/// Return the name of a vara (weekday) in the given language.
pub fn vara_name(vara: Vara, lang: Language) -> &'static str {
    match lang {
        Language::English => vara.name(),
        Language::Hindi | Language::Sanskrit => vara_name_hi(vara),
        Language::Tamil => vara_name_ta(vara),
        Language::Telugu => vara_name_te(vara),
        Language::Kannada => vara_name_kn(vara),
        Language::Malayalam => vara_name_ml(vara),
        Language::Bengali => vara_name_bn(vara),
        Language::Gujarati => vara_name_gu(vara),
        Language::Marathi => vara_name_mr(vara),
        Language::Punjabi => vara_name_pa(vara),
        Language::Odia => vara_name_or(vara),
        Language::Spanish => vara_name_es(vara),
        Language::Portuguese => vara_name_pt(vara),
        Language::French => vara_name_fr(vara),
        Language::German => vara_name_de(vara),
        Language::Japanese => vara_name_ja(vara),
        Language::Thai => vara_name_th(vara),
        Language::Indonesian => vara_name_id(vara),
    }
}

fn vara_name_hi(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "रविवार",
        Vara::Monday => "सोमवार",
        Vara::Tuesday => "मंगलवार",
        Vara::Wednesday => "बुधवार",
        Vara::Thursday => "गुरुवार",
        Vara::Friday => "शुक्रवार",
        Vara::Saturday => "शनिवार",
    }
}

fn vara_name_ta(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "ஞாயிற்றுக்கிழமை",
        Vara::Monday => "திங்கட்கிழமை",
        Vara::Tuesday => "செவ்வாய்க்கிழமை",
        Vara::Wednesday => "புதன்கிழமை",
        Vara::Thursday => "வியாழக்கிழமை",
        Vara::Friday => "வெள்ளிக்கிழமை",
        Vara::Saturday => "சனிக்கிழமை",
    }
}

fn vara_name_te(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "ఆదివారం",
        Vara::Monday => "సోమవారం",
        Vara::Tuesday => "మంగళవారం",
        Vara::Wednesday => "బుధవారం",
        Vara::Thursday => "గురువారం",
        Vara::Friday => "శుక్రవారం",
        Vara::Saturday => "శనివారం",
    }
}

fn vara_name_kn(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "ಭಾನುವಾರ",
        Vara::Monday => "ಸೋಮವಾರ",
        Vara::Tuesday => "ಮಂಗಳವಾರ",
        Vara::Wednesday => "ಬುಧವಾರ",
        Vara::Thursday => "ಗುರುವಾರ",
        Vara::Friday => "ಶುಕ್ರವಾರ",
        Vara::Saturday => "ಶನಿವಾರ",
    }
}

fn vara_name_ml(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "ഞായറാഴ്ച",
        Vara::Monday => "തിങ്കളാഴ്ച",
        Vara::Tuesday => "ചൊവ്വാഴ്ച",
        Vara::Wednesday => "ബുധനാഴ്ച",
        Vara::Thursday => "വ്യാഴാഴ്ച",
        Vara::Friday => "വെള്ളിയാഴ്ച",
        Vara::Saturday => "ശനിയാഴ്ച",
    }
}

fn vara_name_bn(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "রবিবার",
        Vara::Monday => "সোমবার",
        Vara::Tuesday => "মঙ্গলবার",
        Vara::Wednesday => "বুধবার",
        Vara::Thursday => "বৃহস্পতিবার",
        Vara::Friday => "শুক্রবার",
        Vara::Saturday => "শনিবার",
    }
}

fn vara_name_gu(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "રવિવાર",
        Vara::Monday => "સોમવાર",
        Vara::Tuesday => "મંગળવાર",
        Vara::Wednesday => "બુધવાર",
        Vara::Thursday => "ગુરુવાર",
        Vara::Friday => "શુક્રવાર",
        Vara::Saturday => "શનિવાર",
    }
}

fn vara_name_mr(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "रविवार",
        Vara::Monday => "सोमवार",
        Vara::Tuesday => "मंगळवार",
        Vara::Wednesday => "बुधवार",
        Vara::Thursday => "गुरुवार",
        Vara::Friday => "शुक्रवार",
        Vara::Saturday => "शनिवार",
    }
}

fn vara_name_pa(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "ਐਤਵਾਰ",
        Vara::Monday => "ਸੋਮਵਾਰ",
        Vara::Tuesday => "ਮੰਗਲਵਾਰ",
        Vara::Wednesday => "ਬੁੱਧਵਾਰ",
        Vara::Thursday => "ਵੀਰਵਾਰ",
        Vara::Friday => "ਸ਼ੁੱਕਰਵਾਰ",
        Vara::Saturday => "ਸ਼ਨਿੱਚਰਵਾਰ",
    }
}

fn vara_name_or(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "ରବିବାର",
        Vara::Monday => "ସୋମବାର",
        Vara::Tuesday => "ମଙ୍ଗଳବାର",
        Vara::Wednesday => "ବୁଧବାର",
        Vara::Thursday => "ଗୁରୁବାର",
        Vara::Friday => "ଶୁକ୍ରବାର",
        Vara::Saturday => "ଶନିବାର",
    }
}

fn vara_name_es(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "Domingo",
        Vara::Monday => "Lunes",
        Vara::Tuesday => "Martes",
        Vara::Wednesday => "Miércoles",
        Vara::Thursday => "Jueves",
        Vara::Friday => "Viernes",
        Vara::Saturday => "Sábado",
    }
}

fn vara_name_pt(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "Domingo",
        Vara::Monday => "Segunda-feira",
        Vara::Tuesday => "Terça-feira",
        Vara::Wednesday => "Quarta-feira",
        Vara::Thursday => "Quinta-feira",
        Vara::Friday => "Sexta-feira",
        Vara::Saturday => "Sábado",
    }
}

fn vara_name_fr(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "Dimanche",
        Vara::Monday => "Lundi",
        Vara::Tuesday => "Mardi",
        Vara::Wednesday => "Mercredi",
        Vara::Thursday => "Jeudi",
        Vara::Friday => "Vendredi",
        Vara::Saturday => "Samedi",
    }
}

fn vara_name_de(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "Sonntag",
        Vara::Monday => "Montag",
        Vara::Tuesday => "Dienstag",
        Vara::Wednesday => "Mittwoch",
        Vara::Thursday => "Donnerstag",
        Vara::Friday => "Freitag",
        Vara::Saturday => "Samstag",
    }
}

fn vara_name_ja(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "日曜日",
        Vara::Monday => "月曜日",
        Vara::Tuesday => "火曜日",
        Vara::Wednesday => "水曜日",
        Vara::Thursday => "木曜日",
        Vara::Friday => "金曜日",
        Vara::Saturday => "土曜日",
    }
}

fn vara_name_th(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "วันอาทิตย์",
        Vara::Monday => "วันจันทร์",
        Vara::Tuesday => "วันอังคาร",
        Vara::Wednesday => "วันพุธ",
        Vara::Thursday => "วันพฤหัสบดี",
        Vara::Friday => "วันศุกร์",
        Vara::Saturday => "วันเสาร์",
    }
}

fn vara_name_id(v: Vara) -> &'static str {
    match v {
        Vara::Sunday => "Minggu",
        Vara::Monday => "Senin",
        Vara::Tuesday => "Selasa",
        Vara::Wednesday => "Rabu",
        Vara::Thursday => "Kamis",
        Vara::Friday => "Jumat",
        Vara::Saturday => "Sabtu",
    }
}

// ---------------------------------------------------------------------------
// Paksha names
// ---------------------------------------------------------------------------

/// Return the name of a paksha (lunar fortnight) in the given language.
pub fn paksha_name(paksha: crate::panchang::Paksha, lang: Language) -> &'static str {
    use crate::panchang::Paksha;
    match (paksha, lang) {
        (Paksha::Shukla, Language::English) => "Shukla",
        (Paksha::Krishna, Language::English) => "Krishna",
        (Paksha::Shukla, Language::Hindi | Language::Sanskrit | Language::Marathi) => "शुक्ल",
        (Paksha::Krishna, Language::Hindi | Language::Sanskrit | Language::Marathi) => "कृष्ण",
        (Paksha::Shukla, Language::Tamil) => "சுக்ல",
        (Paksha::Krishna, Language::Tamil) => "கிருஷ்ண",
        (Paksha::Shukla, Language::Telugu) => "శుక్ల",
        (Paksha::Krishna, Language::Telugu) => "కృష్ణ",
        (Paksha::Shukla, Language::Kannada) => "ಶುಕ್ಲ",
        (Paksha::Krishna, Language::Kannada) => "ಕೃಷ್ಣ",
        (Paksha::Shukla, Language::Malayalam) => "ശുക്ല",
        (Paksha::Krishna, Language::Malayalam) => "കൃഷ്ണ",
        (Paksha::Shukla, Language::Bengali) => "শুক্ল",
        (Paksha::Krishna, Language::Bengali) => "কৃষ্ণ",
        (Paksha::Shukla, Language::Gujarati) => "શુક્લ",
        (Paksha::Krishna, Language::Gujarati) => "કૃષ્ણ",
        (Paksha::Shukla, Language::Punjabi) => "ਸ਼ੁਕਲ",
        (Paksha::Krishna, Language::Punjabi) => "ਕ੍ਰਿਸ਼ਨ",
        (Paksha::Shukla, Language::Odia) => "ଶୁକ୍ଳ",
        (Paksha::Krishna, Language::Odia) => "କୃଷ୍ଣ",
        // Global languages use transliteration
        (
            Paksha::Shukla,
            Language::Spanish
            | Language::Portuguese
            | Language::French
            | Language::German
            | Language::Japanese
            | Language::Thai
            | Language::Indonesian,
        ) => "Shukla",
        (
            Paksha::Krishna,
            Language::Spanish
            | Language::Portuguese
            | Language::French
            | Language::German
            | Language::Japanese
            | Language::Thai
            | Language::Indonesian,
        ) => "Krishna",
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Planet names
    // -----------------------------------------------------------------------

    #[test]
    fn planet_names_english_match_existing() {
        // English names must match Planet::name()
        for &p in &Planet::ALL {
            assert_eq!(planet_name(p, Language::English), p.name());
        }
    }

    #[test]
    fn planet_names_hindi_navagraha() {
        assert_eq!(planet_name(Planet::Sun, Language::Hindi), "सूर्य");
        assert_eq!(planet_name(Planet::Moon, Language::Hindi), "चन्द्र");
        assert_eq!(planet_name(Planet::Mars, Language::Hindi), "मंगल");
        assert_eq!(planet_name(Planet::Mercury, Language::Hindi), "बुध");
        assert_eq!(planet_name(Planet::Jupiter, Language::Hindi), "गुरु");
        assert_eq!(planet_name(Planet::Venus, Language::Hindi), "शुक्र");
        assert_eq!(planet_name(Planet::Saturn, Language::Hindi), "शनि");
        assert_eq!(planet_name(Planet::Rahu, Language::Hindi), "राहु");
        assert_eq!(planet_name(Planet::Ketu, Language::Hindi), "केतु");
    }

    #[test]
    fn planet_names_sanskrit() {
        assert_eq!(planet_name(Planet::Sun, Language::Sanskrit), "सूर्यः");
        assert_eq!(planet_name(Planet::Mars, Language::Sanskrit), "कुजः");
        assert_eq!(planet_name(Planet::Jupiter, Language::Sanskrit), "बृहस्पतिः");
        assert_eq!(planet_name(Planet::Saturn, Language::Sanskrit), "शनैश्चरः");
    }

    #[test]
    fn planet_names_tamil() {
        assert_eq!(planet_name(Planet::Sun, Language::Tamil), "சூரியன்");
        assert_eq!(planet_name(Planet::Moon, Language::Tamil), "சந்திரன்");
        assert_eq!(planet_name(Planet::Mars, Language::Tamil), "செவ்வாய்");
        assert_eq!(planet_name(Planet::Saturn, Language::Tamil), "சனி");
    }

    #[test]
    fn planet_names_telugu() {
        assert_eq!(planet_name(Planet::Sun, Language::Telugu), "సూర్యుడు");
        assert_eq!(planet_name(Planet::Moon, Language::Telugu), "చంద్రుడు");
        assert_eq!(planet_name(Planet::Jupiter, Language::Telugu), "గురుడు");
    }

    #[test]
    fn planet_names_kannada() {
        assert_eq!(planet_name(Planet::Sun, Language::Kannada), "ಸೂರ್ಯ");
        assert_eq!(planet_name(Planet::Moon, Language::Kannada), "ಚಂದ್ರ");
        assert_eq!(planet_name(Planet::Mars, Language::Kannada), "ಮಂಗಳ");
        assert_eq!(planet_name(Planet::Jupiter, Language::Kannada), "ಗುರು");
        assert_eq!(planet_name(Planet::Rahu, Language::Kannada), "ರಾಹು");
        assert_eq!(planet_name(Planet::Ketu, Language::Kannada), "ಕೇತು");
    }

    #[test]
    fn planet_names_malayalam() {
        assert_eq!(planet_name(Planet::Sun, Language::Malayalam), "സൂര്യന്\u{200d}");
        assert_eq!(planet_name(Planet::Mars, Language::Malayalam), "ചൊവ്വ");
        assert_eq!(planet_name(Planet::Jupiter, Language::Malayalam), "വ്യാഴം");
    }

    #[test]
    fn planet_names_bengali() {
        assert_eq!(planet_name(Planet::Sun, Language::Bengali), "সূর্য");
        assert_eq!(planet_name(Planet::Moon, Language::Bengali), "চন্দ্র");
        assert_eq!(planet_name(Planet::Jupiter, Language::Bengali), "বৃহস্পতি");
    }

    #[test]
    fn planet_names_gujarati() {
        assert_eq!(planet_name(Planet::Sun, Language::Gujarati), "સૂર્ય");
        assert_eq!(planet_name(Planet::Moon, Language::Gujarati), "ચંદ્ર");
        assert_eq!(planet_name(Planet::Saturn, Language::Gujarati), "શનિ");
    }

    #[test]
    fn planet_names_marathi() {
        assert_eq!(planet_name(Planet::Sun, Language::Marathi), "सूर्य");
        assert_eq!(planet_name(Planet::Mars, Language::Marathi), "मंगळ");
        assert_eq!(planet_name(Planet::Saturn, Language::Marathi), "शनी");
    }

    #[test]
    fn planet_names_spanish() {
        assert_eq!(planet_name(Planet::Sun, Language::Spanish), "Sol");
        assert_eq!(planet_name(Planet::Moon, Language::Spanish), "Luna");
        assert_eq!(planet_name(Planet::Mars, Language::Spanish), "Marte");
        assert_eq!(planet_name(Planet::Jupiter, Language::Spanish), "Júpiter");
    }

    #[test]
    fn planet_names_japanese() {
        assert_eq!(planet_name(Planet::Sun, Language::Japanese), "太陽");
        assert_eq!(planet_name(Planet::Moon, Language::Japanese), "月");
        assert_eq!(planet_name(Planet::Mars, Language::Japanese), "火星");
        assert_eq!(planet_name(Planet::Saturn, Language::Japanese), "土星");
    }

    #[test]
    fn planet_names_thai() {
        assert_eq!(planet_name(Planet::Sun, Language::Thai), "อาทิตย์");
        assert_eq!(planet_name(Planet::Moon, Language::Thai), "จันทร์");
        assert_eq!(planet_name(Planet::Rahu, Language::Thai), "ราหู");
        assert_eq!(planet_name(Planet::Ketu, Language::Thai), "เกตุ");
    }

    #[test]
    fn planet_names_all_languages_non_empty() {
        for &p in &Planet::ALL {
            for &lang in &Language::ALL {
                let name = planet_name(p, lang);
                assert!(!name.is_empty(), "Empty name for {p:?} in {lang}");
            }
        }
    }

    #[test]
    fn western_node_aliases_match_vedic_in_indic() {
        // NorthNode and SouthNode should map to Rahu/Ketu in Indic languages
        for &lang in &[
            Language::Hindi,
            Language::Sanskrit,
            Language::Tamil,
            Language::Telugu,
            Language::Kannada,
            Language::Malayalam,
            Language::Bengali,
            Language::Gujarati,
            Language::Marathi,
            Language::Punjabi,
            Language::Odia,
            Language::Thai,
        ] {
            assert_eq!(
                planet_name(Planet::NorthNode, lang),
                planet_name(Planet::Rahu, lang),
                "NorthNode should match Rahu in {lang}"
            );
            assert_eq!(
                planet_name(Planet::SouthNode, lang),
                planet_name(Planet::Ketu, lang),
                "SouthNode should match Ketu in {lang}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Rashi names
    // -----------------------------------------------------------------------

    #[test]
    fn rashi_names_english_match_western() {
        for idx in 0..12 {
            let r = Rashi::from_index(idx);
            assert_eq!(rashi_name(r, Language::English), r.western_name());
        }
    }

    #[test]
    fn rashi_names_hindi_all_twelve() {
        let expected = [
            "मेष",
            "वृषभ",
            "मिथुन",
            "कर्क",
            "सिंह",
            "कन्या",
            "तुला",
            "वृश्चिक",
            "धनु",
            "मकर",
            "कुम्भ",
            "मीन",
        ];
        for (idx, &exp) in expected.iter().enumerate() {
            assert_eq!(rashi_name(Rashi::from_index(idx), Language::Hindi), exp);
        }
    }

    #[test]
    fn rashi_names_tamil_all_twelve() {
        let expected = [
            "மேஷம்",
            "ரிஷபம்",
            "மிதுனம்",
            "கடகம்",
            "சிம்மம்",
            "கன்னி",
            "துலாம்",
            "விருச்சிகம்",
            "தனுசு",
            "மகரம்",
            "கும்பம்",
            "மீனம்",
        ];
        for (idx, &exp) in expected.iter().enumerate() {
            assert_eq!(rashi_name(Rashi::from_index(idx), Language::Tamil), exp);
        }
    }

    #[test]
    fn rashi_names_sanskrit_all_twelve() {
        let expected = [
            "मेषः",
            "वृषभः",
            "मिथुनम्",
            "कर्कटः",
            "सिंहः",
            "कन्या",
            "तुला",
            "वृश्चिकः",
            "धनुः",
            "मकरः",
            "कुम्भः",
            "मीनः",
        ];
        for (idx, &exp) in expected.iter().enumerate() {
            assert_eq!(rashi_name(Rashi::from_index(idx), Language::Sanskrit), exp);
        }
    }

    #[test]
    fn rashi_names_kannada_all_twelve() {
        let expected = [
            "ಮೇಷ",
            "ವೃಷಭ",
            "ಮಿಥುನ",
            "ಕರ್ಕಾಟಕ",
            "ಸಿಂಹ",
            "ಕನ್ಯಾ",
            "ತುಲಾ",
            "ವೃಶ್ಚಿಕ",
            "ಧನು",
            "ಮಕರ",
            "ಕುಂಭ",
            "ಮೀನ",
        ];
        for (idx, &exp) in expected.iter().enumerate() {
            assert_eq!(rashi_name(Rashi::from_index(idx), Language::Kannada), exp);
        }
    }

    #[test]
    fn rashi_names_french_all_twelve() {
        let expected = [
            "Bélier",
            "Taureau",
            "Gémeaux",
            "Cancer",
            "Lion",
            "Vierge",
            "Balance",
            "Scorpion",
            "Sagittaire",
            "Capricorne",
            "Verseau",
            "Poissons",
        ];
        for (idx, &exp) in expected.iter().enumerate() {
            assert_eq!(rashi_name(Rashi::from_index(idx), Language::French), exp);
        }
    }

    #[test]
    fn rashi_names_german_all_twelve() {
        let expected = [
            "Widder",
            "Stier",
            "Zwillinge",
            "Krebs",
            "Löwe",
            "Jungfrau",
            "Waage",
            "Skorpion",
            "Schütze",
            "Steinbock",
            "Wassermann",
            "Fische",
        ];
        for (idx, &exp) in expected.iter().enumerate() {
            assert_eq!(rashi_name(Rashi::from_index(idx), Language::German), exp);
        }
    }

    #[test]
    fn rashi_names_japanese_all_twelve() {
        let expected = [
            "牡羊座",
            "牡牛座",
            "双子座",
            "蟹座",
            "獅子座",
            "乙女座",
            "天秤座",
            "蠍座",
            "射手座",
            "山羊座",
            "水瓶座",
            "魚座",
        ];
        for (idx, &exp) in expected.iter().enumerate() {
            assert_eq!(rashi_name(Rashi::from_index(idx), Language::Japanese), exp);
        }
    }

    #[test]
    fn rashi_names_all_languages_non_empty() {
        for idx in 0..12 {
            let r = Rashi::from_index(idx);
            for &lang in &Language::ALL {
                let name = rashi_name(r, lang);
                assert!(!name.is_empty(), "Empty rashi name for {r:?} in {lang}");
            }
        }
    }

    // -----------------------------------------------------------------------
    // Nakshatra names
    // -----------------------------------------------------------------------

    #[test]
    fn nakshatra_names_hindi_all_27() {
        let expected = [
            "अश्विनी",
            "भरणी",
            "कृत्तिका",
            "रोहिणी",
            "मृगशिरा",
            "आर्द्रा",
            "पुनर्वसु",
            "पुष्य",
            "आश्लेषा",
            "मघा",
            "पूर्वा फाल्गुनी",
            "उत्तरा फाल्गुनी",
            "हस्त",
            "चित्रा",
            "स्वाति",
            "विशाखा",
            "अनुराधा",
            "ज्येष्ठा",
            "मूल",
            "पूर्वाषाढ़ा",
            "उत्तराषाढ़ा",
            "श्रवण",
            "धनिष्ठा",
            "शतभिषा",
            "पूर्वाभाद्रपद",
            "उत्तराभाद्रपद",
            "रेवती",
        ];
        for (i, &exp) in expected.iter().enumerate() {
            assert_eq!(
                nakshatra_name(Nakshatra::ALL[i], Language::Hindi),
                exp,
                "Mismatch at index {i}"
            );
        }
    }

    #[test]
    fn nakshatra_names_tamil_all_27() {
        let expected = [
            "அசுவினி",
            "பரணி",
            "கிருத்திகை",
            "ரோகிணி",
            "மிருகசீரிடம்",
            "திருவாதிரை",
            "புனர்பூசம்",
            "பூசம்",
            "ஆயில்யம்",
            "மகம்",
            "பூரம்",
            "உத்திரம்",
            "அஸ்தம்",
            "சித்திரை",
            "சுவாதி",
            "விசாகம்",
            "அனுஷம்",
            "கேட்டை",
            "மூலம்",
            "பூராடம்",
            "உத்திராடம்",
            "திருவோணம்",
            "அவிட்டம்",
            "சதயம்",
            "பூரட்டாதி",
            "உத்திரட்டாதி",
            "ரேவதி",
        ];
        for (i, &exp) in expected.iter().enumerate() {
            assert_eq!(
                nakshatra_name(Nakshatra::ALL[i], Language::Tamil),
                exp,
                "Mismatch at index {i}"
            );
        }
    }

    #[test]
    fn nakshatra_names_kannada_all_27() {
        let expected = [
            "ಅಶ್ವಿನಿ",
            "ಭರಣಿ",
            "ಕೃತ್ತಿಕಾ",
            "ರೋಹಿಣಿ",
            "ಮೃಗಶಿರಾ",
            "ಆರ್ದ್ರಾ",
            "ಪುನರ್ವಸು",
            "ಪುಷ್ಯ",
            "ಆಶ್ಲೇಷಾ",
            "ಮಘಾ",
            "ಪೂರ್ವ ಫಲ್ಗುಣಿ",
            "ಉತ್ತರ ಫಲ್ಗುಣಿ",
            "ಹಸ್ತ",
            "ಚಿತ್ರಾ",
            "ಸ್ವಾತಿ",
            "ವಿಶಾಖ",
            "ಅನುರಾಧಾ",
            "ಜ್ಯೇಷ್ಠಾ",
            "ಮೂಲಾ",
            "ಪೂರ್ವಾಷಾಢ",
            "ಉತ್ತರಾಷಾಢ",
            "ಶ್ರವಣ",
            "ಧನಿಷ್ಠಾ",
            "ಶತಭಿಷಾ",
            "ಪೂರ್ವಾಭಾದ್ರ",
            "ಉತ್ತರಾಭಾದ್ರ",
            "ರೇವತಿ",
        ];
        for (i, &exp) in expected.iter().enumerate() {
            assert_eq!(
                nakshatra_name(Nakshatra::ALL[i], Language::Kannada),
                exp,
                "Mismatch at index {i}"
            );
        }
    }

    #[test]
    fn nakshatra_names_all_languages_non_empty() {
        for &nak in &Nakshatra::ALL {
            for &lang in &Language::ALL {
                let name = nakshatra_name(nak, lang);
                assert!(
                    !name.is_empty(),
                    "Empty nakshatra name for {nak:?} in {lang}"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Deity names
    // -----------------------------------------------------------------------

    #[test]
    fn deity_names_english_match_existing() {
        for &nak in &Nakshatra::ALL {
            assert_eq!(nakshatra_deity(nak, Language::English), nak.deity());
        }
    }

    #[test]
    fn deity_names_hindi_samples() {
        assert_eq!(
            nakshatra_deity(Nakshatra::Ashwini, Language::Hindi),
            "अश्विनी कुमार"
        );
        assert_eq!(
            nakshatra_deity(Nakshatra::Krittika, Language::Hindi),
            "अग्नि"
        );
        assert_eq!(
            nakshatra_deity(Nakshatra::Shravana, Language::Hindi),
            "विष्णु"
        );
        assert_eq!(nakshatra_deity(Nakshatra::Revati, Language::Hindi), "पूषन्");
    }

    #[test]
    fn deity_names_all_languages_non_empty() {
        for &nak in &Nakshatra::ALL {
            for &lang in &Language::ALL {
                let name = nakshatra_deity(nak, lang);
                assert!(!name.is_empty(), "Empty deity name for {nak:?} in {lang}");
            }
        }
    }

    // -----------------------------------------------------------------------
    // Tithi names
    // -----------------------------------------------------------------------

    #[test]
    fn tithi_names_hindi_all_30() {
        let expected_hi = [
            "प्रतिपदा",
            "द्वितीया",
            "तृतीया",
            "चतुर्थी",
            "पञ्चमी",
            "षष्ठी",
            "सप्तमी",
            "अष्टमी",
            "नवमी",
            "दशमी",
            "एकादशी",
            "द्वादशी",
            "त्रयोदशी",
            "चतुर्दशी",
            "पूर्णिमा",
            "प्रतिपदा",
            "द्वितीया",
            "तृतीया",
            "चतुर्थी",
            "पञ्चमी",
            "षष्ठी",
            "सप्तमी",
            "अष्टमी",
            "नवमी",
            "दशमी",
            "एकादशी",
            "द्वादशी",
            "त्रयोदशी",
            "चतुर्दशी",
            "अमावस्या",
        ];
        for (i, &exp) in expected_hi.iter().enumerate() {
            let t = Tithi {
                number: (i as u8) + 1,
                paksha: if i < 15 {
                    crate::panchang::Paksha::Shukla
                } else {
                    crate::panchang::Paksha::Krishna
                },
            };
            assert_eq!(tithi_name(&t, Language::Hindi), exp, "Tithi {}", i + 1);
        }
    }

    #[test]
    fn tithi_purnima_amavasya_hindi() {
        let purnima = Tithi {
            number: 15,
            paksha: crate::panchang::Paksha::Shukla,
        };
        assert_eq!(tithi_name(&purnima, Language::Hindi), "पूर्णिमा");
        let amavasya = Tithi {
            number: 30,
            paksha: crate::panchang::Paksha::Krishna,
        };
        assert_eq!(tithi_name(&amavasya, Language::Hindi), "अमावस्या");
    }

    #[test]
    fn tithi_english_matches_existing() {
        for i in 1..=30u8 {
            let t = Tithi {
                number: i,
                paksha: if i <= 15 {
                    crate::panchang::Paksha::Shukla
                } else {
                    crate::panchang::Paksha::Krishna
                },
            };
            assert_eq!(tithi_name(&t, Language::English), t.name());
        }
    }

    // -----------------------------------------------------------------------
    // Yoga names
    // -----------------------------------------------------------------------

    #[test]
    fn yoga_names_hindi_all_27() {
        let expected = [
            "विष्कम्भ",
            "प्रीति",
            "आयुष्मान्",
            "सौभाग्य",
            "शोभन",
            "अतिगण्ड",
            "सुकर्मा",
            "धृति",
            "शूल",
            "गण्ड",
            "वृद्धि",
            "ध्रुव",
            "व्याघात",
            "हर्षण",
            "वज्र",
            "सिद्धि",
            "व्यतीपात",
            "वरीयान्",
            "परिघ",
            "शिव",
            "सिद्ध",
            "साध्य",
            "शुभ",
            "शुक्ल",
            "ब्रह्म",
            "इन्द्र",
            "वैधृति",
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let y = Yoga {
                number: (i as u8) + 1,
            };
            assert_eq!(yoga_name(&y, Language::Hindi), exp, "Yoga {}", i + 1);
        }
    }

    #[test]
    fn yoga_english_matches_existing() {
        for i in 1..=27u8 {
            let y = Yoga { number: i };
            assert_eq!(yoga_name(&y, Language::English), y.name());
        }
    }

    // -----------------------------------------------------------------------
    // Karana names
    // -----------------------------------------------------------------------

    #[test]
    fn karana_names_hindi_all_11() {
        let expected = [
            "बव",
            "बालव",
            "कौलव",
            "तैतिल",
            "गर",
            "वणिज",
            "विष्टि",
            "शकुनि",
            "चतुष्पद",
            "नाग",
            "किंस्तुघ्न",
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let k = Karana {
                number: 1,
                name_index: i as u8,
            };
            assert_eq!(karana_name(&k, Language::Hindi), exp, "Karana index {i}");
        }
    }

    #[test]
    fn karana_english_matches_existing() {
        for i in 0..11u8 {
            let k = Karana {
                number: 1,
                name_index: i,
            };
            assert_eq!(karana_name(&k, Language::English), k.name());
        }
    }

    // -----------------------------------------------------------------------
    // Vara names
    // -----------------------------------------------------------------------

    #[test]
    fn vara_names_hindi() {
        assert_eq!(vara_name(Vara::Sunday, Language::Hindi), "रविवार");
        assert_eq!(vara_name(Vara::Monday, Language::Hindi), "सोमवार");
        assert_eq!(vara_name(Vara::Tuesday, Language::Hindi), "मंगलवार");
        assert_eq!(vara_name(Vara::Wednesday, Language::Hindi), "बुधवार");
        assert_eq!(vara_name(Vara::Thursday, Language::Hindi), "गुरुवार");
        assert_eq!(vara_name(Vara::Friday, Language::Hindi), "शुक्रवार");
        assert_eq!(vara_name(Vara::Saturday, Language::Hindi), "शनिवार");
    }

    #[test]
    fn vara_names_tamil() {
        assert_eq!(vara_name(Vara::Sunday, Language::Tamil), "ஞாயிற்றுக்கிழமை");
        assert_eq!(vara_name(Vara::Monday, Language::Tamil), "திங்கட்கிழமை");
        assert_eq!(vara_name(Vara::Saturday, Language::Tamil), "சனிக்கிழமை");
    }

    #[test]
    fn vara_names_telugu() {
        assert_eq!(vara_name(Vara::Sunday, Language::Telugu), "ఆదివారం");
        assert_eq!(vara_name(Vara::Thursday, Language::Telugu), "గురువారం");
    }

    #[test]
    fn vara_names_kannada() {
        assert_eq!(vara_name(Vara::Sunday, Language::Kannada), "ಭಾನುವಾರ");
        assert_eq!(vara_name(Vara::Monday, Language::Kannada), "ಸೋಮವಾರ");
        assert_eq!(vara_name(Vara::Saturday, Language::Kannada), "ಶನಿವಾರ");
    }

    #[test]
    fn vara_names_malayalam() {
        assert_eq!(vara_name(Vara::Sunday, Language::Malayalam), "ഞായറാഴ്ച");
        assert_eq!(vara_name(Vara::Friday, Language::Malayalam), "വെള്ളിയാഴ്ച");
    }

    #[test]
    fn vara_names_bengali() {
        assert_eq!(vara_name(Vara::Sunday, Language::Bengali), "রবিবার");
        assert_eq!(vara_name(Vara::Thursday, Language::Bengali), "বৃহস্পতিবার");
    }

    #[test]
    fn vara_names_spanish() {
        assert_eq!(vara_name(Vara::Sunday, Language::Spanish), "Domingo");
        assert_eq!(vara_name(Vara::Monday, Language::Spanish), "Lunes");
        assert_eq!(vara_name(Vara::Saturday, Language::Spanish), "Sábado");
    }

    #[test]
    fn vara_names_japanese() {
        assert_eq!(vara_name(Vara::Sunday, Language::Japanese), "日曜日");
        assert_eq!(vara_name(Vara::Monday, Language::Japanese), "月曜日");
        assert_eq!(vara_name(Vara::Saturday, Language::Japanese), "土曜日");
    }

    #[test]
    fn vara_names_thai() {
        assert_eq!(vara_name(Vara::Sunday, Language::Thai), "วันอาทิตย์");
        assert_eq!(vara_name(Vara::Friday, Language::Thai), "วันศุกร์");
    }

    #[test]
    fn vara_names_indonesian() {
        assert_eq!(vara_name(Vara::Sunday, Language::Indonesian), "Minggu");
        assert_eq!(vara_name(Vara::Friday, Language::Indonesian), "Jumat");
    }

    #[test]
    fn vara_english_matches_existing() {
        let varas = [
            Vara::Sunday,
            Vara::Monday,
            Vara::Tuesday,
            Vara::Wednesday,
            Vara::Thursday,
            Vara::Friday,
            Vara::Saturday,
        ];
        for v in varas {
            assert_eq!(vara_name(v, Language::English), v.name());
        }
    }

    #[test]
    fn vara_names_all_languages_non_empty() {
        let varas = [
            Vara::Sunday,
            Vara::Monday,
            Vara::Tuesday,
            Vara::Wednesday,
            Vara::Thursday,
            Vara::Friday,
            Vara::Saturday,
        ];
        for v in varas {
            for &lang in &Language::ALL {
                let name = vara_name(v, lang);
                assert!(!name.is_empty(), "Empty vara name for {v:?} in {lang}");
            }
        }
    }

    // -----------------------------------------------------------------------
    // Paksha names
    // -----------------------------------------------------------------------

    #[test]
    fn paksha_names_all_languages() {
        use crate::panchang::Paksha;
        assert_eq!(paksha_name(Paksha::Shukla, Language::English), "Shukla");
        assert_eq!(paksha_name(Paksha::Krishna, Language::English), "Krishna");
        assert_eq!(paksha_name(Paksha::Shukla, Language::Hindi), "शुक्ल");
        assert_eq!(paksha_name(Paksha::Krishna, Language::Hindi), "कृष्ण");
        assert_eq!(paksha_name(Paksha::Shukla, Language::Tamil), "சுக்ல");
        assert_eq!(paksha_name(Paksha::Krishna, Language::Tamil), "கிருஷ்ண");
        assert_eq!(paksha_name(Paksha::Shukla, Language::Telugu), "శుక్ల");
        assert_eq!(paksha_name(Paksha::Krishna, Language::Telugu), "కృష్ణ");
        assert_eq!(paksha_name(Paksha::Shukla, Language::Kannada), "ಶುಕ್ಲ");
        assert_eq!(paksha_name(Paksha::Krishna, Language::Kannada), "ಕೃಷ್ಣ");
        assert_eq!(paksha_name(Paksha::Shukla, Language::Malayalam), "ശുക്ല");
        assert_eq!(paksha_name(Paksha::Krishna, Language::Malayalam), "കൃഷ്ണ");
        assert_eq!(paksha_name(Paksha::Shukla, Language::Bengali), "শুক্ল");
        assert_eq!(paksha_name(Paksha::Krishna, Language::Bengali), "কৃষ্ণ");
        assert_eq!(paksha_name(Paksha::Shukla, Language::Gujarati), "શુક્લ");
        assert_eq!(paksha_name(Paksha::Krishna, Language::Gujarati), "કૃષ્ણ");
        assert_eq!(paksha_name(Paksha::Shukla, Language::Spanish), "Shukla");
        assert_eq!(paksha_name(Paksha::Krishna, Language::Japanese), "Krishna");
    }

    #[test]
    fn paksha_names_all_languages_non_empty() {
        use crate::panchang::Paksha;
        for &lang in &Language::ALL {
            let s = paksha_name(Paksha::Shukla, lang);
            let k = paksha_name(Paksha::Krishna, lang);
            assert!(!s.is_empty(), "Empty Shukla paksha name in {lang}");
            assert!(!k.is_empty(), "Empty Krishna paksha name in {lang}");
        }
    }

    // -----------------------------------------------------------------------
    // Cross-cutting: no duplicates within a language for distinct enum values
    // -----------------------------------------------------------------------

    #[test]
    fn rashi_names_unique_per_language() {
        for &lang in &Language::ALL {
            let mut names: Vec<&str> = (0..12)
                .map(|i| rashi_name(Rashi::from_index(i), lang))
                .collect();
            let len = names.len();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), len, "Duplicate rashi names in {lang}");
        }
    }

    #[test]
    fn nakshatra_names_unique_per_language() {
        for &lang in &Language::ALL {
            let mut names: Vec<&str> = Nakshatra::ALL
                .iter()
                .map(|&n| nakshatra_name(n, lang))
                .collect();
            let len = names.len();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), len, "Duplicate nakshatra names in {lang}");
        }
    }

    // -----------------------------------------------------------------------
    // Language enum coverage
    // -----------------------------------------------------------------------

    #[test]
    fn language_all_count() {
        assert_eq!(Language::ALL.len(), 19);
    }

    #[test]
    fn language_display_all_non_empty() {
        for &lang in &Language::ALL {
            let s = format!("{lang}");
            assert!(!s.is_empty(), "Empty display for {lang:?}");
        }
    }
}
