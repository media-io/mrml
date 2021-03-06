use super::prelude::*;
use super::BodyElement;
use crate::elements::{Component, Error};
use crate::parser::Node;
use crate::util::attributes::*;
use crate::util::context::Context;
use crate::util::header::Header;
use crate::util::size::Size;
use crate::util::tag::Tag;
use log::debug;

lazy_static! {
    static ref DEFAULT_ATTRIBUTES: Attributes = Attributes::default().add("width", "600px");
}

#[derive(Clone, Debug)]
pub struct MJBody {
    attributes: Attributes,
    context: Option<Context>,
    children: Vec<BodyElement>,
    exists: bool,
}

impl MJBody {
    fn default_attributes<'a>(node: &Node<'a>, header: &Header) -> Attributes {
        header
            .default_attributes()
            .get_attributes(node, DEFAULT_ATTRIBUTES.clone())
    }

    pub fn empty() -> MJBody {
        MJBody {
            attributes: Attributes::default(),
            children: vec![],
            context: None,
            exists: false,
        }
    }

    pub fn parse<'a>(node: &Node<'a>, header: &Header) -> Result<MJBody, Error> {
        let mut children = vec![];
        for child in node.children.iter() {
            children.push(BodyElement::parse(child, header, None::<&Attributes>)?);
        }
        Ok(MJBody {
            attributes: Self::default_attributes(node, header).concat(node),
            children,
            context: None,
            exists: true,
        })
    }

    fn set_style_body(&self, tag: Tag) -> Tag {
        tag.maybe_set_style("background-color", self.get_attribute("background-color"))
    }

    pub fn render_preview(&self, header: &Header) -> String {
        let preview = match header.preview() {
            Some(value) => value,
            None => return "".into(),
        };
        Tag::new("div")
            .set_style("display", "none")
            .set_style("font-size", "1px")
            .set_style("color", "#ffffff")
            .set_style("line-height", "1px")
            .set_style("max-height", "0px")
            .set_style("max-width", "0px")
            .set_style("opacity", "0")
            .set_style("overflow", "hidden")
            .render(preview)
    }
}

impl Component for MJBody {
    fn context(&self) -> Option<&Context> {
        self.context.as_ref()
    }

    fn update_header(&self, header: &mut Header) {
        for child in self.children.iter() {
            child.update_header(header);
        }
    }

    fn set_context(&mut self, ctx: Context) {
        self.context = Some(ctx);
        let child_base = Context::new(
            self.get_current_width(),
            self.get_siblings(),
            self.get_raw_siblings(),
            0,
        );
        for (idx, child) in self.children.iter_mut().enumerate() {
            child.set_context(child_base.clone().set_index(idx));
        }
    }

    fn render(&self, header: &Header) -> Result<String, Error> {
        debug!("render");
        let body = self.set_style_body(Tag::new("body"));
        let mut res: Vec<String> = vec![];
        res.push(body.open());
        res.push(self.render_preview(&header));
        if self.exists {
            let div = self
                .set_style_body(Tag::new("div"))
                .maybe_set_class(self.get_attribute("css-class"));
            res.push(div.open());
            for child in self.children.iter() {
                res.push(child.render(header)?);
            }
            res.push(div.close());
        }
        res.push(body.close());
        Ok(res.join(""))
    }
}

impl BodyComponent for MJBody {
    fn attributes(&self) -> Option<&Attributes> {
        Some(&self.attributes)
    }

    fn get_current_width(&self) -> Option<Size> {
        self.get_size_attribute("width")
    }

    fn get_children(&self) -> &Vec<BodyElement> {
        &self.children
    }

    fn set_style(&self, key: &str, tag: Tag) -> Tag {
        match key {
            "body" | "div" => self.set_style_body(tag),
            _ => tag,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::tests::{compare_render, compare_render_with_options};
    use crate::Options;

    #[test]
    fn basic() {
        compare_render(
            include_str!("../../../test/mj-body.mjml"),
            include_str!("../../../test/mj-body.html"),
        );
    }

    #[test]
    fn with_options() {
        let mut opts = Options::default();
        opts.keep_comments = false;
        compare_render_with_options(
            include_str!("../../../test/mj-body.mjml"),
            include_str!("../../../test/mj-body-without-comments.html"),
            opts,
        );
    }

    #[test]
    fn with_background_color() {
        compare_render(
            include_str!("../../../test/mj-body-background-color.mjml"),
            include_str!("../../../test/mj-body-background-color.html"),
        );
    }

    #[test]
    fn with_class() {
        compare_render(
            include_str!("../../../test/mj-body-class.mjml"),
            include_str!("../../../test/mj-body-class.html"),
        );
    }

    #[test]
    fn with_width() {
        compare_render(
            include_str!("../../../test/mj-body-width.mjml"),
            include_str!("../../../test/mj-body-width.html"),
        );
    }
}
