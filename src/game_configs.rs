use std::fmt::format;

use encoding_rs::Encoding;
use itertools::Itertools;

use crate::{message::{MessageAttributes, MessageId, Tag}, utils::{self, get_u16_be}};

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
    Replace,
    Insert((String, MessageId)) //Bank name, id
}


pub fn get_tag_type_default(tag:&Tag) -> TagType {
    get_tag_type_default_inner(tag.group, tag.number,&tag.payload, true, encoding_rs::WINDOWS_1252)
}

fn get_tag_type_default_inner(tag_group : u8, tag_number : u16, payload : &[u8], big_endian : bool, encoding : &'static Encoding) -> TagType {
    let tag_number = if big_endian { tag_number } else {tag_number.swap_bytes()};
    let get_u16 = if big_endian { utils::get_u16_be } else {utils::get_u16_le};
    match tag_group {
        0xFF => match tag_number {
            0x00 => TagType::Style(StyleTagType::Color(
                if payload[0] == 0 {
                    0xFFFF
                } else {
                    payload[0] as u16
                }
            )),
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
            _ => TagType::Replace,
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

pub const ALL_CONFIGS  : [&GameConfig;8]= [&TP, &TWW, &PH, &ST, &FSA, &ALBW, &TFH, &SS];

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
        let idx = if id == 0xFFFF { 0 } else {id};
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

        COLORS_RGB_TWW[idx]
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
        let idx = if id == 0xFFFF { 0 } else {id};
        const COLORS_RGB : [&str; 9] = [
            "#ffffff",
            "#f07878",
            "#aadc8c",
            "#a0b4dc",
            "#dcdc82",
            "#b4c8e6",
            "#c8a0dc",
            "#ffffff",
            "#dcaa78",
        ];

        COLORS_RGB[idx]
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
        let idx = if id == 0xFFFF { 0 } else {id};
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

        COLORS_RGB_TWW[idx]
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
        let idx = if id == 0xFFFF { 0 } else {id};
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

        COLORS_RGB_TWW[idx]
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
        let idx = if id == 0xFFFF { 0 } else {id};
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

        COLORS_RGB_TWW[idx]
    },
    get_tag_type : |tag| {
        match tag.group {
            0x2 => match tag.number {
                0x1E => TagType::Style(StyleTagType::Color(
                    if tag.payload[0] == 0 {
                        0xFFFF
                    } else {
                        tag.payload[0] as u16
                    }
                )),
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
        const LANGUAGES : [(&str, &str);4] = [
            ("jp", "Japanese"),
            ("en", "English"),
            ("fr", "French"),
            // ("sp", "Spanish"),
            ("de", "German"),
            // ("it" "Italian")
        ];

        &LANGUAGES
    },
    get_filenames : || {
        const FILENAMES : [&str;117] = [
            "Action.msbt",
"Collect.msbt",
"Common.msbt",
"EventItemGet.msbt",
"ExtraName.msbt",
"HintGhost.msbt",
"ItemName.msbt",
"ItemNameUpper.msbt",
"ItemSelect.msbt",
"ItemTutorial.msbt",
"LocationName.msbt",
"LocationNameUpper.msbt",
"NPCName.msbt",
"Opening.msbt",
"StaffCredit.msbt",
"System.msbt",
"Demo010.msbt",
"Demo020.msbt",
"Demo030.msbt",
"Demo040.msbt",
"Demo050.msbt",
"Demo060.msbt",
"Demo070.msbt",
"Demo080.msbt",
"Demo090.msbt",
"Demo100.msbt",
"Demo110.msbt",
"Castle.msbt",
"Dark.msbt",
"Dokuro.msbt",
"East.msbt",
"Ganon.msbt",
"Hagure.msbt",
"Hera.msbt",
"Ice.msbt",
"Kame.msbt",
"Sand.msbt",
"Water.msbt",
"Wind.msbt",
"cl_Church_UG.msbt",
"E3_message.msbt",
"Cave.msbt",
"CrossBattle.msbt",
"CrossBoard.msbt",
"CrossForceTalk.msbt",
"CrossOldMan.msbt",
"CrossRecordList.msbt",
"DefaultShadowLink.msbt",
"Ending.msbt",
"Field.msbt",
"FieldDark.msbt",
"FieldDark_00.msbt",
"FieldDark_02.msbt",
"FieldDark_05.msbt",
"FieldDark_0F.msbt",
"FieldDark_13.msbt",
"FieldDark_14.msbt",
"FieldDark_16.msbt",
"FieldDark_17.msbt",
"FieldDark_18.msbt",
"FieldDark_1A.msbt",
"FieldDark_1B.msbt",
"FieldDark_1E.msbt",
"FieldDark_22.msbt",
"FieldDark_28.msbt",
"FieldDark_29.msbt",
"FieldDark_2A.msbt",
"FieldDark_2C.msbt",
"FieldDark_33.msbt",
"FieldDark_35.msbt",
"FieldDark_3A.msbt",
"FieldLight.msbt",
"FieldLight_00.msbt",
"FieldLight_02.msbt",
"FieldLight_03.msbt",
"FieldLight_05.msbt",
"FieldLight_0A.msbt",
"FieldLight_0F.msbt",
"FieldLight_11.msbt",
"FieldLight_12.msbt",
"FieldLight_13.msbt",
"FieldLight_14.msbt",
"FieldLight_16.msbt",
"FieldLight_17.msbt",
"FieldLight_18.msbt",
"FieldLight_1A.msbt",
"FieldLight_1B.msbt",
"FieldLight_1E.msbt",
"FieldLight_22.msbt",
"FieldLight_28.msbt",
"FieldLight_29.msbt",
"FieldLight_2A.msbt",
"FieldLight_2B.msbt",
"FieldLight_2C.msbt",
"FieldLight_2D.msbt",
"FieldLight_2E.msbt",
"FieldLight_33.msbt",
"FieldLight_35.msbt",
"FieldLight_37.msbt",
"FortuneMessage.msbt",
"HintGhostDark.msbt",
"HintGhostLight.msbt",
"ToRentalShopBoard.msbt",
"MiniDungeon_FieldDark_2B.msbt",
"MiniDungeon_FieldLight_07.msbt",
"MiniDungeon_FieldLight_15.msbt",
"MiniDungeon_FieldLight_1E.msbt",
"MiniDungeon_FieldLight_32.msbt",
"MiniDungeon_FieldLight_33.msbt",
"GirigiriGameTest.msbt",
"NpcClimberTest.msbt",
"NpcHinox.msbt",
"NpcTestIwata.msbt",
"StaffCreditTest.msbt",
"npcTest00.msbt",
"test.msbt",
"yamazaki.msbt",
        ];

        &FILENAMES
    },
    get_color_hex: |id| {

        if id == 0xFFFF { 
            "#ffffff" 
        }
        else {
            const COLORS_RGB: [&str; 12] = [
                "#262626",
                "#808080",
                "#FFFFFF",
                "#855C2F",
                "#591710",
                "#006400",
                "#375960",
                "#BAA800",
                "#3A1B4C",
                "#003F97",
                "#F92300",
                "#4AF0D1",
            ];
    
            COLORS_RGB[id]
        }
    },
    get_tag_type : |tag| {
        match tag.group {
            0x2 => {
                let idx = tag.payload[0] as usize;
                let bank_name = match tag.number {
                    0 => "NPCName",
                    1 => if tag.payload[2] == 1 {"LocationNameUpper"} else {"LocationName"},
                    2 => if tag.payload[2] == 1 {"ItemNameUpper"} else {"ItemName"},
                    _ => ""
                };

                TagType::Insert((bank_name.to_string(), MessageId::Int(idx)))
            },
            _ => get_tag_type_default_msbt(tag, false, encoding_rs::UTF_16LE)
        }
    },
    get_tag_replacement : |tag| {
        // let payload = tag.payload.iter().map(|b| format!("{:02X}", b)).join("");
        // let default = format!("[Tag {} {} ]", match tag.group {
        //     0x0 => String::from(match tag.number {
        //         0 => "Ruby ",
        //         1 => "Font ",
        //         2 => "Size ",
        //         3 => "Color ",
        //         _ => ""
        //     }),
            
        //     _ => format!("{}:{}", tag.group, tag.number)
        // }, if !payload.is_empty() { format!("val={{{}}}", payload) } else { "".to_string()});
        
        match tag.group {
            0x1 => match tag.number {
                0 => "[PlayerName]",
                1 => "[UserName]",
                2 => "[ShadowLinkPlayerName]",
                3 => "[ShadowLinkUserName]",
                4 => "[InsertMark]",
                5 => "[IntNumberN]", //TODO : paramete]rs
                6 => "[ChoiceN]", //TODO : paramete]rs
                7 => "" , //AutoForward
                8 => "", // Wa]it
                9 => "[MyRecordNum]",
                10 => "[ShadowLinkRecordNum]",
                11 => "[ShadowLinkPrizeMoney]",
                12 => "[ColoringStart]",
                13 => "[ColoringEnd]",
                14 => "[Flush]",
                15 => "[Vibrate]",
                16 => "[ChoicePositive]",
                17 => "", // Cursor
                _=> "",
            }.to_string(),
            0x2 => {
                let idx = tag.payload[0] as usize;
                match tag.number {
                    0 => {
                        let npc_names = ["[zelda]","[inpa]","[sahaspupil]","[zoraqueen]","[danpei]","[maple]","[priestgirl]","[miner]","[priest]","[sahas]","[ganon]","[darkzelda]","[darklink]","[darkganon]","[commander]","[hitghost]","[darklinkpet]","[blacksmithKid]","[shopmanmagic]","[kinstamother]"];
                        npc_names[idx]
                    },
                    1 => {
                        let location_names = ["[dgn_east]","[dgn_wind]","[dgn_hera]","[dgn_castle]","[dgn_dark]","[dgn_water]","[dgn_dokuro]","[dgn_hagure]","[dgn_ice]","[dgn_sand]","[dgn_kame]","[dgn_ganon]","[loc_name_church]","[loc_name_villagelight]","[loc_name_lake]","[loc_name_linkhouse]","[MtHebra]","[MagicShopLight]","[HoleofHyakkai]","[FortuneHouseDark]","[HakabaDark]","[MagicshopDark]","[BlackSmithDark]","[LinkHouseDark]","[milkbar]","[DevilsMarsh]","[ZorasVillage]","[DeathMountain]","[Boss]","[Cuccos]","[HyruleHotfoot]","[OctballDarby]","[RupeeRush]","[FortunesChoice]","[LostWoods]","[ThievesHideout]","[HyruleCastleCore]"];
                        location_names[idx]
                    },
                    2 => {
                        let item_names = ["[icerod]","[sandrod]","[tornaderod]","[bomb]","[firerod]","[hookshot]","[boomerang]","[hammer]","[bow]","[shield]","[bottle]","[potshop_red]","[potshop_blue]","[potshop_heart]","[item_name_bracelet]","[item_name_lantern]","[item_name_kinsta]","[item_name_gamecoin]","[item_name_stonebeauty]","[item_name_durian]","[item_name_doron]","[item_name_heartpiece]","[item_name_bee]","[item_name_beebadge]","[item_name_powergloves]","[item_name_powerfulglove]","[item_name_pegasus]","[item_name_bell]","[item_name_hintglass]","[item_name_goldenbee]","[item_name_potshop_yellow]","[item_name_potshop_purple]","[item_name_web]","[item_name_net]","[item_name_wisdom]","[item_name_courage]","[item_name_power]","[item_name_fairy]","[item_name_ore]","[postsword]","[charm]","[emptybracelet]","[bigbombflower]","[sword]","[mastersword]","[item_name_liver_blue]","[item_name_liver_purple]","[item_name_liver_yellow]","[item_name_clothes_blue]","[item_name_clothes_red]","[item_name_hyrule_shield]","[item_name_ganbari_power_up]","[item_name_pouch]","[keysmall]","[keyboss]","[heartcontioner]","[compass]","[apple_red]","[apple_blue]","[milk]","[mild_matured]","[message_bottle]","[special_move]","[clothes_blacksmith]","[clothes_green]","[lantern_lv2]","[net_lv2]","[bow_light]","[ganbaritubo]","[trifoce_wisdom]","[triforce_courage]","[triforce_power]","[icerod_LV2]","[sandrod_LV2]","[tornadrod_LV2]","[bomb_LV2]","[firerod_LV2]","[hookshot_LV2]","[boomerang_LV2]","[hammer_LV2]","[bow_LV2]","[icerod_rental]","[sandrod_rental]","[tornaderod_rental]","[bomb_rental]","[firerod_rental]","[hookshot_rental]","[boomerang_rental]","[hammer_rental]","[bow_rental]"];
                        item_names[idx]
                    },
                    _ => "",
                }.to_string()
            },
            _=> "".to_string()
        }
    },

    get_message_style : |_attribs: &MessageAttributes| {
        let centered = false;
        let color = String::new();
        let bg_color = String::new();
    
        
        
        let style_id = String::new();

        StyleInfo { centered, color, bg_color, alt_font : false, style_id }
    }
};


pub const TFH: GameConfig = GameConfig {
    name: "Tri Force Heroes",
    id: "tfh",
    logo : "https://www.nintendo.com/jp/character/zelda/history/img/branch-b/05/pc/logo.png",
    big_endian : false,
    get_languages : || {
        const LANGUAGES : [(&str, &str);3] = [
            ("jp", "Japanese"),
            ("en", "English"),
            ("fr", "French"),
            // ("sp", "Spanish"),
            // ("de", "German"),
            // ("it" "Italian")
        ];

        &LANGUAGES
    },
    get_filenames : || {
        const FILENAMES : [&str;76] = [
"Action.msbt",
"CourseResult.msbt",
"CreateExtraSaveData.msbt",
"E3Flow.msbt",
"ErrorApplet.msbt",
"GetItem.msbt",
"LayoutShopName.msbt",
"Live.msbt",
"Opening.msbt",
"StaffCredit.msbt",
"SystemFlow.msbt",
"AreaSimpleTalk.msbt",
// "KRAreaSimpleTalk.msbt",
// "KRNpcBoy.msbt",
// "KRNpcDressWoman.msbt",
// "KRNpcGentleMan.msbt",
// "KRNpcGirl.msbt",
// "KRNpcMadam.msbt",
// "KRNpcMiddleLady.msbt",
// "KRNpcMiddleman.msbt",
// "KRNpcShopmanDlc.msbt",
// "KRNpcShopmanPhoto.msbt",
// "KRNpcSoldier.msbt",
"NpcBlockMan.msbt",
"NpcBoy.msbt",
"NpcClothesIntern.msbt",
"NpcCommon.msbt",
"NpcDressWoman.msbt",
"NpcGameTreasure.msbt",
"NpcGentleMan.msbt",
"NpcGirl.msbt",
"NpcHeroMan.msbt",
"NpcKing.msbt",
"NpcMadam.msbt",
"NpcMatchingBattle.msbt",
"NpcMatchingBattleInet.msbt",
"NpcMatchingBattleLocal.msbt",
"NpcMatchingDlp.msbt",
"NpcMatchingInet.msbt",
"NpcMatchingLocal.msbt",
"NpcMatchingMulti.msbt",
"NpcMatchingPuppet.msbt",
"NpcMaterial.msbt",
"NpcMiddleLady.msbt",
"NpcMiddleman.msbt",
"NpcMobman.msbt",
"NpcNamingMan.msbt",
"NpcPrincessCursed.msbt",
"NpcPrincessDress.msbt",
"NpcShopmanClothes.msbt",
"NpcShopmanDlc.msbt",
"NpcShopmanGoods.msbt",
"NpcShopmanPhoto.msbt",
"NpcSoldier.msbt",
"NpcWitch.msbt",
"TrialAreaSimpleTalk.msbt",
"TrialNpcBlockMan.msbt",
"TrialNpcClothesIntern.msbt",
"TrialNpcMatchingDlp.msbt",
"TrialNpcMatchingInet.msbt",
"TrialNpcMatchingLocal.msbt",
"TrialNpcMatchingMulti.msbt",
"TrialNpcMaterial.msbt",
"ObjDoorHouse.msbt",
"ObjPuppet.msbt",
"ObjSavePoint.msbt",
"ObjSignboard.msbt",
"CostumeDetail.msbt",
"CostumeExplainLobby.msbt",
"CostumeExplainShop.msbt",
"CostumeFunction.msbt",
"CostumeName.msbt",
"CostumeShortName.msbt",
"FieldName.msbt",
"IntNumberN.msbt",
"ItemExplanation.msbt",
"ItemName.msbt",
"LocationName.msbt",
"MaterialDetail.msbt",
"MaterialName.msbt",
"MaterialNameGet.msbt",
"MaterialNameTalk.msbt",
"Todo.msbt",
"TestIkematsu.msbt",
"TestMessage.msbt",
"TestMouri.msbt",
"TestYamaoka.msbt",
        ];

        &FILENAMES
    },
    get_color_hex: |id| {

        if id == 0xFFFF { 
            "#ffffff" 
        }
        else {
            const COLORS_RGB: [&str; 2] = [
                "#003F97",
                "#F92300",
            ];
    
            COLORS_RGB[id]
        }
    },
    get_tag_type : |tag| {
        match tag.group {
            0x1 => {
                match tag.number {
                    1 => TagType::Insert(("FieldName".to_string(), MessageId::Int(tag.payload[0] as usize))),
                    2 => TagType::Insert(("ItemName".to_string(), MessageId::Int(tag.payload[0] as usize))),
                    5 => TagType::Insert(("CostumeName".to_string(), MessageId::Int(tag.payload[0] as usize))),
                    _=> TagType::Replace,
                }
            },
            _ => get_tag_type_default_msbt(tag, false, encoding_rs::UTF_16LE)
        }
    },
    get_tag_replacement : |tag| {
        let payload = tag.payload.iter().map(|b| format!("{:02X}", b)).join("");
        let default = format!("[Tag {} {} ]", match tag.group {
            0x0 => String::from(match tag.number {
                0 => "Ruby ",
                1 => "Font ",
                2 => "Size ",
                3 => "Color ",
                4 => "PageBreak",
                5 => "Reference",
                _ => ""
            }),
            
            _ => format!("{}:{}", tag.group, tag.number)
        }, if !payload.is_empty() { format!("val={{{}}}", payload) } else { "".to_string()});

        match tag.group {
            0x1 => {
                let get_enum = |arr : &[&str]| {
                    arr[tag.payload[0] as usize].to_string()
                };
                match tag.number {
                    0 => "[PlayerName]".to_string(),
                    1 => {
                        let field_names = ["[Grass]","[Water]","[Fire]","[Ice]","[Fort]","[Sand]","[Dark]","[Sky]"];
                        get_enum(&field_names)
                    },
                    2 => {
                        let item_names = ["[sword]","[bomb]","[bow]","[fireglove]","[boomerang]","[waterrod]","[aircannon]","[armshot]","[hammer]"];
                        get_enum(&item_names)
                    },
                    3 => {
                        let unit_names = ["None","Rupee","Second","Minute","Person","Number","Sheet","Win","Costume"];
                        let idx = tag.payload[4] as usize;
                        if idx > 0 {
                            format!("[Number of {}]", unit_names[idx])
                        } else {
                            String::from("[Number]")
                        }
                    }
                    4 => "[InsertMark]".to_string(),
                    5 => {
                        let costume_names = ["[First]","[Brave]","[Kokiri]","[Zelda]","[Fancy]","[Goron]","[Zora]","[GreatFairy]","[Bomb]","[Gauge]","[AgainstCold]","[RotationAttack]","[DashAttack]","[Rich]","[Boomerang]","[Alike]","[Lucky]","[WaterRod]","[Witch]","[Tights]","[EightBit]","[Kandelaar]","[WalkFast]","[Fairy]","[Normal]","[AirCannon]","[Hammer]","[WalkSand]","[ArmShot]","[FireGlove]","[Balloon]","[Calcify]","[Legend]","[SwordMaster]","[Idol]","[Thorn]","[DLC1]","[DLC2]","[DLC3]","[DLC4]","[DLC5]","[DLC6]"];
                        get_enum(&costume_names)
                    },
                    6 => "[ClearCourseAbyssSelf]".to_string(),
                    7 => "[PlayableCourseAbyssAll]".to_string(),
                    8 => "[KindPointNum]".to_string(),
                    _ => String::new()
                }
            },
            0x2 => match tag.number {
                0 => "", //[Vibrate]",
                1 => "", //[Flush]",
                2 => "", //[Wait]", // TODO params
                3 => "", //[AutoForward]", // TODO params
                4 => "",//[ChoiceN]", // TODO params
                5 => "",//[ChoicePositive]",
                6 => "[ColoringStart]",
                7 => "[ColoringEnd]",
                8 => "",//[LimitForward]", // TODO params
                _=> "",
            }.to_string(),
            _=> "".to_string()
        }
    },

    get_message_style : |_attribs: &MessageAttributes| {
        let centered = false;
        let color = String::new();
        let bg_color = String::new();
    
        
        
        let style_id = String::new();

        StyleInfo { centered, color, bg_color, alt_font : false, style_id }
    }
};


pub const SS: GameConfig = GameConfig {
    name: "Skyward Sword",
    id: "ss",
    logo : "https://www.nintendo.com/jp/character/zelda/history/img/branch-a/01/pc/logo.png",
    big_endian : true,
    get_languages : || {
        const LANGUAGES : [(&str, &str);3] = [
            ("jp", "Japanese"),
            ("en", "English"),
            ("fr", "French"),
            // ("sp", "Spanish"),
            // ("de", "German"),
            // ("it" "Italian")
        ];

        &LANGUAGES
    },
    get_filenames : || {
        const FILENAMES : [&str;80] = [
"0-Common/001-Action.msbt",
"0-Common/002-System.msbt",
"0-Common/003-ItemGet.msbt",
"0-Common/004-Object.msbt",
"0-Common/005-Tutorial.msbt",
"0-Common/006-1KenseiNormal.msbt",
"0-Common/006-2KenseiNormal.msbt",
"0-Common/006-3KenseiNormal.msbt",
"0-Common/006-4KenseiNormal.msbt",
"0-Common/006-5KenseiNormal.msbt",
"0-Common/006-6KenseiNormal.msbt",
"0-Common/006-7KenseiNormal.msbt",
"0-Common/006-8KenseiNormal.msbt",
"0-Common/006-9KenseiNormal.msbt",
"0-Common/006-KenseiNormal.msbt",
"0-Common/007-MapText.msbt",
"0-Common/008-Hint.msbt",
"0-Common/word.msbt",
"1-Town/100-Town.msbt",
"1-Town/101-Shop.msbt",
"1-Town/102-Zelda.msbt",
"1-Town/103-DaiShinkan.msbt",
"1-Town/104-Rival.msbt",
"1-Town/105-Terry.msbt",
"1-Town/106-DrugStore.msbt",
"1-Town/107-Kanban.msbt",
"1-Town/108-ShinkanA.msbt",
"1-Town/109-TakeGoron.msbt",
"1-Town/110-DivingGame.msbt",
"1-Town/111-FortuneTeller.msbt",
"1-Town/112-Trustee.msbt",
"1-Town/113-RemodelStore.msbt",
"1-Town/114-Friend.msbt",
"1-Town/115-Town2.msbt",
"1-Town/116-InsectGame.msbt",
"1-Town/117-Pumpkin.msbt",
"1-Town/118-Town3.msbt",
"1-Town/119-Captain.msbt",
"1-Town/120-Nushi.msbt",
"1-Town/121-AkumaKun.msbt",
"1-Town/122-Town4.msbt",
"1-Town/123-Town5.msbt",
"1-Town/124-Town6.msbt",
"1-Town/125-D3.msbt",
"1-Town/150-Siren.msbt",
"1-Town/199-Demo.msbt",
"2-Forest/200-Forest.msbt",
"2-Forest/201-ForestD1.msbt",
"2-Forest/202-ForestD2.msbt",
"2-Forest/203-ForestF2.msbt",
"2-Forest/204-ForestF3.msbt",
"2-Forest/250-ForestSiren.msbt",
"2-Forest/251-Salvage.msbt",
"2-Forest/299-Demo.msbt",
"3-Mountain/300-Mountain.msbt",
"3-Mountain/301-MountainD1.msbt",
"3-Mountain/302-Anahori.msbt",
"3-Mountain/303-MountainF2.msbt",
"3-Mountain/304-MountainD2.msbt",
"3-Mountain/305-MountainF3.msbt",
"3-Mountain/350-MountainSiren.msbt",
"3-Mountain/351-Salvage.msbt",
"3-Mountain/399-Demo.msbt",
"4-Desert/400-Desert.msbt",
"4-Desert/401-DesertD2.msbt",
"4-Desert/402-DesertF2.msbt",
"4-Desert/403-DesertD1.msbt",
"4-Desert/404-DesertF3.msbt",
"4-Desert/405-DesertD2Clear.msbt",
"4-Desert/406-TrolleyRace.msbt",
"4-Desert/450-DesertSiren.msbt",
"4-Desert/451-Salvage.msbt",
"4-Desert/460-RairyuMinigame.msbt",
"4-Desert/499-Demo.msbt",
"5-CenterField/500-CenterField.msbt",
"5-CenterField/501-Inpa.msbt",
"5-CenterField/502-CenterFieldBack.msbt",
"5-CenterField/503-Goron.msbt",
"5-CenterField/510-Salvage.msbt",
"5-CenterField/599-Demo.msbt",
        ];

        &FILENAMES
    },
    get_color_hex: |id| {

        if id == 0xFFFF { 
            "#ffffff" 
        }
        else {
            const COLORS_RGB: [&str; 15] = [
                "#FF5050",
                "#FF7878",
                "#E6A000",
                "#469BEB",
                "#50DC41",
                "#FF6400",
                "#8C468C",
                "#1EB91E",
                "#009BA5",
                "#F50A32",
                "#919BA0",
                "#EFEF00",
                "#5F9669",
                "#FFFFFF",
                "#000000",
            ];
    
            COLORS_RGB[id]
        }
    },
    get_tag_type : |tag| {
        match tag.group {
            0x02 => match tag.number {
                0x1 => TagType::Insert((String::from("003-ItemGet"), MessageId::Label(format!("NAME_ITEM_{:03}", get_u16_be(&tag.payload, 0))))),
                _ => TagType::Replace
            },
            0x03 => match tag.number {
                0x3 | 0x04 => TagType::Insert((String::from("word"), MessageId::Label(format!("lang:word:{:03}:01", tag.payload[0])))),
                _ => TagType::Replace
            },
            _ => get_tag_type_default_msbt(tag, true, encoding_rs::UTF_16BE)
        }
    },
    get_tag_replacement : |tag| {
        let payload = tag.payload.iter().map(|b| format!("{:02X}", b)).join("");
        let default = format!("[Tag {} {} ]", match tag.group {
            0x0 => String::from(match tag.number {
                0 => "Ruby ",
                1 => "Font ",
                2 => "Size ",
                3 => "Color ",
                _ => ""
            }),
            
            _ => format!("{}:{}", tag.group, tag.number)
        }, if !payload.is_empty() { format!("val={{{}}}", payload) } else { "".to_string()});

        match tag.group {
            0x0 => "".to_string(),
            0x1 => match tag.number {
                0 | 1 | 2 | 3 => "   • ",
                _ => "",
            }.to_string(),
            0x2 => match tag.number {
                0 => "[Link]".to_string(),
                2 => "[Var]".to_string(),
                3 => "[Number]".to_string(), //TODO Params
                4 => format!("[Button {}]", tag.payload[0]),
                _=> default,
            },
            0x3 => match tag.number {
                0 => "".to_string(), // TODO : exposant for 1er, 2e etc, do we handle it ?
                1 => "".to_string(), // Some kind of text action, find out which, but surely invisible
                _ => default,
            },
            _=> default,
        }
    },

    get_message_style : |_attribs: &MessageAttributes| {
        let centered = false;
        let color = String::new();
        let bg_color = String::new();
    
        
        
        let style_id = String::new();

        StyleInfo { centered, color, bg_color, alt_font : false, style_id }
    }
};