use std::{
    cmp::min, fmt::{self}, fs::File, io::{self, Read}, ops::Range, path::Path, str::Utf8Error,
};

use thiserror::Error;

use crate::{message::{self, MessageAttributes, MessageParser, MessageSingleLang, MessageText, Tag, TextPart}, utils::{self}};


#[derive(Error, Debug)]
pub enum BMGParseError {
    #[error("Error Reading String values")]
    InvalidSectionID(#[from] Utf8Error),

    #[error("unknown data store error")]
    #[allow(dead_code)]
    UnknownSectionID,
}

#[derive(Clone)]
#[allow(dead_code)]
struct BMGHeader {
    magic: String,
    filesize: u32,
    sections_cnt: u32,
    encoding: u8,
}


struct INF1Entry {
    offset: u32,
    attributes: Vec<u8>,
}

impl fmt::Display for INF1Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Offset: {}, Attributes: {:X?}", self.offset, self.attributes)
    }
}

#[allow(dead_code)]
struct INF1Data {
    count : u16,
    entry_size : u16,
    _pad : u32,
    entries : Vec<INF1Entry>,
}

struct DAT1Data {
    data : Vec<u8>,
}

impl DAT1Data {
    
    fn get_msg_at(&self,offset:usize, encoding : u8) -> MessageText {

        if self.data[offset] == 0x00 {
            return Vec::new();
        }
        
        let encoding = match encoding {
            1 => encoding_rs::WINDOWS_1252,
            2 => encoding_rs::UTF_16LE, // LE as the only cases we have now are LE, might need to generalise this
            3 => encoding_rs::SHIFT_JIS,
            _ => encoding_rs::WINDOWS_1252, // Default to WINDOWS_1252 if unknown
        };

        let mut it = self.data[offset..].iter();
        let mut end = false;
        let mut full_string = String::new();
        let mut text_parts : Vec<TextPart> = Vec::new();


        while !end {
            let mut stop_value = 0u16;
            let str_bytes = if encoding == encoding_rs::UTF_16LE {
                
                // is easier to try to iterate properly by step of 2 bytes without iterator typing weirdness
                let mut str_end = false;
                let mut str = Vec::new();
                while !str_end {
                    let b1 = *it.next().unwrap();
                    let b2 = *it.next().unwrap();
                    let v = utils::get_u16_le(&[b1,b2], 0);

                    if v != 0x00000 && v != 0x001A {
                        str.push(b1);
                        str.push(b2);
                    } else {
                        stop_value = v;
                        str_end = true;
                    }
                }
                str
            } else {
                it.by_ref().take_while(|&&b| { stop_value = b as u16; b!=0x00 && b!=0x1A }).map(|b| *b).collect::<Vec<_>>()
            };

            let str = encoding.decode(&str_bytes).0;

            full_string += &str;
            text_parts.push(TextPart::Text(str.to_string()));
            
            match stop_value {
                0x00 => end = true,
                0x1A => {
                    let size_bytes = it.next().unwrap_or(&0x00);
                    let payload_size = size_bytes - if encoding == encoding_rs::UTF_16LE { 3 } else {2};
                    let text_tag = it.by_ref().take(payload_size as usize).map(|b| *b).collect::<Vec<_>>(); //Accounting for the 0x1A and size byte
                    text_parts.push(TextPart::Tag(Tag::from(&text_tag)));
                },
                _ => {}
            }
        }

        text_parts
        
    }
}

#[allow(dead_code)]
struct MID1Data {
    count : u16,
    _format : u16,
    _unknown : u32,
    ids : Vec<u32>
}


#[derive(Debug)]

#[allow(dead_code)]
enum FLW1Node {
    Continuation {id : u8, doorquery : u8, inf1_idx : u16, next_node : u16, _pad : u16},
    Branch {id : u8, doorquery : u8, query_fn : u16, param : u16, indir_offset : u16},
    Event {id : u8, event_fn : u8, indir_idx : u16, params : u32}
}

