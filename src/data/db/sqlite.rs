// This code is published under the terms of the GNU GPL license.
// This license requires you to comply with these conditions in order to be valid:
//  * Sharing a modified version of sielo-core require you to share the source code.
//  * Work on program that communicates with the core no needs a GPL compliant license. You are free

//! Abstraction layer implementation for SQLite

extern crate sqlite;

use std::collections::hash_map::HashMap;
use super::{ Error, TableProvider, FieldType, FieldParameter };
use crate::data::history::Field;

pub struct SQLite {
    db : sqlite::Connection,
}

impl SQLite {
    pub fn new<T: AsRef<std::path::Path>>(db_path : T) -> Result<Self, Error> {
        match sqlite::Connection::open(db_path) {
            Ok(t) => Ok(Self { db: t }),
            Err(e) => Err(Error { code: e.code, message: e.message })
        }
    }

    pub fn have_table(&mut self, name : &str) -> Result<bool, Error> {
        match self.db.prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?;") {
            Ok(mut t) => {
                if let Err(e) = t.bind(1, name) {
                    return Err(Error{ code: e.code, message: e.message });
                } else {
                    while let Ok(sqlite::State::Row) = t.next() {
                        match t.read::<i64>(0) {
                            Ok(v) => return Ok(v != 0),
                            Err(e) => return Err(Error{ code: e.code, message: e.message }),
                        }
                    }
                    return Ok(false);
                }
            },
            Err(e) => return Err(Error{ code: e.code, message: e.message }),
        }
    }

    fn convert_format(name : &str, strict : bool) -> Result<String, Error> {
        if Self::use_correct_format(name) {
            Ok(String::from(name))
        } else {
            if strict {
                Err(Error {
                    code: Some(1001),
                    message: Some(format!("Forbidden field format for name {}", name))
                })
            } else {
                println!("Warning: Forbidden field format for name {}", name);
                Ok(Self::convert_correct_format(name))
            }
        }
    }

    fn make_field_command(name : &str,
                  field_t : &FieldType,
                  parameters : &[FieldParameter],
                  strict : bool) -> Result<(String, u8), Error> {
        let mut primary = 0;
        Ok((format!("{} {}{}",
              name,
              match field_t {
                  FieldType::Text => "TEXT",
                  FieldType::Real => "REAL",
                  FieldType::Blob => "BLOB",
                  FieldType::Integer => "INTEGER",
                  FieldType::Unknown => if strict {
                      return Err(Error {code: Some(1002), message: Some(String::from("Unknown field type can not be used"))})
                  } else {
                      println!("Warning: Passed unknown field type, creating with Blob type");
                      "BLOB"
                  },
              },
              {
                  let mut ret = String::new();

                  if parameters.contains(&FieldParameter::NoNull) {
                      ret += " NOT NULL";
                  }
                  if parameters.contains(&FieldParameter::AutoIncrement) {
                      ret += " PRIMARY KEY AUTOINCREMENT";
                      primary = 1;
                  } else if parameters.contains(&FieldParameter::PrimaryKey) {
                      primary = 2;
                  }
                  if parameters.contains(&FieldParameter::Unique) {
                      ret += " UNIQUE";
                  }

                  for i in parameters {
                      if let FieldParameter::Default(def) = i {
                          match field_t {
                              FieldType::Integer =>
                                  ret += &*format!(" DEFAULT {}", match def.parse::<i64>() {
                                      Ok(t) => t,
                                      Err(_) => return Err(Error {
                                          code: Some(1003),
                                          message: Some(format!("Default value ({}) is not an integer", def))
                                      })
                                  }),
                              FieldType::Blob =>
                                  ret += &*format!(" DEFAULT \"{}\"", Self::make_compliant_value(def)),
                              FieldType::Text =>
                                  ret += &*format!(" DEFAULT \"{}\"", Self::make_compliant_value(def)),
                              FieldType::Real =>
                                  ret += &*format!(" DEFAULT {}", match def.parse::<f64>() {
                                      Ok(t) => t,
                                      Err(_) => return Err(Error {
                                          code: Some(1003),
                                          message: Some(format!("Default value ({}) is not a floating-point number", def))
                                      })
                                  }),
                              _ => (),
                          }

                          break;
                      }
                  }

                  ret
              }
        ),primary))
    }

    fn check_fields(&mut self, name : &str)
            -> Result<HashMap<String,(FieldType, Vec<FieldParameter>)>, Error> {
        match self.db.prepare(&*format!("PRAGMA table_info({})", Self::convert_format(name,false).ok().unwrap())) {
            Ok(mut t) => {
                let mut ret = HashMap::<String,(FieldType, Vec<FieldParameter>)>::new();

                loop {
                    match t.next() {
                        Ok(t) => if t == sqlite::State::Done { break; },
                        Err(e) => return Err(Error { code: e.code, message: e.message }),
                    }

                    let name = match t.read::<String>(1) {
                        Ok(t) => t,
                        Err(e) => return Err(Error { code: e.code, message: e.message }),
                    };
                    let tp = match t.read::<String>(2) {
                        Ok(t) => {
                            if t == String::from("INTEGER") { FieldType::Integer }
                            else if t == String::from("TEXT") { FieldType::Text }
                            else if t == String::from("BLOB") { FieldType::Blob }
                            else if t == String::from("REAL") { FieldType::Real }
                            else { FieldType::Unknown }
                        }
                        Err(e) => return Err(Error { code: e.code, message: e.message }),
                    };
                    let mut tags = Vec::<FieldParameter>::new();

                    // Is null or not
                    if match t.read::<i64>(3) {
                        Ok(t) => {
                            if t == 0 {
                                false
                            } else {
                                true
                            }
                        }
                        Err(e) => return Err(Error { code: e.code, message: e.message }),
                    } {
                        tags.push(FieldParameter::NoNull)
                    }

                    // Is primary key or not
                    if match t.read::<i64>(5) {
                        Ok(t) => {
                            if t == 0 {
                                false
                            } else {
                                true
                            }
                        }
                        Err(e) => return Err(Error { code: e.code, message: e.message }),
                    } {
                        tags.push(FieldParameter::PrimaryKey);
                    }

                    // Have default value or not
                    match t.read::<String>(4) {
                        Ok(t) => {
                            tags.push(FieldParameter::Default(t));
                        }
                        Err(e) => if e.code.is_some() {
                            return Err(Error { code: e.code, message: e.message })
                        },
                    };

                    ret.insert(name, (tp, tags));
                }
                return Ok(ret);
            }
            Err(e) => {
                return Err(Error { code: e.code, message: e.message });
            }
        }
        return Err(Error { code: None, message: None });
    }
}

