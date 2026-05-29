use serde::{Deserialize, Serialize};

/// A Sabian Symbol entry for one degree of the zodiac.
///
/// The Sabian Symbols are a set of 360 symbolic images, one for each degree,
/// channeled by Elsie Wheeler and recorded by Marc Edmund Jones (1925),
/// later revised by Dane Rudhyar in *An Astrological Mandala* (1973).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// A Sabian Symbol for one of the 360 zodiac degrees.
pub struct SabianSymbol {
    /// Absolute degree 1-360 (Aries 1 = 1, Pisces 30 = 360)
    pub degree: u16,
    /// Degree within the sign, 1-30
    pub sign_degree: u8,
    /// Sign name
    pub sign: &'static str,
    /// The symbolic image text
    pub symbol: &'static str,
    /// Rudhyar's keynote (1-2 words)
    pub keynote: &'static str,
}

/// Degree classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Classification of a zodiac degree (critical, anaretic, etc.).
pub enum DegreeType {
    /// Critical degrees: 0/13/26 cardinal, 8-9/21-22 fixed, 4/17 mutable
    Critical,
    /// 29th degree of any sign
    Anaretic,
    /// Neither critical nor anaretic
    Normal,
}

const SIGN_NAMES: [&str; 12] = [
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

/// (symbol, keynote) for each of the 360 degrees, indexed 0-359.
/// Order: Aries 1° at index 0 through Pisces 30° at index 359.
///
/// Verified spot-checks against Dane Rudhyar, *An Astrological Mandala* (1973):
///   Aries 1  "A woman just risen from the sea..." -- mindfire.ca, jamesburgess.com
///   Taurus 10 "A Red Cross nurse" -- sabian-calculator.com, sabiansymbols.com
///   Leo 5    "Rock formations..." -- sabian-calculator.com (fixed from prior error)
///   Leo 15   "A pageant, with its spectacular floats..." -- jamesburgess.com
///   Leo 25   "A large camel..." -- cornerstone-astrology.com
///   Scorpio 1 "A sight-seeing bus filled with tourists" -- sabian-calculator.com
///   Capricorn 20 "A hidden choir singing..." -- sabian-calculator.com
///   Pisces 1  "In a crowded marketplace farmers and middlemen..." -- sabiansymbols.com
/// All 30 Leo degrees cross-checked against jamesburgess.com and sabian-calculator.com.
/// Leo 3,5-9,24 were corrected (had duplicated Leo 25-29 content).
const SYMBOL_DATA: [(&str, &str); 360] = [
    // ── Aries (1-30) ──────────────────────────────────────────────
    (
        "A woman just risen from the sea; a seal is embracing her",
        "Emergence",
    ),
    ("A comedian reveals human nature", "Humor"),
    (
        "The cameo profile of a man suggesting the shape of his country",
        "Allegiance",
    ),
    ("Two lovers strolling on a secluded walk", "Receptivity"),
    ("A triangle with wings", "Aspiration"),
    (
        "A square with one of its sides brightly illumined",
        "Realization",
    ),
    (
        "A man successfully expressing himself in two realms at once",
        "Versatility",
    ),
    (
        "A large woman's hat with streamers blown by an east wind",
        "Protection",
    ),
    ("A crystal gazer", "Attunement"),
    (
        "A teacher gives new symbolic forms to traditional images",
        "Revision",
    ),
    ("The president of the country", "Idealization"),
    ("A triangularly shaped flight of wild geese", "Orderliness"),
    (
        "An unexploded bomb reveals an unsuccessful social protest",
        "Impotence",
    ),
    ("A serpent coiling near a man and a woman", "Temptation"),
    ("An Indian weaving a ceremonial blanket", "Artistry"),
    (
        "Nature spirits are seen at work in the light of sunset",
        "Cooperation",
    ),
    ("Two dignified spinsters sitting in silence", "Quiescence"),
    ("An empty hammock stretched between two trees", "Withdrawal"),
    ("The magic carpet of oriental imagery", "Visualization"),
    ("A young girl feeding birds in winter", "Compassion"),
    ("A pugilist enters the ring", "Mobilization"),
    (
        "The gate to the garden of all fulfilled desires",
        "Abundance",
    ),
    ("A pregnant woman in light summer dress", "Fecundity"),
    (
        "An open window and a net curtain blowing into a cornucopia",
        "Imagination",
    ),
    (
        "A double promise reveals its inner and outer meanings",
        "Duality",
    ),
    (
        "A man possessed of more gifts than he can hold",
        "Prodigality",
    ),
    (
        "Through imagination a lost opportunity is regained",
        "Recuperation",
    ),
    ("A large audience confronts the performer", "Performance"),
    ("The music of the spheres", "Plenitude"),
    ("A duck pond and its brood", "Serenity"),
    // ── Taurus (31-60) ────────────────────────────────────────────
    ("A clear mountain stream", "Purity"),
    ("An electrical storm", "Release"),
    ("Steps up to a lawn blooming with clover", "Expansion"),
    ("The pot of gold at the end of the rainbow", "Riches"),
    ("A widow at an open grave", "Catharsis"),
    ("A bridge being built across a gorge", "Enterprise"),
    (
        "A woman of Samaria comes to draw water from the well",
        "Questioning",
    ),
    ("A woman holding a linen bag out of a window", "Reception"),
    ("A Christmas tree decorated", "Gifting"),
    ("A Red Cross nurse", "Dedication"),
    ("A woman watering flowers in her garden", "Nurture"),
    ("A young couple window-shopping", "Longing"),
    ("A porter carrying heavy baggage", "Endurance"),
    (
        "On the beach, children play while shellfish grope at the edge of the water",
        "Play",
    ),
    (
        "A man with a rakish silk hat, muffled against the cold, braves a storm",
        "Fortitude",
    ),
    (
        "An old teacher fails to interest his pupils in traditional knowledge",
        "Inadequacy",
    ),
    ("A battle between the swords and the torches", "Struggle"),
    (
        "A woman airing an old bag through a sunny window",
        "Renewal",
    ),
    ("A new continent rising out of the ocean", "Genesis"),
    (
        "Wisps of winglike clouds streaming across the sky",
        "Awareness",
    ),
    ("A head buyer for a large department store", "Judgment"),
    ("A white dove flying over troubled waters", "Guidance"),
    ("A jewelry shop filled with valuable gems", "Treasure"),
    (
        "An Indian warrior riding fiercely, human scalps hanging from his belt",
        "Conquest",
    ),
    ("A large well-kept public park", "Community"),
    ("A Spaniard serenading his senorita", "Courtship"),
    (
        "An old Indian woman selling the artifacts of her tribe",
        "Preservation",
    ),
    (
        "A woman in middle life stands in rapt sudden realization of forgotten charms",
        "Reminiscence",
    ),
    ("Two cobblers working at a table", "Craftsmanship"),
    (
        "A peacock parading on the terrace of an old castle",
        "Magnificence",
    ),
    // ── Gemini (61-90) ────────────────────────────────────────────
    (
        "A glass-bottomed boat reveals undersea wonders",
        "Discovery",
    ),
    ("Santa Claus filling stockings furtively", "Generosity"),
    ("The garden of the Tuileries in Paris", "Formality"),
    (
        "Holly and mistletoe reawaken old memories of Christmas",
        "Nostalgia",
    ),
    ("A revolutionary magazine asking for action", "Agitation"),
    ("Workmen drilling for oil", "Exploration"),
    ("An old-fashioned well", "Simplicity"),
    ("A labor demonstration", "Protest"),
    ("A quiver filled with arrows", "Readiness"),
    ("An airplane performing a nose dive", "Audacity"),
    (
        "Newly opened lands offer the pioneer new opportunities for experience",
        "Opportunity",
    ),
    (
        "A black slave-girl demands her rights of her mistress",
        "Liberation",
    ),
    ("A great musician at his piano", "Virtuosity"),
    (
        "Two people, far from the crowd, are bridging their differences",
        "Reconciliation",
    ),
    ("Two Dutch children talking to each other", "Companionship"),
    (
        "A woman activist in an emotional speech dramatizing her cause",
        "Advocacy",
    ),
    (
        "The head of a robust youth changes into that of a mature thinker",
        "Maturation",
    ),
    (
        "Two Chinese men converse in their native tongue in an American city",
        "Heritage",
    ),
    (
        "A large archaic volume reveals traditional wisdom",
        "Tradition",
    ),
    (
        "A modern cafeteria displays an abundance of food",
        "Variety",
    ),
    ("A tumultuous labor demonstration", "Revolution"),
    ("Dancing couples in a harvest festival", "Celebration"),
    ("Three fledglings in a nest high in a tree", "Security"),
    ("Children skating over a frozen village pond", "Fun"),
    ("A gardener trimming large palm trees", "Cultivation"),
    ("Frost-covered trees against winter skies", "Endurance"),
    (
        "A gypsy emerging from the forest wherein her tribe is encamped",
        "Independence",
    ),
    ("A bankruptcy granted to the debtor", "Dispensation"),
    ("The first mockingbird of spring", "Quickening"),
    (
        "A parade of bathing beauties before large beach crowds",
        "Exhibition",
    ),
    // ── Cancer (91-120) ───────────────────────────────────────────
    (
        "On a ship the sailors lower an old flag and raise a new one",
        "Reorientation",
    ),
    (
        "A man on a magic carpet hovers over a large area of land",
        "Expansion",
    ),
    (
        "An arctic explorer leads a reindeer through icy canyons",
        "Perseverance",
    ),
    ("A cat arguing with a mouse", "Justification"),
    ("An automobile wrecked by a train", "Loss"),
    ("Game birds feathering their nests", "Preparation"),
    (
        "Two nature spirits dancing under the moonlight",
        "Enchantment",
    ),
    (
        "A group of rabbits dressed in clothes walk as if on parade",
        "Imitation",
    ),
    (
        "A small naked girl bends over a pond trying to catch a fish",
        "Innocence",
    ),
    (
        "A large diamond in the first stages of the cutting process",
        "Crafting",
    ),
    ("A clown caricaturing well-known personalities", "Satire"),
    (
        "A Chinese woman nursing a baby whose aura reveals him to be the reincarnation of a great teacher",
        "Revelation",
    ),
    (
        "A hand with a prominent thumb is held out for study",
        "Determination",
    ),
    (
        "A very old man facing a vast dark space to the northeast",
        "Contemplation",
    ),
    (
        "In a sumptuous dining hall guests relax after partaking of a huge banquet",
        "Satiation",
    ),
    (
        "A man studying a mandala in front of him, with the help of a very ancient book",
        "Meditation",
    ),
    ("The seed grows into knowledge and life", "Germination"),
    (
        "A hen scratching the ground to find nourishment for her progeny",
        "Providence",
    ),
    ("A priest performing a marriage ceremony", "Consecration"),
    ("Venetian gondoliers giving a serenade", "Felicity"),
    (
        "A famous singer is proving her virtuosity during an operatic performance",
        "Mastery",
    ),
    ("A young woman awaiting a sailboat", "Expectancy"),
    ("The meeting of a literary society", "Criticism"),
    (
        "A woman and two men castaways on a small island",
        "Survival",
    ),
    (
        "A will-o'-the-wisp-like phosphorescence crowning the swampland",
        "Allure",
    ),
    (
        "Guests are reading in the library of a luxurious home",
        "Refinement",
    ),
    (
        "A violent storm in a canyon filled with expensive homes",
        "Turbulence",
    ),
    (
        "An Indian girl introduces her white lover to her assembled tribe",
        "Acceptance",
    ),
    (
        "A Greek muse weighing newborn twins in golden scales",
        "Balance",
    ),
    ("A daughter of the American Revolution", "Inheritance"),
    // ── Leo (121-150) ─────────────────────────────────────────────
    // Verified against sabian-calculator.com + jamesburgess.com/leo-sabian-symbols
    (
        "Blood rushes to a man's head as his vital energies are mobilized under the spur of ambition",
        "Mobilization",
    ),
    ("An epidemic of mumps", "Contagion"),
    ("A woman having her hair bobbed", "Transformation"),
    (
        "A formally dressed elderly man stands near trophies he brought back from a hunting expedition",
        "Conquest",
    ),
    ("Rock formations tower over a deep canyon", "Endurance"),
    (
        "An old-fashioned woman is confronted by an up-to-date girl",
        "Contrast",
    ),
    (
        "The constellations of stars shine brilliantly in the night sky",
        "Revelation",
    ),
    (
        "A Bolshevik propagandist spreading his revolutionary ideals",
        "Agitation",
    ),
    (
        "Glass blowers shape beautiful vases with their controlled breathing",
        "Craftsmanship",
    ),
    ("Early morning dew", "Freshness"),
    ("Children on a swing in a huge oak tree", "Exuberance"),
    (
        "An evening party of adults on a lawn illumined by fancy lanterns",
        "Enchantment",
    ),
    (
        "An old sea captain rocking himself on the porch of his cottage",
        "Reminiscence",
    ),
    (
        "A human soul seeking opportunities for outward manifestation",
        "Aspiration",
    ),
    (
        "A pageant, with its spectacular floats, moves along a street crowded with cheering people",
        "Display",
    ),
    (
        "The storm ended, all nature rejoices in brilliant sunshine",
        "Liberation",
    ),
    (
        "A volunteer church choir singing religious hymns",
        "Communion",
    ),
    (
        "A chemist conducts an experiment for his students",
        "Demonstration",
    ),
    ("A houseboat party", "Conviviality"),
    ("Zuni Indians perform a ritual to the sun", "Worship"),
    (
        "Intoxicated chickens dizzily flap their wings trying to fly",
        "Exhilaration",
    ),
    ("A carrier pigeon fulfilling its mission", "Faithfulness"),
    (
        "In a circus the bareback rider displays her dangerous skill",
        "Daring",
    ),
    ("An untidy, unkempt man", "Devotion"),
    (
        "A large camel is seen crossing a vast and forbidding desert",
        "Perseverance",
    ),
    ("After the heavy storm, a rainbow", "Blessing"),
    ("The luminescence of dawn in the eastern sky", "Dawning"),
    ("Many little birds on the limb of a large tree", "Community"),
    (
        "A mermaid emerges from the ocean waves ready for rebirth in human form",
        "Yearning",
    ),
    ("An unsealed letter", "Openness"),
    // ── Virgo (151-180) ───────────────────────────────────────────
    (
        "In a portrait, the significant features of a man's head are artistically emphasized",
        "Characterization",
    ),
    (
        "A large white cross dominates the landscape",
        "Sanctification",
    ),
    ("Two guardian angels", "Custodianship"),
    ("Black and white children play together happily", "Equality"),
    (
        "A man becoming aware of nature spirits and normally unseen spiritual agencies",
        "Sensitivity",
    ),
    ("A merry-go-round", "Diversion"),
    ("A harem", "Seclusion"),
    (
        "A five-year-old child carrying a bag filled with groceries",
        "Responsibility",
    ),
    ("An expressionist painter at work", "Originality"),
    (
        "Two heads looking out and beyond the shadows",
        "Transcendence",
    ),
    (
        "In her baby a mother sees her deep longing for a son answered",
        "Fulfillment",
    ),
    (
        "After the wedding, the groom snatches the veil away from his bride",
        "Unveiling",
    ),
    (
        "A powerful statesman overcomes a state of political hysteria",
        "Authority",
    ),
    ("An aristocratic family tree", "Lineage"),
    ("A fine lace ornamental handkerchief", "Delicacy"),
    (
        "In the zoo, children are brought face to face with an orangutan",
        "Confrontation",
    ),
    ("A volcanic eruption", "Eruption"),
    ("An ouija board", "Channeling"),
    ("A swimming race", "Competition"),
    ("A carpool of businessmen", "Efficiency"),
    ("A girls' basketball team", "Teamwork"),
    (
        "A royal coat of arms enriched with precious stones",
        "Heraldry",
    ),
    ("An animal trainer", "Discipline"),
    ("Mary and her little lamb", "Tenderness"),
    (
        "A flag at half-mast in front of a large public building",
        "Tribute",
    ),
    (
        "A boy with a censer serves the priest near the altar",
        "Service",
    ),
    (
        "A group of aristocratic ladies meet ceremonially at a court's function",
        "Protocol",
    ),
    ("A baldheaded man who has seized power", "Dominance"),
    (
        "A seeker after occult knowledge is reading an ancient scroll",
        "Research",
    ),
    (
        "Totally intent upon completing an immediate task, a man is deaf to any allurement",
        "Concentration",
    ),
    // ── Libra (181-210) ───────────────────────────────────────────
    (
        "In a collection of perfect specimens of many biological forms, a butterfly displays the beauty of its wings, its body impaled by a fine dart",
        "Immortalization",
    ),
    (
        "The transmutation of the fruits of past experiences into the seed-realizations of the forever creative Spirit",
        "Alchemy",
    ),
    (
        "The dawn of a new day reveals everything changed",
        "Transformation",
    ),
    (
        "Around a campfire a group of young people sit in spiritual communion",
        "Fellowship",
    ),
    (
        "A man revealing to his students the foundation of an inner knowledge upon which a new world could be built",
        "Instruction",
    ),
    (
        "The ideals of a man abundantly crystallized",
        "Manifestation",
    ),
    (
        "A woman feeding chickens and protecting them from the hawks",
        "Stewardship",
    ),
    ("A blazing fireplace in a deserted home", "Memory"),
    (
        "Three old masters hanging on the wall of a special room in an art gallery",
        "Mastery",
    ),
    (
        "Having passed through narrow rapids, a canoe reaches calm waters",
        "Relief",
    ),
    (
        "A professor peering over his glasses at his students",
        "Examination",
    ),
    ("Miners are surfacing from a deep coal mine", "Emergence"),
    ("Children blowing soap bubbles", "Playfulness"),
    (
        "In the heat of the noon hour a man takes a siesta",
        "Repose",
    ),
    ("Circular paths", "Wholeness"),
    (
        "After a storm, a boat landing stands in need of reconstruction",
        "Repair",
    ),
    (
        "A retired sea captain watches ships entering and leaving the harbor",
        "Observation",
    ),
    ("Two men placed under arrest", "Accountability"),
    ("A gang of robbers in hiding", "Deception"),
    ("A rabbi performing his duties", "Dedication"),
    ("A Sunday crowd enjoying the beach", "Relaxation"),
    ("A child giving birds a drink at a fountain", "Kindness"),
    (
        "Chanticleer's voice heralds the rising sun with exuberant tones",
        "Annunciation",
    ),
    ("A butterfly with a third wing on its left side", "Mutation"),
    (
        "The sight of an autumn leaf brings to a pilgrim the sudden revelation of the mystery of life and death",
        "Transcendence",
    ),
    (
        "An eagle and a large white dove change into each other",
        "Interplay",
    ),
    ("An airplane sails high in the clear sky", "Elevation"),
    (
        "A man becoming aware of spiritual forces surrounding and assisting him",
        "Awakening",
    ),
    (
        "Humanity seeking to bridge the span of knowledge",
        "Inquiry",
    ),
    (
        "Three mounds of knowledge on a philosopher's head",
        "Wisdom",
    ),
    // ── Scorpio (211-240) ─────────────────────────────────────────
    ("A sight-seeing bus filled with tourists", "Acquaintance"),
    ("A broken bottle and spilled perfume", "Catharsis"),
    (
        "A man and a woman, in love, walk together through a large department store",
        "Bounty",
    ),
    (
        "A youth carries a lighted candle in a devotional ritual",
        "Consecration",
    ),
    (
        "An Indian woman pleading to the chief for the lives of her children",
        "Intercession",
    ),
    (
        "The gold rush tears men away from their native soil",
        "Ambition",
    ),
    ("Deep-sea divers", "Depth"),
    ("The moon shining across a lake", "Stillness"),
    (
        "A parrot repeats the conversation he has overheard",
        "Imitation",
    ),
    ("A fellowship supper reunites old comrades", "Reunion"),
    ("A drowning man is being rescued", "Salvation"),
    ("An embassy ball", "Diplomacy"),
    ("An inventor performs a laboratory experiment", "Innovation"),
    (
        "Telephone linemen at work installing new connections",
        "Communication",
    ),
    ("Children playing around five mounds of sand", "Innocence"),
    ("A girl's face breaking into a smile", "Warmth"),
    (
        "A woman, filled with her own spirit, is the father of her own child",
        "Self-reliance",
    ),
    (
        "A path through woods brilliant with multicolored splendor",
        "Beauty",
    ),
    (
        "A parrot listening and then talking, repeats a conversation he has overheard",
        "Mimicry",
    ),
    (
        "A woman draws away two dark curtains closing the entrance to a sacred pathway",
        "Initiation",
    ),
    (
        "Obeying his conscience, a soldier resists orders",
        "Integrity",
    ),
    ("Hunters shooting wild ducks", "Prowess"),
    ("A rabbit metamorphosed into a fairy", "Transfiguration"),
    (
        "After having heard an inspired individual deliver his 'sermon on the mount,' crowds are returning home",
        "Internalization",
    ),
    ("An x-ray photograph", "Penetration"),
    (
        "Indians making camp after moving into new territory",
        "Adaptation",
    ),
    (
        "A military band marches noisily on through the city streets",
        "Pageantry",
    ),
    (
        "The king of the fairies approaching his domain",
        "Sovereignty",
    ),
    (
        "An Indian squaw pleading to the chief for the lives of her children",
        "Compassion",
    ),
    (
        "Children in Halloween costumes indulge in various pranks",
        "Mischief",
    ),
    // ── Sagittarius (241-270) ─────────────────────────────────────
    (
        "Retired army veterans gather to reawaken old memories",
        "Reminiscence",
    ),
    ("The ocean covered with whitecaps", "Power"),
    ("Two men playing chess", "Strategy"),
    (
        "A little child learning to walk with the encouragement of his parents",
        "Guidance",
    ),
    (
        "A fat owl sitting alone on the limb of a tree",
        "Observation",
    ),
    ("A game of cricket", "Sportsmanship"),
    ("Cupid knocking at the door of a human heart", "Yearning"),
    (
        "Within the depths of the earth, new elements are being formed",
        "Alchemy",
    ),
    (
        "A mother leads her small child step by step up a steep stairway",
        "Nurture",
    ),
    (
        "A theatrical representation of a golden-haired goddess of opportunity",
        "Vision",
    ),
    (
        "In the left section of an archaic temple, a lamp burns in a container shaped like a human body",
        "Embodiment",
    ),
    ("A flag that turns into an eagle that crows", "Apotheosis"),
    ("A widow's past is brought to light", "Exposure"),
    ("The great pyramid and the sphinx", "Endurance"),
    (
        "The groundhog looking for its shadow on Groundhog Day",
        "Anticipation",
    ),
    (
        "Sea gulls fly around a ship in expectation of food",
        "Dependence",
    ),
    (
        "An Easter sunrise service draws a large crowd",
        "Resurrection",
    ),
    ("Tiny children in sunbonnets", "Protection"),
    (
        "Pelicans menaced by the behavior and refuse of men seek safer areas for bringing up their young",
        "Migration",
    ),
    (
        "In an old-fashioned northern village men cut the ice of a frozen pond for use during the summer",
        "Foresight",
    ),
    ("A child and a dog wearing borrowed eyeglasses", "Mimicry"),
    ("A Chinese laundry", "Industry"),
    (
        "A group of immigrants as they fulfill the requirements of entrance into the new country",
        "Transition",
    ),
    ("A bluebird perched on the gate of a cottage", "Happiness"),
    ("A chubby boy on a hobbyhorse", "Imagination"),
    ("A flag-bearer in a battle", "Valor"),
    ("A sculptor at his work", "Creation"),
    (
        "An old bridge over a beautiful stream is still in constant use",
        "Permanence",
    ),
    (
        "A fat boy mowing the lawn of his house on an elegant suburban street",
        "Routine",
    ),
    ("The pope blessing the faithful", "Benediction"),
    // ── Capricorn (271-300) ───────────────────────────────────────
    (
        "An Indian chief claims power from the assembled tribe",
        "Authority",
    ),
    (
        "Three rose windows in a Gothic church, one damaged by war",
        "Sacrifice",
    ),
    (
        "A human soul, in its eagerness for new experience, seeks embodiment",
        "Incarnation",
    ),
    (
        "A group of people outfitting a large canoe at the start of a journey by water",
        "Enterprise",
    ),
    (
        "Indians on the warpath; while some men row a well-filled canoe, others in it perform a war dance",
        "Mobilization",
    ),
    (
        "Ten logs lie under an archway leading to darker woods",
        "Threshold",
    ),
    (
        "A veiled prophet speaks, seized by the power of a god",
        "Prophecy",
    ),
    (
        "In a sun-lit home domesticated birds sing joyously",
        "Happiness",
    ),
    ("An angel carrying a harp", "Attunement"),
    ("An albatross feeding from the hand of a sailor", "Trust"),
    ("A large group of pheasant on a private estate", "Privilege"),
    (
        "An illustrated lecture on natural science reveals little-known aspects of life",
        "Education",
    ),
    (
        "A fire worshipper meditates on the ultimate realities of existence",
        "Devotion",
    ),
    (
        "An ancient bas-relief carved in granite remains a witness to a long-forgotten culture",
        "Legacy",
    ),
    (
        "In a hospital, the children's ward is filled with toys",
        "Comfort",
    ),
    (
        "School grounds filled with boys and girls in gymnasium suits",
        "Training",
    ),
    (
        "A repressed woman finds a psychological release in nudism",
        "Liberation",
    ),
    (
        "The Union Jack flag flies from a British warship",
        "Dominion",
    ),
    (
        "A five-year-old child carrying a huge shopping bag filled with groceries",
        "Precociousness",
    ),
    (
        "A hidden choir is singing during a religious service",
        "Anonymity",
    ),
    ("A relay race", "Cooperation"),
    (
        "By accepting defeat gracefully, a general reveals nobility of character",
        "Magnanimity",
    ),
    (
        "A soldier receiving two awards for bravery in combat",
        "Valor",
    ),
    ("A woman entering a convent", "Withdrawal"),
    (
        "An oriental rug dealer in a store filled with precious ornamental rugs",
        "Discrimination",
    ),
    (
        "A nature spirit dancing in the iridescent mist of a waterfall",
        "Delight",
    ),
    (
        "Pilgrims climbing the steep steps leading to a mountain shrine",
        "Aspiration",
    ),
    ("A large aviary", "Collectivity"),
    ("A woman reading tea leaves", "Divination"),
    (
        "A secret meeting of men responsible for executive decisions in world affairs",
        "Governance",
    ),
    // ── Aquarius (301-330) ────────────────────────────────────────
    ("An old adobe mission in California", "Foundation"),
    ("An unexpected thunderstorm", "Disruption"),
    ("A deserter from the navy", "Defiance"),
    ("A Hindu yogi demonstrates his healing powers", "Mastery"),
    (
        "A council of ancestors is seen implementing the efforts of a young leader",
        "Heritage",
    ),
    (
        "A masked figure performs ritualistic acts in a mystery play",
        "Ceremony",
    ),
    ("A child is seen being born out of an egg", "Emergence"),
    ("Beautifully gowned wax figures on display", "Artifice"),
    ("A flag is seen turning into an eagle", "Metamorphosis"),
    ("A popularity contest reveals a charlatan", "Unmasking"),
    (
        "During a silent hour, a man receives a new inspiration which may change his life",
        "Receptivity",
    ),
    (
        "On a vast staircase stand people of different types, graduated upward",
        "Hierarchy",
    ),
    ("A barometer", "Sensitivity"),
    ("A train entering a tunnel", "Passage"),
    (
        "Two lovebirds sitting on a fence and singing happily",
        "Bliss",
    ),
    ("A big businessman at his desk", "Enterprise"),
    (
        "A watchdog stands guard, protecting his master and his possessions",
        "Vigilance",
    ),
    ("A man being unmasked at a masquerade", "Exposure"),
    (
        "A forest fire is being subdued by the use of water, chemicals and sheer muscular energy",
        "Containment",
    ),
    ("A large white dove bearing a message", "Annunciation"),
    (
        "A disappointed and disillusioned woman courageously faces a seemingly empty life",
        "Resilience",
    ),
    (
        "A rug is placed on the floor of a nursery to allow children to play in comfort and warmth",
        "Provision",
    ),
    (
        "A big bear sitting down and waving all its paws",
        "Playfulness",
    ),
    (
        "A man, having overcome his passions, teaches deep wisdom in terms of his experience",
        "Sagacity",
    ),
    (
        "A butterfly with the right wing more perfectly formed",
        "Differentiation",
    ),
    (
        "A garage man testing a car's battery with a hydrometer",
        "Diagnosis",
    ),
    (
        "An ancient pottery bowl filled with fresh violets",
        "Timelessness",
    ),
    (
        "A tree felled and sawed to ensure a supply of wood for the winter",
        "Preparedness",
    ),
    ("A butterfly emerging from a chrysalis", "Liberation"),
    (
        "Deeply rooted in the past of a very ancient culture, a spiritual brotherhood in which many individual minds are merged into the glowing light of a unanimous consciousness is revealed to one who has traversed the way",
        "Illumination",
    ),
    // ── Pisces (331-360) ──────────────────────────────────────────
    (
        "In a crowded marketplace farmers and middlemen display a great variety of products",
        "Commerce",
    ),
    ("A squirrel hiding from hunters", "Self-preservation"),
    (
        "A petrified forest, an eternal record of a life lived long ago",
        "Immortality",
    ),
    (
        "Heavy car traffic on a narrow isthmus linking two seashore resorts",
        "Congestion",
    ),
    ("A church bazaar", "Charity"),
    ("Officers on dress parade", "Formality"),
    (
        "Illuminated by a shaft of light, a large cross lies on rocks surrounded by sea mist",
        "Epiphany",
    ),
    ("A girl blowing a bugle", "Call"),
    ("A jockey spurs his horse to great speed", "Urgency"),
    (
        "An aviator pursues his journey, flying through ground-obscuring clouds",
        "Perseverance",
    ),
    (
        "Men travelling a narrow path, seeking illumination",
        "Pilgrimage",
    ),
    (
        "In the sanctuary of an occult brotherhood, newly initiated members are being examined and their character tested",
        "Probation",
    ),
    (
        "An ancient sword, used in many battles, is displayed in a museum",
        "Heritage",
    ),
    ("A lady wrapped in a large stole of fox fur", "Luxury"),
    (
        "An officer instructing his men before a simulated assault under a barrage of live shells",
        "Rehearsal",
    ),
    (
        "In the quiet of his study a creative individual experiences a flow of inspiration",
        "Inspiration",
    ),
    ("An Easter promenade", "Renewal"),
    (
        "In a huge tent a famous revivalist conducts his meeting with a spectacular performance",
        "Fervor",
    ),
    ("A master instructing his disciple", "Pedagogy"),
    ("A table set for an evening meal", "Anticipation"),
    (
        "A little white lamb, a child, and a Chinese servant",
        "Innocence",
    ),
    (
        "A prophet carrying tablets of the new law is walking down the slopes of Mount Sinai",
        "Codification",
    ),
    ("A materializing medium giving a seance", "Mediumship"),
    (
        "On a small island surrounded by the vast expanse of the sea, people are seen living in close interaction",
        "Interdependence",
    ),
    (
        "A religious organization succeeds in overcoming the corrupting influence of perverted practices and materialized ideals",
        "Purification",
    ),
    (
        "Watching the very thin moon crescent appearing at sunset, different people realize that the time has come to go ahead with their different projects",
        "Inception",
    ),
    (
        "The harvest moon illumines a clear autumnal sky",
        "Culmination",
    ),
    (
        "A fertile garden under the full moon reveals a variety of full-grown vegetables",
        "Fruition",
    ),
    (
        "Light breaking into many colors as it passes through a prism",
        "Differentiation",
    ),
    (
        "A majestic rock formation resembling a face is idealized by a boy who takes it as his ideal of greatness, and as he grows up, begins to look like it",
        "Idealization",
    ),
];

/// All 360 Sabian symbols, pre-built as a static array.
static SYMBOLS: std::sync::LazyLock<[SabianSymbol; 360]> = std::sync::LazyLock::new(|| {
    let mut arr: [SabianSymbol; 360] = [SabianSymbol {
        degree: 0,
        sign_degree: 0,
        sign: "",
        symbol: "",
        keynote: "",
    }; 360];
    for i in 0..360 {
        let sign_idx = i / 30;
        let deg_in_sign = (i % 30) + 1;
        let (sym, key) = SYMBOL_DATA[i];
        arr[i] = SabianSymbol {
            degree: (i + 1) as u16,
            sign_degree: deg_in_sign as u8,
            sign: SIGN_NAMES[sign_idx],
            symbol: sym,
            keynote: key,
        };
    }
    arr
});

/// Convert ecliptic longitude (0.0-360.0) to the Sabian degree index (0-359).
///
/// Traditional Sabian convention rounds UP: 0°00'01" through 1°00'00" of Aries
/// maps to "1st degree of Aries" (index 0). Exactly 0.0 maps to 360th degree
/// (Pisces 30°, index 359).
fn longitude_to_index(longitude: f64) -> usize {
    let lng = longitude.rem_euclid(360.0);
    if lng == 0.0 {
        359
    } else {
        (lng.ceil() as usize).saturating_sub(1)
    }
}

/// Look up the Sabian symbol for an absolute ecliptic longitude (0.0-360.0).
///
/// Uses the traditional "round up" convention: 0°00'01"-1°00'00" Aries = degree 1.
pub fn sabian_for_degree(absolute_degree: f64) -> &'static SabianSymbol {
    &SYMBOLS[longitude_to_index(absolute_degree)]
}

