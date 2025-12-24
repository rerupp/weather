//! The Visual Crossing configuration data.
use super::*;

#[derive(Clone, Debug)]
pub struct Properties {
    pub endpoint: String,
    pub api_key: String,
}
impl Default for Properties {
    fn default() -> Self {
        Self::from(&Document::default())
    }
}
impl From<&Document> for Properties {
    /// Convert the document into the configuration table.
    fn from(document: &Document) -> Self {
        Self { endpoint: document.endpoint.get().to_string(), api_key: document.api_key.get().to_string() }
    }
}

macros::string_property!(Endpoint, "endpoint", Document::default_endpoint(), "Document::default_endpoint");
macros::string_property!(ApiKey, "api-key", Document::default_api_key(), "Document::default_api_key");

/// The Visual Crossing configuration options.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Document {
    #[serde(flatten)]
    pub endpoint: Endpoint,
    #[serde(flatten)]
    pub api_key: ApiKey
}
impl From<&Properties> for Document {
    fn from(properties: &Properties) -> Self {
        let mut self_ = Document::default();
        self_.endpoint.set(&properties.endpoint);
        self_.api_key.set(&properties.api_key);
        self_
    }
}
impl Document {
    pub fn with_defaults(&self) -> Self {
        Self { endpoint: self.endpoint.with_default(), api_key: self.api_key.with_default() }
    }

    /// The environment variable that can hold the Visual Crossing API key.
    const ENV_KEY: &'static str = "VISUAL_CROSSING_KEY";
    /// The default Visual Crossing API key.
    const DEFAULT_KEY: &'static str = "UNAVAILABLE";
    /// Used by serde when deserializing the document to get the default api key.
    fn default_api_key() -> String {
        env_var_or!(Self::ENV_KEY, Self::DEFAULT_KEY)
    }

    /// The environment variable that can hold the Visual Crossing endpoint.
    const ENV_ENDPOINT: &'static str = "VISUAL_CROSSING_ENDPOINT";
    /// The default Visual Crossing API endpoint.
    const DEFAULT_ENDPOINT: &'static str =
        "https://weather.visualcrossing.com/VisualCrossingWebServices/rest/services/timeline";
    /// Used by serde when deserializing the document to get the default endpoint.
    fn default_endpoint() -> String {
        env_var_or!(Self::ENV_ENDPOINT, Self::DEFAULT_ENDPOINT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_document() {
        // this should be the only test where the process environment needs to be threadsafe
        let env_endpoint = "http:://env/endpoint";
        let env_api_key = "env.api.key";
        env::set_var(Document::ENV_ENDPOINT, env_endpoint);
        env::set_var(Document::ENV_KEY, env_api_key);
        let testcase = Document::default();
        env::remove_var(Document::ENV_ENDPOINT);
        env::remove_var(Document::ENV_KEY);

        // these tests are fragile due to the process environment
        assert_eq!(testcase.endpoint.get(), env_endpoint);
        assert_eq!(testcase.api_key.get(), env_api_key);
    }

    #[test]
    fn toml_document() {
        // these tests are fragile due to the process environment
        env::remove_var(Document::ENV_ENDPOINT);
        env::remove_var(Document::ENV_KEY);

        #[derive(Debug, Default, Serialize, Deserialize)]
        struct DocumentTestcase {
            #[serde(rename = "visual-crossing", default = "Document::default")]
            pub visual_crossing: Document,
        }
        // these tests are fragile because they rely on the process environment state
        let default = Document::default();
        assert_eq!(default.endpoint.get(), Document::default_endpoint());
        assert_eq!(default.api_key.get(), Document::default_api_key());

        // no document
        let testcase = toml::from_str::<DocumentTestcase>("").unwrap();
        assert_eq!(testcase.visual_crossing.endpoint.get(), default.endpoint.get());
        assert_eq!(testcase.visual_crossing.api_key.get(), default.api_key.get());

        // default document
        let testcase_str = toml::to_string(&DocumentTestcase::default()).unwrap();
        let testcase = toml::from_str::<DocumentTestcase>(&testcase_str).unwrap();
        assert_eq!(testcase.visual_crossing.endpoint.get(), default.endpoint.get());
        assert_eq!(testcase.visual_crossing.api_key.get(), default.api_key.get());

        // these tests are fragile due to accessing the process environment
        let set_endpoint = "http://set/end_point";
        let mut endpoint = Endpoint::default();
        endpoint.set(set_endpoint);
        let set_api_key = "testcase.api.key";
        let mut api_key = ApiKey::default();
        api_key.set(set_api_key);
        let document = Document { endpoint, api_key };
        let testcase_str = toml::to_string(&DocumentTestcase { visual_crossing: document }).unwrap();
        let testcase = toml::from_str::<DocumentTestcase>(&testcase_str).unwrap();
        assert_eq!(testcase.visual_crossing.endpoint.get(), set_endpoint);
        assert_eq!(testcase.visual_crossing.api_key.get(), set_api_key);
    }

    #[test]
    fn properties() {
        let mut document = Document::default();
        assert_eq!(document.endpoint.get(), Document::default_endpoint());
        assert_eq!(document.api_key.get(), Document::default_api_key());

        let set_endpoint = "http://set/end_point";
        let set_api_key = "set.api.key";
        document.endpoint.set(set_endpoint);
        document.api_key.set(set_api_key);
        let testcase = Properties::from(&document);
        assert_eq!(testcase.endpoint, set_endpoint);
        assert_eq!(testcase.api_key, set_api_key);

        // these tests are fragile because the process environment is not threadsafe
        let test_document = Document::from(&testcase);
        assert_eq!(test_document.endpoint.get(), set_endpoint);
        assert_eq!(test_document.api_key.get(), set_api_key);
    }
}
