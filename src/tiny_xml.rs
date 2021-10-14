//! Tiny XML generator implementation.
//!
//! Beware: it _does_ let you create incorrect XML, and is therefore useful only for
//! creating **very simple** XML documents.
//!
//! The only goal is to be small, fast, reasonably simple to use and dependency-less.
//!
//! What are the dangers?
//!
//! - XML Namespaces and prefixes are completely up to you
//! - no support for character entities, other than basic escaping
//! - no support for processing instructions, CDATA sections, DOCTYPE declarations
//! - definitely no support for XML schema
//!
//! That being said, you still _can_ create document with namespaces, prefixes, and matching an XML schema.
//! But the api does not guard you to do so correctly.
//!
//! More specifically, it only cares to:
//! - properly close open elements before closing the document
//! - escape attribute values
//! - escape element's text content
use std::io::Result;
use std::io::Write;

pub struct DirtyXml {
    output: Vec<u8>,
    open_elements: Vec<String>,
}

pub type Attribute<'a> = (&'a str, &'a str);

impl DirtyXml {
    pub fn new() -> Result<Self> {
        let mut output = Vec::new();
        writeln!(output, r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#)?;
        Ok(Self { output, open_elements: vec![] })
    }

    fn indent(&mut self) -> Result<()> {
        for _ in &self.open_elements {
            write!(self.output, " ")?;
        }
        Ok(())
    }

    /// Opens a new element.
    pub fn open(&mut self, elem: &str) -> Result<()> {
        self.open_attrs(elem, &[])
    }

    /// Opens a new element with attributes, if any.
    pub fn open_attrs(&mut self, elem: &str, attrs: &[Attribute]) -> Result<()> {
        self.indent()?;
        write!(self.output, "<{}", elem)?;
        for (name, value) in attrs {
            write!(self.output, r#" {}="{}""#, name, escape_attr(value))?;
        }
        writeln!(self.output, ">")?;
        self.open_elements.push(elem.to_string());
        Ok(())
    }

    /// Adds an element with simple text content.
    pub fn simple(&mut self, elem: &str, text: &str) -> Result<()> {
        self.indent()?;
        writeln!(self.output, "<{elem}>{text}</{elem}>",
                 elem = elem,
                 text = escape_textcontent(text),
        )?;
        Ok(())
    }

    /// Close latest unclosed element and return its name if any.
    pub fn close(&mut self) -> Result<Option<String>> {
        match self.open_elements.last() {
            None => Ok(None),
            Some(_) => {
                let element_name = self.open_elements.remove(self.open_elements.len() - 1);
                self.indent()?;
                writeln!(self.output, "</{}>", element_name)?;
                Ok(Some(element_name))
            }
        }
    }

    /// Closes all open elements and converts into a XML string.
    pub fn into_xml(mut self) -> Result<String> {
        while self.close()?.is_some() {};
        Ok(String::from_utf8_lossy(self.output.as_slice()).to_string())
    }
}

fn escape_attr(value: &str) -> String {
    value
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
}

fn escape_textcontent(value: &str) -> String {
    value
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

#[cfg(test)]
mod tests {
    use crate::tiny_xml::DirtyXml;

    #[test]
    fn test_doc() -> anyhow::Result<()> {
        let mut doc = DirtyXml::new()?;
        doc.open_attrs("Import", &[
            ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
            ("xsi:noNamespaceSchemaLocation", "http://www.fio.cz/schema/importIB.xsd"),
        ])?;
        doc.open("Orders")?;
        doc.open("DomesticTransaction")?;
        doc.simple("accountFrom", "1234562")?;
        doc.simple("currency", "CZK")?;
        doc.simple("comment", r#""Vaše" označení & naše <po>kusy"#)?;
        doc.open_attrs("test", &[("hello", r#""earth" & <M>ars"#)])?;
        let xml_string = doc.into_xml()?;
        let expected_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Import xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://www.fio.cz/schema/importIB.xsd">
 <Orders>
  <DomesticTransaction>
   <accountFrom>1234562</accountFrom>
   <currency>CZK</currency>
   <comment>"Vaše" označení &amp; naše &lt;po&gt;kusy</comment>
   <test hello="&quot;earth&quot; &amp; &lt;M&gt;ars">
   </test>
  </DomesticTransaction>
 </Orders>
</Import>
"#;
        assert_eq!(expected_xml, xml_string);
        Ok(())
    }
}
