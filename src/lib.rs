#![deny(missing_docs)]
//! # Markdown Walker
//!
//! A markdown walker trait for traversing markdown AST made by the Comrak crate.
//!
//! ## Example
//!
//! ```rust
//! use std::io;
//! use comrak::nodes::Ast;
//! use markdown_walker::MarkdownWalker;
//!
//! #[derive(Debug, Default, PartialEq)]
//! struct ImageCount(usize);
//!
//! impl MarkdownWalker for ImageCount {
//!     fn visit_image<'arena>(
//!         &mut self,
//!         _node: &'arena Node<'arena, RefCell<Ast>>,
//!         _link: &NodeLink,
//!     ) -> io::Result<()> {
//!         self.0 += 1;
//!         Ok(())
//!     }
//! }
//!
//! #[test]
//! fn test_image_count() {
//!     let markdown = r#"
//! ![Image 1](image1.png)
//! ![Image 2](image2.png)
//! ![Image 3](image3.png)
//! "#;
//!
//!     let image_count = ImageCount::from_markdown(markdown).unwrap();
//!     assert_eq!(image_count, ImageCount(3));
//! }
//!
//! ```

use std::{cell::RefCell, io};

use comrak::{
    arena_tree::Node,
    nodes::{
        Ast, NodeCode, NodeCodeBlock, NodeDescriptionItem, NodeFootnoteDefinition,
        NodeFootnoteReference, NodeHeading, NodeHtmlBlock, NodeLink, NodeList, NodeMath,
        NodeMultilineBlockQuote, NodeShortCode, NodeTable, NodeValue, NodeWikiLink,
    },
};

#[allow(unused_variables)]
/// The main trait we export from this crate. This gives you the ability to build your own
/// types from a given markdown AST made by the Comrak crate. The trait has a default
/// implementation for every one of its methods, so you can choose to override only the
/// methods you need for your type.
///
/// ## From Markdown
///
/// Any type which implements [`Default`] can be created from a markdown string. The default
/// trait lets the walker initialize the type with default values and then parse given markdown
/// string and return the type.
///
/// See the [`from_markdown`] method for more information.
///
/// ## Visit
///
/// The [`visit`] method is the main method you need to implement to build your type from the
/// markdown AST directly. This method leaves a bit more work on the table for you, but you
/// can leverage it for a more performant traversal, since you'll only be marshaling the AST
/// once.
///
/// [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
/// [`from_markdown`]: #method.from_markdown
/// [`visit`]: #method.visit
///
pub trait MarkdownWalker {
    /// Create a new instance of the type from a markdown string. This method is only available
    /// for types that implement the [`Default`] trait. See the [`visit`] method for the lower
    /// level API counterpart.
    ///
    /// [`visit`]: #method.visit
    fn from_markdown(markdown: impl AsRef<str>) -> io::Result<Self>
    where
        Self: Default,
    {
        use comrak::{parse_document, Arena, ExtensionOptions, Options};

        let arena = Arena::new();
        let extension = ExtensionOptions::builder()
            .autolink(true)
            .description_lists(true)
            .footnotes(true)
            .math_code(true)
            .math_dollars(true)
            .multiline_block_quotes(true)
            .shortcodes(true)
            .spoiler(true)
            .strikethrough(true)
            .subscript(true)
            .superscript(true)
            .table(true)
            .tagfilter(true)
            .tasklist(true)
            .underline(true)
            .wikilinks_title_after_pipe(true)
            .wikilinks_title_before_pipe(true)
            .build();

        let options = Options {
            extension,
            ..Options::default()
        };

        let nodes = parse_document(&arena, markdown.as_ref(), &options);

        let mut this = Self::default();
        this.visit(nodes)?;

        Ok(this)
    }

    /// Visit a node in the markdown AST. This method recursively traverses the AST and calls
    /// the appropriate method for each node type. This method powers [`from_markdown`], but
    /// works on the direct AST instead of a markdown string.
    ///
    /// [`from_markdown`]: #method.from_markdown
    fn visit<'arena>(&mut self, node: &'arena Node<'arena, RefCell<Ast>>) -> io::Result<()> {
        use NodeValue::*;

        match &node.data.borrow().value {
            Document => self.visit_document(node)?,

            Code(code) => self.visit_code(node, code)?,
            CodeBlock(code_block) => self.visit_code_block(node, code_block)?,
            EscapedTag(tag) => self.visit_escaped_tag(node, tag)?,
            FrontMatter(frontmatter) => self.visit_front_matter(node, frontmatter)?,
            FootnoteDefinition(footnote_definition) => {
                self.visit_footnote_definition(node, footnote_definition)?
            }
            FootnoteReference(node_footnote_reference) => {
                self.visit_footnote_reference(node, node_footnote_reference)?
            }
            Heading(heading) => self.visit_heading(node, heading)?,
            HtmlBlock(html_block) => self.visit_html_block(node, html_block)?,
            HtmlInline(inline) => self.visit_html_inline(node, inline)?,
            Image(link) => self.visit_image(node, link)?,
            Item(list) => self.visit_item(node, list)?,
            Link(link) => self.visit_link(node, link)?,
            List(list) => self.visit_list(node, list)?,
            Math(math) => self.visit_math(node, math)?,
            MultilineBlockQuote(multiline_block_quote) => {
                self.visit_multiline_block_quote(node, multiline_block_quote)?
            }
            Raw(raw) => self.visit_raw(node, raw)?,
            TaskItem(task_item) => self.visit_task_item(node, task_item)?,
            Text(text) => self.visit_text(node, text)?,
            ShortCode(short_code) => self.visit_short_code(node, short_code)?,
            WikiLink(wiki_link) => self.visit_wiki_link(node, wiki_link)?,

            DescriptionItem(description_item) => {
                self.visit_description_item(node, description_item)?
            }
            DescriptionList => self.visit_description_list(node)?,
            DescriptionTerm => self.visit_description_term(node)?,
            DescriptionDetails => self.visit_description_details(node)?,

            Table(table) => self.visit_table(node, table)?,
            TableRow(table_row) => self.visit_table_row(node, table_row)?,
            TableCell => self.visit_table_cell(node)?,

            BlockQuote => self.visit_block_quote(node)?,
            Emph => self.visit_emph(node)?,
            Escaped => self.visit_escaped(node)?,
            LineBreak => self.visit_line_break(node)?,
            Paragraph => self.visit_paragraph(node)?,
            SoftBreak => self.visit_soft_break(node)?,
            SpoileredText => self.visit_spoilered_text(node)?,
            Strikethrough => self.visit_strikethrough(node)?,
            Strong => self.visit_strong(node)?,
            Subscript => self.visit_subscript(node)?,
            Superscript => self.visit_superscript(node)?,
            ThematicBreak => self.visit_thematic_break(node)?,
            Underline => self.visit_underline(node)?,
        }

