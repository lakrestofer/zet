pub mod ast_nodes;

use crate::preamble::*;

use crate::core::parser::ast_nodes::*;
use clap::ValueEnum;
use color_eyre::eyre::eyre;
use gray_matter::{
    Matter,
    engine::{JSON, TOML, YAML},
};
use pulldown_cmark::{
    CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, OffsetIter, Options, Parser, Tag, TagEnd,
};
use serde::{Deserialize, Serialize};
use std::{
    iter::{FilterMap, Peekable},
    ops::Range,
};

// type Item = (Event<'a>, Range<usize>);

pub struct ParserIterator<'a> {
    inner: Peekable<OffsetIter<'a>>,
    text: &'a str,
}

impl<'a> Iterator for ParserIterator<'a> {
    type Item = (Event<'a>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut val = self.inner.next()?;
        while let (Event::SoftBreak, _) = val {
            val = self.inner.next()?;
        }
        Some(val)
    }
}

pub fn parse(
    frontmatter_parser: FrontMatterParser,
    document_parser: DocumentParser,
    document: String,
) -> Result<(Option<serde_json::Value>, Vec<Node>)> {
    let (frontmatter, content) = frontmatter_parser.parse(document);

    let events = document_parser.parse(content)?;

    Ok((frontmatter, events))
}

#[derive(Copy, Serialize, Deserialize, Clone, PartialEq, Eq, Default, Debug, ValueEnum)]
pub enum FrontMatterFormat {
    #[default]
    #[serde(rename = "yaml")]
    Yaml,
    #[serde(rename = "toml")]
    Toml,
    #[serde(rename = "json")]
    Json,
}

pub enum FrontMatterParser {
    TomlParser(Matter<TOML>),
    JsonParser(Matter<JSON>),
    YamlParser(Matter<YAML>),
}

impl ToString for FrontMatterFormat {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
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
        ((result.data.map(|p| p.into())), result.content)
    }
}

/// The document parser, parameterized over what happens when it encounters each event
#[repr(transparent)]
#[derive(Default)]
pub struct DocumentParser {
    pub options: DocumentParserOptions,
}

pub struct DocumentParserOptions(pub Options);

impl Default for DocumentParserOptions {
    fn default() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_MATH);
        options.insert(Options::ENABLE_WIKILINKS);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_GFM);
        Self(options)
    }
}

impl DocumentParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&self, document: String) -> Result<Vec<Node>> {
        let parser = Parser::new_ext(&document, self.options.0);

        let mut parser_with_offset = ParserIterator {
            inner: parser.into_offset_iter().peekable(),
            text: document.as_str(),
        };

        let mut nodes: Vec<ast_nodes::Node> = Vec::new();

        while let Some((event, range)) = parser_with_offset.next() {
            nodes.push(parse_event(event, range, &mut parser_with_offset)?);
        }

        Ok(nodes)
    }
}

fn parse_event(event: Event, range: Range<usize>, iter: &mut ParserIterator) -> Result<Node> {
    match event {
        Event::Start(tag) => parse_start(tag, range, iter),
        Event::Text(str) => Ok(Node::text(range, str.to_string())),
        Event::Code(str) => parse_code(str, range, iter),
        Event::InlineMath(str) => Ok(Node::inlinemath(range, str.into_string())),
        Event::DisplayMath(str) => Ok(Node::displaymath(range, str.into_string())),
        Event::Html(str) => Ok(Node::html(range, str.into_string())),
        Event::InlineHtml(str) => Ok(Node::html(range, str.into_string())),
        Event::FootnoteReference(str) => {
            Ok(Node::footnotereference(range, String::from(str.as_ref())))
        }
        Event::HardBreak => Ok(Node::hardbreak(range)),
        Event::Rule => Ok(Node::horizontalrule(range)),
        _ => Err(eyre!(
            "unexpected event! event should have been consumed by previous parse rule: {:?} - {:?}",
            event,
            range
        )),
    }
}

