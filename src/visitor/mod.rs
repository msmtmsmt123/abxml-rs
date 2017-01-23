use std::io::Cursor;
use chunks::*;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use std::marker::PhantomData;
use std::collections::HashMap;
use chunks::table_type::Entry;
use std::rc::Rc;
use document::{Namespaces, Element, ElementContainer, Entries};

pub trait ChunkVisitor<'a> {
    fn visit_string_table(&mut self, mut string_table: StringTable<'a>) {}
    fn visit_package(&mut self, mut package: Package<'a>) {}
    fn visit_table_type(&mut self, mut table_type: TableType<'a>) {}
    fn visit_type_spec(&mut self, mut type_spec: TypeSpec<'a>) {}
    fn visit_xml_namespace_start(&mut self, mut namespace_start: XmlNamespaceStart<'a>) {}
    fn visit_xml_namespace_end(&mut self, mut namespace_end: XmlNamespaceEnd<'a>) {}
    fn visit_xml_tag_start(&mut self, mut tag_start: XmlTagStart<'a>) {}
    fn visit_xml_tag_end(&mut self, mut tag_end: XmlTagEnd<'a>) {}
    fn visit_resource(&mut self, mut resource: Resource<'a>) {}
}

pub struct Executor;

impl Executor {
    pub fn arsc<'a, V: ChunkVisitor<'a>>(mut cursor: Cursor<&'a [u8]>, mut visitor: &mut V) -> Result<()> {
        let token = cursor.read_u16::<LittleEndian>()?;
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let chunk_size = cursor.read_u32::<LittleEndian>()?;
        let package_amount = cursor.read_u32::<LittleEndian>()?;

        let stream = ChunkLoaderStream::new(cursor);

        for c in stream {
            match c? {
                Chunk::StringTable(stw) => {
                    let mut st = StringTable::new(stw);
                    visitor.visit_string_table(st);
                },
                Chunk::Package(pw) => {
                    let mut package = Package::new(pw);
                    visitor.visit_package(package);
                },
                Chunk::TableType(ttw) => {
                    let mut tt = TableType::new(ttw);
                    visitor.visit_table_type(tt);
                },
                Chunk::TableTypeSpec(tsw) => {
                    let mut ts = TypeSpec::new(tsw);
                    visitor.visit_type_spec(ts);
                },
                _ => (),
            }
        }

        Ok(())
    }

    pub fn xml<'a, 'b, V: ChunkVisitor<'a>>(mut cursor: Cursor<&'a [u8]>, mut visitor: &mut V, resources: &mut Resources) -> Result<()> {
        let token = cursor.read_u16::<LittleEndian>()?;
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let chunk_size = cursor.read_u32::<LittleEndian>()?;

        let stream = ChunkLoaderStream::new(cursor);

        for c in stream {
            match c? {
                Chunk::StringTable(stw) => {
                    let mut st = StringTable::new(stw);
                    visitor.visit_string_table(st);
                },
                Chunk::Package(pw) => {
                    let mut package = Package::new(pw);
                    visitor.visit_package(package);
                },
                Chunk::TableType(ttw) => {
                    let mut tt = TableType::new(ttw);
                    visitor.visit_table_type(tt);
                },
                Chunk::TableTypeSpec(tsw) => {
                    let mut ts = TypeSpec::new(tsw);
                    visitor.visit_type_spec(ts);
                },
                Chunk::XmlNamespaceStart(xnsw) => {
                    let mut ts = XmlNamespaceStart::new(xnsw);
                    visitor.visit_xml_namespace_start(ts);
                },
                Chunk::XmlNamespaceEnd(xnsw) => {
                    let mut ts = XmlNamespaceEnd::new(xnsw);
                    visitor.visit_xml_namespace_end(ts);
                },
                Chunk::XmlTagStart(xnsw) => {
                    let mut ts = XmlTagStart::new(xnsw);
                    visitor.visit_xml_tag_start(ts);
                },
                Chunk::XmlTagEnd(xnsw) => {
                    let mut ts = XmlTagEnd::new(xnsw);
                    visitor.visit_xml_tag_end(ts);
                },
                Chunk::Resource(rw) => {
                    let mut ts = Resource::new(rw);
                    visitor.visit_resource(ts);
                }
                _ => (),
            }
        }

        Ok(())
    }
}

pub struct DummyVisitor;

impl<'a> ChunkVisitor<'a> for DummyVisitor {}

pub struct PrintVisitor;

impl<'a> ChunkVisitor<'a> for PrintVisitor {
    fn visit_string_table(&mut self, mut string_table: StringTable) {
        println!("String Table!");
        println!("\tLength: {}", string_table.get_strings_len());

        for i in 0..string_table.get_strings_len()-1 {
            match string_table.get_uncached_string(i) {
                Ok(s) => {
                    println!("\tString #{}: {}", i, string_table.get_uncached_string(i).unwrap());
                },
                Err(_) => {
                    println!("ERROR: String not found");
                },
            }
        }
    }

    fn visit_package(&mut self, mut package: Package) {
        println!("Package!");
        println!("\tId: {}", package.get_id());
        println!("\tName: {}", package.get_name().unwrap());
    }

    fn visit_table_type(&mut self, mut table_type: TableType) {
        println!("Table type!");
        println!("\tId: {}", table_type.get_id());
    }

    fn visit_type_spec(&mut self, mut type_spec: TypeSpec) {
        println!("Type spec!");
        println!("\tId: {}", type_spec.get_id());
    }
}

pub struct XmlVisitor<'a> {
    main_string_table: Option<StringTable<'a>>,
    namespaces: Namespaces,
    container: ElementContainer,
}

impl<'a> XmlVisitor<'a> {
    pub fn new() -> Self {
        XmlVisitor {
            main_string_table: None,
            namespaces: Namespaces::new(),
            container: ElementContainer::new(),
        }
    }

    pub fn get_namespaces(&self) -> &Namespaces {
        &self.namespaces
    }

    pub fn get_root(&self) -> &Option<Element> {
        &self.container.get_root()
    }

    pub fn get_string_table(&self) -> &Option<StringTable> {
        &self.main_string_table
    }
}

impl <'a> ChunkVisitor<'a> for XmlVisitor<'a> {
    fn visit_string_table(&mut self, mut string_table: StringTable<'a>) {
        match self.main_string_table {
            Some(_) => {
                println!("Secondary table!");
            },
            None => {
                self.main_string_table = Some(string_table);
            },
        }
    }

    fn visit_xml_namespace_start(&mut self, mut namespace_start: XmlNamespaceStart<'a>) {
        match self.main_string_table {
            Some(ref mut string_table) => {
                self.namespaces.insert(
                    namespace_start.get_namespace(string_table).unwrap(),
                    namespace_start.get_prefix(string_table).unwrap(),
                );
            },
            None => {
                println!("No main string table found!");
            }
        }
    }

    fn visit_xml_tag_start(&mut self, mut tag_start: XmlTagStart<'a>) {
        match self.main_string_table {
            Some(ref mut string_table) => {
                let (attributes, element_name) = tag_start.get_tag(&self.namespaces, string_table).unwrap();
                let element = Element::new(element_name, attributes);
                self.container.start_element(element);
            },
            None => {
                println!("No main string table found!");
            }
        }
    }

    fn visit_xml_tag_end(&mut self, mut tag_end: XmlTagEnd<'a>) {
        self.container.end_element()
    }
}

pub struct ModelVisitor<'a> {
    package_mask: u32,
    resources: Resources<'a>,
    current_spec: Option<TypeSpec<'a>>,
}

impl<'a> ModelVisitor<'a> {
    pub fn new() -> ModelVisitor<'a> {
        ModelVisitor {
            package_mask: 0,
            resources: Resources::new(),
            current_spec: None,
        }
    }
/*
    pub fn get_entries(&self) -> &HashMap<u32, Entry> {
        &self.entries
    }

    pub fn get_string_table(&self) -> &Option<StringTable> {
        &self.main_string_table
    }

    pub fn get_spec_string_table(&self) -> &Option<StringTable> {
        &self.spec_string_table
    }

    pub fn get_entries_string_table(&self) -> &Option<StringTable> {
        &self.entries_string_table
    }
*/
    pub fn get_resources(&self) -> &'a Resources {
        &self.resources
    }

    pub fn get_mut_resources(&mut self) -> &'a mut Resources {
        &mut self.resources
    }
}

impl<'a> ChunkVisitor<'a> for ModelVisitor<'a> {
    fn visit_string_table(&mut self, mut string_table: StringTable<'a>) {
        self.resources.set_string_table(string_table);
    }

    fn visit_package(&mut self, mut package: Package<'a>) {
        self.package_mask = package.get_id() << 24;
        self.resources.add_package(package);
    }

    fn visit_table_type(&mut self, mut table_type: TableType<'a>) {
        match self.current_spec {
            Some(ref ts) => {
                let mask = self.package_mask |
                    ((ts.get_id() as u32) << 16);
                let entries = table_type.get_entries(ts, mask).unwrap();
                self.resources.add_entries(entries);
            },
            None => (),
        }
    }

    fn visit_type_spec(&mut self, mut type_spec: TypeSpec<'a>) {
        self.current_spec = Some(type_spec.clone());
        self.resources.add_type_spec(type_spec);
    }
}

