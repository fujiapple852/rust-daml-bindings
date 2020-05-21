use anyhow::Result;
use daml::lf::{DamlLfPackage, DarFile};
use itertools::Itertools;
use std::str::FromStr;
use prettytable::{Table, Row, color, Attr, Cell};
use prettytable::format;

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

pub enum SortOrder {
    ByIndex,
    ByName,
}

pub fn intern_all_dotted(dar_path: &str, hide_mangled: bool, sort_order: SortOrder) -> Result<()> {
    let dar = DarFile::from_file(dar_path)?;
    match dar.main.payload.package {
        DamlLfPackage::V1(p) => {
            let mut res: Vec<_> = p.interned_dotted_names.iter().enumerate().map(|(idx, dt)| {
                let final_str = dt
                    .segments_interned_str
                    .iter()
                    .map(|&i| &p.interned_strings[i as usize])
                    .join(".");
                (idx, final_str)
            }).collect();
            if let SortOrder::ByName = sort_order {
                res.sort_by(|(_, a), (_, b)| a.cmp(b));
            } else {
                res.sort_by(|(a, _), (b, _)| a.cmp(b));
            }
            let mut table = Table::new();
            table.set_titles(Row::new(vec!["dotted", "index"].into_iter().map(Cell::new).collect()));
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            res.iter().for_each(|(idx, s)| {
                if s.contains("$") {
                    if !hide_mangled {
                        table.add_row(row(s, idx.to_string().as_str(), color::BLUE));
                    }
                } else {
                    table.add_row(row(s, idx.to_string().as_str(), color::WHITE));
                }
            });
            table.printstd();
        },
    }
    Ok(())
}

fn row(dotted: &str, idx: &str, color: color::Color) -> Row {
    Row::new(vec![cell(dotted, color), cell(idx, color)])
}

fn cell(data: &str, color: color::Color) -> Cell {
    Cell::new(data).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color))
}