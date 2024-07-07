#![allow(unused)]

use std::path::Path;
use std::io::Write;
use anyhow::Result;

use crate::stringtbl::StringTableManager;
use crate::itemtbl::ItemTable;
use crate::bin::{BinRecord, Value};

pub struct DataTblsManager {
    pub strtbl: StringTableManager,
    pub weapon: ItemTable,
    pub armor: ItemTable,
    pub misc: ItemTable,
}

impl DataTblsManager {
    pub fn new() -> Self {
        Self {
            strtbl: StringTableManager::new(),
            weapon: ItemTable::new(1),
            armor: ItemTable::new(1001),
            misc: ItemTable::new(2001),
        }
    }

    pub fn load<T: AsRef<std::ffi::OsStr>>(&mut self, data_path: T) -> Result<()> {
        let data_path = Path::new(&data_path);
        self.strtbl.load(
            data_path.join(r"LOCAL\lng\CHI\string.tbl"),
            data_path.join(r"LOCAL\lng\CHI\patchstring.tbl"),
            data_path.join(r"LOCAL\lng\CHI\expansionstring.tbl"),
            data_path.join(r"duck\lng\chi\DuckModString.tbl"),
            data_path.join(r"duck\lng\chi\DuckPermString.tbl"),
        )?;

        self.weapon.load(data_path.join(r"global\excel\weapons.bin"))?;
        self.armor.load(data_path.join(r"global\excel\armor.bin"))?;
        self.misc.load(data_path.join(r"global\excel\misc.bin"))?;

        Ok(())
    }

    pub fn get_string_by_index(&self, index: u16) -> Option<&str> {
        self.strtbl.get_string_by_index(index)
    }

    pub fn dump_fields(&self, rec: &BinRecord, file_name: &str) -> Result<()> {
        let mut lines = Vec::<String>::new();

        // println!("{:#?}", record.fields);

        lines.push("fields = [".to_string());

        for record in rec.iter() {
            lines.push("    {".to_string());

            for f in record.iter() {
                if f.name.starts_with("__pad") {
                    continue;
                }

                lines.push(format!("        '{}': {},", f.name, self.format_value(&f.value)));
            }

            lines.push("    },".to_string());
        }

        lines.push("]".to_string());

        let output = lines.join("\n");

        let mut py = std::fs::File::create(file_name)?;
        py.write_all(output.as_bytes())?;

        Ok(())
    }

    pub fn format_value(&self, value: &Value) -> String {
        match value {
            Value::Int8(v) => format!("\"Int8: 0x{v:02X} ({v})\""),
            Value::Int16(v) => format!("\"Int16: 0x{v:04X} ({v})\""),
            Value::Int32(v) => format!("\"Int32: 0x{v:08X} ({v})\""),
            Value::UInt8(v) => format!("\"UInt8: 0x{v:02X} ({v})\""),
            Value::UInt16(v) => format!("\"UInt16: 0x{v:04X} ({v})\""),
            Value::UInt32(v) => format!("\"UInt32: 0x{v:08X} ({v})\""),
            Value::I8Array(v) => format!("\"Int8Array({}): {v:?}\"", v.len()),
            Value::I16Array(v) => format!("\"Int16Array({}): {v:?}\"", v.len()),
            Value::I32Array(v) => format!("\"Int32Array({}): {v:?}\"", v.len()),
            Value::U8Array(v) => format!("\"UInt8Array({}): {v:?}\"", v.len()),
            Value::U16Array(v) => format!("\"UInt16Array({}): {v:?}\"", v.len()),
            Value::U32Array(v) => format!("\"UInt32Array({}): {v:?}\"", v.len()),

            Value::StringId(v) => {
                let s = self.strtbl.get_string_by_index(*v);
                if let Some(v) = s { format!("\"{}\"", v) } else { format!("\"StringId<0x{:04X}>\"", *v) }
            },

            Value::ItemCode(v) => {
                let b = v.to_le_bytes();
                format!("\"{}\"", String::from_utf8(b.to_vec()).unwrap().trim_end_matches(char::from(0)))
            },

            Value::String(size, s) => {
                format!("\"{}\"", s.as_ref().unwrap())
            },
        }
    }

}
