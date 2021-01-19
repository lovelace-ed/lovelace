//! Tests to check that Malvolio renders structs to the correct HTML strings.
#[test]
fn test_headings() {
    let document = malvolio::Html::default()
        .head(malvolio::Head::default().child(malvolio::Title::new("Some title")))
        .body(
            malvolio::Body::default()
                .child(malvolio::H6::new("Some heading"))
                .child(malvolio::H6::new("Some other heading"))
                .child(malvolio::H5::new("Some other other heading"))
                .child(
                    malvolio::H4::new("Some other other other heading")
                        .attribute("class", "heading-class"),
                ),
        )
        .to_string();
    let document = scraper::Html::parse_document(&document);
    let h6_selector = scraper::Selector::parse("h6").unwrap();
    let h5_selector = scraper::Selector::parse("h5").unwrap();
    let h4_selector = scraper::Selector::parse("h4").unwrap();
    assert_eq!(document.select(&h6_selector).collect::<Vec<_>>().len(), 2);
    assert_eq!(document.select(&h5_selector).collect::<Vec<_>>().len(), 1);
    assert_eq!(
        document
            .select(&h6_selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap(),
        "Some heading"
    );
    assert_eq!(
        document
            .select(&h5_selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap(),
        "Some other other heading"
    );
    let h4 = document.select(&h4_selector).next().unwrap();
    assert_eq!(h4.text().next().unwrap(), "Some other other other heading");
    assert_eq!(h4.value().attr("class").unwrap(), "heading-class");
}
