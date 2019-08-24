// This code is published under the terms of the GNU GPL license.
// This license requires you to comply with these conditions in order to be valid:
//  * Sharing a modified version of sielo-core require you to share the source code.
//  * Work on program that communicates with the core no needs a GPL compliant license. You are free

//! Manage history system for Sielo.
//!
//! # How it works?
//!
//! Currently, they are a main way to manage history in a web browser, each page's information such
//! as the URL, the title, the favicon and the time when the page is loaded in a database. Sielo
//! history database has the same working with one difference: each page will have a link with the
//! parent page.


extern crate url;
extern crate sqlite;

type FieldID = usize;
type ProfileDB = sqlite::Connection;

#[derive(Copy, Clone)]
pub enum Type {
    Page,
    Image,
    Video,
    Text,
    File,
}

pub enum FieldError {
    NoError,
    InternalError
}

enum FieldCode {
    NoError,
    NoFound,
    BadType,
}

pub struct Field<'a, 'b> {
    m_id : FieldID,
    m_item : Item<'a>,
    m_parent : Option<FieldID>,
    m_children : Vec<FieldID>,
    m_provider : &'a History<'b>,
}

pub struct Item<'a> {
    m_mime_type : &'a str,
    m_date : std::time::Instant,
    m_url  : url::Url,
    m_title : Option<&'a str>,
    m_favicon_path : Option<&'a std::path::Path>,
}

pub struct History<'a> {
    m_profile : &'a mut ProfileDB,
}

impl<'a> History<'a> {
    pub fn new(db : &'a mut ProfileDB) -> Result<Self, sqlite::Error> {
        // Check if the table exists
        if {
            let mut cnt = 0;
            if let Err(e) = db.iterate("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='history';", |ret| {
                for i in ret {
                    if i.0 == "COUNT(*)" {
                        cnt = match i.1 {
                            Some(t) => {
                                match t.parse::<i32>() {
                                    Ok(t) => {
                                        t
                                    }
                                    Err(_) => {
                                        return false;
                                    }
                                }
                            },
                            None => return false
                        };
                        return true;
                    }
                }
                false
            }) {
                return Err(e);
            }
            cnt
        } == 0 {
            println!("No history table found, creating one...");
            if let Err(e) = db.execute("CREATE TABLE `history` (\
	            `id`	INTEGER PRIMARY KEY AUTOINCREMENT UNIQUE,\
            	`mimetype`	TEXT NOT NULL,\
            	`url`	TEXT NOT NULL,\
            	`date`	INTEGER NOT NULL,\
            	`title`	TEXT,\
            	`favicon_path`	TEXT,\
               	`parent`	INTEGER,\
	            `children`	BLOB NOT NULL\
            );") {
                return Err(e);
            }
        } else {
            match History::get_table_fields(db) {
                Ok(t) => {
                    let mut cp = vec![("id", "INTEGER"), ("mimetype", "TEXT"),
                                                     ("url", "TEXT"), ("date", "INTEGER"),
                                                     ("title", "TEXT"), ("favicon_path", "TEXT"),
                                                     ("parent", "INTEGER"), ("children", "BLOB")];

                    for i in t {
                        let mut j = 0;
                        while j < cp.len() {
                            if cp[j].0 == i.0 {
                                if cp[j].1 == i.1 {
                                    cp.remove(j);
                                    continue;
                                } else {
                                    return Err(sqlite::Error {
                                        code: None,
                                        message: Some(
                                            format!("Bad type for {}. Expected {}, given {}",
                                                cp[j].0, cp[j].1, i.1)
                                        ),
                                    });
                                }
                            }
                            j += 1;
                        }
                    }
                },
                Err(e) => return Err(e),
            }
        }

        return Ok(Self { m_profile: db });
    }

    fn get_table_fields(db : &ProfileDB) -> Result<Vec<(String,String)>, sqlite::Error> {
        let mut ret = Vec::<(String,String)>::new();

        if let Err(e) = db.iterate("PRAGMA table_info(history);", |data| {
            let mut val = (String::new(),String::new());
            let mut next = false;
            for i in data {
                if i.0 == "name" {
                    if let Some(t) = i.1 {
                        val.0 = String::from(t);
                        if next {
                            ret.push(val);
                            return true;
                        }
                        next = true;
                    }
                }
                if i.0 == "type" {
                    if let Some(t) = i.1 {
                        val.1 = String::from(t);
                        if next {
                            ret.push(val);
                            return true;
                        }
                        next = true;
                    }
                }
            }
            false
        }) {
            return Err(e);
        }
        return Ok(ret);
    }
}

