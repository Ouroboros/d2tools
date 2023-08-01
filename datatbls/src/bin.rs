#![allow(unused)]

use std::rc::Weak;
use std::{path::Path, rc::Rc};
use std::collections::HashMap;
use std::io::{Read, Seek, Cursor};
use ml::io::{File, ReadExt, LittleEndian};
use anyhow::Result;

type LE = LittleEndian;

#[derive(Clone)]
pub enum Value {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    I8Array(Vec::<i8>),
    I16Array(Vec::<i16>),
    I32Array(Vec::<i32>),
    U8Array(Vec::<u8>),
    U16Array(Vec::<u16>),
    U32Array(Vec::<u32>),

    StringId(u16),
    ItemCode(u32),
    String(usize, Option<String>),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int8(v) => write!(f, "Int8: 0x{v:02X} ({v})"),
            Self::Int16(v) => write!(f, "Int16: 0x{v:04X} ({v})"),
            Self::Int32(v) => write!(f, "Int32: 0x{v:08X} ({v})"),
            Self::UInt8(v) => write!(f, "UInt8: 0x{v:02X} ({v})"),
            Self::UInt16(v) => write!(f, "UInt16: 0x{v:04X} ({v})"),
            Self::UInt32(v) => write!(f, "UInt32: 0x{v:08X} ({v})"),
            Self::I8Array(v) => write!(f, "Int8Array({}): {v:?}", v.len()),
            Self::I16Array(v) => write!(f, "Int16Array({}): {v:?}", v.len()),
            Self::I32Array(v) => write!(f, "Int32Array({}): {v:?}", v.len()),
            Self::U8Array(v) => write!(f, "UInt8Array({}): {v:?}", v.len()),
            Self::U16Array(v) => write!(f, "UInt16Array({}): {v:?}", v.len()),
            Self::U32Array(v) => write!(f, "UInt32Array({}): {v:?}", v.len()),

            Self::StringId(v) => write!(f, "StringId: 0x{v:04X} ({v})"),
            Self::ItemCode(v) => write!(f, "ItemCode: 0x{v:08X} ({v})"),
            Self::String(size, s) => write!(f, "String: {}", s.as_ref().unwrap()),
        }
    }
}

macro_rules! value_impl {
    ($num_type:ident, $value_type:ident) => {
        impl From<$num_type> for Value {
            fn from(value: $num_type) -> Self {
                Value::$value_type(value)
            }
        }

        impl Value {
            pub fn $num_type(&self) -> $num_type {
                match self {
                    Self::$value_type(v) => *v,
                    _ => panic!("type is {self:?}"),
                }
            }
        }
    }
}

value_impl!(i8, Int8);
value_impl!(i16, Int16);
value_impl!(i32, Int32);
value_impl!(u8, UInt8);
value_impl!(u16, UInt16);
value_impl!(u32, UInt32);

#[derive(Debug)]
#[derive(Clone)]
pub struct Field {
    pub name    : String,
    pub value   : Value,
    pub offset  : u64,
}

impl Field {
    pub fn new<T: Into<Value>>(name: &str, value: T, offset: u64) -> Field {
        Field{
            name    : name.to_string(),
            value   : value.into(),
            offset,
        }
    }

    pub fn validate_fields_offset(fields: &Vec<Field>) {
        let mut offset = 0u64;

        for f in fields.iter() {
            if f.offset != offset {
                panic!("field {} offset is 0x{:X}, expect 0x{:X}", f.name, offset, f.offset);
            }

            match &f.value {
                Value::Int8(_) => offset += 1,
                Value::Int16(_) => offset += 2,
                Value::Int32(_) => offset += 4,
                Value::UInt8(_) => offset += 1,
                Value::UInt16(_) => offset += 2,
                Value::UInt32(_) => offset += 4,
                Value::I8Array(v) => offset += v.capacity() as u64 * 1,
                Value::I16Array(v) => offset += v.capacity() as u64 * 2,
                Value::I32Array(v) => offset += v.capacity() as u64 * 4,
                Value::U8Array(v) => offset += v.capacity() as u64 * 1,
                Value::U16Array(v) => offset += v.capacity() as u64 * 2,
                Value::U32Array(v) => offset += v.capacity() as u64 * 4,

                Value::StringId(_) => offset += 2,
                Value::ItemCode(_) => offset += 4,
                Value::String(size, s) => offset += *size as u64,
            }
        }
    }

