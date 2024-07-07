mod skill;
mod item;

pub use skill::{SKILLS, SKILL_DESC};
pub use item::ITEMS;

#[macro_export]
macro_rules! __parse_arr_type {
    (u8, $size:expr) => { Value::U8Array(crate::fields::new_vec($size)) };
    (i16, $size:expr) => { Value::I16Array(crate::fields::new_vec($size)) };
    (u16, $size:expr) => { Value::U16Array(crate::fields::new_vec($size)) };
    (u32, $size:expr) => { Value::U32Array(crate::fields::new_vec($size)) };
    (str, $size:expr) => { Value::String($size, None) };
}

#[macro_export]
macro_rules! __parse_type {
    (u8) => { Value::UInt8 };
    (u16) => { Value::UInt16 };
    (u32) => { Value::UInt32 };
    (i8) => { Value::Int8 };
    (i16) => { Value::Int16 };
    (i32) => { Value::Int32 };
    ($type:ident) => { Value::$type }
}

#[macro_export]
macro_rules! __get_default {
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

#[macro_export(local_inner_macros)]
macro_rules! field {
    ($name:expr, $type:tt[$size:expr], $offset:expr) => {
        {
            Field::new($name, __parse_arr_type!($type, $size), $offset)
        }
    };

    ($name:expr, $type:tt, $offset:expr) => {
        {
            Field::new($name, __parse_type!($type)(__get_default!($type)), $offset)
        }
    };

    // ($name:expr, str, $size:expr, $offset:expr) => {
    //     {
    //         Field::new($name, Value::String($size, None), $offset)
    //     }
    // };
}
