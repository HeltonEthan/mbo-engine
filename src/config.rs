use std::path::PathBuf;
use chrono::{
    NaiveDate,
};
use color_eyre::eyre::Result;

use crate::helper;

#[derive(Debug)]
pub struct Config {
    pub dir: PathBuf,
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl Config {
    //dates formatted as %Y-%m-%d (ie. 2024-03-10)
    pub fn new(dir: String, start: String, end: String) -> Result<Self> {
        Ok(Self{
            dir: PathBuf::from(dir),
            start: NaiveDate::parse_from_str(&start, "%Y-%m-%d")?,
            end: NaiveDate::parse_from_str(&end, "%Y-%m-%d")?,
        })
    }

    pub fn dir(&self) -> &PathBuf { &self.dir }
    pub fn start(&self) -> &NaiveDate { &self.start }
    pub fn end(&self) -> &NaiveDate { &self.end }

    pub fn start_unix(&self) -> Result<u64> { helper::to_unix(self.start()) }
    pub fn end_unix(&self) -> Result<u64> { helper::to_unix(self.end()) }
}
