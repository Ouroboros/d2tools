#![allow(unused)]

use std::{io::Write, collections::HashMap, fmt::format};

use {
    crate::{
        bin::*,
        fields,
        stringtbl::*,
        datatbls_mgr::*,
    },

    anyhow::Result,
};

fn test_string_table() -> Result<()> {
    let mut patchstring = StringTable::open(r"D:\Game\Diablo II 暗月\MPQDumped\DATA\LOCAL\lng\CHI\expansionstring.tbl")?;

    let kvs = patchstring.read()?;

    for (i, kv) in kvs.iter().enumerate() {
        println!("{i:04}: {{'{key}': '{value}'}}", key = kv.key, value = kv.value);
    }

    Ok(())
}

pub fn run() -> Result<()> {
    Field::validate_fields_offset(&*fields::SKILLS);
    Field::validate_fields_offset(&*fields::SKILL_DESC);
    Field::validate_fields_offset(&*fields::ITEMS);

    let mut tbls = DataTblsManager::new();

    tbls.load(r"D:\Game\Diablo II 暗月\MPQDumped\DATA\")?;

    tbls.dump_fields(tbls.weapon.records(), r"E:\Desktop\bin2txt\test\bin\weapon.py");
    tbls.dump_fields(tbls.armor.records(), r"E:\Desktop\bin2txt\test\bin\armor.py");
    tbls.dump_fields(tbls.misc.records(), r"E:\Desktop\bin2txt\test\bin\misc.py");

    let mut item_id_lines = Vec::new();

    for items in vec![&tbls.weapon, &tbls.armor, &tbls.misc] {
        let start_index = items.start_index() as usize;

        for (index, item) in items.records().iter().enumerate() {
            let name = tbls.format_value(&item.get("name_str").value);
            let name = name.trim_end();

            if name.is_empty() {
                continue;
            }

            item_id_lines.push(format!("{:>4} {}", index + start_index, name));
        }
    }

    std::fs::File::create(r"D:\Dev\Source\d2tools\DarkMoonData\物品ID.txt")?.write_all(item_id_lines.join("\n").as_bytes())?;

    return Ok(());

    // panic!("{:#?}", fields::SKILL_DESC[fields::SKILL_DESC.len() - 1].value);

    let mut bin_skills = BinFile::open(r"E:\Desktop\bin2txt\test\bin\skills.bin", &*fields::SKILLS)?;
    let skills = bin_skills.read()?;

    let mut bin_skill_desc = BinFile::open(r"E:\Desktop\bin2txt\test\bin\skilldesc.bin", &*fields::SKILL_DESC)?;
    let skill_desc = bin_skill_desc.read()?;

    tbls.dump_fields(&skills, r"E:\Desktop\bin2txt\test\bin\skills.py").unwrap();
    tbls.dump_fields(&skill_desc, r"E:\Desktop\bin2txt\test\bin\skill_desc.py").unwrap();

    let mut strtbl = StringTableManager::new();

    strtbl.load(
        r"D:\Game\Diablo II 暗月\MPQDumped\DATA\LOCAL\lng\CHI\string.tbl",
        r"D:\Game\Diablo II 暗月\MPQDumped\DATA\LOCAL\lng\CHI\patchstring.tbl",
        r"D:\Game\Diablo II 暗月\MPQDumped\DATA\LOCAL\lng\CHI\expansionstring.tbl",
        r"D:\Game\Diablo II 暗月\MPQDumped\DATA\duck\lng\chi\DuckModString.tbl",
        r"D:\Game\Diablo II 暗月\MPQDumped\DATA\duck\lng\chi\DuckPermString.tbl",
    )?;

    let mut m: HashMap<i8, Vec<(&Record, &Record)>> = HashMap::new();

    for skill in skills.iter() {
        let desc = skill.get("skill_desc").value.u16();

        if desc == u16::MAX {
            continue;
        }

        let char_class = skill.get("char_class").value.i8();
        let desc = &skill_desc.records()[desc as usize];
        let str_name = desc.get("str_name").value.u16() as usize;
        let skill_name = strtbl.get_string_by_index(str_name);

        if skill_name.is_none() {
            continue;
        }

        m.entry(char_class).or_insert_with(Vec::new).push((skill, desc));
    }

    let mut lines = Vec::<String>::new();

    lines.push("{".into());

    for (char_class, v) in m.iter() {
        let char_class = match char_class {
            0 => "Ama",
            1 => "Sor",
            2 => "Nec",
            3 => "Pal",
            4 => "Bar",
            5 => "Dru",
            6 => "Ass",
            -1 => "255",
            _ => todo!("wtf"),
        };

        lines.push(format!("  \"{char_class}\": ["));

        for (skill, desc) in v.iter() {
            let name        = desc.get("str_name").value.u16();
            let str_long    = desc.get("str_long").value.u16();
            let req_level   = skill.get("req_level").value.u16();
            let max_lvl     = skill.get("max_lvl").value.u16();
            let skill_id    = skill.get("skill_id").value.i16();

            let name        = strtbl.get_string_by_index(name as usize);
            let str_long    = strtbl.get_string_by_index(str_long as usize);

            if name.is_none() || str_long.is_none() {
                continue;
            }

            let ls = vec![
                "    {".to_string(),
                format!("      \"name\": \"{}\",", name.unwrap()),
                format!("      \"str_long\": \"{}\",", str_long.unwrap()),
                format!("      \"req_level\": {req_level},"),
                format!("      \"max_lvl\": {max_lvl},"),
                format!("      \"id\": {skill_id},"),
                format!("      \"char_class\": \"{char_class}\","),
                "    },".to_string(),
            ];

            lines.extend(ls);
        }

        lines.push("  ],".into());
    }

    lines.push("}".into());

    let output = lines.join("\n");

    let mut py = std::fs::File::create(r"E:\Desktop\bin2txt\test\bin\skills2.py")?;
    py.write_all(output.as_bytes())?;

    // println!("{:#?}", m);

    Ok(())
}