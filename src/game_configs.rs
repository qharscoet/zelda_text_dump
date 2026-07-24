use encoding_rs::Encoding;
use itertools::Itertools;

use crate::{message::{MessageAttributes, Tag}, utils};

#[derive(Default)]
pub struct StyleInfo {
    pub centered:bool,
    pub color:String,
    pub bg_color:String,
    pub alt_font:bool,
    pub style_id:String
}

pub enum StyleTagType {
    Color(u16),
    Size(u16),
    Ruby(u8, String),
    Unknown
}

pub enum TagType {
    Style(StyleTagType),
    Replace
}


pub fn get_tag_type_default(tag:&Tag) -> TagType {
    get_tag_type_default_inner(tag.group, tag.number,&tag.payload, true, encoding_rs::WINDOWS_1252)
}

fn get_tag_type_default_inner(tag_group : u8, tag_number : u16, payload : &[u8], big_endian : bool, encoding : &'static Encoding) -> TagType {
    let tag_number = if big_endian { tag_number } else {tag_number.swap_bytes()};
    let get_u16 = if big_endian { utils::get_u16_be } else {utils::get_u16_le};
    match tag_group {
        0xFF => match tag_number {
            0x00 => TagType::Style(StyleTagType::Color(payload[0] as u16)),
            0x01 => TagType::Style(StyleTagType::Size(get_u16(payload, 0))),
            0x02 => {
                let over_count = payload[0];
                let last_is_zero = payload[payload.len() -1] == 0x00;
                let slice_end = payload.len() - (last_is_zero as usize);
                let raw_bytes = &payload[1..slice_end];

                let decoded_ruby = encoding.decode(&raw_bytes).0.to_string();
                TagType::Style(StyleTagType::Ruby(over_count, decoded_ruby))
            },
            _ => TagType::Style(StyleTagType::Unknown)
        },
        _ => TagType::Replace
    }
}

fn get_tag_type_default_msbt(tag : &Tag, big_endian : bool, encoding : &'static Encoding) -> TagType {
    let get_u16 = if big_endian { utils::get_u16_be } else {utils::get_u16_le};
    match tag.group {
        0x0 => match tag.number {
            0x00 =>  {
                let over_count = get_u16(&tag.payload, 0)/2;
                let ruby_bytes_count = get_u16(&tag.payload, 2);
                let raw_bytes = &tag.payload[4..];

                let decoded_ruby = encoding.decode(&raw_bytes).0.to_string();
                TagType::Style(StyleTagType::Ruby(over_count as u8, decoded_ruby))
            },
            0x02 => TagType::Style(StyleTagType::Size(get_u16(&tag.payload, 0))),
            0x03 => TagType::Style(StyleTagType::Color(get_u16(&tag.payload, 0))),
            _ => TagType::Style(StyleTagType::Unknown)
        },
        _ => TagType::Replace
    }
}

#[derive(Clone)]
pub struct GameConfig {
    pub name: &'static str,
    pub id : &'static str,
    pub logo : &'static str,
    pub big_endian : bool,

    pub get_color_hex: fn(usize) -> &'static str,
    pub get_tag_replacement: fn(&Tag) -> String,
    pub get_tag_type : fn(&Tag) -> TagType,
    pub get_message_style : fn(&MessageAttributes) -> StyleInfo,

    pub get_languages : fn() -> &'static [(&'static str, &'static str)],
    pub get_filenames : fn() -> &'static [&'static str]
}

pub const ALL_CONFIGS  : [&GameConfig;6]= [&TP, &TWW, &PH, &ST, &FSA, &ALBW];

