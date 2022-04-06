use crate::{isa::aarch64::Aarch64, module::DisplayAsm};
use std::fmt::{Display, Formatter, Result};

impl Display for DisplayAsm<'_, Aarch64> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.print(f)
    }
}

impl DisplayAsm<'_, Aarch64> {
    fn print(&self, f: &mut Formatter) -> Result {
        writeln!(f, "  .text")?;
        Ok(())
    }
}
