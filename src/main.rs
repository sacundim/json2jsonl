/// A simple tool to convert big JSON array files to JSON Lines.

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use clap::Parser;
use core::fmt;
use std::error;
use memmap2::Mmap;
use serde::{de, Deserialize, Serialize, Deserializer};
use serde::de::{SeqAccess, Visitor};
use serde_json::{self};
use std::fs::File;
use std::marker::PhantomData;
use std::path::Path;
use strum_macros::EnumString;

/// Parse a Bioportal JSON download
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File to parse. Must not be compressed.
    #[clap(long)]
    infile: String,

    /// Which data set is in the file.
    #[clap(long)]
    dataset: Dataset
}

#[derive(EnumString, Debug)]
enum Dataset {
    Deaths, MinimalInfoUniqueTests
}

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() -> Result<()> {
    let args = Args::parse();
    let path = Path::new(&args.infile);
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file) }?;
    match args.dataset {
        Dataset::MinimalInfoUniqueTests =>
            process::<MinimalInfoUniqueTests>(&mmap),
        Dataset::Deaths => process::<Deaths>(&mmap)
    }
}

fn process<'de, T>(mmap: &'de Mmap) -> Result<()>
where
    T: Serialize,
    T: Deserialize<'de>
{
    let mut deserializer =
        serde_json::Deserializer::from_slice(&mmap);
    for_each(&mut deserializer, |record: T| {
        let output = serde_json::to_string(&record);
        output.map(|s| println!("{}", s)).unwrap()
    }).map_err(|e| e.into())
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Deaths<'a> {
    region: Option<&'a str>,
    age_range: Option<&'a str>,
    sex: Option<&'a str>,
    death_date: Option<DateTime<Utc>>,
    report_date: Option<DateTime<Utc>>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MinimalInfoUniqueTests<'a> {
    age_range: Option<&'a str>,
    city: Option<&'a str>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_option_mmddyyyy")]
    collected_date: Option<NaiveDate>,
    collected_date_utc: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_option_mmddyyyyhhmi")]
    created_at: Option<NaiveDateTime>,
    created_at_utc: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_option_mmddyyyy")]
    reported_date: Option<NaiveDate>,
    reported_date_utc: Option<DateTime<Utc>>,
    result: Option<&'a str>,
    test_type: Option<&'a str>
}

fn deserialize_option_mmddyyyy<'de, D>(deserializer: D)
    -> std::result::Result<Option<NaiveDate>, D::Error>
where D: serde::Deserializer<'de>
{
    let raw: Option<String> = Deserialize::deserialize(deserializer)?;
    raw.map(|s| NaiveDate::parse_from_str(&s, "%m/%d/%Y")
                .map_err(de::Error::custom))
        .transpose()
}

fn deserialize_option_mmddyyyyhhmi<'de, D>(deserializer: D)
    -> std::result::Result<Option<NaiveDateTime>, D::Error>
where D: serde::Deserializer<'de>,
{
    let raw: Option<String> = Deserialize::deserialize(deserializer)?;
    raw.map(|s| NaiveDateTime::parse_from_str(&s, "%m/%d/%Y %H:%M")
                .map_err(de::Error::custom))
        .transpose()
}


/// From: https://github.com/serde-rs/json/issues/160#issuecomment-841344394
fn for_each<'de, D, T, F>(deserializer: D, f: F)
    -> std::result::Result<(), D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
    F: FnMut(T),
{
    struct SeqVisitor<T, F>(F, PhantomData<T>);

    impl<'de, T, F> Visitor<'de> for SeqVisitor<T, F>
    where
        T: Deserialize<'de>,
        F: FnMut(T),
    {
        type Value = ();

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nonempty sequence")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> std::result::Result<(), A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some(value) = seq.next_element::<T>()? {
                self.0(value)
            }
            Ok(())
        }
    }
    let visitor = SeqVisitor(f, PhantomData);
    deserializer.deserialize_seq(visitor)
}