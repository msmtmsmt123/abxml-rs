use model::owned::OwnedBuf;
use byteorder::{LittleEndian, WriteBytesExt};
use chunks::*;
use errors::*;
use std::rc::Rc;
use model::NamespaceEnd;
use model::StringTable;

pub struct XmlNamespaceEndBuf {
    line: u32,
    prefix_index: u32,
    namespace_index: u32,
}

impl XmlNamespaceEndBuf {
    pub fn new(line: u32, prefix_index: u32, namespace_index: u32) -> Self {
        XmlNamespaceEndBuf {
            line: line,
            prefix_index: prefix_index,
            namespace_index: namespace_index,
        }
    }
}

impl NamespaceEnd for XmlNamespaceEndBuf {
    fn get_line(&self) -> Result<u32> {
        Ok(self.line)
    }

    fn get_prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>> {
        let string = string_table.get_string(self.prefix_index)?;

        Ok(string)
    }

    fn get_namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>> {
        let string = string_table.get_string(self.namespace_index)?;

        Ok(string)
    }
}

impl OwnedBuf for XmlNamespaceEndBuf {
    fn get_token(&self) -> u16 {
        TOKEN_XML_END_NAMESPACE
    }

    fn get_body_data(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        out.write_u32::<LittleEndian>(self.prefix_index)?;
        out.write_u32::<LittleEndian>(self.namespace_index)?;

        Ok(out)
    }

    fn get_header(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        out.write_u32::<LittleEndian>(self.line)?;
        out.write_u32::<LittleEndian>(0xFFFFFFFF)?;

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunks::XmlNamespaceEndWrapper;
    use test::compare_chunks;
    use raw_chunks::EXAMPLE_NAMESPACE_END;

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let namespace_end = XmlNamespaceEndBuf::new(99, 1001, 2203);

        assert_eq!(99, namespace_end.get_line().unwrap());
    }

    #[test]
    fn identity() {
        let wrapper = XmlNamespaceEndWrapper::new(EXAMPLE_NAMESPACE_END);

        let owned = wrapper.to_buffer().unwrap();
        let new_raw = owned.to_vec().unwrap();

        compare_chunks(&new_raw, &EXAMPLE_NAMESPACE_END);

    }
}
