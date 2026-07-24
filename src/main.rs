use std::{fmt, fs::File, io::{self, Write}, path::Path};
use rust_xlsxwriter::{Color, Format, FormatAlign};

mod message;

mod bmg_raw_parser;
mod bmg_text_parser;
mod msbt_parser;

mod utils;
mod game_configs;

use message::{Message, Tag, TextPart};

use crate::{message::{MessageParser, MessageSingleLang}, game_configs::{GameConfig, StyleTagType, TagType}};


const BANK_COUNT : usize = 32;

const DEFAULT_COLORS : [&str; 9] = [
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


impl Tag {
    
    fn get_simple_replacement(&self, config: Option<&GameConfig>) -> String {
        match config {
            Some(conf) => (conf.get_tag_replacement)(&self),
            None => String::from("[Tag]")
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}", match self.group {
            0xFF => {
                match self.number {
                    0x00 => "[Color]",
                    0x01 => "[Size]",
                    0x02 => "[Ruby]",
                    _ => "",
                }
            },
            _ => "[Tag]"
        })
    }
}

impl Message {
    fn get_html_formatted(&self, lang_id : usize, ignore_tags : bool, config : Option<&GameConfig>) -> String {
        
        if ignore_tags {
            self.get_raw(lang_id, config).replace("\n", "<br>")
            //RE_TAG.replace_all(&s, |c : &Captures| c[0].parse::<Tag>().unwrap_or_default().get_simple_replacement().to_owned()).to_string()
        } else {

            let mut res_str = String::new();

            let mut current_color = "#ffffff";
            let mut current_size = 100;

            let mut needs_ruby : Option<(u8, String)> = None;

            if self.text.len() > lang_id {
                for part in &self.text[lang_id] {
                    match part {
                        TextPart::Text(text) => {
                            if text != "" {
                                let text = text.replace("\n", "<br>");
                                if let Some((over_count, ruby_text)) = needs_ruby {
                                    let mut chars = text.chars();
                                    let base_text : String = chars.by_ref().take(over_count as usize).collect();
                                    let remaining_text : String = chars.collect();
                                    res_str += &format!("<ruby>{}<rp>(</rp><rt>{}</rt><rp>)</rp></ruby>{}", base_text, ruby_text, remaining_text);
                                    needs_ruby = None;
                                } else {
                                    res_str += &text;
                                }
                            }
                        },
                        TextPart::Tag(tag) => {
                            let get_tag_type = config.map(|c| c.get_tag_type).unwrap_or(game_configs::get_tag_type_default);
                            match get_tag_type(&tag) {
                                TagType::Style(style_type) => {
                                    match style_type {
                                        StyleTagType::Color(id) => { // change color
                                            if current_color != "#ffffff" {
                                                res_str += "</span>";
                                            }

                                            let new_color = id as usize;
                                            let c = if let Some(conf) = config { (conf.get_color_hex)(new_color)} else { "#ffffff"};
                                            if c != "#ffffff" {
                                                res_str += &format!("<span style='color:{};'>", c);
                                            }
                                            current_color = c;
                                        },
                                        StyleTagType::Size(percent) => {
                                            let new_size = percent;// get_u16(&tag.payload, 0);
                                            if current_size != 100 {
                                                res_str += "</span>"
                                            }
                                            if new_size != 100 {
                                                res_str += &format!("<span style='font-size:{}%;'>", new_size);
                                            }
                    
                                            current_size = new_size;
                                        },
                                        StyleTagType::Ruby(over_count, decoded_ruby) => {
                                            // let over_count = tag.payload[0];
                                            // let last_is_zero = tag.payload[tag.payload.len() -1] == 0x00;
                                            // let slice_end = tag.payload.len() - (last_is_zero as usize);
                                            // let raw_bytes = &tag.payload[1..slice_end];

                                            // let encoding = config.map(|c| 
                                            //     match c.id  {
                                            //        "ph" | "st" => encoding_rs::UTF_16LE,
                                            //        _ => encoding_rs::SHIFT_JIS, 
                                            //     }).unwrap_or(encoding_rs::SHIFT_JIS);



                                            // let decoded_ruby = encoding.decode(&raw_bytes).0;
                                            needs_ruby = Some((over_count, decoded_ruby));
                                        },
                                        _ => {}
                                    }
                                }
                                TagType::Replace => { res_str += &tag.get_simple_replacement(config); }
                            }
                        }
                    }                
                }
            }

            res_str
        }
        
    }