pub struct Resources<'a> {
    packages: Vec<Package<'a>>,
    specs: Vec<TypeSpec<'a>>,
    string_table: Option<StringTable<'a>>,
    spec_string_table: Option<StringTable<'a>>,
    entries_string_table: Option<StringTable<'a>>,
    entries: Entries,
}

impl<'a> Resources<'a> {
    pub fn new() -> Self {
        Resources {
            packages: Vec::new(),
            specs: Vec::new(),
            string_table: None,
            spec_string_table: None,
            entries_string_table: None,
            entries: Entries::new(),
        }
    }

    pub fn set_string_table(&mut self, string_table: StringTable<'a>) {
        if self.packages.is_empty() {
            self.string_table = Some(string_table);
        } else {
            if self.spec_string_table.is_none() {
                self.spec_string_table = Some(string_table);
            } else {
                self.entries_string_table = Some(string_table);
            }
        }
    }

    pub fn add_package(&mut self, package: Package<'a>) {
        self.packages.push(package);
    }

    pub fn add_entries(&mut self, entries: Entries) {
        self.entries.extend(entries);
    }

    pub fn add_type_spec(&mut self, type_spec: TypeSpec<'a>) {
        self.specs.push(type_spec);
    }

    pub fn get_entries(&self) -> &Entries {
        &self.entries
    }
}