use crate::{Error, Result};
use std::fmt::Display;
use std::io::Write;

use pulldown_cmark::MetadataBlockKind;

pub type Range = std::ops::Range<usize>;

pub trait Ranged {
    fn range(&self) -> &Range;
}

macro_rules! make_node_struct_impls {
    (
        $(
            $(#[$meta:meta])*
            $vis:vis struct $name:ident {
                $(
                    $fvis:vis $field:ident : $ty:ty
                ),* $(,)?
            }
        )*
    ) => {
        $(
            $(#[$meta])*
            #[derive(Debug)]
            $vis struct $name {
                $(
                    $fvis $field : $ty,
                )*
            }

            impl $name {
                pub fn new($($field: $ty),*) -> Self {
                    Self { $($field),* }
                }
            }
        )*
    }
}

macro_rules! make_node_enum_impls {
    (
        $(
            $(#[$emeta:meta])*
            $evis:vis enum $ename:ident {
                $(
                    $variant:ident $( ( $($vty:ty),* ) )?
                ),* $(,)?
            }
        )*
    ) => {
        $(
            $(#[$emeta])*
            #[derive(Debug)]
            $evis enum $ename {
                $(
                    $variant $( ( $($vty),* ) )?,
                )*
            }
        )*
    }
}

make_node_struct_impls! {
    pub struct SourceFile {
        pub range: Range,
        pub children: Vec<Node>,
    }

    pub struct MetadataBlock {
        pub range: Range,
        pub kind: MetadataBlockKind,
        // pub text: String,
    }

    pub struct Heading {
        pub id: Option<String>,
        pub classes: Vec<String>,
        pub attributes: Vec<(String, Option<String>)>,
        pub range: Range,
        pub level: u8,
        pub children: Vec<Node>,
    }

    pub struct Paragraph {
        pub range: Range,
        pub children: Vec<Node>,
        // pub marker: Option<TaskListMarker>,
    }

    pub struct BlockQuote {
        pub range: Range,
        pub children: Vec<Node>,
    }

    pub struct Text {
        pub range: Range,
        pub text: String,
    }


    pub struct TextDecoration {
        pub range: Range,
        pub kind: TextDecorationKind,
        pub children: Vec<Node>,
    }

    pub struct Html {
        pub range: Range,
    }

    pub struct DisplayMath {
        pub range: Range,
    }

    pub struct InlineMath {
        pub range: Range,
    }

    pub struct FootnoteReference {
        pub range: Range,
        pub name: String,
    }

    pub struct FootnoteDefinition {
        pub range: Range,
        pub name: String,
        pub children: Vec<Node>,
    }

    pub struct DefinitionList {
        pub range: Range,
        pub children: Vec<Node>,
    }
    pub struct DefinitionListTitle {
        pub range: Range,
        pub children: Vec<Node>,
    }

    pub struct DefinitionListDefinition{
        pub range: Range,
        pub children: Vec<Node>,
    }

    pub struct InlineLink {
        pub range: Range,
        pub children: Vec<Node>,
        pub url: String,
        pub title: Option<String>,
    }

    pub struct ReferenceLink {
        pub range: Range,
        pub children: Vec<Node>,
        pub reference: String,
    }

    pub struct ShortcutLink {
        pub range: Range,
        pub children: Vec<Node>,
    }

    pub struct AutoLink {
        pub range: Range,
        pub children: Vec<Node>,
    }

    pub struct WikiLink {
        pub range: Range,
        pub children: Vec<Node>,
    }

    pub struct LinkReference {
        pub range: Range,
        pub name: String,
        pub link: String,
        pub title: Option<String>,
    }

    pub struct InlineImage {
        pub range: Range,
        // pub text: String,
        // pub url: String,
        // pub title: Option<String>,
    }

    pub struct ReferenceImage {
        pub range: Range,
        // pub text: String,
        // pub reference: String,
    }

    pub struct SoftBreak {
        pub range: Range,
    }

    pub struct HardBreak {
        pub range: Range,
    }

    pub struct List {
        pub range: Range,
        pub start_index: Option<u64>,
        pub children: Vec<Node>,
    }

    pub struct Item {
        pub range: Range,
        // pub marker: Option<TaskListMarker>,
        pub children: Vec<Node>,
        pub sub_lists: Vec<Node>,
    }

    pub struct TaskListMarker {
        pub range: Range,
        pub is_checked: bool,
    }

    /// Inline code.
    pub struct Code {
        pub range: Range,
        pub code: String,
    }

    pub struct CodeBlock {
        pub range: Range,
        pub tag: Option<String>,
        pub is_fenced: bool,
        pub children: Vec<Node>,
    }

    pub struct HorizontalRule {
        pub range: Range,
    }

    pub struct Table {
        pub range: Range,
        pub header: TableHead,
        pub column_alignment: Vec<ColumnAlignment>,
        pub rows: Vec<TableRow>,
    }

    pub struct TableHead {
        pub range: Range,
        pub cells: Vec<TableCell>,
    }

    pub struct TableRow {
        pub range: Range,
        pub cells: Vec<TableCell>,
    }

    pub struct TableCell {
        pub range: Range,
        pub children: Vec<Node>,
    }

    pub struct NotImplemented {
        pub range: Range,
    }
}

impl SourceFile {
    pub fn render_tree(&self) -> Result<String> {
        let mut buffer: Vec<u8> = Vec::new();

        writeln!(buffer, "SourceFile {{")?;

        for node in &self.children {
            render_tree_node(&mut buffer, node, 1);
        }

        writeln!(buffer, "}}")?;

        let buffer = String::from_utf8(buffer)
            .map_err(|_| Error::ParseError("could not format into tree".into()))?;

        return Ok(buffer);
    }
}

fn render_tree_node<W: Write>(buffer: &mut W, node: &Node, level: usize) {
    for _ in 0..level {
        write!(buffer, " ")?;
    }
    match node {
        Node::NotImplemented(not_implemented) => todo!(),
        Node::SourceFile(source_file) => todo!(),
        Node::Heading(heading) => todo!(),
        Node::Paragraph(paragraph) => todo!(),
        Node::BlockQuote(block_quote) => todo!(),
        Node::Text(text) => todo!(),
        Node::TextDecoration(text_decoration) => todo!(),
        Node::Html(html) => todo!(),
        Node::FootnoteReference(footnote_reference) => todo!(),
        Node::FootnoteDefinition(footnote_definition) => todo!(),
        Node::DefinitionList(definition_list) => todo!(),
        Node::DefinitionListTitle(definition_list_title) => todo!(),
        Node::DefinitionListDefinition(definition_list_definition) => todo!(),
        Node::InlineLink(inline_link) => todo!(),
        Node::ReferenceLink(reference_link) => todo!(),
        Node::ShortcutLink(shortcut_link) => todo!(),
        Node::AutoLink(auto_link) => todo!(),
        Node::WikiLink(wiki_link) => todo!(),
        Node::LinkReference(link_reference) => todo!(),
        Node::InlineImage(inline_image) => todo!(),
        Node::ReferenceImage(reference_image) => todo!(),
        Node::List(list) => todo!(),
        Node::Item(item) => todo!(),
        Node::TaskListMarker(task_list_marker) => todo!(),
        Node::SoftBreak(soft_break) => todo!(),
        Node::HardBreak(hard_break) => todo!(),
        Node::Code(code) => todo!(),
        Node::CodeBlock(code_block) => todo!(),
        Node::HorizontalRule(horizontal_rule) => todo!(),
        Node::Table(table) => todo!(),
        Node::TableHead(table_head) => todo!(),
        Node::TableRow(table_row) => todo!(),
        Node::TableCell(table_cell) => todo!(),
        Node::MetadataBlock(metadata_block) => todo!(),
        Node::DisplayMath(display_math) => todo!(),
        Node::InlineMath(inline_math) => todo!(),
    }
}

impl Display for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SourceFile {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl Display for MetadataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "MetadataBlock({:?})", self.kind)?;
        Ok(())
    }
}

impl Display for Heading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heading {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for Paragraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Paragraph {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for BlockQuote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BlockQuote {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Text")?;
        Ok(())
    }
}

impl Display for TextDecoration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TextDecoration({})  {{", self.kind)?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for Html {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Html")?;

        Ok(())
    }
}

impl Display for DisplayMath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "DisplayMath")?;
        Ok(())
    }
}