    pub fn read<T: ReadExt>(&mut self, fs: &mut T) -> Result<()> {
        match &mut self.value {
            Value::UInt8(_) => {
                self.value = Value::from(fs.u8());
            },
            Value::UInt16(_) => {
                self.value = Value::from(fs.u16::<LE>());
            },
            Value::UInt32(_) => {
                self.value = Value::from(fs.u32::<LE>());
            },
            Value::Int8(_) => {
                self.value = Value::from(fs.i8());
            },
            Value::Int16(_) => {
                self.value = Value::from(fs.i16::<LE>());
            },
            Value::Int32(_) => {
                self.value = Value::from(fs.i32::<LE>());
            },
            Value::I8Array(v) => {
                for i in 0..v.capacity() {
                    v[i] = fs.i8();
                }
            },
            Value::I16Array(v) => {
                for i in 0..v.capacity() {
                    v[i] = fs.i16::<LE>();
                }
            },
            Value::I32Array(v) => {
                for i in 0..v.capacity() {
                    v[i] = fs.i32::<LE>();
                }
            },
            Value::U8Array(v) => {
                // println!("capacity: {}", v.capacity());
                // println!("len: {}", v.len());
                for i in 0..v.capacity() {
                    // println!("  {i}");
                    v[i] = fs.u8();
                }
                // println!("capacity: {}", v.capacity());
                // println!("len: {}", v.len());
            },
            Value::U16Array(v) => {
                for i in 0..v.capacity() {
                    v[i] = fs.u16::<LE>();
                }
            },
            Value::U32Array(v) => {
                for i in 0..v.capacity() {
                    v[i] = fs.u32::<LE>();
                }
            },

            Value::StringId(_) => {
                self.value = Value::StringId(fs.u16::<LE>());
            },

            Value::ItemCode(_) => {
                self.value = Value::ItemCode(fs.u32::<LE>());
            },

            Value::String(size, s) => {
                let mut b = fs.read_bytes(*size)?;
                let s = String::from_utf8(b).unwrap().trim_end_matches(char::from(0)).to_string();
                self.value = Value::String(*size, Some(s));
            },
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Record {
    fields: Vec<Field>,
    hm: HashMap<String, usize>,
}

impl From<Vec<Field>> for Record {
    fn from(fields: Vec<Field>) -> Self {
        let mut hm = HashMap::new();
        for (i, f) in fields.iter().enumerate() {
            hm.insert(f.name.clone(), i);
        }

        Self {
            fields,
            hm,
        }
    }
}

impl Record {
    pub fn iter(&self) -> std::slice::Iter<'_, Field> {
        self.fields.iter()
    }

    pub fn get(&self, key: &str) -> &Field {
        let idx = self.hm.get(key).expect(format!("{key} not exists").as_str());
        &self.fields[*idx]
    }
}

pub struct BinRecord {
    records: Vec<Record>,
}

impl BinRecord {
    pub fn new() -> BinRecord {
        BinRecord{
            records: Vec::new(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Record> {
        self.records.iter()
    }

    pub fn records(&self) -> &Vec<Record> {
        &self.records
    }
}

pub struct BinFile {
    file: File,
    fields: Vec<Field>,
}

impl BinFile {
    pub fn open<T: AsRef<Path>>(path: T, fields: &[Field]) -> Result<BinFile> {
        Ok(BinFile{
            file    : File::open(path)?,
            fields  : Vec::from(fields),
        })
    }

    pub fn read(&mut self) -> Result<BinRecord> {
        let mut buf = Vec::new();
        self.file.read_to_end(&mut buf)?;
        let mut r = Cursor::new(&buf);

        let mut record = BinRecord::new();
        let record_count = r.u32::<LE>();

        for i in 0..record_count {
            let mut fields = self.fields.clone();

            for f in fields.iter_mut() {
                let pos = r.stream_position().unwrap();
                f.read(&mut r)?;
            }

            record.records.push(fields.into());
        }

        // println!("records: {record_count}");

        Ok(record)
    }
}
