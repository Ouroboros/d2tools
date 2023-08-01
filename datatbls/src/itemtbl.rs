#![allow(unused)]

use std::path::Path;
use std::io::{Seek, SeekFrom, BufReader, BufRead};
use ml::io::{File, ReadExt, LittleEndian};
use anyhow::Result;
use crate::bin::*;
use crate::fields;

pub struct ItemTable {
    start_index : u32,
    records     : BinRecord,
}

impl ItemTable {
    pub fn new(start_index: u32) -> Self {
        Self{
            start_index,
            records: BinRecord::new(),
        }
    }

    pub fn load<T: AsRef<Path>>(&mut self, path: T) -> Result<&BinRecord> {
        let p = path.as_ref().as_os_str().to_str().unwrap().to_string();
        self.records = BinFile::open(path, &*fields::ITEMS)?.read()?;
        Ok(&self.records)
    }

    pub fn start_index(&self) -> u32 {
        self.start_index
    }

    pub fn records(&self) -> &BinRecord {
        &self.records
    }
}
