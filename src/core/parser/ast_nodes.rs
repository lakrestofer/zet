use serde::{Deserialize, Serialize};

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
            #[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockQuoteKind {
    Inline,
    Fenced(String),
}

make_node_struct_impls! {
    pub struct Document {
        pub range: Range,
        pub children: Vec<Node>,
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
        pub text: String
    }

    pub struct DisplayMath {
        pub range: Range,
        pub text: String,
    }

    pub struct InlineMath {
        pub range: Range,
        pub text: String,
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

make_node_enum_impls! {
    #[derive(PartialEq , Copy, Clone, Serialize, Deserialize)]
    pub enum TextDecorationKind {
        Emphasis,
        Strong,
        Strikethrough,
        Superscript,
        Subscript
    }


    #[derive(PartialEq , Copy, Clone, Serialize, Deserialize)]
    pub enum ColumnAlignment {
        None,
        Left,
        Center,
        Right,
    }

}

macro_rules! generate_node {
($($node_name:ident),*) => {

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Node {
        $($node_name($node_name)),*,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum NodeKind {
        $($node_name),*,
    }

    impl Node {
        #[allow(dead_code)]
        pub fn kind(&self) -> NodeKind {
            match self {
                $(Node::$node_name(_) => NodeKind::$node_name),*
            }
        }
    }

    impl Ranged for Node {
        fn range(&self) -> &Range {
            match self {
                $(Node::$node_name(node) => node.range()),*
            }
        }

    }


    $(
    impl Ranged for $node_name {
        fn range(&self) -> &Range {
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
    // DefinitionList,
    // DefinitionListTitle,
    // DefinitionListDefinition,
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
    InlineMath
];
