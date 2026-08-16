use std::{cmp::min, env::temp_dir, fs::File, io::{self, Read}, ops::Range, path::Path, str::Utf8Error};

use thiserror::Error;

use crate::{message::{MessageAttributes, MessageId, MessageParser, MessageSingleLang, MessageText, Tag, TextPart}, utils};

#[derive(Error, Debug)]
pub enum MSBTParseError {
    #[error("Error Reading String values")]
    InvalidSectionID(#[from] Utf8Error),

    #[error("unknown data store error")]
    UnknownSectionID,

    #[error("block offset is outside file")]
    #[allow(dead_code)]
    OffsetOutOfBounds
}

#[derive(Default, Debug)]
struct LMSHeader {
    magic: String,
    big_endian : bool,
    _unknown : u16,
    encoding: u8,
    version : u8,
    blocks_cnt: u16,
    _unknown2 : u16,
    filesize: u32,
}

struct LBL1Data {
    label_groups:u32,
    labels : Vec<String>,
}

struct TXT2Data {
    count : u32,
    offsets : Vec<u32>,
    data : Vec<u8>
}


impl TXT2Data {

    fn get_msg(&self, idx:usize, encoding : &'static encoding_rs::Encoding) -> MessageText {
        
        if idx > self.offsets.len(){
            return Vec::new();
        }

        let offset = self.offsets[idx] as usize;
        if self.data[offset] == 0x00 && self.data[offset+1] == 0x00 {
            return Vec::new();
        }

        // TODO : generalise with bmg
    

        let mut it = self.data[offset..].iter();
        let mut end = false;
        let mut full_string = String::new();
        let mut text_parts : Vec<TextPart> = Vec::new();

        let big_endian = encoding == encoding_rs::UTF_16BE;
        let get_u16 = if big_endian { utils::get_u16_be } else {utils::get_u16_le};

        while !end {
            let mut stop_value = 0u16;
            let str_bytes = if encoding == encoding_rs::UTF_16LE || encoding == encoding_rs::UTF_16BE {
                
                // is easier to try to iterate properly by step of 2 bytes without iterator typing weirdness
                let mut str_end = false;
                let mut str = Vec::new();
                while !str_end {
                    let b1 = *it.next().unwrap();
                    let b2 = *it.next().unwrap();
                    let v = get_u16(&[b1,b2], 0);

                    if v != 0x00000 && v != 0x000E {
                        str.push(b1);
                        str.push(b2);
                    } else {
                        stop_value = v;
                        str_end = true;
                    }
                }
                str
            } else {
                it.by_ref().take_while(|&&b| { stop_value = b as u16; b!=0x00 && b!=0x0E }).map(|b| *b).collect::<Vec<_>>()
            };

            let str = encoding.decode(&str_bytes).0;

            full_string += &str;
            text_parts.push(TextPart::Text(str.to_string()));
            
            match stop_value {
                0x00 => end = true,
                0x0E => {
                    let mut read_u16 = || {get_u16(&it.by_ref().take(2).map(|b| *b).collect::<Vec<_>>(), 0)};
                    let group = read_u16() as u8;
                    let number = read_u16();
                    let params_size = read_u16();
                    let payload = it.by_ref().take(params_size as usize).map(|b| *b).collect::<Vec<_>>();

                    text_parts.push(TextPart::Tag(Tag{
                        group,number, payload
                    }));
                },
                _ => {}
            }

        }

        text_parts
    }
}

struct ATR1Data {
    attr_count : u32,
    attr_size : u32,
    attribs : Vec<Vec<u8>>,
    strings : Vec<Vec<u8>>
}

struct TSY1Data {
    // TODO : implement TSY1 parsing
}

enum MSBTBlockData {
    LBL1(LBL1Data),
    TXT2(TXT2Data),
    ATR1(ATR1Data),
    TSY1(TSY1Data),
}

struct MSBTBlock {
    block_type: String,
    size: u32,
    range: Range<usize>,
    data : MSBTBlockData
}

struct MSBTData {
    header : LMSHeader,
    blocks : [Option<MSBTBlock>; MSBTData::SECTION_COUNT],
}


impl MSBTData {
    const SECTION_COUNT : usize = 4;
    const LBL1 : usize = 0;
    const TXT2 : usize = 1;
    const ATR1 : usize = 2;
    const TSY1 : usize = 3;

