use std::env;

use crate::walker::Walker;

pub struct W;

impl Walker for W {
    fn get_root(&self) -> std::io::Result<()> {
        let path = env::current_dir()?;
        println!("The current directory is {}", path.display());
        Ok(())
    }
}
