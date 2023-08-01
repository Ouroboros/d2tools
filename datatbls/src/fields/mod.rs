mod skill;
mod item;

pub use skill::{SKILLS, SKILL_DESC};
pub use item::ITEMS;

#[macro_export]
macro_rules! parse_arr_type {
    (u8) => { Value::U8Array };
    (i16) => { Value::I16Array };
    (u16) => { Value::U16Array };
    (u32) => { Value::U32Array };
}

#[macro_export]
macro_rules! parse_type {
    (u8) => { Value::UInt8 };
    (u16) => { Value::UInt16 };
    (u32) => { Value::UInt32 };
    (i8) => { Value::Int8 };
    (i16) => { Value::Int16 };
    (i32) => { Value::Int32 };
    ($type:ident) => { Value::$type }
}

#[macro_export]
macro_rules! get_default {
    (str) => { u8::default() };
    ($_:ident) => { 0 };
}

fn new_vec<T: std::default::Default>(size: usize) -> Vec<T> {
    let mut v = Vec::with_capacity(size);
    for _ in 0..size {
        v.push(T::default());
    }
    v
}

#[macro_export]
macro_rules! field {
    ($name:expr, $type:tt[$size:expr], $offset:expr) => {
        {
            use crate::parse_arr_type;
            use crate::fields::new_vec;
            Field::new($name, parse_arr_type!($type)(new_vec($size)), $offset)
        }
    };

    ($name:expr, $type:tt, $offset:expr) => {
        {
            use crate::{parse_type, get_default};
            Field::new($name, parse_type!($type)(get_default!($type)), $offset)
        }
    };

    ($name:expr, str, $size:expr, $offset:expr) => {
        {
            Field::new($name, Value::String($size, None), $offset)
        }
    };
}