/// Look up the Sabian symbol by sign index (0 = Aries .. 11 = Pisces) and
/// degree within the sign (1-30).
///
/// Returns `None` if `sign_index > 11` or `degree` is 0 or > 30.
pub fn sabian_for_sign_degree(sign_index: usize, degree: u8) -> Option<&'static SabianSymbol> {
    if sign_index >= 12 || !(1..=30).contains(&degree) {
        return None;
    }
    let idx = sign_index * 30 + (degree as usize - 1);
    Some(&SYMBOLS[idx])
}

/// Return the Sabian symbol at the opposite point (180° away).
pub fn opposite_sabian(absolute_degree: f64) -> &'static SabianSymbol {
    sabian_for_degree(absolute_degree + 180.0)
}

/// Classify a degree as Critical, Anaretic, or Normal.
///
/// Critical degrees (traditional):
/// - Cardinal signs (Aries/Cancer/Libra/Capricorn): 0°, 13°, 26°
/// - Fixed signs (Taurus/Leo/Scorpio/Aquarius): 8°-9°, 21°-22°
/// - Mutable signs (Gemini/Virgo/Sagittarius/Pisces): 4°, 17°
///
/// Anaretic: 29th degree of any sign (28°00'-28°59').
///
/// Degree values here refer to the integer floor of the degree within the sign
/// (i.e. 13°30' = degree 13).
pub fn degree_type(absolute_degree: f64) -> DegreeType {
    let lng = absolute_degree.rem_euclid(360.0);
    let sign_idx = (lng / 30.0).floor() as usize % 12;
    let deg_in_sign = (lng % 30.0).floor() as u8; // 0-29

    // Anaretic: 29th degree (29°00'-29°59')
    if deg_in_sign == 29 {
        return DegreeType::Anaretic;
    }

    // Modality: 0=cardinal, 1=fixed, 2=mutable
    let modality = match sign_idx {
        0 | 3 | 6 | 9 => 0,  // cardinal
        1 | 4 | 7 | 10 => 1, // fixed
        _ => 2,              // mutable
    };

    match modality {
        0 if deg_in_sign == 0 || deg_in_sign == 13 || deg_in_sign == 26 => DegreeType::Critical,
        1 if (8..=9).contains(&deg_in_sign) || (21..=22).contains(&deg_in_sign) => {
            DegreeType::Critical
        }
        2 if deg_in_sign == 4 || deg_in_sign == 17 => DegreeType::Critical,
        _ => DegreeType::Normal,
    }
}

