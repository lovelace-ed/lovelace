/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

//! Handles timezones

use std::borrow::Cow;

use chrono_tz::TZ_VARIANTS;
use malvolio::prelude::{Div, Label, Name, Select, SelectOption, Value};

use crate::css_names::FORM_GROUP;

/// Creates a field from which a timezone can be selected which should be added to an existing form
/// (it is just an input field; not a free-standing form on its own).
pub fn timezone_form<T>(name: T, message: Option<&'static str>) -> Div
where
    T: Into<Cow<'static, str>>,
{
    Div::new()
        .attribute(malvolio::prelude::Class::from(FORM_GROUP))
        .child(Label::new(
            message.unwrap_or("Please select your timezone:"),
        ))
        .child(
            Select::default()
                .attribute(Name::new(name))
                .children(TZ_VARIANTS.iter().map(|timezone| {
                    SelectOption::default()
                        .attribute(Value::new(timezone.to_string()))
                        .text(timezone.to_string())
                })),
        )
}
