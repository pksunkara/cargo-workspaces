use crate::utils::Writer;
use serde::Serialize;
use std::io::Result;

pub trait Listable: Serialize {
    fn json(&self, w: &mut Writer) -> Result<()> {
        w.none(&serde_json::to_string_pretty(self)?)?;
        w.none("\n")?;

        Ok(())
    }

    fn list(&self, w: &mut Writer, json: bool, long: bool, all: bool) -> Result<()>;
}