pub const TWW: GameConfig = GameConfig {
    name: "The Wind Waker",
    id: "tww",
    logo : "https://www.nintendo.com/jp/character/zelda/history/img/branch-d/01/pc/logo.png",
    big_endian : true,
    get_languages : || {
        const LANGUAGES : [(&str, &str);4] = [
            ("jp", "Japanese"),
            ("uk", "UK English"),
            ("fr", "French"),
            // ("sp", "Spanish"),
            ("de", "German"),
            // ("it" "Italian")
        ];

        &LANGUAGES
    },
    get_filenames : || {
        const FILENAMES : [&str;1] = [
            "zel_00.bmg",
        ];

        &FILENAMES
    },
    get_color_hex: |id| {
        const COLORS_RGB_TWW: [&str; 9] = [
            "#ffffff",
            "#ff6400",
            "#00ff00",
            "#7878ff",
            "#ffff3c",
            "#00ffff",
            "#ff00ff",
            "#828282",
            "#ff8000",
        ];

        COLORS_RGB_TWW[id]
    },
    get_tag_type : |tag| {
        get_tag_type_default_inner(tag.group, tag.number, &tag.payload, true, encoding_rs::SHIFT_JIS)
    },
    get_tag_replacement : |tag| {
        match tag.group {
            0x00 => {
                match tag.number {
                    0x00 => "[Link]",
                    0x08 => "• ",
                    0x09 => "• ",
                    0x0A => "[A] ",
                    0x0B => "[B] ",
                    0x0C => "[C] ",
                    0x0D => "[L] ",
                    0x0E => "[R] ",
                    0x0F => "[X] ",
                    0x10 => "[Y] ",
                    0x11 => "[Z] ",
                    0x12 => "[DPad] ",
                    0x13 => "[Analog] ",
                    0x14 => "🡄 ",
                    0x15 => "🡆 ",
                    0x16 => "🡅 ",
                    0x17 => "🡇 ",
                    0x18 => "[AnalogUp] ",
                    0x19 => "[AnalogDown] ",
                    0x1A => "[AnalogLeft] ",
                    0x1B => "[AnalogRight] ",
                    0x1C => "[AnalogVertical] ",
                    0x1D => "[AnalogHorizontal] ",
                    0x1E => " ",
                    0x1F => " ",
                    0x20 => "[CanonBalls]",
                    0x21 => "[BrokenVasePayment]",
                    0x22 => "[AuctionCharacter]",
                    0x23 => "[AuctionItem]",
                    0x24 => "[AuctionBid]",
                    0x25 => "[AuctionStartingBid]",
                    0x26 => "[PlayerActionBidSelector]",
                    0x27 => "[FlashingA]",
                    0x28 => "[OrcaBlowCount]",
                    0x29 => "[PiratePassword]",
                    0x2A => "[Starburst]",
                    0x2B => "[PostOfficeGameLetterCount]",
                    0x2C => "[PostOfficeGameRupeeReward]",
                    0x2D => "[PostBoxLetterCount]",
                    0x2E => "[RemainingKorokCount]",
                    0x2F => "[RemainingForestWaterTime]",
                    0x30 => "[FlightPlatformTime]",
                    0x31 => "[FlightPlatformRecord]",
                    0x32 => "[BeedlePointCount]",
                    0x33 => "[MsMariePendantCount]",
                    0x34 => "[MsMariePendantTotal]",
                    0x35 => "[PigGameTime]",
                    0x36 => "[SailingGameRupeeReward]",
                    0x37 => "[CurrentBombCapacity]",
                    0x38 => "[CurrentArrowCapacity]",
                    0x39 => "[Heart]",
                    0x3A => "[MusicNote]",
                    0x3B => "[TargetLetterCount]",
                    0x3C => "[FishmanHitCount]",
                    0x3D => "[FishmanRupeeReward]",
                    0x3E => "[BokoBabaSeedCount]",
                    0x3F => "[SkullNecklaceCount]",
                    0x40 => "[ChuJellyCount]",
                    0x41 => "[JoyPendantCount]",
                    0x42 => "[GoldenFeatherCount]",
                    0x43 => "[KnightsCrestCount]",
                    0x44 => "[BeedleRupeeOffer]",
                    0x45 => "[BokoBabaSellSelector]",
                    0x46 => "[SkullNecklaceSellSelector]",
                    0x47 => "[ChuJellySellSelector]",
                    0x48 => "[JoyPendantSellSelector]",
                    0x49 => "[GoldenFeatherSellSelector]",
                    0x4A => "[KnightsCrestSellSelector]",
                    _ => ""
                }
            }
            _=> ""
        }.to_string()
    },

    get_message_style : |attribs: &MessageAttributes| {
        let mut centered = false;
        let mut color = String::new();
        let mut bg_color = String::new();
        
        match attribs.payload[0x08] {
            0x01 => { bg_color = String::from("#3F48CC");}
            0x02 => { bg_color = String::from("#A68752"); color = String::from("#000000");}
            0x06 => { bg_color = String::from("#84795A"); color = String::from("#000000");}
            0x07 => { bg_color = String::from("#BDA273"); color = String::from("#000000");}
            0x09 => { bg_color = String::from("#3F48CC");}
            0x0D => { centered = true; }
            0x0E => { bg_color = String::from("#3F48CC"); }
            _ => {}
        }
        
        
        let style_id = match attribs.payload[0x08] {
            0x01|0x02|0x06|0x07|0x09|0x0D|0x0E => format!("display-{}", attribs.payload[0x08]),
            _  => String::new()
        };

        StyleInfo { centered, color, bg_color, alt_font : false, style_id }
    }
};