impl TableProvider for SQLite {
    type TableProviderType = SQLite;
    type DataBaseType = sqlite::Connection;

    fn use_table(&mut self,
                 name : &str,
                 fields : &[(&str, &FieldType, &[FieldParameter])],
                 auto_create_field : bool,
                 strict : bool) -> Result<(), Error> {
        let name = match Self::convert_format(name, strict) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };
        let fields = {
            let mut ret = Vec::<(String, &FieldType, &[FieldParameter])>
                ::with_capacity(fields.len());

            for i in fields {
                ret.push((match Self::convert_format(i.0, strict) {
                    Ok(t) => t,
                    Err(e) => return Err(e),
                }, i.1, i.2));
            }

            ret
        };
        match self.have_table(name.as_str()) {
            Ok(t) => {
                if t {
                    match self.check_fields(name.as_str()) {
                        Ok(t) => {
                            for i in fields {
                                if t.contains_key(i.0.as_str()) {
                                    let val = &t[i.0.as_str()];
                                    if &val.0 == i.1 {
                                        if !i.2.contains(&FieldParameter::NoNull) && val.1.contains(&FieldParameter::NoNull) {
                                            println!("Warning: field {} cannot be null instead of the program wish", i.0);
                                        }
                                    } else {
                                        // The type is not the one expected.
                                        // TODO: Add support of type modifier
                                        unimplemented!();
                                    }
                                } else {
                                    // Field doesn't exist, create it
                                    let com = format!("ALTER TABLE {} ADD COLUMN {}",
                                                      name,
                                                      match Self::make_field_command(i.0.as_str(),
                                                                                     i.1,
                                                                                     i.2,
                                                                                     strict) {
                                                          Ok(t) => t.0,
                                                          Err(e) => return Err(e),
                                                      });
                                    if let Err(e) = self.db.execute(com) {
                                        return Err(Error { code: e.code, message: e.message });
                                    }
                                }
                            }
                        },
                        Err(e) => return Err(e),
                    }
                } else {
                    // Create table from scratch
                    let mut first = true;
                    let mut primary_key = None;
                    let mut pk_decl_later = false;
                    let mut com = format!("CREATE TABLE {} (",
                        match Self::convert_format(name.as_str(), strict) {
                            Ok(t) => t,
                            Err(e) => return Err(e),
                        }
                    );
                    for i in fields {
                        if !first {
                            com += ",";
                        }
                        let ret = match Self::make_field_command(i.0.as_str(),
                                                                 i.1,
                                                                 i.2,
                                                                 strict) {
                            Ok(value) => value,
                            Err(e) => return Err(e),
                        };

                        com += ret.0.as_str();

                        if ret.1 == 1 {
                            if primary_key != None {
                                return Err(Error {
                                    code: Some(1004),
                                    message: Some(String::from("Multiple declaration of primary key"))
                                })
                            } else {
                                primary_key = Some(i.0);
                                pk_decl_later = false;
                            }
                        } else if ret.1 == 2 {
                            if primary_key != None {
                                return Err(Error {
                                    code: Some(1004),
                                    message: Some(String::from("Multiple declaration of primary key"))
                                })
                            } else {
                                primary_key = Some(i.0);
                                pk_decl_later = true;
                            }
                        }

                        first = false;
                    }
                    if pk_decl_later {
                        if let Some(pk) = primary_key {
                            com += format!(",PRIMARY KEY({})", Self::use_correct_format(pk.as_str())).as_str();
                        }
                    }
                    com += ");";
                    println!("{}", com);
                    if let Err(e) = self.db.execute(com) {
                        return Err(Error { code: e.code, message: e.message });
                    }
                }
            },
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn request(&mut self, req : &str, arguments : &[&str])
               -> Result<Vec<HashMap<String,String>>, Error>
    {
        return Err(Error { code: None, message: None });
    }
}