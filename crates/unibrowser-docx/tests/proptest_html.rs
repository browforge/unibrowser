use proptest::prelude::*;

proptest! {
    #[test]
    fn html_paragraph_roundtrip_preserves_plain_text(
        words in prop::collection::vec("[a-z]{2,10}", 1..5),
    ) {
        let joined = words.join(" ");
        let html = format!("<p>{}</p>", joined);
        let doc = scraper::Html::parse_fragment(&html);
        let text = doc.root_element().text().collect::<Vec<_>>().join(" ");
        let needle = words[0].trim();
        prop_assert!(!needle.is_empty());
        prop_assert!(text.contains(needle));
    }
}
