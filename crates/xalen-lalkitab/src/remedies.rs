//! Lal Kitab remedy catalog -- 108 planet-house combinations.
//!
//! Each of the 9 planets in each of the 12 houses has specific material
//! remedies prescribed by Pt. Roop Chand Joshi. These involve wearing items,
//! donating goods, keeping objects at home, and feeding animals.

use crate::houses::Planet;
use serde::{Deserialize, Serialize};

/// A Lal Kitab remedy entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// A Lal Kitab remedy for a specific planet-house combination.
pub struct LalKitabRemedy {
    pub planet: Planet,
    /// House number (1-12).
    pub house: usize,
    /// Textual remedy descriptions.
    pub remedies: Vec<String>,
    /// Specific items involved (donations, wearing, keeping at home).
    pub items: Vec<String>,
}

/// Look up remedies for `planet` in `house` (1-12).
///
/// Returns a `LalKitabRemedy` with 2-3 remedies and associated items.
/// Panics if house is out of 1-12 range.
pub fn remedy_lookup(planet: Planet, house: usize) -> LalKitabRemedy {
    let house = house.clamp(1, 12);

    let (remedies, items) = remedy_data(planet, house);
    LalKitabRemedy {
        planet,
        house,
        remedies: remedies.into_iter().map(String::from).collect(),
        items: items.into_iter().map(String::from).collect(),
    }
}

