/// A simple tool to convert big JSON array files to JSON Lines.
/// It just reads a big JSON array from stdin and writes JSONL
/// to stdout, that's it.

use core::fmt;
use std::{error, io};
use serde::{Deserialize, Deserializer};
use serde::de::{SeqAccess, Visitor};
use serde_json::{self, Value};
use std::marker::PhantomData;


type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() -> Result<()> {
    let mut deserializer =
        serde_json::Deserializer::from_reader(io::stdin());
    for_each(&mut deserializer, |record: Value| {
        let output = serde_json::to_string(&record);
        output.map(|s| println!("{}", s)).unwrap()
    }).map_err(|e| e.into())
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