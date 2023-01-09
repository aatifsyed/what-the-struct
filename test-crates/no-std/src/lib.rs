#![no_std]

pub struct UnitStruct;

pub struct EmptyPlainStruct {}

pub struct PlainStruct {
    pub unit_field: (),
    pub bool_field: bool,
    pub i8_array_field: [i8; 1],
    pub sibling_field: EmptyPlainStruct,
    pub child_field: nested::UnitStruct,
    _private_char_field: char,
    pub u8_slice_field: [u8],
}

pub mod nested {
    pub struct UnitStruct;
}