/// Internal lookup returning (remedies, items) for each of the 108 combos.
fn remedy_data(planet: Planet, house: usize) -> (Vec<&'static str>, Vec<&'static str>) {
    match (planet, house) {
        // =====================================================================
        // SUN in houses 1-12
        // =====================================================================
        (Planet::Sun, 1) => (
            vec![
                "Offer water to the Sun at sunrise",
                "Keep a solid silver ball",
                "Avoid accepting free gifts or charity",
            ],
            vec!["wheat", "jaggery", "copper"],
        ),
        (Planet::Sun, 2) => (
            vec![
                "Donate coconut and almonds at a temple",
                "Keep solid gold or copper item",
                "Serve father and elders with respect",
            ],
            vec!["coconut", "almonds", "copper vessel"],
        ),
        (Planet::Sun, 3) => (
            vec![
                "Offer water to the Sun daily",
                "Donate red cloth at a religious place",
                "Keep a ruby or copper ring",
            ],
            vec!["red cloth", "copper ring", "wheat"],
        ),
        (Planet::Sun, 4) => (
            vec![
                "Offer wheat and jaggery to a temple",
                "Maintain good relations with the government",
                "Keep a Surya Yantra at home",
            ],
            vec!["wheat", "jaggery", "Surya Yantra"],
        ),
        (Planet::Sun, 5) => (
            vec![
                "Offer water with jaggery to the Sun",
                "Feed monkeys with jaggery and wheat",
                "Wear a copper bangle",
            ],
            vec!["jaggery", "wheat", "copper bangle"],
        ),
        (Planet::Sun, 6) => (
            vec![
                "Feed jaggery and wheat to monkeys",
                "Keep a dark room in the house",
                "Donate red lentils at a temple",
            ],
            vec!["jaggery", "red lentils", "dark room"],
        ),
        (Planet::Sun, 7) => (
            vec![
                "Marry before 24th year of age",
                "Donate wheat and jaggery",
                "Keep a silver ball with you",
            ],
            vec!["wheat", "jaggery", "silver ball"],
        ),
        (Planet::Sun, 8) => (
            vec![
                "Throw wheat and jaggery into flowing water",
                "Do not accept free items",
                "Keep a solid copper ball",
            ],
            vec!["wheat", "jaggery", "copper ball"],
        ),
        (Planet::Sun, 9) => (
            vec![
                "Offer water to the Sun from a copper vessel",
                "Serve elders and father",
                "Keep saffron at home",
            ],
            vec!["copper vessel", "saffron", "wheat"],
        ),
        (Planet::Sun, 10) => (
            vec![
                "Offer water to the Sun at sunrise",
                "Maintain a wheat flour lamp at home",
                "Donate jaggery at a temple",
            ],
            vec!["wheat", "jaggery", "copper"],
        ),
        (Planet::Sun, 11) => (
            vec![
                "Keep a copper vessel filled with water at bedside",
                "Donate wheat and jaggery",
                "Avoid non-vegetarian food on Sundays",
            ],
            vec!["copper vessel", "wheat", "jaggery"],
        ),
        (Planet::Sun, 12) => (
            vec![
                "Throw jaggery and wheat in flowing water",
                "Donate almonds at a temple",
                "Avoid starting new ventures on Sundays",
            ],
            vec!["jaggery", "wheat", "almonds"],
        ),

        // =====================================================================
        // MOON in houses 1-12
        // =====================================================================
        (Planet::Moon, 1) => (
            vec![
                "Keep a silver square piece",
                "Offer milk to Shiva Linga",
                "Avoid selling milk or water",
            ],
            vec!["silver piece", "milk", "rice"],
        ),
        (Planet::Moon, 2) => (
            vec![
                "Keep a square piece of silver in your pocket",
                "Serve your mother with devotion",
                "Donate white items",
            ],
            vec!["silver piece", "rice", "white cloth"],
        ),
        (Planet::Moon, 3) => (
            vec![
                "Wear silver ornaments",
                "Donate white sweets at a temple",
                "Keep water in a silver vessel at bedside",
            ],
            vec!["silver ornaments", "white sweets", "silver vessel"],
        ),
        (Planet::Moon, 4) => (
            vec![
                "Offer milk at a Shiva temple",
                "Keep rainwater at home",
                "Wear a pearl or moonstone",
            ],
            vec!["milk", "pearl", "moonstone"],
        ),
        (Planet::Moon, 5) => (
            vec![
                "Donate milk and rice at a temple",
                "Avoid keeping empty vessels at home",
                "Serve widows and elderly women",
            ],
            vec!["milk", "rice", "white cloth"],
        ),
        (Planet::Moon, 6) => (
            vec![
                "Keep a silver ball or rice in a silver box",
                "Donate milk daily",
                "Avoid keeping a well or pond at home",
            ],
            vec!["silver box", "milk", "rice"],
        ),
        (Planet::Moon, 7) => (
            vec![
                "Offer rain or well water at a temple",
                "Keep a square silver piece",
                "Respect mother and elderly women",
            ],
            vec!["silver piece", "water vessel", "rice"],
        ),
        (Planet::Moon, 8) => (
            vec![
                "Keep solid silver at home",
                "Throw rice into flowing water",
                "Donate white cloth to a temple",
            ],
            vec!["solid silver", "rice", "white cloth"],
        ),
        (Planet::Moon, 9) => (
            vec![
                "Offer rice and milk at a religious place",
                "Keep silver items in the home",
                "Avoid selling milk",
            ],
            vec!["rice", "milk", "silver items"],
        ),
        (Planet::Moon, 10) => (
            vec![
                "Keep a silver thali at home",
                "Offer milk to Banyan tree roots",
                "Use silver glass for drinking water",
            ],
            vec!["silver thali", "milk", "silver glass"],
        ),
        (Planet::Moon, 11) => (
            vec![
                "Keep a silver nail in your house",
                "Donate white cloth",
                "Offer milk to a Shiva temple",
            ],
            vec!["silver nail", "white cloth", "milk"],
        ),
        (Planet::Moon, 12) => (
            vec![
                "Throw rice into flowing water",
                "Keep a vessel of fresh water at head of bed",
                "Donate silver at a temple",
            ],
            vec!["rice", "fresh water", "silver"],
        ),

        // =====================================================================
        // MARS in houses 1-12
        // =====================================================================
        (Planet::Mars, 1) => (
            vec![
                "Keep a red handkerchief",
                "Serve your brother",
                "Donate sweet chapatis to workers",
            ],
            vec!["red handkerchief", "sweet chapati", "copper"],
        ),
        (Planet::Mars, 2) => (
            vec![
                "Keep a deer skin at home",
                "Feed sweet chapati to dogs",
                "Wear a silver chain",
            ],
            vec!["deer skin", "sweet chapati", "silver chain"],
        ),
        (Planet::Mars, 3) => (
            vec![
                "Keep ivory or saffron at home",
                "Donate sweet chapati to dogs",
                "Avoid being cruel to siblings",
            ],
            vec!["saffron", "sweet chapati", "ivory"],
        ),
        (Planet::Mars, 4) => (
            vec![
                "Keep a square piece of silver at home",
                "Pour sweet milk into a well or Bawdi",
                "Offer sindoor at Hanuman temple",
            ],
            vec!["silver piece", "sweet milk", "sindoor"],
        ),
        (Planet::Mars, 5) => (
            vec![
                "Offer pomegranate in flowing water",
                "Keep silver square at home",
                "Feed dogs with sweet chapatis",
            ],
            vec!["pomegranate", "silver", "sweet chapati"],
        ),
        (Planet::Mars, 6) => (
            vec![
                "Keep honey at home",
                "Donate sweet chapati and red lentils",
                "Wear a copper ring",
            ],
            vec!["honey", "sweet chapati", "copper ring"],
        ),
        (Planet::Mars, 7) => (
            vec![
                "Donate sweet chapatis to dogs",
                "Keep a silver square",
                "Avoid arrogance and aggression",
            ],
            vec!["sweet chapati", "silver square", "red lentils"],
        ),
        (Planet::Mars, 8) => (
            vec![
                "Float sweet chapati in flowing water",
                "Donate blood if health permits",
                "Keep a deer skin at home",
            ],
            vec!["sweet chapati", "deer skin", "red cloth"],
        ),
        (Planet::Mars, 9) => (
            vec![
                "Keep sweet food items at home",
                "Donate sweet chapatis to dogs",
                "Offer sindoor at a Hanuman temple",
            ],
            vec!["sweet food", "sindoor", "sweet chapati"],
        ),
        (Planet::Mars, 10) => (
            vec![
                "Keep a red handkerchief in pocket",
                "Feed sweet chapatis to dogs",
                "Donate jaggery to workers",
            ],
            vec!["red handkerchief", "sweet chapati", "jaggery"],
        ),
        (Planet::Mars, 11) => (
            vec![
                "Keep a solid silver piece",
                "Offer sweet chapati to dogs daily",
                "Donate red cloth",
            ],
            vec!["silver piece", "sweet chapati", "red cloth"],
        ),
        (Planet::Mars, 12) => (
            vec![
                "Offer sweet chapatis to dogs",
                "Keep honey in a copper vessel",
                "Donate red lentils at a temple",
            ],
            vec!["sweet chapati", "honey", "red lentils"],
        ),

        // =====================================================================
        // MERCURY in houses 1-12
        // =====================================================================
        (Planet::Mercury, 1) => (
            vec![
                "Wear a copper coin with a hole in it as pendant",
                "Donate green moong dal",
                "Keep a parrot at home or feed parrots",
            ],
            vec!["copper coin", "green moong dal", "parrot"],
        ),
        (Planet::Mercury, 2) => (
            vec![
                "Wear emerald or green thread",
                "Donate green vegetables",
                "Keep an earthen pot of water at home",
            ],
            vec!["emerald", "green vegetables", "earthen pot"],
        ),
        (Planet::Mercury, 3) => (
            vec![
                "Donate goat's milk at a temple",
                "Keep a parrot or feed green pulses to parrots",
                "Avoid quarreling with in-laws",
            ],
            vec!["goat's milk", "green pulses", "parrot"],
        ),
        (Planet::Mercury, 4) => (
            vec![
                "Wear a ring made from the metal of a door hinge",
                "Donate green moong to a temple",
                "Feed birds green grains",
            ],
            vec!["iron ring", "green moong", "green grains"],
        ),
        (Planet::Mercury, 5) => (
            vec![
                "Donate green cloth at a temple",
                "Wear a copper coin as pendant",
                "Keep fennel seeds at home",
            ],
            vec!["green cloth", "copper coin", "fennel seeds"],
        ),
        (Planet::Mercury, 6) => (
            vec![
                "Donate moong dal and green vegetables",
                "Keep a parrot or feed parrots",
                "Wear emerald on little finger",
            ],
            vec!["moong dal", "green vegetables", "emerald"],
        ),
        (Planet::Mercury, 7) => (
            vec![
                "Keep a parrot or feed parrots daily",
                "Donate moong dal",
                "Wear a copper coin pendant",
            ],
            vec!["parrot", "moong dal", "copper coin"],
        ),
        (Planet::Mercury, 8) => (
            vec![
                "Float green moong in flowing water",
                "Feed parrots green grains",
                "Wear iron ring on little finger",
            ],
            vec!["green moong", "iron ring", "green grains"],
        ),
        (Planet::Mercury, 9) => (
            vec![
                "Donate green items at a temple",
                "Keep a clean nose; wash face frequently",
                "Avoid speculation and gambling",
            ],
            vec!["green items", "mint", "green cloth"],
        ),
        (Planet::Mercury, 10) => (
            vec![
                "Donate green cloth and moong at a temple",
                "Feed parrots green grains daily",
                "Wear emerald or copper coin",
            ],
            vec!["green cloth", "moong", "emerald"],
        ),
        (Planet::Mercury, 11) => (
            vec![
                "Keep fennel seeds at home",
                "Donate green cloth",
                "Feed parrots",
            ],
            vec!["fennel seeds", "green cloth", "green grains"],
        ),
        (Planet::Mercury, 12) => (
            vec![
                "Float green moong in flowing water",
                "Donate green bangles",
                "Feed cows with green grass",
            ],
            vec!["green moong", "green bangles", "green grass"],
        ),

        // =====================================================================
        // JUPITER in houses 1-12
        // =====================================================================
        (Planet::Jupiter, 1) => (
            vec![
                "Apply saffron tilak on forehead",
                "Donate yellow items at a temple",
                "Serve sadhus and Brahmins",
            ],
            vec!["saffron", "yellow cloth", "turmeric"],
        ),
        (Planet::Jupiter, 2) => (
            vec![
                "Offer saffron at a temple",
                "Donate turmeric and yellow items",
                "Keep a gold piece in the locker",
            ],
            vec!["saffron", "turmeric", "gold piece"],
        ),
        (Planet::Jupiter, 3) => (
            vec![
                "Donate yellow items at a temple",
                "Wear a yellow thread or sapphire",
                "Feed crows with sweet rice",
            ],
            vec!["yellow items", "yellow thread", "sweet rice"],
        ),
        (Planet::Jupiter, 4) => (
            vec![
                "Apply saffron on tongue daily",
                "Donate yellow cloth and turmeric",
                "Serve aged saints and Brahmins",
            ],
            vec!["saffron", "yellow cloth", "turmeric"],
        ),
        (Planet::Jupiter, 5) => (
            vec![
                "Offer saffron and turmeric at a temple",
                "Keep gold with you",
                "Feed cows with jaggery and gram",
            ],
            vec!["saffron", "turmeric", "gold"],
        ),
        (Planet::Jupiter, 6) => (
            vec![
                "Donate turmeric and yellow lentils",
                "Apply saffron tilak",
                "Feed crows",
            ],
            vec!["turmeric", "yellow lentils", "saffron"],
        ),
        (Planet::Jupiter, 7) => (
            vec![
                "Wear yellow thread around neck",
                "Donate yellow lentils to a temple",
                "Apply saffron on forehead before leaving home",
            ],
            vec!["yellow thread", "yellow lentils", "saffron"],
        ),
        (Planet::Jupiter, 8) => (
            vec![
                "Apply saffron tilak daily",
                "Donate almonds at a temple",
                "Keep gold piece in safe",
            ],
            vec!["saffron", "almonds", "gold piece"],
        ),
        (Planet::Jupiter, 9) => (
            vec![
                "Serve elders and saints",
                "Donate saffron and yellow cloth",
                "Keep a gold piece with you",
            ],
            vec!["saffron", "yellow cloth", "gold piece"],
        ),
        (Planet::Jupiter, 10) => (
            vec![
                "Donate yellow items and turmeric",
                "Offer saffron at a temple daily",
                "Feed Brahmins yellow rice",
            ],
            vec!["yellow items", "turmeric", "saffron"],
        ),
        (Planet::Jupiter, 11) => (
            vec![
                "Wear a yellow sapphire",
                "Donate turmeric at a temple",
                "Keep saffron tilak on forehead",
            ],
            vec!["yellow sapphire", "turmeric", "saffron"],
        ),
        (Planet::Jupiter, 12) => (
            vec![
                "Offer saffron tilak at temple",
                "Donate yellow lentils",
                "Serve old saints and gurus",
            ],
            vec!["saffron", "yellow lentils", "turmeric"],
        ),

        // =====================================================================
        // VENUS in houses 1-12
        // =====================================================================
        (Planet::Venus, 1) => (
            vec![
                "Donate cow ghee at a temple",
                "Offer camphor at a Lakshmi temple",
                "Avoid alcoholic drinks",
            ],
            vec!["cow ghee", "camphor", "white rice"],
        ),
        (Planet::Venus, 2) => (
            vec![
                "Offer white items at a Lakshmi temple",
                "Donate rice and white cloth",
                "Keep a silver idol at home",
            ],
            vec!["white cloth", "rice", "silver idol"],
        ),
        (Planet::Venus, 3) => (
            vec![
                "Donate white items at a temple",
                "Keep perfumed items at home",
                "Wear diamond or opal ring",
            ],
            vec!["white items", "perfume", "diamond"],
        ),
        (Planet::Venus, 4) => (
            vec![
                "Donate white cow ghee at a temple",
                "Keep a white piece of cloth under pillow",
                "Offer flowers to Lakshmi",
            ],
            vec!["cow ghee", "white cloth", "flowers"],
        ),
        (Planet::Venus, 5) => (
            vec![
                "Offer yogurt and rice at a temple",
                "Donate white items",
                "Keep curd culture at home",
            ],
            vec!["yogurt", "rice", "white items"],
        ),
        (Planet::Venus, 6) => (
            vec![
                "Float rice and white items in flowing water",
                "Donate white cow to a Brahmin",
                "Avoid extra-marital affairs",
            ],
            vec!["rice", "white items", "white cow"],
        ),
        (Planet::Venus, 7) => (
            vec![
                "Donate cow ghee to a temple",
                "Wear a diamond or opal ring",
                "Keep white flowers at home",
            ],
            vec!["cow ghee", "diamond", "white flowers"],
        ),
        (Planet::Venus, 8) => (
            vec![
                "Float rice in flowing water",
                "Donate white cloth at a temple",
                "Bury white items in a deserted place",
            ],
            vec!["rice", "white cloth", "white items"],
        ),
        (Planet::Venus, 9) => (
            vec![
                "Donate rice and white cloth at a temple",
                "Offer camphor at a Lakshmi temple",
                "Avoid wearing dirty clothes",
            ],
            vec!["rice", "white cloth", "camphor"],
        ),
        (Planet::Venus, 10) => (
            vec![
                "Keep a silver piece in your wallet",
                "Donate rice and white items",
                "Worship Lakshmi on Fridays",
            ],
            vec!["silver piece", "rice", "white items"],
        ),
        (Planet::Venus, 11) => (
            vec![
                "Donate cow ghee at a temple",
                "Wear silver ornaments",
                "Keep white flowers at home",
            ],
            vec!["cow ghee", "silver ornaments", "white flowers"],
        ),
        (Planet::Venus, 12) => (
            vec![
                "Donate white items at a temple",
                "Offer camphor daily",
                "Keep a clean white handkerchief",
            ],
            vec!["white items", "camphor", "white handkerchief"],
        ),

        // =====================================================================
        // SATURN in houses 1-12
        // =====================================================================
        (Planet::Saturn, 1) => (
            vec![
                "Keep mustard oil in an earthen pot at home",
                "Donate black urad dal at a temple",
                "Avoid drinking milk at night",
            ],
            vec!["mustard oil", "earthen pot", "black urad dal"],
        ),
        (Planet::Saturn, 2) => (
            vec![
                "Keep a solid iron ball at home",
                "Donate mustard oil on Saturdays",
                "Feed crows",
            ],
            vec!["iron ball", "mustard oil", "black sesame"],
        ),
        (Planet::Saturn, 3) => (
            vec![
                "Donate mustard oil on Saturdays",
                "Keep a solid iron piece",
                "Feed ants with flour and sugar",
            ],
            vec!["mustard oil", "iron piece", "flour"],
        ),
        (Planet::Saturn, 4) => (
            vec![
                "Offer mustard oil at a Hanuman temple",
                "Donate black urad dal on Saturdays",
                "Keep an iron piece under pillow",
            ],
            vec!["mustard oil", "black urad dal", "iron piece"],
        ),
        (Planet::Saturn, 5) => (
            vec![
                "Donate almonds at a temple",
                "Offer mustard oil on Saturdays",
                "Feed monkeys",
            ],
            vec!["almonds", "mustard oil", "black sesame"],
        ),
        (Planet::Saturn, 6) => (
            vec![
                "Keep a black dog at home",
                "Donate mustard oil and urad dal on Saturdays",
                "Float coconut in flowing water",
            ],
            vec!["black dog", "mustard oil", "coconut"],
        ),
        (Planet::Saturn, 7) => (
            vec![
                "Keep an iron nail in your pocket",
                "Donate mustard oil and urad dal",
                "Feed crows on Saturdays",
            ],
            vec!["iron nail", "mustard oil", "urad dal"],
        ),
        (Planet::Saturn, 8) => (
            vec![
                "Float iron items in flowing water on Saturdays",
                "Donate mustard oil",
                "Keep a solid iron ball at home",
            ],
            vec!["iron items", "mustard oil", "iron ball"],
        ),
        (Planet::Saturn, 9) => (
            vec![
                "Donate mustard oil at a Hanuman temple",
                "Keep an iron piece in pocket",
                "Feed ants with sugar and flour",
            ],
            vec!["mustard oil", "iron piece", "flour"],
        ),
        (Planet::Saturn, 10) => (
            vec![
                "Donate mustard oil and urad dal on Saturdays",
                "Keep a solid iron ball at work",
                "Offer water to a Peepal tree",
            ],
            vec!["mustard oil", "urad dal", "iron ball"],
        ),
        (Planet::Saturn, 11) => (
            vec![
                "Keep an iron vessel at home",
                "Donate mustard oil on Saturdays",
                "Feed dogs and crows",
            ],
            vec!["iron vessel", "mustard oil", "black sesame"],
        ),
        (Planet::Saturn, 12) => (
            vec![
                "Float mustard oil and iron in flowing water",
                "Donate black urad dal",
                "Avoid consuming alcohol",
            ],
            vec!["mustard oil", "iron", "black urad dal"],
        ),

        // =====================================================================
        // RAHU in houses 1-12
        // =====================================================================
        (Planet::Rahu, 1) => (
            vec![
                "Keep a solid silver ball",
                "Float barley in flowing water",
                "Donate coconut at a temple",
            ],
            vec!["silver ball", "barley", "coconut"],
        ),
        (Planet::Rahu, 2) => (
            vec![
                "Keep a solid silver ball at home",
                "Donate barley and coconut at a temple",
                "Avoid wearing blue sapphire",
            ],
            vec!["silver ball", "barley", "coconut"],
        ),
        (Planet::Rahu, 3) => (
            vec![
                "Keep an ivory item at home",
                "Donate barley at a temple",
                "Wear a silver ring",
            ],
            vec!["ivory item", "barley", "silver ring"],
        ),
        (Planet::Rahu, 4) => (
            vec![
                "Keep barley under pillow while sleeping",
                "Donate coal or dark items at a temple",
                "Wear silver chain",
            ],
            vec!["barley", "coal", "silver chain"],
        ),
        (Planet::Rahu, 5) => (
            vec![
                "Float coconut in flowing water",
                "Donate barley and radish at a temple",
                "Keep an elephant tusk at home",
            ],
            vec!["coconut", "barley", "elephant tusk"],
        ),
        (Planet::Rahu, 6) => (
            vec![
                "Keep a lead piece at home",
                "Float coal in flowing water",
                "Donate coconut and barley",
            ],
            vec!["lead piece", "coal", "coconut"],
        ),
        (Planet::Rahu, 7) => (
            vec![
                "Float barley in flowing water",
                "Donate coconut at a temple",
                "Keep a solid silver ball at home",
            ],
            vec!["barley", "coconut", "silver ball"],
        ),
        (Planet::Rahu, 8) => (
            vec![
                "Float barley and coconut in flowing water",
                "Keep a solid silver ball",
                "Donate fennel seeds at a temple",
            ],
            vec!["barley", "coconut", "fennel seeds"],
        ),
        (Planet::Rahu, 9) => (
            vec![
                "Float barley in flowing water",
                "Keep a silver ball",
                "Donate coconut at a temple",
            ],
            vec!["barley", "silver ball", "coconut"],
        ),
        (Planet::Rahu, 10) => (
            vec![
                "Keep barley under pillow",
                "Donate coconut at a Bhairav temple",
                "Wear a silver chain",
            ],
            vec!["barley", "coconut", "silver chain"],
        ),
        (Planet::Rahu, 11) => (
            vec![
                "Float coconut in flowing water",
                "Donate barley at a temple",
                "Wear a silver ring",
            ],
            vec!["coconut", "barley", "silver ring"],
        ),
        (Planet::Rahu, 12) => (
            vec![
                "Float barley and coconut in flowing water",
                "Keep a silver ball at home",
                "Donate fennel seeds",
            ],
            vec!["barley", "coconut", "fennel seeds"],
        ),

        // =====================================================================
        // KETU in houses 1-12
        // =====================================================================
        (Planet::Ketu, 1) => (
            vec![
                "Keep a silver dog figurine at home",
                "Donate black and white sesame at a temple",
                "Feed stray dogs",
            ],
            vec!["silver dog figurine", "black sesame", "white sesame"],
        ),
        (Planet::Ketu, 2) => (
            vec![
                "Donate a brown and white patterned blanket at a temple",
                "Feed dogs with sweet items",
                "Wear a gold ring on ring finger",
            ],
            vec!["patterned blanket", "sweet items", "gold ring"],
        ),
        (Planet::Ketu, 3) => (
            vec![
                "Keep saffron at home",
                "Donate sesame seeds",
                "Wear gold in the ear",
            ],
            vec!["saffron", "sesame seeds", "gold earring"],
        ),
        (Planet::Ketu, 4) => (
            vec![
                "Float black and white sesame in flowing water",
                "Donate a brown blanket",
                "Feed stray dogs with milk",
            ],
            vec!["sesame seeds", "brown blanket", "milk"],
        ),
        (Planet::Ketu, 5) => (
            vec![
                "Keep a silver piece at home",
                "Donate sweet chapatis to dogs",
                "Float sesame seeds in flowing water",
            ],
            vec!["silver piece", "sweet chapati", "sesame seeds"],
        ),
        (Planet::Ketu, 6) => (
            vec![
                "Keep a brown and white dog at home",
                "Donate sesame and blankets",
                "Wear a gold piece on body",
            ],
            vec!["brown dog", "sesame seeds", "gold piece"],
        ),
        (Planet::Ketu, 7) => (
            vec![
                "Donate sesame seeds and a blanket",
                "Feed dogs sweet food",
                "Wear gold earring",
            ],
            vec!["sesame seeds", "blanket", "gold earring"],
        ),
        (Planet::Ketu, 8) => (
            vec![
                "Float sesame seeds in flowing water",
                "Keep a silver dog figurine",
                "Donate a multicolored blanket at a temple",
            ],
            vec![
                "sesame seeds",
                "silver dog figurine",
                "multicolored blanket",
            ],
        ),
        (Planet::Ketu, 9) => (
            vec![
                "Feed stray dogs daily",
                "Donate sesame seeds at a temple",
                "Keep a piece of gold at home",
            ],
            vec!["sesame seeds", "gold piece", "sweet food"],
        ),
        (Planet::Ketu, 10) => (
            vec![
                "Keep a silver ball at home",
                "Donate sesame seeds and a blanket",
                "Feed dogs with milk and chapati",
            ],
            vec!["silver ball", "sesame seeds", "blanket"],
        ),
        (Planet::Ketu, 11) => (
            vec![
                "Keep saffron and honey at home",
                "Donate sesame seeds at a temple",
                "Feed stray dogs",
            ],
            vec!["saffron", "honey", "sesame seeds"],
        ),
        (Planet::Ketu, 12) => (
            vec![
                "Feed stray dogs sweet items",
                "Donate a brown and white blanket",
                "Keep gold ornament at home",
            ],
            vec!["sweet items", "brown blanket", "gold ornament"],
        ),

        _ => (vec!["No Lal Kitab remedies for this planet"], vec![]),
    }
}

