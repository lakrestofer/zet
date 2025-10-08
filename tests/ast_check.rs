use insta::assert_yaml_snapshot;
use insta::glob;
use std::fs;
use zet::parser::FrontMatterFormat;
use zet::parser::FrontMatterParser;

#[test]
fn test_input_files() {
    glob!("input_files/*.md", |path| {
        let input = fs::read_to_string(path).unwrap();

        let frontmatter_parser = FrontMatterParser::new(FrontMatterFormat::Toml);
        let content_parser = zet::parser::DocumentParser::new();

        let res = zet::parser::parse(frontmatter_parser, content_parser, input).unwrap();

        assert_yaml_snapshot!(res);
    });
}
