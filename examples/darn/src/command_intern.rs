use crate::DarnCommand;
use anyhow::{Context, Result};
use clap::{App, Arg, ArgGroup, ArgMatches, SubCommand};
use daml::lf::{DamlLfPackage, DarFile};
use itertools::Itertools;
use prettytable::color::Color;
use prettytable::format;
use prettytable::{color, Attr, Cell, Row, Table};
use std::str::FromStr;

/// Darn command for displaying interned strings and dotted names.
pub struct CommandIntern {}

impl DarnCommand for CommandIntern {
    fn name(&self) -> &str {
        "intern"
    }

    fn args<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name("intern")
            .about("Show interned strings and dotted names in a dar")
            .arg(Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1))
            .arg(Arg::with_name("string").short("s").long("string").help("Show interned strings"))
            .arg(Arg::with_name("dotted").short("d").long("dotted").help("Show interned dotted names"))
            .arg(
                Arg::with_name("index")
                    .short("i")
                    .long("index")
                    .multiple(true)
                    .use_delimiter(true)
                    .takes_value(true)
                    .required(false)
                    .help("the intern indices"),
            )
            .arg(
                Arg::with_name("show-mangled")
                    .short("f")
                    .long("show-mangled")
                    .required(false)
                    .help("show mangled names"),
            )
            .arg(Arg::with_name("order-by-index").required(false).long("order-by-index").help("order by index"))
            .arg(Arg::with_name("order-by-name").required(false).long("order-by-name").help("order by name"))
            .group(ArgGroup::with_name("mode").required(true).arg("string").arg("dotted"))
            .group(ArgGroup::with_name("order").required(false).arg("order-by-index").arg("order-by-name"))
    }

    fn execute(&self, matches: &ArgMatches<'_>) -> Result<()> {
        let dar_path = matches.value_of("dar").unwrap();
        let filter: Vec<usize> = matches
            .values_of("index")
            .unwrap_or_default()
            .map(|i|usize::from_str(i).context(format!("parsing index from '{}'", i)))
            .collect::<Result<Vec<_>>>()?;
        let show_mangled = matches.is_present("show-mangled");
        let sort = match (matches.is_present("order-by-index"), matches.is_present("order-by-name")) {
            (true, false) => SortOrder::ByIndex,
            _ => SortOrder::ByName,
        };
        if matches.is_present("dotted") {
            intern_dotted(dar_path, show_mangled, &sort, filter.as_slice())
        } else if matches.is_present("string") {
            intern_string(dar_path, show_mangled, &sort, filter.as_slice())
        } else {
            unreachable!()
        }
    }
}

enum SortOrder {
    ByIndex,
    ByName,
}

fn intern_string(dar_path: &str, show_mangled: bool, sort_order: &SortOrder, filter: &[usize]) -> Result<()> {
    let dar = DarFile::from_file(dar_path)?;
    match dar.main.payload.package {
        DamlLfPackage::V1(package) => {
            let mut res: Vec<_> = package
                .interned_strings
                .iter()
                .enumerate()
                .filter_map(|(idx, rendered)| {
                    if (filter.is_empty() || filter.contains(&idx)) && (show_mangled || !rendered.contains('$')) {
                        Some((idx, rendered))
                    } else {
                        None
                    }
                })
                .collect();
            if res.is_empty() {
                println!("no interned strings matched indices {}", filter.iter().join(", "));
                return Ok(());
            }
            if let SortOrder::ByName = sort_order {
                res.sort_by(|(_, rendered_a), (_, rendered_b)| rendered_a.cmp(rendered_b));
            } else {
                res.sort_by(|(index_a, _), (index_b, _)| index_a.cmp(index_b));
            }
            let mut table = Table::new();
            table.set_titles(Row::new(vec!["index", "rendered"].into_iter().map(Cell::new).collect()));
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            res.iter().for_each(|(idx, rendered)| {
                table.add_row(string_row(idx.to_string().as_str(), rendered, pick_color(rendered)));
            });
            table.printstd();
        },
    }
    Ok(())
}

fn intern_dotted(dar_path: &str, show_mangled: bool, sort_order: &SortOrder, filter: &[usize]) -> Result<()> {
    let dar = DarFile::from_file(dar_path)?;
    match dar.main.payload.package {
        DamlLfPackage::V1(package) => {
            let mut res: Vec<_> = package
                .interned_dotted_names
                .iter()
                .enumerate()
                .filter_map(|(idx, dt)| {
                    if filter.is_empty() || filter.contains(&idx) {
                        let segments = dt
                            .segments_interned_str
                            .iter()
                            .map(|&i| format!("{}({})", package.interned_strings[i as usize], i))
                            .join(".");
                        let rendered =
                            dt.segments_interned_str.iter().map(|&i| &package.interned_strings[i as usize]).join(".");
                        if show_mangled || !rendered.contains('$') {
                            Some((idx, rendered, segments))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            if res.is_empty() {
                println!("no interned dotted names matched indices {}", filter.iter().join(", "));
                return Ok(());
            }
            if let SortOrder::ByName = sort_order {
                res.sort_by(|(_, rendered_a, _), (_, rendered_b, _)| rendered_a.cmp(rendered_b));
            } else {
                res.sort_by(|(index_a, ..), (index_b, ..)| index_a.cmp(index_b));
            }
            let mut table = Table::new();
            table.set_titles(Row::new(vec!["index", "rendered", "segments"].into_iter().map(Cell::new).collect()));
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            res.iter().for_each(|(idx, rendered, segments)| {
                table.add_row(dotted_row(idx.to_string().as_str(), rendered, segments, pick_color(rendered)));
            });
            table.printstd();
        },
    }
    Ok(())
}

fn string_row(idx: &str, rendered: &str, color: color::Color) -> Row {
    Row::new(vec![cell(idx, color), cell(rendered, color)])
}

fn dotted_row(idx: &str, rendered: &str, segments: &str, color: color::Color) -> Row {
    Row::new(vec![cell(idx, color), cell(rendered, color), cell(segments, color)])
}

fn cell(data: &str, color: color::Color) -> Cell {
    Cell::new(data).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color))
}

fn pick_color(data: &str) -> Color {
    if data.contains('$') {
        color::BLUE
    } else {
        color::WHITE
    }
}
