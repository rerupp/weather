//! The Visual Crossing configuration data.

use super::*;

/// The properties for US Cities.
#[derive(Clone, Debug)]
pub struct Properties {
    pub filename: String,
}
impl Default for Properties {
    fn default() -> Self {
        Self::from(&Document::default())
    }
}
impl From<&Document> for Properties {
    fn from(document: &Document) -> Self {
        Self { filename: document.filename.get().to_string() }
    }
}

macros::string_property!(Filename, "filename", Document::default_filename(), "Document::default_filename");

/// The US Cities configuration options document.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Document {
    #[serde(flatten)]
    pub filename: Filename,
}
impl From<&Properties> for Document {
    /// Creates the document from a properties instance filtering out default values.
    fn from(properties: &Properties) -> Self {
        let mut self_ = Document::default();
        self_.filename.set(&properties.filename);
        self_
    }
}
impl Document {
    /// Creates a clone of the document setting empty serializable attributes to default values.
    pub fn with_defaults(&self) -> Self {
        Self { filename: self.filename.with_default() }
    }

    /// The environment variable name that can hold the US cities filename.
    const ENV_FILENAME: &'static str = "USCITIES_FILENAME";
    /// The default US cities filename.
    const DEFAULT_FILENAME: &'static str = "uscities.csv";
    /// The default filename.
    fn default_filename() -> String {
        env_var_or!(Self::ENV_FILENAME, Self::DEFAULT_FILENAME)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_document() {
        // these tests are fragile due to the process environment

        // this should be the only test where the process state needs to be threadsafe
        let env_filename = "env_filename.ext";
        env::set_var(Document::ENV_FILENAME, env_filename);
        let testcase = Document::default();
        env::remove_var(Document::ENV_FILENAME);

        // assert_eq!(testcase.default_filename, env_filename);
        assert_eq!(testcase.filename.get(), env_filename);
    }

    #[test]
    fn toml_document() {
        env::remove_var(Document::ENV_FILENAME);
        
        #[derive(Debug, Default, Serialize, Deserialize)]
        struct DocumentTestcase {
            #[serde(rename = "us-cities", default = "Document::default")]
            pub us_cities: Document,
        }

        // the serde deserialization makes this test fragile because it can use environment variables
        // to get the default properties and cargo tests are multithreaded
        let default = Document::default();

        // no document
        let document_testcase = toml::from_str::<DocumentTestcase>("").unwrap();
        assert_eq!(document_testcase.us_cities.filename.get(), default.filename.get());

        // default document
        let testcase = DocumentTestcase { us_cities: Document::default() };
        let testcase_str = toml::to_string(&testcase).unwrap();
        let document_testcase = toml::from_str::<DocumentTestcase>(&testcase_str).unwrap();
        assert_eq!(document_testcase.us_cities.filename.get(), default.filename.get());

        // not a default document
        let set_filename = "toml_filename.file";
        let mut testcase_document = Document::default();
        // testcase_document.set_filename(toml_filename);
        testcase_document.filename.set(set_filename);
        let testcase = DocumentTestcase { us_cities: testcase_document };
        let testcase_str = toml::to_string(&testcase).unwrap();
        let document_testcase = toml::from_str::<DocumentTestcase>(&testcase_str).unwrap();
        assert_eq!(document_testcase.us_cities.filename.get(), set_filename);
    }

    #[test]
    fn properties() {
        let mut document = Document::default();
        assert_eq!(document.filename.get(), Document::default_filename());

        // with nondefault filename
        let set_filename = "set_filename.txt";
        document.filename.set(set_filename);
        let testcase = Properties::from(&document);
        assert_eq!(testcase.filename, set_filename);

        // these tests are fragile because the process environment is not threadsafe
        let document_testcase = Document::from(&testcase);
        assert_eq!(document_testcase.filename.get(), set_filename);
    }
}
