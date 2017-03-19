use xml::writer::{EventWriter, EmitterConfig, XmlEvent};
use xml::common::XmlVersion;
use model::Element as AbxmlElement;
use std::ops::Deref;
use std::io::Write;
use errors::*;
use model::Namespaces;

pub struct Xml;

impl Xml {
    pub fn encode(namespaces: &Namespaces, element: &AbxmlElement) -> Result<String> {
        let target: Vec<u8> = Vec::new();
        let mut writer = EmitterConfig::new().perform_indent(true).create_writer(target);

        let version = XmlVersion::Version10;
        writer.write(XmlEvent::StartDocument {
                         version: version,
                         encoding: None,
                         standalone: Some(false),
                     })?;
        Self::encode_element(&mut writer, namespaces, element)
            .chain_err(|| "Error decoding an element")?;

        let inner = writer.into_inner();
        String::from_utf8(inner).chain_err(|| "Could not export XML")
    }

    fn encode_element<W: Write>(writer: &mut EventWriter<W>,
                                namespaces: &Namespaces,
                                element: &AbxmlElement)
                                -> Result<()> {

        let tag = element.get_tag();
        let tag_name = tag.get_name();
        let prefix = tag.get_prefix();
        let mut xml_element = XmlEvent::start_element(tag_name.deref().as_str());

        for (k, v) in element.get_attributes() {
            xml_element = xml_element.attr(k.as_str(), v);
        }

        if prefix.is_some() {
            let prefix = prefix.unwrap();
            let uri_result = namespaces.get(prefix.deref());

            if uri_result.is_some() {
                let uri_result = uri_result.unwrap();
                let str_prefix = prefix.as_str();

                xml_element = xml_element.ns(uri_result.as_str(), str_prefix);
            }
        }

        writer.write(xml_element)?;

        for child in element.get_children() {
            Self::encode_element(writer, namespaces, child)?;
        }

        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