    fn get_xlsx_formatted(&self, lang_id : usize, ignore_tags : bool, default_color : Color, config : Option<&GameConfig> ) -> Vec<(Format, String)> {
        let mut segments : Vec<(Format, String)> = Vec::new();

        if ignore_tags {
            segments.push((Format::new(), self.get_raw(lang_id, config)));
        } else {
            
            let mut current_color = "#ffffff";
            let mut current_size = 100;

            const DEFAULT_SIZE : f32 = 11.0;

            if self.text.len() > lang_id {

                for part in &self.text[lang_id] {
                    match part {
                        TextPart::Text(text) => {
                            if !text.is_empty() {
                                // let config_color = if let Some(conf) = config { (conf.get_color_hex)(current_color)} else { DEFAULT_COLORS[current_color]};
                                let color = if current_color == "#ffffff" { default_color } else { Color::from(current_color)};
                                let size = DEFAULT_SIZE * (current_size as f32/100.0);
                                let format = Format::new().set_font_color(color).set_font_size(size);
                                segments.push((format, text.to_string()));
                            }
                        },
                        TextPart::Tag(tag) => {
                            let get_tag_type = config.map(|c| c.get_tag_type).unwrap_or(game_configs::get_tag_type_default);
                            match get_tag_type(&tag) {
                                TagType::Style(style_type) => {
                                    match style_type {
                                        StyleTagType::Color(id) => { // change color
                                            let c = if let Some(conf) = config { (conf.get_color_hex)(id as usize)} else { "#ffffff"};
                                            current_color = c
                                        },
                                        StyleTagType::Size(percent) => {                          
                                            current_size = percent;

                                        },
                                        StyleTagType::Ruby(_,_) => {
                                            
                                        },
                                        _ => {}
                                    }
                                }
                                TagType::Replace => { 
                                    let s = tag.get_simple_replacement(config).to_string();
                                    if !s.is_empty() {
                                        let color = if current_color == "#ffffff" { default_color } else { Color::from(current_color)};
                                        let size = DEFAULT_SIZE * (current_size as f32/100.0);
                                        let format = Format::new().set_font_color(color).set_font_size(size);
    
                                        segments.push((format, s)); 
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
        
        segments
    }

    fn get_raw(&self, lang_id : usize, config : Option<&GameConfig>) -> String {
        if self.text.len() > lang_id {
            message::get_raw_msg(&self.text[lang_id], config)
        } else {
            String::new()
        }
    }

}


#[derive(Default, Debug)]
struct BMGParser {
    msgs : [Vec<Message>; BANK_COUNT],
    encoding : Option<&'static encoding_rs::Encoding>,
}

impl BMGParser {
    
    #[allow(dead_code)]
    fn print(self) {
        for (bank_id,msg_bank) in self.msgs.iter().enumerate() {
            for (idx, msg) in msg_bank.iter().filter(|msg| !msg.is_empty()).enumerate() {
                println!("{} : {:#x} : {}", bank_id, idx, msg);
            }
        }
    }
}

trait Exporter {
    fn new(filepath: &Path) -> Self;
    fn set_config(&mut self, config:&GameConfig);
    fn begin(&mut self);
    fn set_headers(&mut self);
    fn add_row(&mut self , msg : &Message, ignore_tags : bool);
    fn end(&mut self);
}

struct HTMLExporter {
    file: Option<File>,
    config : Option<GameConfig>
}

impl Exporter for HTMLExporter  {
    fn new(filepath: &Path) -> Self {
        if let Ok(f) = File::create(filepath) {
            HTMLExporter { file: Some(f), config :None }
        } else {
            println!("Can't open {}", filepath.display());
            HTMLExporter {file: None, config : None}
        }
    }

    fn set_config(&mut self, config : &GameConfig) {
        self.config = Some(config.clone());
    }

    fn begin(&mut self) {
        if let Some(f) = &mut self.file {

        let font =  match &self.config {
            Some(conf) => match conf.id {
                "tp" => "fot-rodin-prondb",
                "tww" | "ph" | "st" | "fsa" => "rock",
                _ => "fot-rodin-prondb"
            }
            None => "fot-rodin-prondb"
            
        };

        let ruby_font =  match &self.config {
            Some(conf) => match conf.id {
                "tp" => "reishotai",
                "tww" | "ph" | "st" | "fsa"  => "fot-rodin-prondb",
                _ => "fot-rodin-prondb"
            }
            None => "fot-rodin-prondb"
        };

        let id = self.config.as_ref().map(|c| c.id).unwrap_or_default();

        let logo_url = self.config.as_ref().map(|conf| conf.logo).unwrap_or("https://upload.wikimedia.org/wikipedia/commons/thumb/2/2a/Zelda_Logo.svg/1280px-Zelda_Logo.svg.png");
        let _ = f.write(format!("<!DOCTYPE html>
<html>
<head>
<style>
    @font-face {{
        font-family: 'fot-rodin-prondb';
        src: url(\"assets/FOT-RodinProN-DB.otf\");
        font-weight: normal;
        font-style: normal;
    }}

    @font-face {{
        font-family: 'reishotai';
        src: url(\"assets/Reishotai.otf\");
        font-weight: normal;
        font-style: normal;
        size-adjust: 120%;
    }}

    @font-face {{
        font-family: 'rock';
        src: url(\"assets/RocknRollOne-Regular.ttf\");
        font-weight: 400;
        font-style: normal;
    }}

    body rt {{
        color : white;
        font-family: '{ruby_font}', 'ＭＳ 明朝', serif;
    }}

    body.nofuri rt {{
        display:none;
    }}


    header {{
            text-align:center;
        }}

  table {{
    table-layout: fixed;
    width: 100%;
    overflow:auto;
    font-family: '{font}';
    
}}
td {{
    border: 1px solid white;
    border-radius: 10px;
    padding:1em;
    }}

thead tr {{
    color:black; 
    background-color: initial;
}}

tr {{
    color: white;
    height: 48px;
    background: rgb(0 0 0 / 90%);
}}


nav {{
    position: sticky;
    background: white;
    top: 0;
}}

nav a:link,  nav a:visited {{
  background-color: #0d6efd;
  color: white;
  padding: 14px 25px;
  text-align: center;
  text-decoration: none;
  display: inline-block;
  border-radius: 10px;
    padding:1em;
}}

nav a:hover, nav a:active {{
  background-color: #0b5ed7;
}}
</style>
<link href=\"styles/{id}.css\" rel=\"stylesheet\" />
</head>
<body>
<header>
  <img src=\"{logo_url}\"/>
</header>
<nav id=\"options\">
    <input id=\"hide-furi\" type=\"checkbox\" name=\"HideFuri\" />
    <label for=\"HideFuri\">Hide Japanese Furigana</label>
    <a href=\"download/{id}.csv\">Download CSV</a>
    <a href=\"download/{id}.xlsx\">Download Excel</a>
</nav>
<table>").as_bytes());
        }
        
    }

    fn set_headers(&mut self) {
        if let Some(f) = &mut self.file {
            let mut s = "<thead>
    <tr>".to_string();

        let languages = if let Some(conf) = &self.config {(conf.get_languages)()} else { &[] };
        for lang in languages {
            s +=  &format!("<th>{}</th>", lang.1);
        }

      s += "
    </tr>
  </thead>
  <tbody>";

          let _ = f.write(s.as_bytes());
        }
    }

    fn add_row(&mut self, msg : &Message, ignore_tags : bool) {
        if let Some(f) = &mut self.file {
            let style_info = self.config.as_ref().map(|c| (c.get_message_style)(&msg.attribs)).unwrap_or_default();

            let mut style_str = String::new();
            if style_info.centered { style_str += "text-align: center; "};
            if style_info.alt_font { style_str += "font-family: \"reishotai\", \"ＭＳ 明朝\", serif;"}
            if !style_info.color.is_empty() { style_str += &format!("color:{};", style_info.color)}
            if !style_info.bg_color.is_empty() { style_str += &format!("background-color:{};", style_info.bg_color)}

            if !style_str.is_empty() { style_str = format!("style='{style_str}'")}

            let class_str = if !style_info.style_id.is_empty() { format!(" class=\"{}\"", style_info.style_id )} else {String::new()};

            let mut s  = format!("<tr {style_str}{class_str}>");
    
            let lang_count =  if let Some(config) = &self.config { (config.get_languages)().len()} else {0};
            for i in 0..lang_count {
                s += &format!("<td>{}</td>\n", msg.get_html_formatted(i, ignore_tags, self.config.as_ref()));
            }
    
            s += "</tr>";
    
            let _ = f.write(s.as_bytes());
        }
    }

    fn end(&mut self) {
        if let Some(f) = &mut self.file {
            let _ = f.write(b"</tbody>
</table>
<script> 

const nofuriCheckbox = document.querySelector('#hide-furi');
    nofuriCheckbox.addEventListener('change', () => {
    document.querySelector('body').classList.toggle('nofuri', nofuriCheckbox.checked );
});

</script>
</body>
</html>");
        }
    }
    
}

struct CSVExporter {
    file: Option<File>,
    config : Option<GameConfig>
}


impl Exporter for CSVExporter {
    fn new(filepath: &Path) -> Self {
        if let Ok(f) = File::create(filepath) {
            CSVExporter { file: Some(f), config : None }
        } else {
            println!("Can't open {}", filepath.display());
            CSVExporter {file: None, config: None}
        }
    }

    fn set_config(&mut self, config : &GameConfig) {
        self.config = Some(config.clone());
    }

    fn begin(&mut self) {
       
    }

    fn set_headers(&mut self) {
        if let Some(f) = &mut self.file {
            let mut s = "".to_string();

            let languages = if let Some(conf) = &self.config {(conf.get_languages)()} else { &[] };
            for lang in languages {
                s +=  &format!("{};", lang.1);
            }
            s += "\n";
            let _ = f.write(s.as_bytes());
        }

    }

    fn add_row(&mut self , msg : &Message, _ : bool) {
        if let Some(f) = &mut self.file {
            let mut s =  "".to_string();
    
            let lang_count =  if let Some(config) = &self.config { (config.get_languages)().len()} else {0};
            for i in 0..lang_count {
                s += &format!("\"{}\";", msg.get_raw(i, self.config.as_ref()));
            }

            s += "\n";
    
            let _ = f.write(s.as_bytes());
        }
    }

    fn end(&mut self) {
        
    }
}

struct XLSXExporter {
    filepath: String,
    workbook : rust_xlsxwriter::Workbook,
    current_row: usize,
    config : Option<GameConfig>
}

impl Exporter for XLSXExporter {
    fn new(filepath: &Path) -> Self {
        println!("Creating XLSX file : {}", filepath.display());
        XLSXExporter { filepath: filepath.display().to_string(), workbook: rust_xlsxwriter::Workbook::new(), current_row : 0, config:None }
    }

    fn set_config(&mut self, config : &GameConfig) {
        self.config = Some(config.clone());
    }

    fn begin(&mut self) {
        // Add a worksheet to the workbook.
        let _worksheet = self.workbook.add_worksheet();
    }

    fn set_headers(&mut self) {
        if let Ok(worksheet) = self.workbook.worksheet_from_index(0) {
            let bold = Format::new().set_bold();
            let dark_bg = Format::new().set_font_color(Color::White).set_background_color(Color::Gray);

            let languages = if let Some(conf) = &self.config {(conf.get_languages)()} else { &[] };
            let lang_count =  if let Some(config) = &self.config { (config.get_languages)().len()} else {0};
            let _ = worksheet.write_row_with_format(0, 0, languages.iter().map(|l| l.1), &bold);
            let _ = worksheet.set_column_range_format(0, lang_count as u16, &dark_bg);
            if let Err(e) = worksheet.set_column_range_width(0, lang_count as u16, 50) {
                println!("Error setting col width : {e}");
            }
           

            self.current_row = 1;
        }
    }

    fn add_row(&mut self , msg : &Message, ignore_tags : bool) {
        if let Ok(worksheet) = self.workbook.worksheet_from_index(0) {

            let lang_count =  if let Some(config) = &self.config { (config.get_languages)().len()} else {0};
            for i in 0..lang_count {
                if ignore_tags {
                    let _ = worksheet.write(self.current_row as u32 , i as u16, msg.get_raw(i, self.config.as_ref()));
                } else {
                    let mut cell_color = Color::White;
                    let mut cell_align = FormatAlign::default();
                    let mut cell_bg_color = Color::Gray;

                    let style_info = self.config.as_ref().map(|c| (c.get_message_style)(&msg.attribs)).unwrap_or_default();
                    if style_info.centered { cell_align = FormatAlign::Center}
                    if !style_info.color.is_empty() { cell_color = Color::from(style_info.color.as_str())}
                    if !style_info.bg_color.is_empty() { cell_bg_color = Color::from(style_info.bg_color.as_str())}
                    
                    let cell_format = Format::new().set_font_color(cell_color)
                                                    .set_background_color(cell_bg_color)
                                                    .set_align(cell_align)
                                                    .set_align(FormatAlign::VerticalCenter)
                                                    .set_text_wrap();
                                                    

                    let segments = msg.get_xlsx_formatted(i, ignore_tags, cell_color, self.config.as_ref());

                    if !segments.is_empty() {
                        let segments_ref : Vec<_>= segments.iter().map(|(a,b)| (a,b.as_str())).collect();
                        match worksheet.write_rich_string_with_format(self.current_row as u32 , i as u16, &segments_ref, &cell_format) {
                            Ok(_) => {},
                            Err(e) => {
                                println!("Error rich {e}");
                                // println!("row {}, col {} segments {:?}", self.current_row, i , segments.iter().map(|(_,s)| s).collect::<Vec<_>>());
                            },
                        }
                    }
                }
            }
            self.current_row += 1;
    
        }
    }

    fn end(&mut self) {
        // Save the file to disk.
        if let Ok(worksheet) = self.workbook.worksheet_from_index(0) {
            worksheet.autofit();
        }
        
        match self.workbook.save(&self.filepath) {
            Ok(_) => {},
            Err(e) => println!("Error saving : {e}")
        };
    }
}


impl BMGParser {
    fn add_message(&mut self, msg: &MessageSingleLang, lang_idx : usize, bank_id : usize) {

        if msg.is_empty() {
            return;
        }

        let idx = msg.id - 1;//if msg.id > 0 {} else {self.msgs[bank_id].len()};

        if idx + 1> self.msgs[bank_id].len() { self.msgs[bank_id].resize_with(idx + 1, || Message::default() );}
        
        self.msgs[bank_id][idx].id = msg.id;
        
        if self.msgs[bank_id][idx].attribs.is_empty() {
            self.msgs[bank_id][idx].attribs = msg.attribs.clone();
        }
        
        
        if lang_idx >= self.msgs[bank_id][idx].text.len() { self.msgs[bank_id][idx].text.resize(lang_idx +1, Vec::new()); }
        if !self.msgs[bank_id][idx].text[lang_idx].is_empty()
        {
            println!("ALREADY USED : {}, {:#x}, lang {}", bank_id, idx, lang_idx);
            println!("Prev : {}", self.msgs[bank_id][idx] );
            println!("New: {:?}", msg.text );
        } else 
        {
            self.msgs[bank_id][idx].text[lang_idx] = msg.text.clone();
        }  
    }

    fn export_html(&self, filepath: &Path, ignore_tags : bool, config : &GameConfig) {
        
        let mut exporter = HTMLExporter::new(filepath);
        exporter.set_config(config);
        exporter.begin();
        exporter.set_headers();

        for bank in &self.msgs {
            for msg in bank.iter().filter(|msg| !msg.is_empty()) {
                exporter.add_row(msg, ignore_tags);
            }
        }
        exporter.end();
    }

    fn export_csv(&self, filepath: &Path, config : &GameConfig) {
        
        let mut exporter = CSVExporter::new(filepath);
        exporter.set_config(config);
        exporter.begin();
        exporter.set_headers();

        for bank in &self.msgs {
            for msg in bank.iter().filter(|msg| !msg.is_empty()) {
                exporter.add_row(msg, true);
            }
        }
        exporter.end();
    }


    fn export_xlsx(&self, filepath: &Path, ignore_tags : bool, config : &GameConfig) {
        let mut exporter = XLSXExporter::new(filepath);
        exporter.set_config(config);
        exporter.begin();
        exporter.set_headers();

        for bank in &self.msgs {
            for msg in bank.iter().filter(|msg| !msg.is_empty()) {
                exporter.add_row(msg, ignore_tags);
            }
        }
        exporter.end();
    }
}

fn process_file(filename : &Path, lang_id : usize, bank_id : usize, parser : &mut BMGParser, big_endian : bool) -> io::Result<()> {

    println!("opening file {}", filename.display());
    // Tried some shennaningans
    let p : Box<dyn MessageParser> = match filename.extension().and_then(|s| s.to_str()) {
        Some("txt") => Box::new(bmg_text_parser::open_bmg(filename)?),
        Some("bmg") => Box::new(bmg_raw_parser::open_bmg(filename, big_endian)?),
        Some("msbt") => Box::new(msbt_parser::open_msbt(filename)?),
        None => todo!(),
        _ => todo!()
    };

    for m in p.get_all_messages() {
        parser.add_message(&m, lang_id, bank_id);
    }

    parser.encoding = Some(p.get_encoding());

    Ok(())
}

fn process_config(parser : &mut BMGParser, config : &GameConfig)
{
    for (lang_idx, lang) in (config.get_languages)().iter().enumerate() {

        let str_path = &format!("./res/{}/{}", config.id, lang.1);
        let folder_path = Path::new(&str_path);

        for (bank_id,&basename) in (config.get_filenames)().iter().enumerate() {
            let _ = process_file(&folder_path.join(&basename), lang_idx, bank_id, parser, config.big_endian);
        }
    }
}

fn generate_index(filepath : &Path) {

    if let Ok(mut f) = File::create(filepath) {
        let _ = f.write(b"<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>Zelda Text Dumps</title>
</head>
<body style=\"text-align:center\">
<h1> Zelda comparison tables </h1>
<div>");

        for conf in game_configs::ALL_CONFIGS {
            let _ = f.write(format!("<a href=\"{}.html\"><img src=\"{}\"/></a>", conf.id, conf.logo).as_bytes());
        }

        let _ = f.write(b"</div></body>
</html>");
    }
}

fn main() {

    generate_index(Path::new("./www/index.html"));

    // for config in &[game_configs::ALBW] {
    for config in game_configs::ALL_CONFIGS {
        
        let id = config.id;
        let mut parser : BMGParser = Default::default();
        process_config(&mut parser, &config);
        parser.export_html(Path::new(&format!("./www/{id}.html")), false, &config);
        parser.export_csv(Path::new(&format!("./www/download/{id}.csv")), &config);
        parser.export_xlsx(Path::new(&format!("./www/download/{id}.xlsx")), false, &config);
    }


    //msbt_parser::print_msbt(Path::new("./res/albw/Japanese/Common.msbt"));

}