#[derive(Debug)]
#[allow(dead_code)]
struct FLW1Data {
    node_count : u16,
    ind_count : u16,
    _pad : u32,

    nodes : Vec<FLW1Node>,
    ind_table : Vec<u16>

}

#[derive(Debug)]
#[allow(dead_code)]
struct FLI1Entry {
    id : u16,
    _pad : u16,
    node_idx : u16,
    _pad2 : u16
}

#[derive(Debug)]
#[allow(dead_code)]
struct FLI1Data {
    count : u16,
    entries : Vec<FLI1Entry>,
}

#[allow(dead_code)]
enum BMGSectionData {
    INF1(INF1Data),
    DAT1(DAT1Data),
    MID1(MID1Data),
    FLW1(FLW1Data),
    FLI1(FLI1Data)
}

#[allow(dead_code)]
struct BMGSection {
    section_type: String,
    size: u32,
    range: Range<usize>,
    data: BMGSectionData
}

struct BMGData {
    header: BMGHeader,
    sections: [Option<BMGSection>; BMGData::SECTION_COUNT],
}

impl BMGData {
    const SECTION_COUNT : usize = 6;
    const INF1 : usize = 0;
    const DAT1 : usize = 1;
    const MID1 : usize = 2;
    const STR1 : usize = 3;
    const FLW1 : usize = 4;
    const FLI1 : usize = 5;

    fn get_idx(type_str : &str) -> Option<usize> {
        match type_str {
            "INF1" => Some(BMGData::INF1),
            "DAT1" => Some(BMGData::DAT1),
            "MID1" => Some(BMGData::MID1),
            "STR1" => Some(BMGData::STR1),
            "FLW1" => Some(BMGData::FLW1),
            "FLI1" => Some(BMGData::FLI1),
            _ => None
        }
    }
}

pub struct BMGRawParser {
    _data: Vec<u8>,
    data_parsed: BMGData,
}


impl BMGRawParser {
    fn new(data: Vec<u8>, big_endian : bool) -> Self {

        let parsed = BMGRawParser::parse_data(&data, big_endian).unwrap_or( BMGData {
            header: BMGHeader {
                magic: String::new(),
                filesize: 0,
                sections_cnt: 0,
                encoding: 0,
            },
            sections: [const { None } ;6],
        });

        
       BMGRawParser { _data : data, data_parsed: parsed}
    }


    #[allow(dead_code)]
    fn get_header(&self) -> &BMGHeader {
        &self.data_parsed.header
    }

    #[allow(dead_code)]
    fn get_sections(&self) -> &[Option<BMGSection>; BMGData::SECTION_COUNT] {
        &self.data_parsed.sections
    }

    fn get_section(&self, idx : usize) -> Option<&BMGSectionData> {
        if let Some(section) = &self.data_parsed.sections[idx] {
            Some(&section.data)
        } else {
            None
        }
    }

