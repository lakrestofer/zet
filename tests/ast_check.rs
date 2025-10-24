use insta::assert_yaml_snapshot;
use insta::glob;
use std::fs;
use zet::core::parser::FrontMatterFormat;
use zet::core::parser::FrontMatterParser;

#[test]
fn test_input_files() {
    glob!("input_files/*.md", |path| {
        let input = fs::read_to_string(path).unwrap();

        let frontmatter_parser = FrontMatterParser::new(FrontMatterFormat::Toml);
        let content_parser = zet::core::parser::DocumentParser::new();

        let res = zet::core::parser::parse(frontmatter_parser, content_parser, input).unwrap();

        assert_yaml_snapshot!(res);
    });
}
