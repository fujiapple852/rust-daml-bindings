use anyhow::Result;
use daml::lf::{DamlLfPackage, DarFile};
use itertools::Itertools;
use std::str::FromStr;

pub fn intern_string(dar_path: &str, index: &str) -> Result<()> {
    let dar = DarFile::from_file(dar_path)?;
    let index = usize::from_str(index).unwrap();
    match dar.main.payload.package {
        DamlLfPackage::V1(p) => {
            println!("{}", p.interned_strings[index]);
        },
    }
    Ok(())
}

pub fn intern_dotted(dar_path: &str, index: &str) -> Result<()> {
    let dar = DarFile::from_file(dar_path)?;
    let index = usize::from_str(index).unwrap();
    match dar.main.payload.package {
        DamlLfPackage::V1(p) => {
            let final_str = p.interned_dotted_names[index]
                .segments_interned_str
                .iter()
                .map(|&i| &p.interned_strings[i as usize])
                .join(".");
            println!("{}", final_str);
        },
    }
    Ok(())
}
