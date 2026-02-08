use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum TaskListMarker {
    // - some list
    NoCheckmark,
    // - [ ] some list
    UnChecked,
    // - [x] some list
    Checked,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Node {
    // container nodes
    Heading {
        range: Range,
        id: Option<String>,
        classes: Vec<String>,
        attributes: Vec<(String, Option<String>)>,
        level: u8,
        content: String,
        children: Vec<Node>,
    },
    Paragraph {
        range: Range,
        children: Vec<Node>,
    },
    BlockQuote {
        range: Range,
        children: Vec<Node>,
    },
    List {
        range: Range,
        start_index: Option<u64>,
        children: Vec<Node>,
    },
    Item {
        range: Range,
        task_list_marker: TaskListMarker,
        children: Vec<Node>,
        sub_lists: Vec<Node>,
    },
    CodeBlock {
        range: Range,
        tag: Option<String>,
        is_fenced: bool,
        children: Vec<Node>,
    },
    Table {
        range: Range,
        header: TableHead,
        column_alignment: Vec<ColumnAlignment>,
        rows: Vec<TableRow>,
    },
    // leaf nodes
    HardBreak {
        range: Range,
    },
    FootnoteDefinition {
        range: Range,
        id: String,
        target: String,
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
    InlineLink {
        range: Range,
        title: String,
        target: String,
    },
    /// [foo][bar]
    ///
    /// [bar]: <https://some.url>
    ReferenceLink {
        range: Range,
        title: String,
        id: String,
        target: String,
    },
    /// [bar]
    ///
    /// [bar]: <https://some.url>
    ShortcutLink {
        range: Range,
        id: String,
        target: String,
    },
    /// <www.foo.bar>
    AutoLink {
        range: Range,
        target: String,
    },
    /// `[[foo|bar]]`
    WikiLink {
        range: Range,
        title: String,
        target: String,
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
    Code {
        range: Range,
        code: String,
    },
    HorizontalRule {
        range: Range,
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
    /// Given a node, convert it into a serde_json::Value,
    /// and return the inner object, without the range field.
    ///
    /// Node::Code { range, code } -> {"Code": {"range": 42, "code": "..."}}
    /// is converted to
    /// {"code": "..."}
    pub fn inner_json_data(&self) -> Map<String, Value> {
        let value = serde_json::to_value(self).unwrap();

        // we may assume that the value is an object and that the
        // inner value is one as well.
        let Value::Object(mut map) = value else {
            unreachable!()
        };
        let Some(Value::Object(mut map)) = map.remove(&self.kind().to_string()) else {
            unreachable!()
        };
        map.remove("range");

        map
    }
}

impl Node {
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
        children: Vec<Node>,
    ) -> Self {
        Self::Heading {
            range,
            id,
            classes,
            attributes,
            level,
            content,
            children,
        }
    }
    pub fn paragraph(range: Range, children: Vec<Node>) -> Self {
        Self::Paragraph { children, range }
    }
    pub fn blockquote(range: Range, children: Vec<Node>) -> Self {
        Self::BlockQuote { range, children }
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
    pub fn footnotedefinition(range: Range, id: String, target: String) -> Self {
        Self::FootnoteDefinition { range, id, target }
    }
    pub fn inlinelink(range: Range, title: String, target: String) -> Self {
        Self::InlineLink {
            title,
            target,
            range,
        }
    }
    pub fn referencelink(range: Range, title: String, id: String, target: String) -> Self {
        Self::ReferenceLink {
            range,
            title,
            id,
            target,
        }
    }
    pub fn shortcutlink(range: Range, id: String, target: String) -> Self {
        Self::ShortcutLink { range, id, target }
    }
    pub fn autolink(range: Range, target: String) -> Self {
        Self::AutoLink { target, range }
    }
    pub fn wikilink(range: Range, title: String, target: String) -> Self {
        Self::WikiLink {
            range,
            title,
            target,
        }
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
    pub fn item(
        range: Range,
        task_list_marker: TaskListMarker,
        children: Vec<Node>,
        sub_lists: Vec<Node>,
    ) -> Self {
        Self::Item {
            children,
            task_list_marker,
            sub_lists,
            range,
        }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, Hash)]
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

impl ToString for NodeKind {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl From<NodeKind> for String {
    fn from(value: NodeKind) -> Self {
        value.to_string()
    }
}

impl Node {
    pub fn kind(&self) -> NodeKind {
        use NodeKind::*;

        match &self {
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
            Node::Code { .. } => Code,
            Node::CodeBlock { .. } => CodeBlock,
            Node::HorizontalRule { .. } => HorizontalRule,
            Node::Table { .. } => Table,
            Node::DisplayMath { .. } => DisplayMath,
            Node::InlineMath { .. } => InlineMath,
            Node::HardBreak { .. } => HardBreak,
        }
    }
}
