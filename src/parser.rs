pub mod ast_nodes;

use std::ops::Range;

use crate::{
    parser::ast_nodes::{Heading, *},
    *,
};
use gray_matter::{
    Matter,
    engine::{JSON, TOML, YAML},
};
use pulldown_cmark::{
    CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, OffsetIter, Options, Parser, Tag, TagEnd,
};
use serde::{Deserialize, Serialize};

pub struct ParserIterator<'a> {
    inner: OffsetIter<'a>,
}

impl<'a> Iterator for ParserIterator<'a> {
    type Item = (Event<'a>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        let e = self.inner.next();
        if let Some(e) = &e {
            log::debug!("event: {:?}", e);
        }
        e
    }
}

pub fn parse(
    frontmatter_parser: FrontMatterParser,
    document_parser: DocumentParser,
    document: String,
) -> Result<()> {
    let (frontmatter, content) = frontmatter_parser.parse(document);

    log::debug!("frontmatter: {:?}", frontmatter);

    let _events = document_parser.parse(content);

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
        // options.insert(Options::ENABLE_GFM);
        // options.remove(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        Self { options }
    }
}

impl DocumentParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&self, document: String) -> Result<()> {
        let parser = Parser::new_ext(&document, self.options);

        let mut parser_with_offset = ParserIterator {
            inner: parser.into_offset_iter(),
        };

        let mut children: Vec<ast_nodes::Node> = Vec::new();

        while let Some((event, range)) = parser_with_offset.next() {
            children.push(parse_event(event, range, &mut parser_with_offset)?);
        }

        // log::debug!("children: {:?}", children);

        Ok(())
    }
}

fn parse_event(event: Event, range: Range<usize>, iter: &mut ParserIterator) -> Result<Node> {
    match event {
        Event::Start(tag) => parse_start(tag, range, iter),
        Event::End(_) => Ok(NotImplemented::new(range).into()),
        Event::Text(cow_str) => parse_text(range, iter),
        Event::Code(cow_str) => Ok(NotImplemented::new(range).into()),
        Event::InlineMath(cow_str) => Ok(NotImplemented::new(range).into()),
        Event::DisplayMath(cow_str) => Ok(NotImplemented::new(range).into()),
        Event::Html(cow_str) => Ok(NotImplemented::new(range).into()),
        Event::InlineHtml(cow_str) => Ok(NotImplemented::new(range).into()),
        Event::FootnoteReference(cow_str) => Ok(NotImplemented::new(range).into()),
        Event::SoftBreak => Ok(NotImplemented::new(range).into()),
        Event::HardBreak => Ok(NotImplemented::new(range).into()),
        Event::Rule => Ok(NotImplemented::new(range).into()),
        Event::TaskListMarker(_) => Ok(NotImplemented::new(range).into()),
    }
}

fn parse_text(
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    todo!()
}

fn parse_start(start_tag: Tag, range: Range<usize>, iter: &mut ParserIterator) -> Result<Node> {
    match start_tag {
        Tag::Heading {
            level,
            id,
            classes,
            attrs,
        } => parse_heading(level, id, classes, attrs, range, iter),
        Tag::Paragraph => parse_paragraph(range, iter),
        Tag::BlockQuote(kind) => parse_blockquote(kind, range, iter),
        Tag::CodeBlock(kind) => parse_code_block(kind, range, iter),
        Tag::HtmlBlock => parse_htmlblock(range, iter),
        Tag::List(n) => parse_list(n, range, iter),
        Tag::Item => parse_item(range, iter),
        Tag::FootnoteDefinition(str) => parse_footnote_def(str, range, iter),
        Tag::DefinitionList => parse_def_list(range, iter),
        Tag::DefinitionListTitle => parse_def_list_title(range, iter),
        Tag::DefinitionListDefinition => parse_def_list_def(range, iter),
        Tag::Table(alignments) => parse_table(alignments, range, iter),
        Tag::TableHead => parse_table_head(range, iter).map(|h| h.into()),
        Tag::TableRow => parse_table_row(range, iter).map(|r| r.into()),
        Tag::TableCell => parse_table_cell(range, iter).map(|c| c.into()),
        Tag::Emphasis => parse_text_decor(TextDecorationKind::Emphasis, range, iter),
        Tag::Strong => parse_text_decor(TextDecorationKind::Strong, range, iter),
        Tag::Strikethrough => parse_text_decor(TextDecorationKind::Strikethrough, range, iter),
        Tag::Superscript => parse_text_decor(TextDecorationKind::Superscript, range, iter),
        Tag::Subscript => parse_text_decor(TextDecorationKind::Subscript, range, iter),
        Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        } => parse_link(link_type, dest_url, title, id, range, iter),
        Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        } => parse_image(link_type, dest_url, title, id, range, iter),
        Tag::MetadataBlock(metadata_block_kind) => {
            parse_metadata_block(metadata_block_kind, range, iter)
        }
    }
}

fn parse_metadata_block(
    kind: pulldown_cmark::MetadataBlockKind,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Image) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(MetadataBlock::new(range, kind).into())
}

fn parse_image(
    link_type: LinkType,
    dest_url: CowStr<'_>,
    title: CowStr<'_>,
    id: CowStr<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    while let Some((event, _)) = iter.next() {
        match event {
            Event::End(TagEnd::Image) => break,
            _ => {} // ignore link children
        }
    }
    match link_type {
        LinkType::Inline => Ok(InlineImage::new(range).into()),
        LinkType::Reference
        | LinkType::ReferenceUnknown
        | LinkType::Collapsed
        | LinkType::CollapsedUnknown => Ok(ReferenceImage::new(range).into()),
        _ => Err(Error::ParseError("not implemented yet".into())),
    }
}