/// Return all 108 remedy entries.
pub fn all_remedies() -> Vec<LalKitabRemedy> {
    let mut out = Vec::with_capacity(108);
    for p in Planet::VEDIC_NINE {
        for h in 1..=12 {
            out.push(remedy_lookup(p, h));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_108_entries_exist() {
        let all = all_remedies();
        assert_eq!(all.len(), 108);
    }

    #[test]
    fn every_entry_has_at_least_two_remedies() {
        for r in all_remedies() {
            assert!(
                r.remedies.len() >= 2,
                "{} in house {} has only {} remedies",
                r.planet,
                r.house,
                r.remedies.len()
            );
        }
    }

    #[test]
    fn every_entry_has_items() {
        for r in all_remedies() {
            assert!(
                !r.items.is_empty(),
                "{} in house {} has no items",
                r.planet,
                r.house
            );
        }
    }

    #[test]
    fn sun_in_1_specific() {
        let r = remedy_lookup(Planet::Sun, 1);
        assert!(r.remedies[0].contains("sunrise"));
        assert!(r.items.contains(&"wheat".to_string()));
    }

    #[test]
    fn saturn_in_10_specific() {
        let r = remedy_lookup(Planet::Saturn, 10);
        assert!(r.items.contains(&"mustard oil".to_string()));
    }

    #[test]
    fn ketu_in_6_has_dog() {
        let r = remedy_lookup(Planet::Ketu, 6);
        let joined = r.remedies.join(" ");
        assert!(joined.to_lowercase().contains("dog"));
    }
}
