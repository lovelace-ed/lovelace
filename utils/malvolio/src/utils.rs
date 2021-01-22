use std::collections::HashMap;

#[cfg(feature = "yew")]
pub fn write_attributes_to_vtag(
    attrs: HashMap<&'static str, String>,
    vtag: &mut ::yew::virtual_dom::VTag,
) {
    for (key, value) in attrs.into_iter() {
        vtag.add_attribute(key, &value);
    }
}

pub fn write_attributes(
    attrs: &HashMap<&'static str, String>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    for (key, value) in attrs {
        f.write_str(key)?;
        f.write_str("=\"")?;
        f.write_str(&value)?;
        f.write_str("\"")?;
    }
    Ok(())
}