pub const TP: GameConfig = GameConfig {
    name: "Twilight Princess",
    id:"tp",
    logo : "https://www.nintendo.com/jp/character/zelda/history/img/branch-c/02/pc/logo.png",
    big_endian : true,
    get_languages : || {
        const LANGUAGES : [(&str, &str);4] = [
            ("jp", "Japanese"),
            ("us", "US English"),
            ("fr", "French"),
            // ("sp", "Spanish"),
            ("de", "German"),
            // ("it" "Italian")
        ];

        &LANGUAGES
    },
    get_filenames : || {
        const FILENAMES : [&str;10] = [
            "zel_00.bmg",
            "zel_01.bmg",
            "zel_02.bmg",
            "zel_03.bmg",
            "zel_04.bmg",
            "zel_05.bmg",
            "zel_06.bmg",
            "zel_07.bmg",
            "zel_08.bmg",
            "zel_99.bmg",
        ];

        &FILENAMES
    },
    get_color_hex: |id| {
        const COLORS_RGB : [&str; 9] = [
            "#FFFFFF",
            "#f07878",
            "#aadc8c",
            "#a0b4dc",
            "#dcdc82",
            "#b4c8e6",
            "#c8a0dc",
            "#ffffff",
            "#dcaa78",
        ];

        COLORS_RGB[id]
    },
    get_tag_type : |tag| {
        get_tag_type_default_inner(tag.group, tag.number, &tag.payload, true, encoding_rs::SHIFT_JIS)
    },
    get_tag_replacement : |tag| {
        match tag.group {
            0x00 => {
                match tag.number {
                    0x00 =>	"[Link]",
                    0x08 => "• ",
                    0x09 => "• ",
                    0x0A => "[A] ",
                    0x0B => "[B] ",
                    0x0C => "[C] ",
                    0x0D => "[L] ",
                    0x0E => "[R] ",
                    0x0F => "[X] ",
                    0x10 => "[Y] ",
                    0x11 => "[Z] ",
                    0x12 => "[DPad] ",
                    0x13 => "[Analog] ",
                    0x14 => "🡄 ",
                    0x15 => "🡆 ",
                    0x16 => "🡅 ",
                    0x17 => "🡇 ",
                    0x18 => "[AnalogUp] ",
                    0x19 => "[AnalogDown] ",
                    0x1A => "[AnalogLeft] ",
                    0x1B => "[AnalogRight] ",
                    0x1C => "[AnalogVertical] ",
                    0x1D => "[AnalogHorizontal] ",
                    0x1E => " ",
                    0x1F => " ",
                    0x23 => "[RedTarget] ",
                    0x24 => "[YellowTarget] ",
                    0x2E => "[XorY] ",
                    0x39 => "♥ ",
                    0x22 =>	"[Epona]",
                    0x29 =>	"[CurrentScent]",
                    0x2B =>	"[WarpingTo]",
                    0x2D =>	"[Bomb-Name]",
                    0x31 =>	"[Bomb-Count]",
                    0x32 =>	"[Bomb-Price]",
                    0x35 =>	"[nop000035]",
                    0x37 =>	"[Bombcap]",
                    0x3B =>	"[ReturnedBug]",
                    0x3C =>	"[LetterSender]",
                    0x3E =>	"[CurrentLetterPage]",
                    0x3F =>	"[MaxLetterPage]",
                    _ => ""
                }
            },
            0x03 => {
                match tag.number {
                    0x01 =>	"[WiiA]",
                    0x02 =>	"[WiiB]",
                    0x03 =>	"[WiiHome]",
                    0x04 =>	"[WiiMinus]",
                    0x05 =>	"[WiiPlus]",
                    0x06 =>	"[Wii1]",
                    0x07 =>	"[Wii2]",
                    0x08 =>	"[WiiD-WE]",
                    0x09 =>	"[WiiD-N]",
                    0x0A =>	"[WiiD-S]",
                    0x0B =>	"[WiiD-WE]",
                    0x0C =>	"[WiiD-E]",
                    0x0D =>	"[WiiD-W]",
                    0x0E =>	"[Wiimote]",
                    0x0F =>	"[WReticule]",
                    0x10 =>	"[WNunchunk]",
                    0x11 =>	"[Wiimote]",
                    0x12 =>	"[Fairy]",
                    0x13 =>	"[WiiC]",
                    0x14 =>	"[WiiZ]",
                    _ => ""
                }
            },
            0x04 => {
                match tag.number {
                    0x00 =>	"巫",
                    0x01 =>	"嗅",
                    0x02 =>	"眷",
                    0x03 =>	"蜀",
                    0x04 =>	"蟲",
                    0x05 =>	"裔",
                    0x06 =>	"惧",
                    0x07 =>	"綺",
                    0x08 =>	"罠",
                    0x09 =>	"祓",
                    0x0A =>	"墟",
                    0x0B =>	"絆",
                    0x0C =>	"僭",
                    0x0D =>	"憑",
                    _ => ""
                }
            },
            0x05 => {
                match tag.number {
                    0x00 =>	"[Time]",
                    0x03 =>	if tag.payload[0] == 0  {"[ReturnedBugs]" } else {"[RemainingBugs]"},
                    0x04 =>	"noop",
                    0x07 =>	"[RiverPoints]",
                    0x08 =>	"[FishLength]",
                    0x09 =>	"[MartGoalLeft]",
                    0x0A =>	"[LetterCount]",
                    0x0B =>	"[PoesNeeded]",
                    0x0C =>	if tag.payload[0] == 0 {"[LatestScore]" } else {"[HighScore]"},
                    0x0D =>	"[FishCount]",
                    0x0E =>	"[RollGoal]",
                    _ => ""
                }
            },
            0x06 => {
                match tag.number {
                    0x02 => "♂",	
                    0x03 => "♀",	
                    0x04 => "★",	
                    0x05 => "※",	
                    0x06 => "←",	
                    0x07 => "→",	
                    0x08 => "↑",	
                    0x09 => "↓",	
                    0x0A => "⧫",
                    0x0B => " ",    
                    _ => "",
                }
            },
            _=> "",
        }.to_string()
    },

    get_message_style : |attribs: &MessageAttributes| {
        let mut centered = false;
        let mut color = String::new();
        let mut alt_font = false;

        match attribs.payload[0x05] {
            0x00 => {}, //TODO : add dark background
            0x01 => {}, // no background
            0x07 => centered = true,
            0x0C => alt_font = true,
            0x0D => color = String::from("#b4c8e6"),
            0x0E => color = String::from("#aadc8c"),
            0x13 => {centered = true; alt_font = true;},
            _ => {}
        }

        let style_id = match attribs.payload[0x05] {
            0x00 | 0x0D |0x0E => format!("display-{}", attribs.payload[0x05]),
            _  => String::new()
        };

        StyleInfo { centered, color, bg_color : String::new(), alt_font, style_id }
    }
};