fn parse_code(
    _cow: CowStr<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> Result<Node> {
    let mut raw_text = &iter.text[range.clone()];
    while raw_text.starts_with("`") && raw_text.ends_with("`") {
        raw_text = &raw_text[1..(raw_text.len() - 1)];
    }
    Ok(Node::code(range, raw_text.to_string()))
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
        Tag::BlockQuote(_) => parse_blockquote(range, iter),
        Tag::CodeBlock(kind) => parse_code_block(kind, range, iter),
        Tag::HtmlBlock => parse_htmlblock(range, iter),
        Tag::List(n) => parse_list(n, range, iter),
        Tag::Item => parse_item(range, iter),
        Tag::FootnoteDefinition(id) => parse_footnote_def(id, range, iter),
        Tag::Table(alignments) => parse_table(alignments, range, iter),
        // Tag::TableHead => parse_table_head(range, iter).map(|h| h.into()),
        // Tag::TableRow => parse_table_row(range, iter).map(|r| r.into()),
        // Tag::TableCell => parse_table_cell(range, iter).map(|c| c.into()),
        Tag::Emphasis => parse_text_decor(TextDecorationKind::Emphasis, range, iter),
        Tag::Strong => parse_text_decor(TextDecorationKind::Strong, range, iter),
        Tag::Strikethrough => parse_text_decor(TextDecorationKind::Strikethrough, range, iter),
        Tag::Superscript => parse_text_decor(TextDecorationKind::Superscript, range, iter),
        Tag::Subscript => parse_text_decor(TextDecorationKind::Subscript, range, iter),
        Tag::Link {
            link_type,
            dest_url,
            // TODO: in which situation is this populated?
            title: _,
            id,
        } => match link_type {
            LinkType::Inline => parse_inline_link(dest_url, range, iter),
            LinkType::Reference => parse_reference_link(dest_url, id, range, iter),
            LinkType::Shortcut => parse_shortcut_link(dest_url, id, range, iter),
            LinkType::WikiLink { .. } => parse_wiki_link(dest_url, range, iter),
            LinkType::Autolink => parse_auto_link(dest_url, range, iter),
            LinkType::Email => parse_email_link(dest_url, range, iter),
            // with a broken_link_callback setup, this could be used to have
            // an external reference list (bibtex perhaps?)
            LinkType::CollapsedUnknown | LinkType::ReferenceUnknown | LinkType::ShortcutUnknown => {
                todo!()
            }
            LinkType::Collapsed => {
                return Err(eyre!("unsupported link type: {link_type:?}"));
            }
        },

        // parse_link(link_type, dest_url, title, id, range, iter),
        Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        } => parse_image(link_type, dest_url, title, id, range, iter),
        _ => Err(eyre!("unsupported tag")),
    }
}

fn parse_email_link(
    dest_url: CowStr<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, color_eyre::eyre::Error> {
    while let Some((event, _)) = iter.next() {
        match event {
            Event::End(TagEnd::Link) => break,
            _ => {}
        }
    }

    Ok(Node::autolink(range, dest_url.to_string()))
}

fn parse_auto_link(
    dest_url: CowStr<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, color_eyre::eyre::Error> {
    while let Some((event, _)) = iter.next() {
        match event {
            Event::End(TagEnd::Link) => break,
            _ => {}
        }
    }

    Ok(Node::autolink(range, dest_url.to_string()))
}

fn parse_wiki_link(
    dest_url: CowStr<'_>,
    // has_pothole: bool,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, color_eyre::eyre::Error> {
    let mut title = String::new();

    // independent of whether this wiki link is "piped" or not (not sure why
    // pulldown_cmark uses the term pothole) we can retrieve the title by
    // consuming the events. When the wikilink has no pothole, the title
    // is the same as the dest_url, but we get the title here by consuming anyway
    while let Some((event, _)) = iter.next() {
        match event {
            Event::End(TagEnd::Link) => break,
            Event::Text(t) => title.push_str(&t),
            _ => {}
        }
    }

    Ok(Node::wikilink(range, title, dest_url.to_string()))
}

fn parse_shortcut_link(
    dest_url: CowStr<'_>,
    id: CowStr<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, color_eyre::eyre::Error> {
    while let Some((event, _)) = iter.next() {
        match event {
            Event::End(TagEnd::Link) => break,
            _ => {}
        }
    }

    return Ok(Node::shortcutlink(
        range,
        id.to_string(),
        dest_url.to_string(),
    ));
}

fn parse_reference_link(
    dest_url: CowStr<'_>,
    id: CowStr<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, color_eyre::eyre::Error> {
    let mut title = String::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Link) => break,
            Event::Text(t) => title.push_str(&t),
            _ => {}
        }
    }

    Ok(Node::referencelink(
        range,
        title,
        id.to_string(),
        dest_url.to_string(),
    ))
}

fn parse_image(
    link_type: LinkType,
    _dest_url: CowStr<'_>,
    _title: CowStr<'_>,
    _id: CowStr<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> Result<Node> {
    for (event, _) in iter.by_ref() {
        match event {
            Event::End(TagEnd::Image) => break,
            _ => {} // ignore link children
        }
    }
    match link_type {
        LinkType::Inline => Ok(Node::inlineimage(range)),
        LinkType::Reference
        | LinkType::ReferenceUnknown
        | LinkType::Collapsed
        | LinkType::CollapsedUnknown => Ok(Node::referenceimage(range)),
        _ => Err(eyre!("not implemented yet")),
    }
}

fn parse_inline_link(
    dest_url: CowStr<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> std::result::Result<Node, color_eyre::eyre::Error> {
    // [hello *there*](what)
    // dest_url == "what"
    // title = "hello there"

    let mut title = String::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Link) => break,
            Event::Text(t) => title.push_str(&t),
            _ => {}
        }
    }

    Ok(Node::inlinelink(range, title, dest_url.to_string()))
}

