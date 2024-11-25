#![allow(unused)]

use std::path::Path;
use std::io::{Seek, SeekFrom, BufReader, BufRead};
use ml::io::{File, ReadExt, LittleEndian};
use anyhow::Result;

type LE = LittleEndian;

struct StringTableHeader {
    crc                 : u16,
    count               : u16,
    hash_table_size     : u32,
    unknown_08          : u8,
    string_start_offset : u32,
    max_miss_times      : u32,
    string_end_offset   : u32,
}

impl std::fmt::Debug for StringTableHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\
            crc                 = 0x{crc:04X}\n\
            count               = 0x{count:04X}\n\
            hash_table_size     = 0x{hash_table_size:08X}\n\
            unknown_08          = 0x{unknown_08:02X}\n\
            string_start_offset = 0x{string_start_offset:08X}\n\
            max_miss_times      = 0x{max_miss_times:08X}\n\
            string_end_offset   = 0x{string_end_offset:08X}",
            crc                 = self.crc,
            count               = self.count,
            hash_table_size     = self.hash_table_size,
            unknown_08          = self.unknown_08,
            string_start_offset = self.string_start_offset,
            max_miss_times      = self.max_miss_times,
            string_end_offset   = self.string_end_offset,
        )
    }
}

impl From<&mut File> for StringTableHeader {
    fn from(fs: &mut File) -> Self {
        Self {
            crc                 : fs.u16::<LE>(),
            count               : fs.u16::<LE>(),
            hash_table_size     : fs.u32::<LE>(),
            unknown_08          : fs.u8(),
            string_start_offset : fs.u32::<LE>(),
            max_miss_times      : fs.u32::<LE>(),
            string_end_offset   : fs.u32::<LE>(),
        }
    }
}

pub struct StringTableEntry {
    pub key     : String,
    pub value   : String,
}

pub struct StringTable {
    file: File,
}

impl StringTable {
    pub fn open<T: AsRef<Path>>(path: T) -> Result<StringTable> {
        Ok(Self{
            file: File::open(path)?,
        })
    }

    pub fn read(&mut self) -> Result<Vec<StringTableEntry>> {
        let mut header = StringTableHeader::from(&mut self.file);
        // println!("{:#?}", header);

        let mut fs = &mut self.file;
        let mut offset_into_hash_array = Vec::with_capacity(header.count as usize);

        for i in 0..header.count {
            offset_into_hash_array.push(fs.u16::<LE>());
        }

        let node_start_offset = fs.pos()?;

        // println!("node_start_offset = 0x{node_start_offset:08X}");

        let mut entries = Vec::new();

        for i in 0..header.count {
            let entry_offset = node_start_offset + u64::from(offset_into_hash_array[i as usize]) * 17;

            fs.seek(SeekFrom::Start(entry_offset))?;

            let used = fs.u8();
            let index = fs.u16::<LE>();
            let hash_value = fs.u32::<LE>();
            let key_offset = fs.u32::<LE>();
            let val_offset = fs.u32::<LE>();
            let val_length = fs.u16::<LE>();

            fs.seek(SeekFrom::Start(u64::from(key_offset)))?;

            let key = {
                let mut reader = BufReader::new(&mut fs);
                let mut buf = Vec::new();
                reader.read_until(0, &mut buf)?;
                buf.pop();

                String::from_utf8(buf)?
            };

            fs.seek(SeekFrom::Start(u64::from(val_offset)))?;

            let value = {
                let mut reader = BufReader::new(&mut fs);
                let mut buf = Vec::new();
                reader.read_until(0, &mut buf)?;
                buf.pop();

                String::from_utf8(buf)?.replace("\n", "\\n")
            };

            entries.push(StringTableEntry{key, value});
        }

        Ok(entries)
    }
}

pub struct StringTableManager {
    string          : Vec<StringTableEntry>,
    patchstring     : Vec<StringTableEntry>,
    expansionstring : Vec<StringTableEntry>,
    duckmodstring   : Vec<StringTableEntry>,
    duckpermstring  : Vec<StringTableEntry>,
}

impl StringTableManager {
    pub fn new() -> Self {
        Self {
            string          : Vec::new(),
            patchstring     : Vec::new(),
            expansionstring : Vec::new(),
            duckmodstring   : Vec::new(),
            duckpermstring  : Vec::new(),
        }
    }

    pub fn load<T: AsRef<Path>>(&mut self, string: T, patchstring: T, expansionstring: T, duckmodstring: Option<T>, duckpermstring: Option<T>) -> Result<()> {
        self.string = StringTable::open(string)?.read()?;
        self.patchstring = StringTable::open(patchstring)?.read()?;
        self.expansionstring = StringTable::open(expansionstring)?.read()?;

        match duckmodstring {
            Some(ref x) => self.duckmodstring = StringTable::open(x)?.read()?,
            None => (),
        };

        match duckmodstring {
            Some(ref x) => self.duckpermstring = StringTable::open(x)?.read()?,
            None => (),
        };
        Ok(())
    }

    pub fn get_string_by_index(&self, index: u16) -> Option<&str> {
        let index = index as usize;
        match index {
            0..=9999 => {
                Some(self.string[index].value.as_str())
            },
            10000..=19999 => {
                self.get_string_from_tbl(&self.patchstring, index - 10000)
                // Some(self.patchstring[index - 10000].value.as_str())
            },
            20000..=29999 => {
                Some(self.expansionstring[index - 20000].value.as_str())
            },
            0x86E8..=0xFC18 => {
                Some(self.duckmodstring[-(index as i16 + 1000) as usize].value.as_str())
            },
            0xFC19..=0xFFFE => {
                Some(self.duckpermstring[-(index as i16 + 2) as usize].value.as_str())
            },
            _ => {
                None
                // todo!("unsupported index: {index}");
            },
        }
    }

    fn get_string_from_tbl<'a>(&self, str_tbl: &'a Vec<StringTableEntry>, index: usize) -> Option<&'a str> {
        let mut index = index;

        if index >= str_tbl.len() {
            index = 500;
            // return None;
        }

        Some(str_tbl[index].value.as_str())
    }
}
