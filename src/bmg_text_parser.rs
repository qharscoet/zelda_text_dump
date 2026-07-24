use itertools::Itertools;
use regex::Regex;
use std::{fs::File, io::{self, BufRead, BufReader}, path::Path, sync::LazyLock};

use crate::{message::{MessageParser,MessageAttributes, MessageSingleLang, Tag, TextPart}, utils::unpack_u16};

static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?P<ID>[[:xdigit:]]+) (@(?P<slot>[[:xdigit:]]{4}) )?(?P<attribs>\[.+\]) = (?P<str>.+)?").unwrap());
static RE_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\\[x|z]\{(.*?)\}").unwrap());
static RE_ENCODING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"@ENCODING\s+= (\d)").unwrap());

impl std::str::FromStr for Tag {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        
        if let Some(caps) = RE_TAG.captures(s) {
            let args = caps.get(1).map_or("", |m| m.as_str());
            let values = args.split(",").collect::<Vec<_>>();

            let x_escape = s.starts_with(r"\x");
            let start_idx = x_escape as usize;

            if x_escape {
                let (total_size, group) = unpack_u16(u16::from_str_radix(values[start_idx], 16).unwrap_or_default());
                //real payload size, BMG text format adds a padding 0 and odd sizes ??
                let even_size = (((total_size/2) * 2) - 5) as usize;
                
                
                let number = u16::from_str_radix(values[start_idx + 1], 16).unwrap();
                let payload : Vec<_> = values[start_idx+2..].iter().flat_map(|s| u16::from_str_radix(s, 16)).map(|v| unpack_u16(v)).flat_map(|(v1,v2)| [v1,v2]).take(even_size).collect();
                // println!("{:#x}, {:#x}, {:?}", total_size, group, payload);

                Ok(Tag { group:group, number:number, payload:payload})
            } else {
                Err("Z escapes not implemented yet")
            }
        } else  {
            Err("Couldn't capture data")
        }
    }
}


impl std::str::FromStr for MessageAttributes {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut values = s.split_inclusive(&[',', '/']).fold(Vec::new(), |mut acc, s| {
            if s.ends_with(',') {
                acc.push(u8::from_str_radix(&s[0..s.len() -1], 16).unwrap_or_default());
            } else if s.ends_with('/') {
                acc.push(u8::from_str_radix(&s[0..s.len() -1], 16).unwrap_or_default());
                let curr_len = acc.len();
                let next_len = ((curr_len + 4)/4) * 4; //align to next 32
                acc.resize(next_len, 0);

            } else {
                acc.push(u8::from_str_radix(s, 16).unwrap_or_default());
            };
            acc
        });
        values.resize(16, 0);
        Ok(MessageAttributes { payload: values })
    }
}

impl std::str::FromStr for MessageSingleLang {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("Line is empty");
        }

        if let Some(groups) = RE.captures(s) {
            let id: usize = usize::from_str_radix(&groups["ID"], 16).unwrap_or_default();

            if id > 0 {
                //let slot = usize::from_str_radix(&groups["slot"], 16).unwrap();

                let attribs_str = &groups["attribs"][1..groups["attribs"].len()-1];
                let attribs : MessageAttributes = attribs_str.parse().unwrap_or_default();

                let mut text_parts = Vec::new();

                if let Some(str) = groups.name("str")
                {
                        let s = str.as_str().replace(r"\n", "\n");
                        let tags_it = RE_TAG.find_iter(&s).flat_map(|m| m.as_str().parse::<Tag>()).map(|t| TextPart::Tag(t));
                        let str_it = RE_TAG.split(&s).map(|s| TextPart::Text(s.to_string()));

                        text_parts = str_it.interleave(tags_it).collect::<Vec<_>>();
                }

                Ok(MessageSingleLang { text: text_parts, attribs, id })
            } else {
                Err("Invalid ID")
            }
        } else  {
            Err("Ill formed string")
        }
    }
}

pub struct BMGTextParser{
    messages : Vec<MessageSingleLang>,
    encoding : &'static encoding_rs::Encoding,
}

impl BMGTextParser {

    fn new(lines : impl Iterator<Item=std::string::String>) -> Self
    {
        let  mut iter = lines.skip_while(|l|  !RE_ENCODING.is_match(l) );

        let s = iter.next().unwrap();
        let iter = iter.skip_while(|l|  !RE.is_match(l) );

        let mut encoding = 0;

        if let Some(caps) = RE_TAG.captures(&s) {
            let val = caps.get(1).map_or("0", |m| m.as_str());

            encoding = val.parse::<u32>().unwrap_or_default();
        }

       let encoding = match encoding {
            1 => encoding_rs::WINDOWS_1252,
            2 => encoding_rs::UTF_16LE, // LE as the only cases we have now are LE, might need to generalise this
            3 => encoding_rs::SHIFT_JIS,
            _ => encoding_rs::WINDOWS_1252, // Default to WINDOWS_1252 if unknown
        };

        BMGTextParser {
            messages :  iter.flat_map(|l| l.parse()).collect(),
            encoding
        }
    }   
}

impl MessageParser for BMGTextParser {
    fn get_all_messages(&self) -> Vec<MessageSingleLang> {
       self.messages.clone()
    }

    fn get_encoding(&self) -> &'static encoding_rs::Encoding {
        encoding_rs::SHIFT_JIS
    }
}


pub fn open_bmg(filename: &Path) -> Result<BMGTextParser, io::Error> {

    let file = File::open(filename)?;
    let buf_reader = BufReader::new(file);

    Ok(BMGTextParser::new(buf_reader.lines().flatten()))
}