    fn get_idx(type_str : &str) -> Option<usize> {
        match type_str {
            "LBL1" => Some(MSBTData::LBL1),
            "TXT2" => Some(MSBTData::TXT2),
            "ATR1" => Some(MSBTData::ATR1),
            "TSY1" => Some(MSBTData::TSY1),
            _ => None
        }
    }
}

pub struct MSBTParser {
    _data: Vec<u8>,
    data_parsed: MSBTData,
}

impl MSBTParser {
    fn new(data: Vec<u8>) -> Self {

        let parsed = match MSBTParser::parse_data(&data) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error parsing MSBT data: {}", e);
                MSBTData { header: Default::default(), blocks: [const {None}; MSBTData::SECTION_COUNT] }
            },
        };

        
       MSBTParser { _data : data, data_parsed: parsed}
    }

    fn parse_data(data: &[u8]) -> Result<MSBTData, MSBTParseError>{
        
        let header = MSBTParser::parse_header(data)?;

        let big_endian = header.big_endian;
        let get_u32 = if big_endian { utils::get_u32_be } else {utils::get_u32_le};

        let mut blocks = [const {None}; MSBTData::SECTION_COUNT];
        let mut offset = 0x20;

        for i in 0..header.blocks_cnt {
            if offset + 4 < data.len()
            {
                let section_type = str::from_utf8(&data[offset..offset + 4])?.to_string();
                let section_size = get_u32(&data, offset + 4);
    
                let range_start = offset + 0x10;
                let range_end = min(range_start + section_size as usize, data.len()); //size includes the header
                let range = range_start..range_end as usize;
                
                if let Some(idx) = MSBTData::get_idx(&section_type) {
                    blocks[idx] = Some(MSBTBlock {
                        block_type : section_type,
                        size : section_size,
                        range : range.clone(),
                        data: MSBTParser::parse_section(&data[offset..range_end], big_endian)?
                    });
                }
            
                let block_end = offset + 0x10 + section_size as usize; //0x10 is block header size
                offset = ((block_end + 15) / 16) * 16; //next 16-bytes aligned address
            } else {
                println!("Invalid offset for section {i}");
            }
        }

        Ok(MSBTData { header, blocks})
    }

    fn parse_header(data : &[u8]) -> Result<LMSHeader, MSBTParseError> {
        let big_endian = utils::get_u16_be(&data, 0x8) == 0xFEFF;

        let get_u32 = if big_endian { utils::get_u32_be } else {utils::get_u32_le};
        let get_u16 = if big_endian { utils::get_u16_be } else {utils::get_u16_le};

        Ok(LMSHeader {
            magic: str::from_utf8(&data[0..8])?.to_string(),
            big_endian : big_endian,
            _unknown : get_u16(data, 0xA),
            encoding : data[0xC],
            version : data[0xD],
            blocks_cnt : get_u16(data, 0xE),
            _unknown2 : get_u16(data, 0x10),
            filesize: get_u32(&data, 0x12),
        })
    }

    fn parse_section(data : &[u8], big_endian : bool) -> Result<MSBTBlockData, MSBTParseError> {

        let get_u32 = if big_endian { utils::get_u32_be } else {utils::get_u32_le};

        let section_type = str::from_utf8(&data[0..4])?;
        let section_size = get_u32(&data, 4);

        let range_start = 0x10;
        let range_end = min(range_start + section_size as usize, data.len()); //size includes the header
        let range = range_start..range_end as usize;

        let section_data = &data[range];

        match section_type {
            "LBL1" => {
                let label_groups = get_u32(section_data, 0x0);

                let mut a : Vec<_>= section_data[0x04..].chunks_exact(8).take(label_groups as usize).flat_map(|bucket| {
                    let label_count = get_u32(bucket, 0x0) as usize;
                    let mut offset = get_u32(bucket, 0x4) as usize;

                    let mut labels = Vec::new();
                    //labels.resize(label_count, String::new());
                
                    for i in 0..label_count {
                        let label_len = section_data[offset] as usize;
                        let label_range = (offset+1)..(offset+1+label_len);

                        let index = get_u32(section_data, label_range.end) as usize;
                        let label = str::from_utf8(&section_data[label_range.clone()])?.to_string();

                        labels.push((label, index));

                        offset = label_range.end + 4; //4 bytes for index
                    }

                    Ok::<Vec<(String, usize)>, MSBTParseError>(labels)
                }).flatten().collect();
              
                a.sort_by_key(|e| e.1);
                let labels = a.iter().map(|e| e.0.clone()).collect();
                
                Ok(MSBTBlockData::LBL1(LBL1Data {
                    label_groups,
                    labels
                }))
            },
            "TXT2" => {
                let count = get_u32(section_data, 0);
                let data_begin = 0x04 + count as usize * 4; 
                let offsets = section_data[0x04..].chunks_exact(4).take(count as usize).map(|offset| get_u32(offset, 0) - data_begin as u32).collect();


                let data = Vec::from(&section_data[data_begin..]);
                    
                Ok(MSBTBlockData::TXT2(TXT2Data { count, offsets, data }))
            },
            "ATR1" => {
                let attr_count = get_u32(section_data, 0);
                let attr_size = get_u32(section_data, 0x4);
                let total_size = attr_count as usize * attr_size as usize;
                let attribs = if attr_size > 0 {
                    Vec::from(&section_data[0x8..(0x8 + total_size)]).chunks_exact(attr_size as usize).map(|b| Vec::from(b)).collect()
                } else {
                    vec![vec![]; attr_count as usize]
                };


                let mut strings = Vec::new();
                let mut curr_str = Vec::new();
                for b in section_data[(0x08 + total_size)..].chunks_exact(2) {

                    if b[0] != 0x00 || b[1] != 0x00 {
                        curr_str.extend_from_slice(b);
                    } else {
                        strings.push(curr_str.clone());
                        curr_str.clear();
                    }
                }

                Ok(MSBTBlockData::ATR1(ATR1Data{
                    attr_count,attr_size, attribs, strings
                }))
            },
            "TSY1" => {
                // TODO : implement TSY1 parsing
                Ok(MSBTBlockData::TSY1(TSY1Data {}))
            },
            _ => Err(MSBTParseError::UnknownSectionID)
        }

    }


    pub fn get_msg(&self, idx : usize) -> MessageSingleLang {
        if let Some(MSBTBlockData::LBL1(lbl1)) = self.get_block(MSBTData::LBL1) {
            let label = &lbl1.labels[idx];

            if let Some(MSBTBlockData::TXT2(txt2)) = self.get_block(MSBTData::TXT2) {
                let text = txt2.get_msg(idx, self.get_encoding());

                let attribs = if let Some(MSBTBlockData::ATR1(atr1)) = self.get_block(MSBTData::ATR1) {
                    MessageAttributes{payload : atr1.attribs[idx].clone()}
                } else {
                    Default::default()
                };
                
                MessageSingleLang {
                    id : MessageId::Label(label.clone()),
                    attribs,
                    text,
                }
            } else {
                MessageSingleLang::default()
            }
        } else {
            MessageSingleLang::default()
        }
    }
    
    #[allow(dead_code)]
    fn get_header(&self) -> &LMSHeader {
        &self.data_parsed.header
    }

    #[allow(dead_code)]
    fn get_blocks(&self) -> &[Option<MSBTBlock>; MSBTData::SECTION_COUNT] {
        &self.data_parsed.blocks
    }

    fn get_block(&self, idx : usize) -> Option<&MSBTBlockData> {
        if let Some(section) = &self.data_parsed.blocks[idx] {
            Some(&section.data)
        } else {
            None
        }
    }

    fn print(&self) {
        let header = self.get_header();
        println!("Magic: {}", header.magic);
        println!("Endian: {}", header.big_endian);
        println!("Version: {}", header.version);
        println!("Filesize: {}", header.filesize);
        println!("Blocks count: {}", header.blocks_cnt);
        println!("Encoding: {}", header.encoding);

        let encoding = self.get_encoding();


        for section in self.get_blocks().iter().flatten() {
            println!("Section type: {}", section.block_type);
            println!("Section size: {}", section.size);
            println!("Section data range: {:X?}", section.range);

            match &section.data {
                MSBTBlockData::LBL1(lbl1_data) => {
                    for l in &lbl1_data.labels {
                        println!("\t{}", l);
                    }
                },
                MSBTBlockData::TXT2(txt2_data) => {
                    println!("\tcount : {}", txt2_data.count);

                    for i in 0..3 {
                        println!("\t offset {} : {:X}", i, txt2_data.offsets[i]);
                    }
                },
                MSBTBlockData::ATR1(atr1_data) => {
                    println!("\tnumber of attribs {}", atr1_data.attr_count);
                    println!("\tattribs size {}", atr1_data.attr_size);

                     for i in 0..5 {
                        println!("\t string {i} : {}",encoding.decode(&atr1_data.strings[i]).0.to_string());
                    }
                },
                MSBTBlockData::TSY1(_) => {
                    println!("\tEmpty TSY1 section");
                }
            }
        }
        
    }
}

