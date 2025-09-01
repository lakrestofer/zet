use std::ops::Range;

use crate::*;
use gray_matter::{
    Matter,
    engine::{JSON, TOML, YAML},
};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};

pub fn parse(
    frontmatter_parser: FrontMatterParser,
    document_parser: DocumentParser,
    document: String,
) -> Result<()> {
    let (frontmatter, content) = frontmatter_parser.parse(document);

    log::debug!("frontmatter: {:?}", frontmatter);

    let events = document_parser.parse(content);

    Ok(())
}

#[derive(Copy, Serialize, Deserialize, Clone, PartialEq, Eq, Default, Debug)]
pub enum FrontMatterFormat {
    #[default]
    #[serde(rename = "toml")]
    Toml,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "yaml")]
    Yaml,
}

pub enum FrontMatterParser {
    TomlParser(Matter<TOML>),
    JsonParser(Matter<JSON>),
    YamlParser(Matter<YAML>),
}

impl FrontMatterParser {
    pub fn new(format: FrontMatterFormat) -> Self {
        match format {
            FrontMatterFormat::Toml => FrontMatterParser::TomlParser(Matter::<TOML>::new()),
            FrontMatterFormat::Json => FrontMatterParser::JsonParser(Matter::<JSON>::new()),
            FrontMatterFormat::Yaml => FrontMatterParser::YamlParser(Matter::<YAML>::new()),
        }
    }

    pub fn parse(&self, content: String) -> (Option<serde_json::Value>, String) {
        let result = match self {
            FrontMatterParser::TomlParser(matter) => matter.parse(&content),
            FrontMatterParser::JsonParser(matter) => matter.parse(&content),
            FrontMatterParser::YamlParser(matter) => matter.parse(&content),
        };
        (result.data.map(|p| p.into()), result.content)
    }
}

/// The document parser, parameterized over what happens when it encounters each event
#[repr(transparent)]
pub struct DocumentParser {
    options: Options,
}

impl Default for DocumentParser {
    fn default() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_MATH);
        options.insert(Options::ENABLE_WIKILINKS);
        Self { options }
    }
}

impl DocumentParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&self, document: String) -> Result<()> {
        let parser = Parser::new_ext(&document, self.options);

        let parser_with_offset = parser.into_offset_iter();

        let mut heading_processor: HeadingProcessor<DefaultHeadingHandler> =
            HeadingProcessor::default();

        for (event, range) in parser_with_offset {
            // log::debug!("location: {:?}, event: {:?}", range, event);
            heading_processor.process(event, range);
        }

        Ok(())
    }
}

#[derive(Default)]
struct HeadingProcessor<T: HeadingHandler = DefaultHeadingHandler> {
    state: HeadingParserState,
    handler: T,
}

impl<T: HeadingHandler> HeadingProcessor<T> {
    pub fn process(&mut self, event: Event<'_>, range: Range<usize>) {
        use HeadingParserState::*;

        if let Some(new_state) = match (&self.state, event) {
            (
                Start,
                Event::Start(Tag::Heading {
                    level,
                    id,
                    classes,
                    attrs,
                }),
            ) => Some(Content {
                level,
                id: id.map(|id| id.into_string()),
                classes: classes
                    .into_iter()
                    .map(|class| class.into_string())
                    .collect(),
                attrs: attrs
                    .into_iter()
                    .map(|(key, value)| (key.into_string(), value.map(|v| v.into_string())))
                    .collect(),
            }),
            (
                Content {
                    level,
                    id,
                    classes,
                    attrs,
                },
                Event::Text(content),
            ) => Some(End {
                level: *level,
                id: id.clone(),
                classes: classes.clone(),
                attrs: attrs.clone(),
                content: content.to_string(),
            }),
            (
                Content {
                    level,
                    id,
                    classes,
                    attrs,
                },
                Event::End(TagEnd::Heading(_)),
            ) => {
                self.handler.handle(Heading::new(
                    *level,
                    id.clone(),
                    classes.clone(),
                    attrs.clone(),
                    "".into(),
                ));
                Some(Start)
            }
            (
                End {
                    level,
                    id,
                    classes,
                    attrs,
                    content,
                },
                Event::End(TagEnd::Heading(_)),
            ) => {
                self.handler.handle(Heading::new(
                    *level,
                    id.clone(),
                    classes.clone(),
                    attrs.clone(),
                    content.clone(),
                ));
                Some(Start)
            }
            _ => None,
        } {
            self.state = new_state
        }
    }
}

#[derive(Default)]
enum HeadingParserState {
    #[default]
    Start, // we are waiting for a Start(Tag::Heading) event
    Content {
        level: HeadingLevel,
        id: Option<String>,
        classes: Vec<String>,
        attrs: Vec<(String, Option<String>)>,
    }, // we are waiting for a paragraph
    End {
        level: HeadingLevel,
        id: Option<String>,
        classes: Vec<String>,
        attrs: Vec<(String, Option<String>)>,
        content: String,
    }, // we are waiting for an End(Tag::Heading) event
}
trait HeadingHandler {
    fn handle(&mut self, heading: Heading);
}

#[derive(Copy, Clone, Default, Debug)]
struct DefaultHeadingHandler;

impl HeadingHandler for DefaultHeadingHandler {
    fn handle(&mut self, heading: Heading) {
        log::debug!("handling heading: {:?}", heading);
    }
}

#[derive(Debug)]
struct Heading {
    level: HeadingLevel,
    id: Option<String>,
    classes: Vec<String>,
    attrs: Vec<(String, Option<String>)>,
    content: String,
}

impl Heading {
    pub fn new(
        level: HeadingLevel,
        id: Option<String>,
        classes: Vec<String>,
        attrs: Vec<(String, Option<String>)>,
        content: String,
    ) -> Self {
        Self {
            level,
            id,
            classes,
            attrs,
            content,
        }
    }
}

pub type FrontMatter = serde_json::Value;
