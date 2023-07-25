//! A table of statuses.

use crate::availability::{AvailabilityData, AvailabilityRow};
use chrono::NaiveDate;
use std::{borrow::Borrow, iter};

/// A ready-to-render table of packages statuses per target.
#[derive(Debug, serde::Serialize)]
pub struct Table<'a, Additional: 'a = ()> {
    /// A target which the table is built for.
    pub current_target: &'a str,
    /// Table's title.
    pub title: Vec<String>,
    /// A list of packages and their availabilities sorted by package name in an ascending order.
    pub packages_availability: Vec<AvailabilityRow<'a>>,
    /// Additional data to render.
    pub additional: Additional,
}

/// Sorts a given container (in a form of an iterator) into a vector of its items in an ascending
/// order.
fn sort<T: Ord>(data: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut vec: Vec<_> = data.into_iter().collect();
    vec.sort_unstable();
    vec
}

/// Table builder.
#[derive(Debug, Clone)]
pub struct TableBuilder<'a, Dates = iter::Empty<NaiveDate>, DateFmt = &'static str, Additional = ()>
{
    data: &'a AvailabilityData,
    target: &'a str,
    dates: Dates,
    first_cell: String,
    date_fmt: DateFmt,
    additional_data: Additional,
}

impl<'a> TableBuilder<'a> {
    /// Initializes a table builder for given data and target.
    ///
    /// By default list of `dates` is empty (which is probably not what you want), as is the first
    /// cell in the table (which is probably what you want).
    pub fn default(data: &'a AvailabilityData, target: &'a str) -> Self {
        TableBuilder {
            data,
            target,
            dates: iter::empty(),
            first_cell: String::new(),
            date_fmt: "%Y-%m-%d",
            additional_data: (),
        }
    }
}

impl<'a, Dates, DateFmt, Additional> TableBuilder<'a, Dates, DateFmt, Additional> {
    /// Sets the very first cell in the table (top-left corner).
    pub fn first_cell(self, first_cell: &impl ToString) -> Self {
        TableBuilder {
            first_cell: first_cell.to_string(),
            ..self
        }
    }

    /// Sets the dates range to a given iterator over dates.
    ///
    /// Please note that the iterator (not your object, but rather the iterator it resolves to)
    /// should be cloneable. If you provide a `Vec`, you are on the safe side :)
    pub fn dates<I>(self, dates: I) -> TableBuilder<'a, I::IntoIter, DateFmt, Additional>
    where
        I: IntoIterator,
        I::IntoIter: Clone,
        I::Item: Borrow<NaiveDate>,
    {
        TableBuilder {
            data: self.data,
            target: self.target,
            dates: dates.into_iter(),
            first_cell: self.first_cell,
            date_fmt: self.date_fmt,
            additional_data: self.additional_data,
        }
    }

    /// Sets a format in which the dates will be formatted. Here's a formatting syntax for your
    /// convenience:
    /// [chrono::format::strftime](https://docs.rs/chrono/0.4.6/chrono/format/strftime/index.html).
    ///
    /// The default is `"%Y-%m-%d"`.
    pub fn date_format<T>(self, date_fmt: T) -> TableBuilder<'a, Dates, T, Additional>
    where
        T: AsRef<str>,
    {
        TableBuilder {
            data: self.data,
            target: self.target,
            dates: self.dates,
            first_cell: self.first_cell,
            date_fmt,
            additional_data: self.additional_data,
        }
    }

    /// Sets the additional data.
    pub fn additional<NewAdditional>(
        self,
        data: NewAdditional,
    ) -> TableBuilder<'a, Dates, DateFmt, NewAdditional> {
        TableBuilder {
            data: self.data,
            target: self.target,
            dates: self.dates,
            first_cell: self.first_cell,
            date_fmt: self.date_fmt,
            additional_data: data,
        }
    }

    /// Builds a table using all the supplied data.
    pub fn build(self) -> Table<'a, Additional>
    where
        Dates: Iterator + Clone,
        Dates::Item: Borrow<NaiveDate>,
        DateFmt: AsRef<str>,
    {
        Table::new(
            self.data,
            self.target,
            &self.dates,
            self.first_cell,
            self.date_fmt.as_ref(),
            self.additional_data,
        )
    }
}

impl<'a> Table<'a> {
    /// Initializes a table builder.
    pub fn builder(data: &'a AvailabilityData, target: &'a str) -> TableBuilder<'a> {
        TableBuilder::default(data, target)
    }
}

impl<'a, Additional> Table<'a, Additional> {
    /// Construct an availability table for a target for specific dates.
    fn new<I>(
        data: &'a AvailabilityData,
        target: &'a str,
        dates: &I,
        first_cell: String,
        date_fmt: &str,
        additional_data: Additional,
    ) -> Self
    where
        I: Iterator + Clone,
        I::Item: Borrow<NaiveDate>,
    {
        let title = iter::once(first_cell)
            .chain(
                dates
                    .clone()
                    .map(|date| date.borrow().format(date_fmt).to_string()),
            )
            .collect();
        let packages = sort(data.get_available_packages());
        let availability = packages
            .into_iter()
            .filter_map(|pkg| data.get_availability_row(target, pkg, dates.clone()))
            .collect();
        Table {
            current_target: target,
            title,
            packages_availability: availability,
            additional: additional_data,
        }
    }
}