    fn print(&self) {
        let header = self.get_header();
        println!("Magic: {}", header.magic);
        println!("Filesize: {}", header.filesize);
        println!("Sections count: {}", header.sections_cnt);
        println!("Encoding: {}", header.encoding);

        for section in self.get_sections().iter().flatten() {
            println!("Section type: {}", section.section_type);
            println!("Section size: {}", section.size);
            println!("Section data range: {:X?}", section.range);

            match &section.data {
                BMGSectionData::INF1(INF1Data{count,entry_size, _pad, entries})=> {
                    println!("\t INF1 Section: count={}, entry_size={}, entry_len={}", count, entry_size, entries.len());
                    println!("\t Entry 0: {}", entries[0]);
                    println!("\t Entry 1: {}", entries[1]);
                    println!("\t Entry 2: {}", entries[2]);
                },
                BMGSectionData::DAT1(dat1data) => {
                    println!("\t DAT1 Section: data length={}", dat1data.data.len());
                    println!("\t First 10 bytes: {:X?}", &dat1data.data[0..10]);
                    if let Some(BMGSectionData::INF1(inf1)) = self.get_section(BMGData::INF1) {
                        println!("\t First message: {:#?}", dat1data.get_msg_at(inf1.entries[0].offset as usize, header.encoding));
                        println!("\t Second message: {:#?}", dat1data.get_msg_at(inf1.entries[1].offset as usize, header.encoding));
                        println!("\t Third message: {:#?}", dat1data.get_msg_at(inf1.entries[2].offset as usize, header.encoding));
                    }
                },
                BMGSectionData::MID1(mid1data) => {
                    println!("\t MID1 Section: count {}", mid1data.count);
                    println!("\t Length {}", mid1data.count);
                    println!("\t ID[0] : {}", mid1data.ids[0]);
                    println!("\t ID[1] : {}", mid1data.ids[1]);
                    println!("\t ID[2] : {}", mid1data.ids[2]);
                },
                BMGSectionData::FLW1(flw1data) => {
                    println!("\t FLW1 Section: Node count {}", flw1data.node_count);
                    println!("\t FLW1 Section: Indirection count {}", flw1data.ind_count);
                    println!("\t FLW1 Section: Nodes[0] {:X?}", flw1data.nodes[0]);
                    println!("\t FLW1 Section: Nodes[1] {:X?}", flw1data.nodes[1]);
                    println!("\t FLW1 Section: Nodes[2] {:X?}", flw1data.nodes[2]);
                    println!("\t FLW1 Section: Indir[0] {:X?}", flw1data.ind_table[0]);
                    println!("\t FLW1 Section: Indir[1] {:X?}", flw1data.ind_table[1]);
                    println!("\t FLW1 Section: Indir[2] {:X?}", flw1data.ind_table[2]);

                },
                BMGSectionData::FLI1(fli1data) => {
                    println!("\t FLI1 Section: Node count {}", fli1data.count);
                    println!("\t FLI1 Section: Nodes[0] {:X?}", fli1data.entries[0]);
                    println!("\t FLI1 Section: Nodes[1] {:X?}", fli1data.entries[1]);
                    println!("\t FLI1 Section: Nodes[2] {:X?}", fli1data.entries[2]);
                },
            }
        }
    }


    fn parse_header(data : &[u8], big_endian : bool) -> Result<BMGHeader, BMGParseError> {
        let get_u32 = if big_endian { utils::get_u32_be } else {utils::get_u32_le};

        Ok(BMGHeader {
            magic: str::from_utf8(&data[0..8])?.to_string(),
            filesize: get_u32(&data, 8),
            sections_cnt: get_u32(&data, 0x0C),
            encoding: data[0x10],
        })
    }