/// Decanate (face) of a degree.
///
/// Each sign is divided into 3 decanates of 10° each. The ruling sign of each
/// decanate follows the triplicity (element) order:
/// - 1st decanate (0°-9°59'): same sign
/// - 2nd decanate (10°-19°59'): next sign of same element
/// - 3rd decanate (20°-29°59'): third sign of same element
///
/// Returns `(decanate_number, ruler_sign_name)` where decanate_number is 1-3.
pub fn decanate(absolute_degree: f64) -> (usize, &'static str) {
    let lng = absolute_degree.rem_euclid(360.0);
    let sign_idx = (lng / 30.0).floor() as usize % 12;
    let deg_in_sign = (lng % 30.0).floor() as u8;

    let decanate_num = match deg_in_sign {
        0..=9 => 1,
        10..=19 => 2,
        _ => 3,
    };

    // Element order: signs sharing the same element are 4 apart
    // Fire: Ari(0), Leo(4), Sag(8)
    // Earth: Tau(1), Vir(5), Cap(9)
    // Air: Gem(2), Lib(6), Aqu(10)
    // Water: Can(3), Sco(7), Pis(11)
    let ruler_idx = match decanate_num {
        1 => sign_idx,
        2 => (sign_idx + 4) % 12,
        _ => (sign_idx + 8) % 12,
    };

    (decanate_num, SIGN_NAMES[ruler_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aries_1_degree() {
        let s = sabian_for_degree(0.5);
        assert_eq!(s.degree, 1);
        assert_eq!(s.sign, "Aries");
        assert_eq!(s.sign_degree, 1);
        assert!(s.symbol.contains("woman just risen from the sea"));
        assert_eq!(s.keynote, "Emergence");
    }

    #[test]
    fn aries_30_degree() {
        let s = sabian_for_sign_degree(0, 30).unwrap();
        assert_eq!(s.degree, 30);
        assert_eq!(s.sign, "Aries");
        assert_eq!(s.sign_degree, 30);
        assert!(s.symbol.contains("duck pond"));
    }

    #[test]
    fn taurus_1_degree() {
        let s = sabian_for_degree(30.1);
        assert_eq!(s.degree, 31);
        assert_eq!(s.sign, "Taurus");
        assert_eq!(s.sign_degree, 1);
        assert!(s.symbol.contains("mountain stream"));
    }

    #[test]
    fn pisces_30_degree_wrap() {
        // Exactly 0.0 should map to Pisces 30 (degree 360)
        let s = sabian_for_degree(0.0);
        assert_eq!(s.degree, 360);
        assert_eq!(s.sign, "Pisces");
        assert_eq!(s.sign_degree, 30);
    }

    #[test]
    fn sign_degree_lookup() {
        let s = sabian_for_sign_degree(5, 15).unwrap(); // Virgo 15
        assert_eq!(s.sign, "Virgo");
        assert_eq!(s.sign_degree, 15);
        assert_eq!(s.degree, 165);
    }

    #[test]
    fn opposite_degree() {
        // Aries 1 (0.5°) opposite is Libra 1 (180.5°)
        let opp = opposite_sabian(0.5);
        assert_eq!(opp.sign, "Libra");
        assert_eq!(opp.sign_degree, 1);
        assert_eq!(opp.degree, 181);
    }

    #[test]
    fn wrap_around_360() {
        // 359.5° should be Pisces 30
        let s = sabian_for_degree(359.5);
        assert_eq!(s.degree, 360);
        assert_eq!(s.sign, "Pisces");

        // 360.5° wraps to Aries 1
        let s2 = sabian_for_degree(360.5);
        assert_eq!(s2.degree, 1);
        assert_eq!(s2.sign, "Aries");
    }

    #[test]
    fn critical_degrees_cardinal() {
        // Aries 0° (0.0-0.999...) = critical (cardinal sign, degree 0)
        assert_eq!(degree_type(0.5), DegreeType::Critical);
        assert_eq!(degree_type(0.0), DegreeType::Critical);
        // Aries 1° = normal
        assert_eq!(degree_type(1.0), DegreeType::Normal);
        // Cancer 13°
        assert_eq!(degree_type(90.0 + 13.5), DegreeType::Critical);
        // Libra 26°
        assert_eq!(degree_type(180.0 + 26.3), DegreeType::Critical);
    }

    #[test]
    fn critical_degrees_fixed() {
        // Taurus 9° (fixed sign, 8-9 critical)
        assert_eq!(degree_type(30.0 + 9.0), DegreeType::Critical);
        // Leo 21°
        assert_eq!(degree_type(120.0 + 21.5), DegreeType::Critical);
        // Taurus 10° = normal
        assert_eq!(degree_type(30.0 + 10.5), DegreeType::Normal);
    }

    #[test]
    fn critical_degrees_mutable() {
        // Gemini 4°
        assert_eq!(degree_type(60.0 + 4.2), DegreeType::Critical);
        // Virgo 17°
        assert_eq!(degree_type(150.0 + 17.9), DegreeType::Critical);
        // Gemini 5° = normal
        assert_eq!(degree_type(60.0 + 5.0), DegreeType::Normal);
    }

    #[test]
    fn anaretic_degree() {
        // Aries 29°
        assert_eq!(degree_type(29.5), DegreeType::Anaretic);
        // Pisces 29°
        assert_eq!(degree_type(330.0 + 29.1), DegreeType::Anaretic);
    }

    #[test]
    fn decanate_first() {
        // Aries 5° → 1st decanate, ruler = Aries
        let (dec, ruler) = decanate(5.0);
        assert_eq!(dec, 1);
        assert_eq!(ruler, "Aries");
    }

    #[test]
    fn decanate_second() {
        // Aries 15° → 2nd decanate, ruler = Leo (next fire sign)
        let (dec, ruler) = decanate(15.0);
        assert_eq!(dec, 2);
        assert_eq!(ruler, "Leo");
    }

    #[test]
    fn decanate_third() {
        // Aries 25° → 3rd decanate, ruler = Sagittarius (3rd fire sign)
        let (dec, ruler) = decanate(25.0);
        assert_eq!(dec, 3);
        assert_eq!(ruler, "Sagittarius");
    }

    #[test]
    fn decanate_water_sign() {
        // Cancer 0° → 1st decanate, ruler = Cancer
        let (dec, ruler) = decanate(90.0);
        assert_eq!(dec, 1);
        assert_eq!(ruler, "Cancer");

        // Cancer 12° → 2nd decanate, ruler = Scorpio
        let (dec2, ruler2) = decanate(102.0);
        assert_eq!(dec2, 2);
        assert_eq!(ruler2, "Scorpio");

        // Cancer 22° → 3rd decanate, ruler = Pisces
        let (dec3, ruler3) = decanate(112.0);
        assert_eq!(dec3, 3);
        assert_eq!(ruler3, "Pisces");
    }

    #[test]
    fn all_360_symbols_present() {
        for i in 0..360 {
            let s = &SYMBOLS[i];
            assert_eq!(s.degree, (i + 1) as u16);
            assert!(!s.symbol.is_empty(), "Empty symbol at degree {}", i + 1);
            assert!(!s.keynote.is_empty(), "Empty keynote at degree {}", i + 1);
        }
    }

    #[test]
    fn sign_boundaries() {
        // Last degree of each sign -> first degree of next sign
        for sign_idx in 0..12 {
            let last = sabian_for_sign_degree(sign_idx, 30).unwrap();
            assert_eq!(last.sign, SIGN_NAMES[sign_idx]);
            assert_eq!(last.sign_degree, 30);

            if sign_idx < 11 {
                let first_next = sabian_for_sign_degree(sign_idx + 1, 1).unwrap();
                assert_eq!(first_next.sign, SIGN_NAMES[sign_idx + 1]);
                assert_eq!(first_next.sign_degree, 1);
                assert_eq!(first_next.degree, last.degree + 1);
            }
        }
    }

    #[test]
    fn invalid_sign_index_returns_none() {
        assert!(sabian_for_sign_degree(12, 1).is_none());
    }

    #[test]
    fn invalid_degree_returns_none() {
        assert!(sabian_for_sign_degree(0, 0).is_none());
        assert!(sabian_for_sign_degree(0, 31).is_none());
    }
}