pub const PH: GameConfig = GameConfig {
    name: "Phantom Hourglass",
    id: "ph",
    logo : "https://www.nintendo.com/jp/character/zelda/history/img/branch-d/02/pc/logo.png",
    big_endian : false,
    get_languages : || {
        const LANGUAGES : [(&str, &str);4] = [
            ("jp", "Japanese"),
            ("us", "English"),
            ("fr", "French"),
            // // ("sp", "Spanish"),
            ("de", "German"),
            // ("it" "Italian")
        ];

        &LANGUAGES
    },
    get_filenames : || {
        const FILENAMES : [&str;32] = [
            "battle.bmg",
            "battleCommon.bmg",
            "bossLast1.bmg", 
            "bossLast3.bmg", 
            "brave.bmg", 
            "collect.bmg",   
            "demo.bmg",
            "field.bmg",
            "flame.bmg",
            "frost.bmg",
            "ghost.bmg",
            "hidari.bmg",
            "kaitei_F.bmg",
            "kaitei.bmg",
            "kojima1.bmg",
            "kojima2.bmg",
            "kojima3.bmg",
            "kojima5.bmg",
            "main_isl.bmg",
            "mainselect.bmg",
            "myou.bmg",
            "power.bmg",
            "regular.bmg",
            "sea.bmg",
            "sennin.bmg",
            "ship.bmg",
            "staff.bmg",
            "system.bmg",
            "torii.bmg",
            "wind.bmg",
            "wisdom_dngn.bmg",
            "wisdom.bmg",
        ];

        &FILENAMES
    },
    get_color_hex: |id| {
        const COLORS_RGB_TWW: [&str; 9] = [
            "#ffffff",
            "#ff6400",
            "#00ff00",
            "#7878ff",
            "#ffff3c",
            "#00ffff",
            "#ff00ff",
            "#828282",
            "#ff8000",
        ];

        COLORS_RGB_TWW[id]
    },
    get_tag_type : |tag| {
        get_tag_type_default_inner(tag.group, tag.number, &tag.payload, false, encoding_rs::UTF_16LE)
    },
    get_tag_replacement : |tag| {
        let tag_number = tag.number.swap_bytes();
        match tag.group {
            0xFE => match tag_number {
                0x00 => "[Link]",
                0x0E => "[Number]",
                _ => "[Unknown_Tag]"
            },
            _=> ""
        }.to_string()
    },

    get_message_style : |_attribs: &MessageAttributes| {
        let centered = false;
        let color = String::new();
        let bg_color = String::new();
        
        // match attribs.payload[0x08] {
        //     0x01 => { bg_color = String::from("#3F48CC");}
        //     0x02 => { bg_color = String::from("#A68752"); color = String::from("#000000");}
        //     0x06 => { bg_color = String::from("#84795A"); color = String::from("#000000");}
        //     0x07 => { bg_color = String::from("#BDA273"); color = String::from("#000000");}
        //     0x09 => { bg_color = String::from("#3F48CC");}
        //     0x0D => { centered = true; }
        //     0x0E => { bg_color = String::from("#3F48CC"); }
        //     _ => {}
        // }
        
        
        let style_id = String::new();

        StyleInfo { centered, color, bg_color, alt_font : false, style_id }
    }
};