impl Display for InlineMath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "InlineMath")?;
        Ok(())
    }
}

impl Display for FootnoteReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "FootnoteReference({})", self.name)?;
        Ok(())
    }
}

impl Display for FootnoteDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "FootnoteDefinition ({}) {{}}", self.name)?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for DefinitionList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heading {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for DefinitionListTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heading {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for DefinitionListDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heading {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for InlineLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heading {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for ReferenceLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heading {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for ShortcutLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heading {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for AutoLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heading {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for WikiLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heading {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for LinkReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "LinkReference")?;
        Ok(())
    }
}

impl Display for InlineImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "InlineImage")?;
        Ok(())
    }
}

impl Display for ReferenceImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ReferenceImage")?;
        Ok(())
    }
}

impl Display for SoftBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SoftBreak")?;
        Ok(())
    }
}

impl Display for HardBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "HardBreak")?;
        Ok(())
    }
}

impl Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "List {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heading {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for TaskListMarker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TaskListMarker")?;
        Ok(())
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Code")?;
        Ok(())
    }
}

impl Display for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CodeBlock")?;

        Ok(())
    }
}

impl Display for HorizontalRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "HorizontalRule")?;
        Ok(())
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Table {{")?;
        writeln!(f, "{}", &self.header)?;
        for row in &self.rows {
            writeln!(f, "{}", row)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for TableHead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TableHead {{")?;
        for node in &self.cells {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for TableRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TableRow {{")?;
        for node in &self.cells {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for TableCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TableCell {{")?;
        for node in &self.children {
            writeln!(f, "{}", node)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for NotImplemented {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "NotImplemented")?;

        Ok(())
    }
}

make_node_enum_impls! {
    pub enum TextDecorationKind {
        Emphasis,
        Strong,
        Strikethrough,
        Superscript,
        Subscript
    }


    #[derive(PartialEq , Copy, Clone)]
    pub enum ColumnAlignment {
        None,
        Left,
        Center,
        Right,
    }

}

impl Display for TextDecorationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)?;
        Ok(())
    }
}

macro_rules! generate_node {
($($node_name:ident),*) => {

    #[derive(Debug)]
    pub enum Node {
        $($node_name($node_name)),*,
    }

    #[cfg(debug_assertions)]
    #[derive(Debug)]
    pub enum NodeKind {
        $($node_name),*,
    }

    #[cfg(debug_assertions)]
    impl Node {
        #[allow(dead_code)]
        pub fn kind(&self) -> NodeKind {
            match self {
                $(Node::$node_name(_) => NodeKind::$node_name),*
            }
        }
    }

    impl Ranged for Node {
        fn range<'a>(&'a self) -> &'a Range {
            match self {
                $(Node::$node_name(node) => node.range()),*
            }
        }

    }


    $(
    impl Ranged for $node_name {
        fn range<'a>(&'a self) -> &'a Range {
            &self.range
        }

    }

    impl Into<Node> for $node_name {
        fn into(self) -> Node {
            Node::$node_name(self)
        }
    }
    )*
};
}

impl Node {
    pub fn has_preceding_space(&self, file_text: &str) -> bool {
        let range = self.range();
        if range.start == 0 {
            false
        } else {
            file_text.as_bytes().get(range.start - 1) == " ".as_bytes().first()
        }
    }

    pub fn starts_with_punctuation(&self, file_text: &str) -> bool {
        let range = self.range();
        let text = &file_text[range.start..range.end];
        if let Some(first_char) = text.chars().next() {
            first_char.is_ascii_punctuation()
        } else {
            false
        }
    }

    pub fn ends_with_punctuation(&self, file_text: &str) -> bool {
        let range = self.range();
        let text = &file_text[range.start..range.end];
        if let Some(last_char) = text.chars().last() {
            last_char.is_ascii_punctuation()
        } else {
            false
        }
    }
}

generate_node![
    SourceFile,
    NotImplemented,
    Heading,
    Paragraph,
    BlockQuote,
    Text,
    TextDecoration,
    Html,
    FootnoteReference,
    FootnoteDefinition,
    DefinitionList,
    DefinitionListTitle,
    DefinitionListDefinition,
    InlineLink,
    ReferenceLink,
    ShortcutLink,
    AutoLink,
    WikiLink,
    LinkReference,
    InlineImage,
    ReferenceImage,
    List,
    Item,
    TaskListMarker,
    SoftBreak,
    HardBreak,
    Code,
    CodeBlock,
    HorizontalRule,
    Table,
    TableHead,
    TableRow,
    TableCell,
    MetadataBlock,
    DisplayMath,
    InlineMath
];

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::NotImplemented(inner) => inner.fmt(f),
            Node::SourceFile(inner) => inner.fmt(f),
            Node::Heading(inner) => inner.fmt(f),
            Node::Paragraph(inner) => inner.fmt(f),
            Node::BlockQuote(inner) => inner.fmt(f),
            Node::Text(inner) => inner.fmt(f),
            Node::TextDecoration(inner) => inner.fmt(f),
            Node::Html(inner) => inner.fmt(f),
            Node::FootnoteReference(inner) => inner.fmt(f),
            Node::FootnoteDefinition(inner) => inner.fmt(f),
            Node::DefinitionList(inner) => inner.fmt(f),
            Node::DefinitionListTitle(inner) => inner.fmt(f),
            Node::DefinitionListDefinition(inner) => inner.fmt(f),
            Node::InlineLink(inner) => inner.fmt(f),
            Node::ReferenceLink(inner) => inner.fmt(f),
            Node::ShortcutLink(inner) => inner.fmt(f),
            Node::AutoLink(inner) => inner.fmt(f),
            Node::WikiLink(inner) => inner.fmt(f),
            Node::LinkReference(inner) => inner.fmt(f),
            Node::InlineImage(inner) => inner.fmt(f),
            Node::ReferenceImage(inner) => inner.fmt(f),
            Node::List(inner) => inner.fmt(f),
            Node::Item(inner) => inner.fmt(f),
            Node::TaskListMarker(inner) => inner.fmt(f),
            Node::SoftBreak(inner) => inner.fmt(f),
            Node::HardBreak(inner) => inner.fmt(f),
            Node::Code(inner) => inner.fmt(f),
            Node::CodeBlock(inner) => inner.fmt(f),
            Node::HorizontalRule(inner) => inner.fmt(f),
            Node::Table(inner) => inner.fmt(f),
            Node::TableHead(inner) => inner.fmt(f),
            Node::TableRow(inner) => inner.fmt(f),
            Node::TableCell(inner) => inner.fmt(f),
            Node::MetadataBlock(inner) => inner.fmt(f),
            Node::DisplayMath(inner) => inner.fmt(f),
            Node::InlineMath(inner) => inner.fmt(f),
        }
    }
}
