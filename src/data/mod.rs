// This code is published under the terms of the GNU GPL license.
// This license requires you to comply with these conditions in order to be valid:
//  * Sharing a modified version of sielo-core require you to share the source code.
//  * Work on program that communicates with the core no needs a GPL compliant license. You are free

//! Collection of tools used by Sielo for data management purposes.
//! It implements toolset of:
//!  * [History system]()
//!  * [Database interface]() between SQLite and Sielo.
//!  * [Settings system]() using TOML files
//!  * [Modules management]() using OpenSSL and SQLite.

pub mod history;
pub mod db;