fn parse_link(
    link_type: LinkType,
    dest_url: CowStr<'_>,
    title: CowStr<'_>,
    id: CowStr<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> Result<Node> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Link) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    match link_type {
        LinkType::Inline => Ok(InlineLink::new(
            range,
            children,
            dest_url.to_string(),
            Some(title.to_string()),
        )
        .into()),
        LinkType::Reference
        | LinkType::ReferenceUnknown
        | LinkType::Collapsed
        | LinkType::CollapsedUnknown => {
            Ok(ReferenceLink::new(range, children, "todo".into()).into())
        }
        LinkType::Shortcut | LinkType::ShortcutUnknown => {
            Ok(ShortcutLink::new(range, children).into())
        }
        LinkType::Autolink | LinkType::Email => Ok(AutoLink::new(range, children).into()),
        LinkType::WikiLink { .. } => Ok(WikiLink::new(range, children).into()),
    }
}

// TODO: do we need to constraint the end tag such that we only search for the
// same end as `kind`?
fn parse_text_decor(
    kind: TextDecorationKind,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> Result<Node> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Emphasis) => break,
            Event::End(TagEnd::Strong) => break,
            Event::End(TagEnd::Strikethrough) => break,
            Event::End(TagEnd::Superscript) => break,
            Event::End(TagEnd::Subscript) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(TextDecoration {
        range,
        kind,
        children,
    }
    .into())
}

fn parse_table_cell(range: Range<usize>, iter: &mut ParserIterator<'_>) -> Result<TableCell> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::TableCell) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(TableCell::new(range, children))
}

fn parse_table_row(
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<TableRow, Error> {
    let mut cells = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::TableRow) => break,
            Event::Start(Tag::TableCell) => cells.push(parse_table_cell(range, iter)?),
            _ => return Err(Error::ParseError("expected table row".into())),
        }
    }

    Ok(TableRow::new(range, cells))
}

fn parse_table(
    alignments: Vec<pulldown_cmark::Alignment>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let header = if let Some((Event::Start(Tag::TableHead), range)) = iter.next() {
        parse_table_head(range, iter)?
    } else {
        return Err(Error::ParseError(
            "header expected but recieved none".into(),
        ));
    };

    let mut rows = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Table) => break,
            Event::Start(Tag::TableRow) => rows.push(parse_table_row(range, iter)?),
            _ => return Err(Error::ParseError("expected table row".into())),
        }
    }

    Ok(Table::new(
        range,
        header,
        alignments
            .into_iter()
            .map(|a| match a {
                pulldown_cmark::Alignment::None => ColumnAlignment::None,
                pulldown_cmark::Alignment::Left => ColumnAlignment::Left,
                pulldown_cmark::Alignment::Center => ColumnAlignment::Center,
                pulldown_cmark::Alignment::Right => ColumnAlignment::Right,
            })
            .collect(),
        rows,
    )
    .into())
}

fn parse_table_head(range: Range<usize>, iter: &mut ParserIterator<'_>) -> Result<TableHead> {
    todo!()
}

fn parse_def_list_def(
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::DefinitionListDefinition) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }
    Ok(DefinitionListDefinition::new(range, children).into())
}

fn parse_def_list_title(
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::DefinitionListTitle) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(DefinitionListTitle::new(range, children).into())
}

fn parse_def_list(
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::DefinitionList) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(DefinitionList::new(range, children).into())
}

fn parse_footnote_def(
    name: CowStr<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::FootnoteDefinition) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(FootnoteDefinition::new(range, name.to_string(), children).into())
}

fn parse_item(
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let mut children = Vec::new();
    let mut sub_lists = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Item) => break,
            Event::Start(Tag::List(_)) => sub_lists.push(parse_event(event, range, iter)?),
            _ => {
                children.append(&mut sub_lists);
                children.push(parse_event(event, range, iter)?)
            }
        }
    }

    Ok(Item::new(range, children, sub_lists).into())
}

fn parse_list(
    n: Option<u64>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::List(_)) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(List::new(range, n, children).into())
}

fn parse_htmlblock(
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::HtmlBlock) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(Html::new(range, children).into())
}

fn parse_code_block(
    kind: pulldown_cmark::CodeBlockKind<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::CodeBlock) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    let is_fenced = matches!(kind, CodeBlockKind::Fenced(_));
    let tag = match kind {
        CodeBlockKind::Indented => None,
        CodeBlockKind::Fenced(tag) => {
            let tag = String::from(tag.as_ref().trim());
            if tag.is_empty() { None } else { Some(tag) }
        }
    };

    Ok(CodeBlock::new(range, tag, is_fenced, children).into())
}

fn parse_blockquote(
    kind: Option<pulldown_cmark::BlockQuoteKind>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::BlockQuote(_)) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(BlockQuote::new(range, children).into())
}

fn parse_paragraph(
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, Error> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Paragraph) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(Paragraph::new(range, children).into())
}

fn parse_heading(
    level: HeadingLevel,
    id: Option<CowStr<'_>>,
    classes: Vec<CowStr<'_>>,
    attrs: Vec<(CowStr<'_>, Option<CowStr<'_>>)>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> Result<Node> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Heading(end_level)) => {
                if end_level == level {
                    break;
                }
            }
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(Heading {
        id: id.map(|s| s.to_string()),
        range: range,
        level: level as u8,
        children,
        classes: classes.iter().map(|s| s.to_string()).collect(),
        attributes: attrs
            .iter()
            .map(|(k, v)| (k.to_string(), v.as_ref().map(|s| s.to_string())))
            .collect(),
    }
    .into())
}

pub type FrontMatter = serde_json::Value;
