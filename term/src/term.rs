//! Printing data to a terminal.

use prettytable::{cell::Cell, color, row::Row, Attr::ForegroundColor, Table as PrettyTable};
use rustup_available_packages::table::Table as DataTable;
use std::iter;

fn cell_from_bool(val: bool) -> Cell {
    use prettytable::format::Alignment;
    if val {
        Cell::new_align("+", Alignment::CENTER).with_style(ForegroundColor(color::GREEN))
    } else {
        Cell::new_align("-", Alignment::CENTER).with_style(ForegroundColor(color::RED))
    }
}

fn row_from_pkgdata(package: &str, data: Vec<bool>) -> Row {
    let first = Cell::new(package).with_style(ForegroundColor(color::CYAN));
    let rest = data.into_iter().map(cell_from_bool);
    Row::new(iter::once(first).chain(rest).collect())
}

/// Prints availability history as a coloured table to the terminal.
pub fn print_table(source: DataTable) {
    let first = iter::once(Row::from(source.title));
    let rest = source
        .packages_availability
        .into_iter()
        .map(|(pkg, avail)| row_from_pkgdata(pkg, avail));
    let table: PrettyTable = first.chain(rest).collect();
    table.printstd();
}