impl MessageParser for MSBTParser {
    fn get_all_messages(&self) -> Vec<crate::message::MessageSingleLang> {
        if let Some(MSBTBlockData::LBL1(lbl1)) = self.get_block(MSBTData::LBL1) {
            (0..lbl1.labels.len()).map(|i| {
                self.get_msg(i as usize)
            }).collect()
       } else {
        Vec::new()
       }
    }

    fn get_encoding(&self) -> &'static encoding_rs::Encoding {
        match self.get_header().encoding {
            0 => encoding_rs::UTF_8,
            1 => if self.get_header().big_endian { encoding_rs::UTF_16BE } else {encoding_rs::UTF_16LE}, // LE as the only cases we have now are LE, might need to generalise this
            // 2 => encoding_rs::UTF_32; // does not seem to exists :'(
            _ => encoding_rs::UTF_8, // Default to WINDOWS_1252 if unknown
        }
    }
}


pub fn open_msbt(filename: &Path) -> io::Result<MSBTParser> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(MSBTParser::new(buffer))
}


#[allow(dead_code)]
pub fn print_msbt(path : &Path) {
    match open_msbt(path) {
        Ok(parser) => {
            parser.print();
            // parser.print_flow();
            for i in 0..3 {
                println!("Message {i} : {:?}", parser.get_msg(i).text);//parser.get_msg(0x66).text));
            }
        }
        Err(e) => {
            eprintln!("Error opening BMG file: {}", e);
        }
    }
}