// fn parse_link(
//     link_type: LinkType,
//     dest_url: CowStr<'_>,
//     _title: CowStr<'_>,
//     _id: CowStr<'_>,
//     range: Range<usize>,
//     iter: &mut ParserIterator<'_>,
// ) -> Result<Node> {
//     let mut children = Vec::new();

//     while let Some((event, range)) = iter.next() {
//         match event {
//             Event::End(TagEnd::Link) => break,
//             _ => children.push(parse_event(event, range, iter)?),
//         }
//     }

//     // match link_type {
//     //     LinkType::WikiLink { has_pothole } => {

//     //     },
//     //     LinkType::Inline => {

//     //     }

//     //     // not implemented yet
//     //     LinkType::Reference
//     //     | LinkType::ReferenceUnknown
//     //     | LinkType::Collapsed
//     //     | LinkType::CollapsedUnknown
//     //     | LinkType::Shortcut
//     //     | LinkType::ShortcutUnknown => todo!(),
//     // }

//     match link_type {
//         LinkType::Inline => {
//             // the children are the title
//             let mut title = String::new();

//             for event in children {
//                 match &event {
//                     Node::Text { text, range: _ } => title.push_str(&text.to_string()),
//                     Node::TextDecoration {
//                         kind: _,
//                         content,
//                         range: _,
//                     } => title.push_str(content),
//                     Node::Code { code, range: _ } => title.push_str(code),

//                     _ => panic!(
//                         "parser encounter unexpected event when parsing link: {:?}",
//                         event
//                     ),
//                 }
//             }

//             Ok(Node::inlinelink(range, title, dest_url.to_string()))
//         }
//         LinkType::Autolink | LinkType::Email => Ok(Node::autolink(range, dest_url.to_string())),
//         LinkType::WikiLink { .. } => Ok(Node::wikilink(range, children)),
//         // not implemented
//         LinkType::Reference
//         | LinkType::ReferenceUnknown
//         | LinkType::Collapsed
//         | LinkType::Shortcut
//         | LinkType::ShortcutUnknown
//         | LinkType::CollapsedUnknown => unimplemented!(),
//     }
// }

// TODO: do we need to constraint the end tag such that we only search for the
// same end as `kind`?
fn parse_text_decor(
    kind: TextDecorationKind,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> Result<Node> {
    let mut content = String::new();

    let target_end = match kind {
        TextDecorationKind::Emphasis => TagEnd::Emphasis,
        TextDecorationKind::Strong => TagEnd::Strong,
        TextDecorationKind::Strikethrough => TagEnd::Strikethrough,
        TextDecorationKind::Superscript => TagEnd::Superscript,
        TextDecorationKind::Subscript => TagEnd::Subscript,
    };

    for (event, _range) in iter.by_ref() {
        if let Event::End(tag) = event
            && tag == target_end
        {
            break;
        }

        if let Event::Text(text) = event {
            content = text.to_string();
        } else {
            return Err(eyre!("unexpected event!: {:?}", event));
        }
    }

    Ok(Node::textdecoration(range, kind, content))
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

fn parse_table_row(range: Range<usize>, iter: &mut ParserIterator<'_>) -> Result<TableRow> {
    let mut cells = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::TableRow) => break,
            Event::Start(Tag::TableCell) => cells.push(parse_table_cell(range, iter)?),
            _ => return Err(eyre!("expected table row")),
        }
    }

    Ok(TableRow::new(range, cells))
}

fn parse_table(
    alignments: Vec<pulldown_cmark::Alignment>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> Result<Node> {
    let header = if let Some((Event::Start(Tag::TableHead), range)) = iter.next() {
        parse_table_head(range, iter)?
    } else {
        return Err(eyre!("header expected but received none"));
    };

    let mut rows = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Table) => break,
            Event::Start(Tag::TableRow) => rows.push(parse_table_row(range, iter)?),
            _ => return Err(eyre!("expected table row")),
        }
    }

    Ok(Node::table(
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
    ))
}

fn parse_table_head(range: Range<usize>, iter: &mut ParserIterator<'_>) -> Result<TableHead> {
    let mut cells = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::Start(Tag::TableCell) => cells.push(parse_table_cell(range, iter)?),
            Event::End(TagEnd::TableHead) => break,
            e => {
                return Err(eyre!("Received unexpected event: {:?}", e));
            }
        }
    }

    Ok(TableHead { range, cells })
}

