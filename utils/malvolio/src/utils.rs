use std::{borrow::Cow, collections::HashMap};

#[cfg(feature = "yew")]
pub fn write_attributes_to_vtag(
    attrs: &HashMap<&'static str, Cow<'static, str>>,
    vtag: &mut ::yew::virtual_dom::VTag,
) {
    for (key, value) in attrs.into_iter() {
        vtag.add_attribute(key, value);
    }
}

pub fn write_attributes(
    attrs: &HashMap<&'static str, Cow<'static, str>>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    for (key, value) in attrs {
        f.write_str(key)?;
        f.write_str("=\"")?;
        match value {
            Cow::Borrowed(b) => {
                f.write_str(b)?;
            }
            Cow::Owned(string) => {
                f.write_str(&string)?;
            }
        }
        f.write_str("\"")?;
    }
    Ok(())
}
