// This code is published under the terms of the GNU GPL license.
// This license requires you to comply with these conditions in order to be valid:
//  * Sharing a modified version of sielo-core require you to share the source code.
//  * Work on program that communicates with the core no needs a GPL compliant license. You are free

//! Abstraction layer for database system
//!
//!

use std::collections::hash_map::HashMap;
use std::ops::{Index, Add};
use std::borrow::Borrow;

pub mod sqlite;

pub enum FieldType {
    Integer,
    Real,
    Text,
    Blob,
    Unknown,
}

pub enum FieldValue<'a> {
    Integer(i64),
    Real(f64),
    Text(&'a str),
    Blob(&'a [u8]),
}

#[derive(Eq, PartialEq)]
pub enum FieldParameter {
    PrimaryKey,
    NoNull,
    AutoIncrement,
    Unique,
    Default(String),
}

pub struct Error {
    code : Option<isize>,
    message : Option<String>,
}

pub trait TableProvider {
    type TableProviderType : TableProvider;
    type DataBaseType;

    fn use_table(&mut self,
                  name : &str,
                 fields : &[(&str, &FieldType, &[FieldParameter])],
                  auto_create_field : bool,
                  strict : bool) -> Result<(), Error>;

    fn request(&mut self, req : &str, arguments : &[&str])
               -> Result<Vec<HashMap<String,String>>, Error>;

    fn use_correct_format(val : &str) -> bool {
        const LETTER_RANGE : (&u8,&u8) = (&97u8, &122u8);
        const DIGIT_RANGE : (&u8,&u8) = (&48u8, &57u8);
        const UNDERSCORE : &u8 = &95u8;

        for i in val.as_bytes() {
            if (i < LETTER_RANGE.0 || i > LETTER_RANGE.1) &&
               (i < DIGIT_RANGE.0 || i > DIGIT_RANGE.1) &&
               i != UNDERSCORE {
                return false;
            }
        }
        return true;
    }

    fn convert_correct_format(val : &str) -> String {
        let mut ret = String::new();
        let mut first = true;

        const LETTER_RANGE : (&u8,&u8) = (&97u8, &122u8);
        const DIGIT_RANGE : (&u8,&u8) = (&48u8, &57u8);
        const UNDERSCORE : &u8 = &95u8;

        const UPPERCASE_RANGE : (&u8,&u8) = (&65u8, &90u8);

        for i in val.as_bytes() {
            if (i >= LETTER_RANGE.0 && i <= LETTER_RANGE.1) ||
                (i >= DIGIT_RANGE.0 && i <= DIGIT_RANGE.1) ||
                i == UNDERSCORE {
                ret += unsafe {
                    std::str::from_utf8_unchecked(std::slice::from_ref(i))
                };
            } else if i >= UPPERCASE_RANGE.0 && i <= UPPERCASE_RANGE.1 {
                if !first {ret += "_"};
                ret += unsafe {
                    std::str::from_utf8_unchecked(&[i.to_ascii_lowercase()])
                };
            }
            first = false;
        }

        return ret;
    }

    fn make_compliant_value(val : &str) -> String {
        let mut ret = String::new();

        let to_escape = ['\"','\'', '?', '\\'];

        for i in val.chars() {
            if to_escape.contains(&i) {
                ret += "\\";
            }
            ret += &*i.to_string();
        }

        ret
    }
}