fn parse_footnote_def(
    id: CowStr<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> Result<Node> {
    let mut target = String::new();

    while let Some((event, _)) = iter.next() {
        match event {
            Event::End(TagEnd::FootnoteDefinition) => break,
            Event::Text(text) => target.push_str(&text),
            _ => {}
        }
    }

    Ok(Node::footnotedefinition(range, id.to_string(), target))
}

fn parse_item(range: Range<usize>, iter: &mut ParserIterator<'_>) -> Result<Node> {
    let mut children = Vec::new();
    let mut sub_lists = Vec::new();
    let mut checkmark = TaskListMarker::NoCheckmark;

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Item) => break,
            Event::TaskListMarker(checked) => {
                if checked {
                    checkmark = TaskListMarker::Checked;
                } else {
                    checkmark = TaskListMarker::UnChecked;
                }
            }
            Event::Start(Tag::List(_)) => sub_lists.push(parse_event(event, range, iter)?),
            Event::Start(Tag::Paragraph) => {
                // Handle paragraph wrapper around task list items
                // The TaskListMarker might appear inside the paragraph
                let para_children = parse_paragraph_in_item(&mut checkmark, range, iter)?;
                children.append(&mut sub_lists);
                children.extend(para_children);
            }
            // TODO: this structure looks a bit weird. Maybe we should reset the
            // sublist list? Check out the snapshot test for it and see if it
            // makes sense
            _ => {
                children.append(&mut sub_lists);
                children.push(parse_event(event, range, iter)?)
            }
        }
    }

    Ok(Node::item(range, checkmark, children, sub_lists))
}

fn parse_paragraph_in_item(
    checkmark: &mut TaskListMarker,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> Result<Vec<Node>> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Paragraph) => break,
            Event::TaskListMarker(checked) => {
                if checked {
                    *checkmark = TaskListMarker::Checked;
                } else {
                    *checkmark = TaskListMarker::UnChecked;
                }
            }
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(children)
}

fn parse_list(n: Option<u64>, range: Range<usize>, iter: &mut ParserIterator<'_>) -> Result<Node> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::List(_)) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(Node::list(range, n, children))
}

fn parse_htmlblock(range: Range<usize>, iter: &mut ParserIterator<'_>) -> Result<Node> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::HtmlBlock) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(Node::html(range, "TODO".into()))
}

fn parse_code_block(
    kind: pulldown_cmark::CodeBlockKind<'_>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> Result<Node> {
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

    Ok(Node::codeblock(range, tag, is_fenced, children))
}

fn parse_blockquote(range: Range<usize>, iter: &mut ParserIterator<'_>) -> Result<Node> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::BlockQuote(_)) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(Node::blockquote(range, children))
}

fn parse_paragraph(range: Range<usize>, iter: &mut ParserIterator<'_>) -> Result<Node> {
    let mut children = Vec::new();

    while let Some((event, range)) = iter.next() {
        match event {
            Event::End(TagEnd::Paragraph) => break,
            _ => children.push(parse_event(event, range, iter)?),
        }
    }

    Ok(Node::paragraph(range, children))
}

fn parse_heading(
    orig_level: HeadingLevel,
    id: Option<CowStr<'_>>,
    classes: Vec<CowStr<'_>>,
    attrs: Vec<(CowStr<'_>, Option<CowStr<'_>>)>,
    range: Range<usize>,
    iter: &mut ParserIterator<'_>,
) -> Result<Node> {
    // 1. we first parse the title itself.
    // 2. we then continue parsing the "section" under the title. We stop when
    // we find the start of a another section on the same "level" or higher, or
    // the end.

    let mut heading_content = String::new();
    let mut children = Vec::new();

    // 1. parse the heading
    while let Some((event, _)) = iter.next() {
        match event {
            Event::End(TagEnd::Heading(end_level)) => {
                if end_level == orig_level {
                    break;
                }
            }
            // only consider the text events, we remove any special formatting
            Event::Text(content) => heading_content.push_str(content.as_ref()),
            _ => {}
        }
    }

    // 2. And then the section
    //
    // We do not want to consume the `Event::Start(TagStart::Heading)` event,
    // which is why the below iteration looks different than above.
    while let Some((event, _)) = iter.inner.peek() {
        if let Event::Start(Tag::Heading { level, .. }) = event
            && *level >= orig_level
        {
            break;
        }
        // consume the event
        let Some((event, range)) = iter.next() else {
            unreachable!();
        };

        children.push(parse_event(event, range, iter)?);
    }

    Ok(Node::heading(
        range,
        id.map(|s| s.to_string()),
        classes.iter().map(|s| s.to_string()).collect(),
        attrs
            .iter()
            .map(|(k, v)| (k.to_string(), v.as_ref().map(|s| s.to_string())))
            .collect(),
        orig_level as u8,
        heading_content,
        children,
    ))
}

pub type FrontMatter = serde_json::Value;
