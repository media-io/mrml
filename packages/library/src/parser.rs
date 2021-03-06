use xmlparser::{StrSpan, Token, Tokenizer};

pub type Attributes<'a> = Vec<(StrSpan<'a>, StrSpan<'a>)>;
pub type Children<'a> = Vec<Element<'a>>;

#[derive(Debug)]
pub enum Error {
    InvalidFormat,
    /// The input string should be smaller than 4GiB.
    SizeLimit,
    /// Errors detected by the `xmlparser` crate.
    ParserError(xmlparser::Error),
    /// The MJML document must have at least one element.
    NoRootNode,
}

impl From<xmlparser::Error> for Error {
    fn from(err: xmlparser::Error) -> Self {
        Error::ParserError(err)
    }
}

pub struct Node<'a> {
    pub name: StrSpan<'a>,
    pub attributes: Attributes<'a>,
    pub children: Children<'a>,
}

impl<'a> Node<'a> {
    pub fn new(name: StrSpan<'a>, attributes: Attributes<'a>, children: Children<'a>) -> Self {
        Node {
            name,
            attributes,
            children,
        }
    }

    fn parse(parser: &mut Tokenizer<'a>, tag: StrSpan<'a>) -> Result<Self, Error> {
        let mut attributes = vec![];
        loop {
            let token = match parser.next() {
                Some(value) => value,
                None => return Err(Error::InvalidFormat),
            };
            let token = token?;
            match token {
                Token::Attribute {
                    local,
                    value,
                    span: _,
                    prefix: _,
                } => {
                    attributes.push((local, value));
                }
                Token::ElementEnd { end, span: _ } => match end {
                    xmlparser::ElementEnd::Empty => {
                        return Ok(Node::new(tag, attributes, vec![]));
                    }
                    xmlparser::ElementEnd::Open => {
                        return Ok(Node::new(
                            tag,
                            attributes,
                            Element::parse_children(parser, tag)?,
                        ));
                    }
                    _ => return Err(Error::InvalidFormat),
                },
                _ => return Err(Error::InvalidFormat),
            };
        }
    }

    pub fn parse_root(parser: &mut Tokenizer<'a>) -> Result<Self, Error> {
        let token = match parser.next() {
            Some(value) => value,
            None => return Err(Error::NoRootNode),
        };
        let token = token?;
        match token {
            Token::ElementStart {
                local,
                prefix: _,
                span: _,
            } => Node::parse(parser, local),
            _ => Err(Error::NoRootNode),
        }
    }
}

pub enum Element<'a> {
    Comment(StrSpan<'a>),
    Node(Node<'a>),
    Text(StrSpan<'a>),
}

impl<'a> Element<'a> {
    pub fn is_comment(&self) -> bool {
        matches!(self, Element::Comment(_))
    }
    pub fn as_comment(&self) -> Option<&StrSpan<'a>> {
        match self {
            Element::Comment(value) => Some(value),
            _ => None,
        }
    }
    pub fn is_node(&self) -> bool {
        matches!(self, Element::Node(_))
    }
    pub fn as_node(&self) -> Option<&Node<'a>> {
        match self {
            Element::Node(value) => Some(value),
            _ => None,
        }
    }
    pub fn is_text(&self) -> bool {
        matches!(self, Element::Text(_))
    }
    pub fn as_text(&self) -> Option<&StrSpan<'a>> {
        match self {
            Element::Text(value) => Some(value),
            _ => None,
        }
    }
    fn parse_children(parser: &mut Tokenizer<'a>, tag: StrSpan<'a>) -> Result<Vec<Self>, Error> {
        let mut children: Vec<Element<'a>> = vec![];
        loop {
            let token = match parser.next() {
                Some(value) => value,
                // end before having the closing element
                None => return Err(Error::InvalidFormat),
            };
            let token = token?;
            match token {
                Token::ElementStart {
                    local,
                    prefix: _,
                    span: _,
                } => {
                    children.push(Element::Node(Node::parse(parser, local)?));
                }
                Token::Text { text } => {
                    if !text.as_str().trim().is_empty() {
                        children.push(Element::Text(text));
                    }
                }
                Token::ElementEnd { end, span: _ } => match end {
                    xmlparser::ElementEnd::Close(_prefix, local) => {
                        if local.as_str() == tag.as_str() {
                            return Ok(children);
                        }
                        // end before having the closing element
                        return Err(Error::InvalidFormat);
                    }
                    _ => return Err(Error::InvalidFormat),
                },
                // TODO handle comments
                Token::Comment { text, span: _ } => {
                    children.push(Element::Comment(text));
                }
                _ => return Err(Error::InvalidFormat),
            };
        }
    }
}

/// parse the mjml template into an mjml object
///
/// ```rust
/// let result = mrml::parse(r#"
///     <mjml>
///         <mj-head>
///             <mj-title>Something</mj-title>
///         </mj-head>
///     </mjml>
/// "#, mrml::Options::default());
/// assert!(result.is_ok());
/// ```
///
/// ```rust
/// let result = mrml::parse("<mjml", mrml::Options::default());
/// assert!(result.is_err());
/// ```
pub fn parse(text: &'_ str) -> Result<Node<'_>, Error> {
    if text.len() > std::u32::MAX as usize {
        return Err(Error::SizeLimit);
    }
    let mut parser = Tokenizer::from(text);
    Node::parse_root(&mut parser)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_root() {
        let root = parse("<mjml><mj-head></mj-head><mj-body /></mjml>");
        let root = root.unwrap();
        assert_eq!(root.children.len(), 2);
    }

    #[test]
    fn parse_with_weird_text() {
        let root = parse("<mjml><mj-body>&copy;</mj-body></mjml>");
        assert!(root.is_ok());
    }

    #[test]
    fn parse_with_html() {
        let root = parse("<mjml><mj-body><a href=\"toto\">yolo</a></mj-body></mjml>");
        assert!(root.is_ok());
    }

    #[test]
    fn parse_with_comment() {
        let root = parse("<mjml><mj-body><!--<a href=\"toto\">yolo</a>--></mj-body></mjml>");
        assert!(root.is_ok());
        let root = root.unwrap();
        match root.children.get(0).as_ref().unwrap() {
            Element::Node(node) => {
                assert_eq!(node.children.len(), 1);
            }
            _ => assert!(false),
        };
    }
}