pub const ST: GameConfig = GameConfig {
    name: "Spirit Tracks",
    id: "st",
    logo : "https://www.nintendo.com/jp/character/zelda/history/img/branch-d/03/pc/logo.png",
    big_endian : false,
    get_languages : || {
        const LANGUAGES : [(&str, &str);4] = [
            ("jp", "Japanese"),
            ("us", "English"),
            ("fr", "French"),
            // ("sp", "Spanish"),
            ("de", "German"),
            // ("it" "Italian")
        ];

        &LANGUAGES
    },
    get_filenames : || {
        const FILENAMES : [&str;30] = [
            "battle_common.bmg",
            "battle_parent.bmg",
            "castle_town.bmg",
            "castle.bmg",
            "collect.bmg",
            "demo.bmg",
            "demo01_05.bmg",
            "demo06_10.bmg",
            "demo11_15.bmg",
            "demo16_20.bmg",
            "demo21_25.bmg",
            "desert.bmg",
            "dungeon.bmg",
            "field.bmg",
            "flame_fld.bmg",
            "flame.bmg",
            "forest.bmg",
            "intrain.bmg",
            "maingame.bmg",
            "post.bmg",
            "regular.bmg",
            "select.bmg",
            "shop.bmg",
            "snow.bmg",
            "tower_lobby.bmg",
            "tower.bmg",
            "train_extra.bmg",
            "train.bmg",
            "village.bmg",
            "water.bmg",
        ];

        &FILENAMES
    },
    get_color_hex: |id| {
        const COLORS_RGB_TWW: [&str; 9] = [
            "#ffffff",
            "#ff6400",
            "#00ff00",
            "#7878ff",
            "#ffff3c",
            "#00ffff",
            "#ff00ff",
            "#828282",
            "#ff8000",
        ];

        COLORS_RGB_TWW[id]
    },
    get_tag_type : |tag| {
        get_tag_type_default_inner(tag.group, tag.number, &tag.payload, false, encoding_rs::UTF_16LE)
    },
    get_tag_replacement : |tag| {
        let tag_number = tag.number.swap_bytes();
        match tag.group {
            0xFE => match tag_number {
                0x00 => "[Link]",
                0x0E => "[Number]",
                _ => "[Unknown_Tag]"
            },
            _=> ""
        }.to_string()
    },

    get_message_style : |_attribs: &MessageAttributes| {
        let centered = false;
        let color = String::new();
        let bg_color = String::new();
        
        // match attribs.payload[0x08] {
        //     0x01 => { bg_color = String::from("#3F48CC");}
        //     0x02 => { bg_color = String::from("#A68752"); color = String::from("#000000");}
        //     0x06 => { bg_color = String::from("#84795A"); color = String::from("#000000");}
        //     0x07 => { bg_color = String::from("#BDA273"); color = String::from("#000000");}
        //     0x09 => { bg_color = String::from("#3F48CC");}
        //     0x0D => { centered = true; }
        //     0x0E => { bg_color = String::from("#3F48CC"); }
        //     _ => {}
        // }
        
        
        let style_id = String::new();

        StyleInfo { centered, color, bg_color, alt_font : false, style_id }
    }
};