        for child in node.children() {
            self.visit(child)?;
        }

        Ok(())
    }

    /// Visit a block quote node.
    fn visit_block_quote<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a code node, which is a code span that contains a [`NodeCode`] ref.
    fn visit_code<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        code: &NodeCode,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a code block node, which contains a [`NodeCodeBlock`] ref.
    fn visit_code_block<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        code_block: &NodeCodeBlock,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a description item node, which contains a [`NodeDescriptionItem`] ref.
    fn visit_description_item<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        description_item: &NodeDescriptionItem,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a description list node.
    fn visit_description_list<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a description term node.
    fn visit_description_term<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a description details node.
    fn visit_description_details<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a document node.
    fn visit_document<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit an escaped content node.
    fn visit_escaped<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit an escaped tag node. The tag is a [`String`] that contains the escaped tag content.
    fn visit_escaped_tag<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        tag: &String,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit an emphasis node.
    fn visit_emph<'arena>(&mut self, node: &'arena Node<'arena, RefCell<Ast>>) -> io::Result<()> {
        Ok(())
    }

    /// Visit a footnote definition node, which contains a [`NodeFootnoteDefinition`] ref.
    fn visit_footnote_definition<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        footnote_definition: &NodeFootnoteDefinition,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a footnote reference node, which contains a [`NodeFootnoteReference`] ref.
    fn visit_footnote_reference<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        footnote_reference: &NodeFootnoteReference,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a front matter node. The front matter is a [`String`] that contains the front matter content.
    fn visit_front_matter<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        frontmatter: &String,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a heading node, which contains a [`NodeHeading`] ref.
    fn visit_heading<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        heading: &NodeHeading,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit an HTML block node, which is a [`NodeHtmlBlock`] ref.
    fn visit_html_block<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        html_block: &NodeHtmlBlock,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit an HTML inline node. The inline is a [`String`] that contains the HTML inline content.
    fn visit_html_inline<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        inline: &String,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit an image node, which contains a [`NodeLink`] ref for the image link.
    fn visit_image<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        link: &NodeLink,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit an item node, which contains a [`NodeList`] ref describing the container list.
    fn visit_item<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        list: &NodeList,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a link node, which contains a [`NodeLink`] ref for the link.
    fn visit_link<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        link: &NodeLink,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a line break node.
    fn visit_line_break<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a list node, which contains a [`NodeList`] ref describing the list.
    fn visit_list<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        list: &NodeList,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a math node, which contains a [`NodeMath`] ref.
    fn visit_math<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        math: &NodeMath,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a multiline block quote node, which contains a [`NodeMultilineBlockQuote`] ref.
    fn visit_multiline_block_quote<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        multiline_block_quote: &NodeMultilineBlockQuote,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a paragraph node.
    fn visit_paragraph<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a raw node. The raw is a [`String`] that contains the raw content.
    fn visit_raw<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        raw: &String,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a short code node, which contains a [`NodeShortCode`] ref.
    fn visit_short_code<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        short_code: &NodeShortCode,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a soft break node.
    fn visit_soft_break<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a spoilered text node.
    fn visit_spoilered_text<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a strong node.
    fn visit_strong<'arena>(&mut self, node: &'arena Node<'arena, RefCell<Ast>>) -> io::Result<()> {
        Ok(())
    }

    /// Visit a strikethrough node.
    fn visit_strikethrough<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a subscript node.
    fn visit_subscript<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a superscript node.
    fn visit_superscript<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a table node, which contains a [`NodeTable`] ref.
    fn visit_table<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        table: &NodeTable,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a table cell node.
    fn visit_table_cell<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a table row node.
    fn visit_table_row<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        table_row: &bool,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a task item node.
    fn visit_task_item<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        task_item: &Option<char>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a text node. The text is a [`String`] that contains the text content.
    fn visit_text<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        text: &String,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a thematic break node.
    fn visit_thematic_break<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit an underline node.
    fn visit_underline<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Visit a wiki link node, which contains a [`NodeWikiLink`] ref.
    fn visit_wiki_link<'arena>(
        &mut self,
        node: &'arena Node<'arena, RefCell<Ast>>,
        wiki_link: &NodeWikiLink,
    ) -> io::Result<()> {
        Ok(())
    }
}