    fn parse_section(data: &[u8], offset: usize, big_endian : bool) -> Result<BMGSectionData, BMGParseError> {

        let get_u32 = if big_endian { utils::get_u32_be } else {utils::get_u32_le};
        let get_u16 = if big_endian { utils::get_u16_be } else {utils::get_u16_le};

        let section_type = str::from_utf8(&data[offset..offset + 4])?;
        let section_size = get_u32(&data, offset + 4);

        let range_start = offset + 8;
        let range_end = min(offset + section_size as usize, data.len()); //size includes the header
        let range = range_start..range_end as usize;


        let section_data = &data[range];

        match section_type {
            "INF1" => {
                let count = get_u16(&section_data, 0x00);
                let entry_size = get_u16(&section_data,  0x02);
                let pad = get_u32(&section_data, 0x04);

                let entries = section_data[0x08..].chunks_exact(entry_size as usize).map(|entry| {
                    let entry_offset_value = get_u32(&entry, 0x00);
                    let attributes = entry[0x04..].to_vec();

                    INF1Entry{offset:entry_offset_value, attributes}
                }).take(count as usize).collect();

                Ok(BMGSectionData::INF1(INF1Data {
                    count,
                    entry_size,
                    _pad : pad,
                    entries,
                }))
            }
            "DAT1" => {
                Ok(BMGSectionData::DAT1(DAT1Data { data: section_data.to_vec()}))
            },
            "MID1" => {
                let count = get_u16(&section_data, 0x00);
                let format = get_u16(&section_data, 0x02);
                let unknown  = get_u32(&section_data, 0x04);

                let ids : Vec<_> = section_data[0x08..].chunks_exact(4).map(|v| get_u32(v, 0)).collect();


                Ok(BMGSectionData::MID1(MID1Data { count, _format : format, _unknown: unknown, ids: ids}))
            }
            "FLW1" => {
                let count = get_u16(&section_data, 0x00);
                let indir_count = get_u16(&section_data, 0x02);
                let pad = get_u32(&section_data, 0x04);

                let nodes = section_data[0x08..].chunks_exact(8).filter_map(|bytes| 
                    match bytes[0x00] {
                        0x01 => Some(FLW1Node::Continuation { id: bytes[0x00], doorquery: bytes[0x01], inf1_idx: get_u16(&bytes, 0x02), next_node: get_u16(&bytes, 0x04), _pad: get_u16(&bytes, 0x06) }),
                        0x02 => Some(FLW1Node::Branch { id: bytes[0x00], doorquery: bytes[0x01], query_fn: get_u16(&bytes, 0x02), param: get_u16(&bytes, 0x04), indir_offset: get_u16(&bytes, 0x06) }),
                        0x03 => Some(FLW1Node::Event { id: bytes[0x00], event_fn: bytes[0x01], indir_idx: get_u16(&bytes, 0x02), params: get_u32(&bytes, 0x04)}),
                        _ => None
                    }
                ).take(count as usize);

                let indir_start_offset = (0x08 + count * 8) as usize;
                let indir_table = section_data[indir_start_offset..].chunks_exact(2).map(|v| get_u16(v, 0)).take(indir_count as usize);
                Ok(BMGSectionData::FLW1(FLW1Data { node_count:count , ind_count: indir_count, _pad: pad, nodes: nodes.collect(), ind_table: indir_table.collect() }))
            },
            "FLI1" => {
                 let count = get_u16(&section_data, 0x00);

                 let indices = section_data[0x08..].chunks_exact(8).map(|bytes|
                    FLI1Entry {id : get_u16(&bytes,0x00), _pad : get_u16(&bytes, 0x02), node_idx : get_u16(&bytes, 0x04), _pad2 : get_u16(&bytes, 0x06)}
                    ).take(count as usize);
                Ok(BMGSectionData::FLI1(FLI1Data { count: count, entries: indices.collect() }))
            }
            _ => Err(BMGParseError::UnknownSectionID)

        }


    }

    fn parse_data(data: &[u8], big_endian : bool) -> Result<BMGData, BMGParseError> {
        let header = BMGRawParser::parse_header(&data, big_endian)?;
        let mut sections = [const {None}; BMGData::SECTION_COUNT];
        let mut offset = 0x20;

        let get_u32 = if big_endian { utils::get_u32_be } else {utils::get_u32_le};

        println!("Number of sections : {}", header.sections_cnt);
        for i in 0..header.sections_cnt {
            if offset + 4 < data.len()
            {
                let section_type = str::from_utf8(&data[offset..offset + 4])?.to_string();
                let section_size = get_u32(&data, offset + 4);
    
                let range_start = offset + 8;
                let range_end = min(offset + section_size as usize, data.len()); //size includes the header
                let range = range_start..range_end as usize;
                
                if let Some(idx) = BMGData::get_idx(&section_type) {
                    sections[idx] = Some(BMGSection {
                        section_type : section_type,
                        size : section_size,
                        range : range,
                        data: BMGRawParser::parse_section(data, offset, big_endian)?
                    });
                }
            
                offset += section_size as usize;
            } else {
                println!("Invalid offset for section {i}");
            }
        }

        Ok(BMGData { header, sections })
    }