pub const FSA: GameConfig = GameConfig {
    name: "Four Swords Adventures",
    id: "fsa",
    logo : "https://www.nintendo.com/jp/character/zelda/history/img/branch-c/03/pc/logo.png",
    big_endian : true,
    get_languages : || {
        const LANGUAGES : [(&str, &str);4] = [
            ("jp", "Japanese"),
            ("us", "US English"),
            ("fr", "French"),
            // ("sp", "Spanish"),
            ("de", "German"),
            // ("it" "Italian")
        ];

        &LANGUAGES
    },
    get_filenames : || {
        const FILENAMES : [&str;1] = [
            "gc_four_swords_text.bmg",
        ];

        &FILENAMES
    },
    get_color_hex: |id| {
        const COLORS_RGB_TWW: [&str; 9] = [
            "#ffffff",
            "#ff6400",
            "#00ff00",
            "#7878ff",
            "#ffff3c",
            "#00ffff",
            "#ff00ff",
            "#828282",
            "#ff8000",
        ];

        COLORS_RGB_TWW[id]
    },
    get_tag_type : |tag| {
        match tag.group {
            0x2 => match tag.number {
                0x1E => TagType::Style(StyleTagType::Color(tag.payload[0] as u16)),
                _ => TagType::Replace
            },
            _ => TagType::Replace
        }
    },
    get_tag_replacement : |_tag| {
        "".to_string()
    },

    get_message_style : |_attribs: &MessageAttributes| {
        let centered = false;
        let color = String::new();
        let bg_color = String::new();
        
        // match attribs.payload[0x08] {
        //     0x01 => { bg_color = String::from("#3F48CC");}
        //     0x02 => { bg_color = String::from("#A68752"); color = String::from("#000000");}
        //     0x06 => { bg_color = String::from("#84795A"); color = String::from("#000000");}
        //     0x07 => { bg_color = String::from("#BDA273"); color = String::from("#000000");}
        //     0x09 => { bg_color = String::from("#3F48CC");}
        //     0x0D => { centered = true; }
        //     0x0E => { bg_color = String::from("#3F48CC"); }
        //     _ => {}
        // }
        
        
        let style_id = String::new();

        StyleInfo { centered, color, bg_color, alt_font : false, style_id }
    }
};

pub const ALBW: GameConfig = GameConfig {
    name: "A Link Between Worlds",
    id: "albw",
    logo : "https://www.nintendo.com/jp/character/zelda/history/img/branch-b/04/pc/logo.png",
    big_endian : false,
    get_languages : || {
        const LANGUAGES : [(&str, &str);2] = [
            ("jp", "Japanese"),
            // ("us", "US English"),
            ("fr", "French"),
            // ("sp", "Spanish"),
            // ("de", "German"),
            // ("it" "Italian")
        ];

        &LANGUAGES
    },
    get_filenames : || {
        const FILENAMES : [&str;6] = [
            "Common.msbt",
            "Field.msbt",
            "FieldLight.msbt",
            "FieldDark.msbt",
            "Ganon.msbt",
            "Ending.msbt",
        ];

        &FILENAMES
    },
    get_color_hex: |id| {

        if id == 0xFFFF { 
            "#ffffff" 
        }
        else {
            const COLORS_RGB: [&str; 11] = [
                "#ffffff",
                "#ff6400",
                "#00ff00",
                "#7878ff",
                "#ffff3c",
                "#00ffff",
                "#ff00ff",
                "#828282",
                "#ff8000",
                "#123456",
                "#789abc",
            ];
    
            COLORS_RGB[id]
        }
    },
    get_tag_type : |tag| {
        get_tag_type_default_msbt(tag, false, encoding_rs::UTF_16LE)
    },
    get_tag_replacement : |tag| {
        let payload = tag.payload.iter().map(|b| format!("{:02X}", b)).join(",");
        format!("[Tag {}  {} ]", match tag.group {
            0x0 => String::from(match tag.number {
                0 => "Ruby ",
                1 => "Font ",
                2 => "Size ",
                3 => "Color ",
                _ => ""
            }),
            _ => format!("{}:{}", tag.group, tag.number)
        }, payload)
    },

    get_message_style : |_attribs: &MessageAttributes| {
        let centered = false;
        let color = String::new();
        let bg_color = String::new();
    
        
        
        let style_id = String::new();

        StyleInfo { centered, color, bg_color, alt_font : false, style_id }
    }
};
