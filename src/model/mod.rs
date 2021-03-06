//! Representations of logical structures found on android binary files

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use model::owned::Entry;
use errors::*;

mod element;
mod value;
pub mod owned;
pub mod builder;

pub use self::element::Element;
pub use self::element::Tag;
pub use self::element::ElementContainer;
pub use self::value::Value;

use visitor::Origin;

pub type Namespaces = BTreeMap<String, String>;
pub type Entries = HashMap<u32, Entry>;

pub trait Identifier {
    fn get_package(&self) -> u8;
    fn get_spec(&self) -> u8;
    fn get_id(&self) -> u16;
}

impl Identifier for u32 {
    fn get_package(&self) -> u8 {
        let mut package_id = (self >> 24) as u8;

        if package_id == 0 {
            package_id = 1;
            info!("Resource with package id 0 found. Recreate id with current package id");
        }

        package_id
    }

    fn get_spec(&self) -> u8 {
        ((self & 0x00FF0000) >> 16) as u8
    }

    fn get_id(&self) -> u16 {
        (self & 0xFFFF) as u16
    }
}

// Traits
pub trait StringTable {
    fn get_strings_len(&self) -> u32;
    fn get_styles_len(&self) -> u32;
    fn get_string(&self, idx: u32) -> Result<Rc<String>>;
}

// TODO: Decide if the trait should return Results or not
pub trait Package {
    fn get_id(&self) -> Result<u32>;
    fn get_name(&self) -> Result<String>;
}

pub trait Library {
    fn get_name(&self) -> Option<String>;
    fn format_reference(
        &self,
        id: u32,
        key: u32,
        namespace: Option<String>,
        prefix: &str,
    ) -> Result<String>;
    // fn get_entries(&self) -> &Entries;
    fn get_entry(&self, id: u32) -> Result<&Entry>;
    fn get_entries_string(&self, str_id: u32) -> Result<Rc<String>>;
    fn get_spec_string(&self, str_id: u32) -> Result<Rc<String>>;
}

pub trait LibraryBuilder<'a> {
    type StringTable: StringTable;
    type TypeSpec: TypeSpec;

    fn set_string_table(&mut self, string_table: Self::StringTable, origin: Origin);
    fn add_entries(&mut self, entries: Entries);
    fn add_type_spec(&mut self, type_spec: Self::TypeSpec);
}

pub trait Resources<'a> {
    type Library: Library + LibraryBuilder<'a>;

    fn get_package(&self, package_id: u8) -> Option<&Self::Library>;
    fn get_mut_package(&mut self, package_id: u8) -> Option<&mut Self::Library>;
    fn get_main_package(&self) -> Option<&Self::Library>;
    fn is_main_package(&self, package_id: u8) -> bool;
}

/// Trait that represents a XML tag start
pub trait TagStart {
    /// Type of the attributes
    type Attribute: AttributeTrait;

    /// Return the ¿line in which the tag appear?
    fn get_line(&self) -> Result<u32>;
    /// Return the content of the unknown field1
    fn get_field1(&self) -> Result<u32>;
    /// Return the namespace index. If there is no namespace, it will return 0xFFFFFFFF
    fn get_namespace_index(&self) -> Result<u32>;
    /// Returns the index of the tag name on the string table
    fn get_element_name_index(&self) -> Result<u32>;
    /// Return the content of the unknown field1
    fn get_field2(&self) -> Result<u32>;
    /// Return the amount of attributes this tag contains
    fn get_attributes_amount(&self) -> Result<u32>;
    /// Returns the ¿class?
    fn get_class(&self) -> Result<u32>;
    /// Returns the attribute on the `index` position or error if it is greater than
    /// `get_attributes_amount`
    fn get_attribute(&self, index: u32) -> Result<Self::Attribute>;
}

pub trait AttributeTrait {
    /// Return the namespace index. If there is no namespace, it will return 0xFFFFFFFF
    fn get_namespace(&self) -> Result<u32>;
    /// Returns the index of the attribute on the string table
    fn get_name(&self) -> Result<u32>;
    /// Returns the ¿class?
    fn get_class(&self) -> Result<u32>;
    /// Returns the data type (see `Values`)
    fn get_resource_value(&self) -> Result<u32>;
    /// Returns the value (see `Values`)
    fn get_data(&self) -> Result<u32>;

    /// Creates a `Value` depending on the data type and data value
    fn get_value(&self) -> Result<Value> {
        let data_type = ((self.get_resource_value()? >> 24) & 0xFF) as u8;
        let data_value = self.get_data()?;
        let class = self.get_class()?;

        let value = if class == 0xFFFFFFFF {
            Value::new(data_type, data_value)?
        } else {
            Value::StringReference(class)
        };

        Ok(value)
    }
}

pub trait TagEnd {
    fn get_id(&self) -> Result<u32>;
}

pub trait NamespaceStart {
    fn get_line(&self) -> Result<u32>;
    fn get_prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>>;
    fn get_namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>>;
}

pub trait NamespaceEnd {
    fn get_line(&self) -> Result<u32>;
    fn get_prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>>;
    fn get_namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>>;
}

pub trait TypeSpec {
    fn get_id(&self) -> Result<u16>;
    fn get_amount(&self) -> Result<u32>;
    fn get_flag(&self, index: u32) -> Result<u32>;
}

pub trait TableType {
    type Configuration: Configuration;

    fn get_id(&self) -> Result<u8>;
    fn get_amount(&self) -> Result<u32>;
    fn get_configuration(&self) -> Result<Self::Configuration>;
    fn get_entry(&self, index: u32) -> Result<Entry>;
}

pub trait Configuration {
    fn get_size(&self) -> Result<u32>;
    fn get_mcc(&self) -> Result<u16>;
    fn get_mnc(&self) -> Result<u16>;
    fn get_language(&self) -> Result<String>;
    fn get_region(&self) -> Result<String>;
    fn get_orientation(&self) -> Result<u8>;
    fn get_touchscreen(&self) -> Result<u8>;
    fn get_density(&self) -> Result<u16>;
    fn get_keyboard(&self) -> Result<u8>;
    fn get_navigation(&self) -> Result<u8>;
    fn get_input_flags(&self) -> Result<u8>;
    fn get_width(&self) -> Result<u16>;
    fn get_height(&self) -> Result<u16>;
    fn get_sdk_version(&self) -> Result<u16>;
    fn get_min_sdk_version(&self) -> Result<u16>;
    fn get_screen_layout(&self) -> Result<u8>;
    fn get_ui_mode(&self) -> Result<u8>;
    fn get_smallest_screen(&self) -> Result<u16>;
    fn get_screen_width(&self) -> Result<u16>;
    fn get_screen_height(&self) -> Result<u16>;
    fn get_locale_script(&self) -> Result<Option<String>>;
    fn get_locale_variant(&self) -> Result<Option<String>>;
    fn get_secondary_layout(&self) -> Result<Option<u8>>;
}

#[cfg(test)]
mod tests {
    use model::Identifier;

    #[test]
    fn it_extracts_package_id() {
        assert_eq!(2130837685.get_package(), 127)
    }

    #[test]
    fn it_converts_package_id_if_is_0() {
        assert_eq!(131253.get_package(), 1)
    }

    #[test]
    fn it_extracts_spec_id() {
        assert_eq!(2130837685.get_spec(), 2)
    }

    #[test]
    fn it_extracts_id() {
        assert_eq!(2130837685.get_id(), 181)
    }
}