    pub fn get_msg(&self, idx : usize) -> MessageSingleLang {
        if let Some(BMGSectionData::INF1(inf1)) = self.get_section(BMGData::INF1) {
            if let Some(BMGSectionData::DAT1(dat1)) = self.get_section(BMGData::DAT1) {
                let inf1_entry = &inf1.entries[idx];
                let data = dat1.get_msg_at(inf1_entry.offset as usize, self.data_parsed.header.encoding);

                
                //let attribs = .clone();
                let attribs = MessageAttributes{payload : inf1_entry.attributes.clone()};

                let id = if let Some(BMGSectionData::MID1(mid1)) = self.get_section(BMGData::MID1) {
                    mid1.ids[idx] as usize
                } else { 
                    match attribs.get_message_id() {
                        Some(id) => id as usize,
                        None => idx +1
                    }
                 };

                MessageSingleLang {
                    id : id,
                    attribs : attribs,
                    text : data
                }
            } else {
                MessageSingleLang::default()
            }
        } else {
            MessageSingleLang::default()
        }
    }

    fn print_flow_chain(&self,flow_node_idx : usize, visited: &mut Vec<bool>) {
        if let Some(BMGSectionData::FLW1(flw1)) = self.get_section(BMGData::FLW1) {
            visited[flow_node_idx] = true;

            match &flw1.nodes[flow_node_idx] {
                FLW1Node::Continuation { id : _, doorquery : _, inf1_idx, next_node, _pad } =>  {
                    let m = self.get_msg(*inf1_idx as usize);
                    println!("INF1 {:X} : {}",inf1_idx,  message::get_raw_msg(&m.text, None));
    
                    if *next_node != 0xFFFF {
                        self.print_flow_chain(*next_node as usize, visited);
                    }
                }
                FLW1Node::Branch { id:_, doorquery:_, query_fn:_, param:_, indir_offset } => {
                    println!("Branch");

                    let base_next_node = &flw1.ind_table[*indir_offset as usize];
                    if *base_next_node != 0xFFFF {
                        self.print_flow_chain(*base_next_node as usize, visited);
                    }

                },
                FLW1Node::Event { id : _, event_fn, indir_idx, params: _ } => {
                    println!("Running Event {}", event_fn);

                    let next_node = &flw1.ind_table[*indir_idx as usize];
                    if *next_node != 0xFFFF {
                        self.print_flow_chain(*next_node as usize, visited);
                    }
                    
                },
            }
        }
    }

    fn print_flow(&self) {
        if let Some(BMGSectionData::FLW1(flw1)) = self.get_section(BMGData::FLW1) {

            let mut visited = vec![false; flw1.node_count as usize];
            while let Some(idx) = (0..flw1.node_count).find(|node| !visited[*node as usize] && matches!(flw1.nodes[*node as usize] ,FLW1Node::Continuation { .. })) {
                println!("------------");
                self.print_flow_chain(idx as usize, &mut visited);
            }


        }
    }
}

impl MessageParser for BMGRawParser {
    fn get_all_messages(&self) -> Vec<MessageSingleLang> {
       if let Some(BMGSectionData::INF1(inf1)) = self.get_section(BMGData::INF1) {
            (0..inf1.count).map(|i| {
                self.get_msg(i as usize)
            }).collect()
       } else {
        Vec::new()
       }
    }

    fn get_encoding(&self) -> &'static encoding_rs::Encoding {
        match self.get_header().encoding {
            1 => encoding_rs::WINDOWS_1252,
            2 => encoding_rs::UTF_16LE, // LE as the only cases we have now are LE, might need to generalise this
            3 => encoding_rs::SHIFT_JIS,
            _ => encoding_rs::WINDOWS_1252, // Default to WINDOWS_1252 if unknown
        }
    }
}

pub fn open_bmg(filename: &Path, big_endian : bool) -> Result<BMGRawParser, io::Error> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(BMGRawParser::new(buffer, big_endian))
}


#[allow(dead_code)]
pub fn print_bmg(path : &Path) {
    match open_bmg(path, false) {
        Ok(parser) => {
            parser.print();
            // parser.print_flow();
            // println!("Message 0x66 : {}", get_raw_msg(parser.get_msg(0x66).text));
        }
        Err(e) => {
            eprintln!("Error opening BMG file: {}", e);
        }
    }
}
