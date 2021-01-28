use anyhow::Result;

pub fn module(_dar_path: &str, _module: Option<&str>) -> Result<()> {
    // fn cell(data: &str, color: color::Color) -> Cell {
    //     Cell::new(data).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color))
    // }
    // fn row(name: &str, version: &str, package_id: &str, language_version: &str, color: color::Color) -> Row {
    //     Row::new(vec![cell(name, color), cell(version, color), cell(package_id, color), cell(language_version,
    // color)]) }
    // let dar = DarFile::from_file(dar_path).unwrap();
    // let package_name = "TestingTypes";
    // dar.apply(|archive| {
    // let mut table = Table::new();
    // table.set_titles(Row::new(vec!["name", "version", "package_id", "lf"].into_iter().map(Cell::new).collect()));
    // table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    // let package = archive.packages.get(package_name).unwrap();

    // package.root_module

    // let name = package.name;
    // let version = package.version.unwrap_or_else(|| "n/a");
    // let package_id = package.package_id;
    // let language_version = &package.language_version.to_string();
    // if package.package_id == dar.main.hash {
    //     table.add_row(row(name, version, package_id, language_version, color::GREEN));
    // } else {
    //     table.add_row(row(name, version, package_id, language_version, color::WHITE));
    // }

    // table.printstd();
    // })
    // .unwrap();
    Ok(())
}
