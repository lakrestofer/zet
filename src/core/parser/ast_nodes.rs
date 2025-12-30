use serde::{Deserialize, Serialize};

pub type Range = std::ops::Range<usize>;

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockQuoteKind {
    Inline,
    Fenced(String),
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum TextDecorationKind {
    Emphasis,
    Strong,
    Strikethrough,
    Superscript,
    Subscript,
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum ColumnAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Node {
    NotImplemented {
        range: Range,
    },
    SoftBreak {
        range: Range,
    },
    HardBreak {
        range: Range,
    },
    Heading {
        range: Range,
        id: Option<String>,
        classes: Vec<String>,
        attributes: Vec<(String, Option<String>)>,
        level: u8,
        content: String,
    },
    Paragraph {
        range: Range,
        children: Vec<Node>,
    },
    BlockQuote {
        range: Range,
        children: Vec<Node>,
    },
    Text {
        range: Range,
        text: String,
    },
    TextDecoration {
        range: Range,
        kind: TextDecorationKind,
        content: String,
    },
    Html {
        range: Range,
        text: String,
    },
    FootnoteReference {
        range: Range,
        name: String,
    },
    FootnoteDefinition {
        range: Range,
        name: String,
        children: Vec<Node>,
    },
    InlineLink {
        range: Range,
        title: String,
        target: String,
    },
    ReferenceLink {
        range: Range,
        children: Vec<Node>,
        reference: String,
    },
    ShortcutLink {
        range: Range,
        children: Vec<Node>,
    },
    /// <www.foo.bar>
    AutoLink {
        range: Range,
        target: String,
    },
    /// `[[foo|bar]]`
    WikiLink {
        range: Range,
        children: Vec<Node>,
    },

    LinkReference {
        range: Range,
        name: String,
        link: String,
        title: Option<String>,
    },
    InlineImage {
        range: Range,
    },
    ReferenceImage {
        range: Range,
    },
    List {
        range: Range,
        start_index: Option<u64>,
        children: Vec<Node>,
    },
    Item {
        range: Range,
        children: Vec<Node>,
        sub_lists: Vec<Node>,
    },
    TaskListMarker {
        range: Range,
        is_checked: bool,
    },
    Code {
        range: Range,
        code: String,
    },
    CodeBlock {
        range: Range,
        tag: Option<String>,
        is_fenced: bool,
        children: Vec<Node>,
    },
    HorizontalRule {
        range: Range,
    },
    Table {
        range: Range,
        header: TableHead,
        column_alignment: Vec<ColumnAlignment>,
        rows: Vec<TableRow>,
    },
    DisplayMath {
        range: Range,
        text: String,
    },
    InlineMath {
        range: Range,
        text: String,
    },
}

impl Node {
    pub fn range(&self) -> Range {
        let range = match self {
            Node::NotImplemented { range } => range,
            Node::SoftBreak { range } => range,
            Node::HardBreak { range } => range,
            Node::Heading {
                range,
                id,
                classes,
                attributes,
                level,
                content,
            } => range,
            Node::Paragraph { range, children } => range,
            Node::BlockQuote { range, children } => range,
            Node::Text { range, text } => range,
            Node::TextDecoration {
                range,
                kind,
                content,
            } => range,
            Node::Html { range, text } => range,
            Node::FootnoteReference { range, name } => range,
            Node::FootnoteDefinition {
                range,
                name,
                children,
            } => range,
            Node::InlineLink {
                range,
                title,
                target,
            } => range,
            Node::ReferenceLink {
                range,
                children,
                reference,
            } => range,
            Node::ShortcutLink { range, children } => range,
            Node::AutoLink { range, target } => range,
            Node::WikiLink { range, children } => range,
            Node::LinkReference {
                range,
                name,
                link,
                title,
            } => range,
            Node::InlineImage { range } => range,
            Node::ReferenceImage { range } => range,
            Node::List {
                range,
                start_index,
                children,
            } => range,
            Node::Item {
                range,
                children,
                sub_lists,
            } => range,
            Node::TaskListMarker { range, is_checked } => range,
            Node::Code { range, code } => range,
            Node::CodeBlock {
                range,
                tag,
                is_fenced,
                children,
            } => range,
            Node::HorizontalRule { range } => range,
            Node::Table {
                range,
                header,
                column_alignment,
                rows,
            } => range,
            Node::DisplayMath { range, text } => range,
            Node::InlineMath { range, text } => range,
        };
        range.clone()
    }
}

impl Node {
    pub fn notimplemented(range: Range) -> Self {
        Self::NotImplemented { range }
    }
    pub fn softbreak(range: Range) -> Self {
        Self::SoftBreak { range }
    }
    pub fn hardbreak(range: Range) -> Self {
        Self::HardBreak { range }
    }
    pub fn heading(
        range: Range,
        id: Option<String>,
        classes: Vec<String>,
        attributes: Vec<(String, Option<String>)>,
        level: u8,
        content: String,
    ) -> Self {
        Self::Heading {
            range,
            id,
            classes,
            attributes,
            level,
            content,
        }
    }
    pub fn paragraph(range: Range, children: Vec<Node>) -> Self {
        Self::Paragraph { children, range }
    }
    pub fn blockquote(range: Range, children: Vec<Node>) -> Self {
        Self::Paragraph { children, range }
    }
    pub fn text(range: Range, text: String) -> Self {
        Self::Text { text, range }
    }
    pub fn textdecoration(range: Range, kind: TextDecorationKind, content: String) -> Self {
        Self::TextDecoration {
            kind,
            content,
            range,
        }
    }
    pub fn html(range: Range, text: String) -> Self {
        Self::Html { text, range }
    }
    pub fn footnotereference(range: Range, name: String) -> Self {
        Self::FootnoteReference { name, range }
    }
    pub fn footnotedefinition(range: Range, name: String, children: Vec<Node>) -> Self {
        Self::FootnoteDefinition {
            name,
            children,
            range,
        }
    }
    pub fn inlinelink(range: Range, title: String, target: String) -> Self {
        Self::InlineLink {
            title,
            target,
            range,
        }
    }
    pub fn referencelink(range: Range, children: Vec<Node>, reference: String) -> Self {
        Self::ReferenceLink {
            children,
            reference,
            range,
        }
    }
    pub fn shortcutlink(range: Range, children: Vec<Node>) -> Self {
        Self::ShortcutLink { children, range }
    }
    pub fn autolink(range: Range, target: String) -> Self {
        Self::AutoLink { target, range }
    }
    pub fn wikilink(range: Range, children: Vec<Node>) -> Self {
        Self::WikiLink { children, range }
    }
    pub fn linkreference(range: Range, name: String, link: String, title: Option<String>) -> Self {
        Self::LinkReference {
            name,
            link,
            title,
            range,
        }
    }
    pub fn inlineimage(range: Range) -> Self {
        Self::InlineImage { range }
    }
    pub fn referenceimage(range: Range) -> Self {
        Self::ReferenceImage { range }
    }
    pub fn list(range: Range, start_index: Option<u64>, children: Vec<Node>) -> Self {
        Self::List {
            start_index,
            children,
            range,
        }
    }
    pub fn item(range: Range, children: Vec<Node>, sub_lists: Vec<Node>) -> Self {
        Self::Item {
            children,
            sub_lists,
            range,
        }
    }
    pub fn tasklistmarker(range: Range, is_checked: bool) -> Self {
        Self::TaskListMarker { is_checked, range }
    }
    pub fn code(range: Range, code: String) -> Self {
        Self::Code { code, range }
    }
    pub fn codeblock(
        range: Range,
        tag: Option<String>,
        is_fenced: bool,
        children: Vec<Node>,
    ) -> Self {
        Self::CodeBlock {
            tag,
            is_fenced,
            children,
            range,
        }
    }
    pub fn horizontalrule(range: Range) -> Self {
        Self::HorizontalRule { range }
    }
    pub fn table(
        range: Range,
        header: TableHead,
        column_alignment: Vec<ColumnAlignment>,
        rows: Vec<TableRow>,
    ) -> Self {
        Self::Table {
            header,
            column_alignment,
            rows,
            range,
        }
    }
    pub fn displaymath(range: Range, text: String) -> Self {
        Self::DisplayMath { text, range }
    }
    pub fn inlinemath(range: Range, text: String) -> Self {
        Self::InlineMath { text, range }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableHead {
    pub range: Range,
    pub cells: Vec<TableCell>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct TableRow {
    pub range: Range,
    pub cells: Vec<TableCell>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct TableCell {
    pub range: Range,
    pub children: Vec<Node>,
}

impl TableHead {
    pub fn new(range: Range, cells: Vec<TableCell>) -> Self {
        Self { range, cells }
    }
}
impl TableRow {
    pub fn new(range: Range, cells: Vec<TableCell>) -> Self {
        Self { range, cells }
    }
}
impl TableCell {
    pub fn new(range: Range, children: Vec<Node>) -> Self {
        Self { range, children }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeKind {
    Document,
    NotImplemented,
    Heading,
    Paragraph,
    BlockQuote,
    Text,
    TextDecoration,
    Html,
    FootnoteReference,
    FootnoteDefinition,
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
    DisplayMath,
    InlineMath,
}
impl Node {
    pub fn kind(&self) -> NodeKind {
        use NodeKind::*;

        match &self {
            Node::NotImplemented { .. } => NotImplemented,
            Node::Heading { .. } => Heading,
            Node::Paragraph { .. } => Paragraph,
            Node::BlockQuote { .. } => BlockQuote,
            Node::Text { .. } => Text,
            Node::TextDecoration { .. } => TextDecoration,
            Node::Html { .. } => Html,
            Node::FootnoteReference { .. } => FootnoteReference,
            Node::FootnoteDefinition { .. } => FootnoteDefinition,
            Node::InlineLink { .. } => InlineLink,
            Node::ReferenceLink { .. } => ReferenceLink,
            Node::ShortcutLink { .. } => ShortcutLink,
            Node::AutoLink { .. } => AutoLink,
            Node::WikiLink { .. } => WikiLink,
            Node::LinkReference { .. } => LinkReference,
            Node::InlineImage { .. } => InlineImage,
            Node::ReferenceImage { .. } => ReferenceImage,
            Node::List { .. } => List,
            Node::Item { .. } => Item,
            Node::TaskListMarker { .. } => TaskListMarker,
            Node::Code { .. } => Code,
            Node::CodeBlock { .. } => CodeBlock,
            Node::HorizontalRule { .. } => HorizontalRule,
            Node::Table { .. } => Table,
            Node::DisplayMath { .. } => DisplayMath,
            Node::InlineMath { .. } => InlineMath,
            Node::SoftBreak { .. } => SoftBreak,
            Node::HardBreak { .. } => HardBreak,
        }
    }
}
