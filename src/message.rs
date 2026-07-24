use std::fmt;

use itertools::Itertools;

use crate::{game_configs::GameConfig, utils::get_u16_be};

#[derive(Debug, Default, Clone)]
pub struct MessageAttributes {
    pub payload : Vec<u8>,
}


impl MessageAttributes {
    pub fn get_message_id(&self) -> Option<u16> {
        if self.payload.len() > 16 { //This should exclude PH
            Some(get_u16_be(&self.payload, 0))
        } else {
            None
        }
    }

    fn _get_printing_style(&self) -> u8 {
        self.payload[0x06]
    }

    pub fn is_empty(&self) -> bool {
        self.payload.iter().all(|v| *v == 0)
    }
}

impl fmt::Display for MessageAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:X?}", self.payload)
    }
}


pub type MessageText = Vec<TextPart>;
pub struct Message {
    pub text : Vec<MessageText>,
    pub attribs : MessageAttributes,
    pub id : usize
}

#[derive(Default, Clone)]
pub struct MessageSingleLang {
    pub text : MessageText,
    pub attribs : MessageAttributes,
    pub id : usize
}


impl Message {
    pub fn is_empty(&self) -> bool {
        self.text.iter().all(|parts| parts.is_empty())
    }
}

impl MessageSingleLang {
    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && self.attribs.is_empty()
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:?}", self.text.iter().map(|text_parts| {
            text_parts.iter().map(|part|
                part.to_string() ).join("")
        }).collect::<Vec<_>>())
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}",self.text.iter().map(|lang_msg| 
            lang_msg.iter().map(|part| format!("{:?}", part)).collect::<Vec<_>>().join("\n")
            ).join("\n ------------------\n")   
        )
    }
}

impl Default for Message {
    fn default() -> Self {
        Message { text: Default::default(), attribs: MessageAttributes::default(), id : 0}
    }
}



pub fn get_raw_msg(msg : &MessageText, config : Option<&GameConfig>) -> String {
    msg.iter().map(|text_part| match text_part {
            TextPart::Text(s) => s.to_string(),
            TextPart::Tag(t) => t.get_simple_replacement(config).to_string()
        }).join("")
}



#[derive(Default, Clone)]
pub struct Tag {
    pub group : u8,
    pub number : u16,
    pub payload : Vec<u8>
}

#[derive(Clone)]
pub enum TextPart {
    Text(String),
    Tag(Tag)
}

impl fmt::Display for TextPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextPart::Text(s) => write!(f, "{}", s),
            TextPart::Tag(t) => write!(f, "{}", t),
        }
    }
}

impl fmt::Debug for TextPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextPart::Text(s) => write!(f, "Text : {}", s),
            TextPart::Tag(t) => write!(f, "Tag : {:?}", t),
        }
    }
}

impl From<&Vec<u8>> for Tag {
    fn from(value: &Vec<u8>) -> Self {
        if value.len() > 2 {
            Tag {
                group : value[0],
                number : get_u16_be(&value, 0x1),
                payload : Vec::from(&value[0x03..]),
            }
        } else {
            Tag::default()
        }
    }
}


impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:0>2X}/{:0>4X} payload : {:X?}", self.group, self.number, self.payload)
    }
}




pub trait MessageParser {
    fn get_all_messages(&self) -> Vec<MessageSingleLang>;
    fn get_encoding(&self) -> &'static encoding_rs::Encoding;
}