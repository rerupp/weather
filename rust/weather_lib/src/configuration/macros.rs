//! The configuration document macros that create the different document properties.
//!

/// This macro generates a `String` property. The property consists of an optional value and a default
/// value. If the value is not set, the default value is used. The resulting `struct` can be
/// serialized and deserialized by serde. The default value is skipped by `serde` when serializing and
/// deserializing the property.
///
/// # macro Arguments
///
/// * `$name` is used to as the `struct` name.
/// * `$serde_name` is the `serde` document field name.
/// * `$default_value` is used to initialize the default value.
/// * `$serde_default` is used by `serde` to initialize the default value.
///
macro_rules! string_property {
    ($name: ident, $serde_name: literal, $default_value: expr, $serde_default: literal) => {
        /// A serde compatible g property used by configuration documents.string property
        #[derive(Debug, Serialize, Deserialize)]
        pub struct $name {
            /// The set string property value.
            #[serde(rename = $serde_name)]
            value: Option<String>,
            /// The property default value.
            #[serde(skip_serializing, skip_deserializing, default = $serde_default)]
            default_value: String,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { value: None, default_value: $default_value }
            }
        }
        impl $name {
            /// Return the property value if it has been set otherwise return the default value.
            pub fn get(&self) -> &str {
                self.value.as_ref().unwrap_or_else(|| &self.default_value)
            }
            /// Set the property value unless what is being set equals the default value.
            ///
            /// # Arguments
            ///
            /// * `value` is the properties value.
            pub fn set(&mut self, value: &str) {
                // todo: do you need to pull the test or add a clear if default?
                if value != self.default_value.as_str() {
                    self.value.replace(value.to_string());
                }
            }
            /// If the value has not been set, force it to be the default value.
            pub fn with_default(&self) -> Self {
                let value = match &self.value {
                    None => self.default_value.clone(),
                    Some(v) => v.to_string(),
                };
                Self { value: Some(value), default_value: self.default_value.clone() }
            }
        }
    };
}
pub(super) use string_property;

/// This macro generates a `bool` property. The property consists of an optional value and a default
/// value. If the value is not set, the default value is used. The resulting `struct` can be
/// serialized and deserialized by serde. The default value is skipped by `serde` when serializing and
/// deserializing the property.
///
/// # macro Arguments
///
/// * `$name` is used to as the `struct` name.
/// * `$serde_name` is the `serde` document field name.
/// * `$default_value` is used to initialize the default value.
/// * `$serde_default` is used by `serde` to initialize the default value.
///
macro_rules! bool_property {
    ($name: ident, $serde_name: literal, $default_value: expr, $serde_default: literal) => {
        /// A serde compatible boolean property used by configuration documents.
        #[derive(Debug, Serialize, Deserialize)]
        pub struct $name {
            /// The set boolean property value.
            #[serde(rename = $serde_name)]
            /// The property default value.
            value: Option<bool>,
            #[serde(skip_serializing, skip_deserializing, default = $serde_default)]
            default_value: bool,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { value: None, default_value: $default_value }
            }
        }
        impl $name {
            /// Return the property value if it has been set otherwise return the default value.
            pub fn get(&self) -> bool {
                *self.value.as_ref().unwrap_or_else(|| &self.default_value)
            }
            /// Set the property value unless what is being set equals the default value.
            ///
            /// # Arguments
            ///
            /// * `value` is the properties value.
            pub fn set(&mut self, value: bool) {
                if value != self.default_value {
                    self.value.replace(value);
                }
            }
            /// If the value has not been set, force it to be the default value.
            pub fn with_default(&self) -> Self {
                let value = match &self.value {
                    None => self.default_value,
                    Some(v) => *v,
                };
                Self { value: Some(value), default_value: self.default_value }
            }
        }
    };
}
pub(super) use bool_property;

/// This macro generates a `usize` property. The property consists of an optional value and a default
/// value. If the value is not set, the default value is used. The resulting `struct` can be
/// serialized and deserialized by serde. The default value is skipped by `serde` when serializing and
/// deserializing the property.
///
/// # macro Arguments
///
/// * `$name` is used to as the `struct` name.
/// * `$serde_name` is the `serde` document field name.
/// * `$default_value` is used to initialize the default value.
/// * `$serde_default` is used by `serde` to initialize the default value.
///
macro_rules! usize_property {
    ($name: ident, $serde_name: literal, $default_value: expr, $serde_default: literal) => {
        #[derive(Debug, Serialize, Deserialize)]
        pub struct $name {
            #[serde(rename = $serde_name)]
            value: Option<usize>,
            #[serde(skip_serializing, skip_deserializing, default = $serde_default)]
            default_value: usize,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { value: None, default_value: $default_value }
            }
        }
        impl $name {
            /// Return the property value if it has been set otherwise return the default value.
            pub fn get(&self) -> usize {
                *self.value.as_ref().unwrap_or_else(|| &self.default_value)
            }
            /// Set the property value unless what is being set equals the default value.
            ///
            /// # Arguments
            ///
            /// * `value` is the properties value.
            pub fn set(&mut self, value: usize) {
                if value != self.default_value {
                    self.value.replace(value);
                }
            }
            /// If the value has not been set, force it to be the default value.
            pub fn with_default(&self) -> Self {
                let value = match &self.value {
                    None => self.default_value,
                    Some(v) => *v,
                };
                Self { value: Some(value), default_value: self.default_value }
            }
        }
    };
}
pub(super) use usize_property;

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[test]
    fn string_property() {
        fn default_string_property() -> String {
            "default_string_property".to_string()
        }
        string_property!(TestProperty, "string-field", default_string_property(), "default_string_property");
        let testcase = TestProperty::default();
        assert_eq!(testcase.get(), default_string_property());

        let default_value = "default_value";
        let testcase = TestProperty { value: None, default_value: default_value.to_string() };
        assert_eq!(testcase.get(), default_value);
        let toml_str = toml::to_string(&testcase).unwrap();
        assert!(toml_str.is_empty());

        // after deserialization the default value comes from serde
        let mut testcase = toml::from_str::<TestProperty>(&toml_str).unwrap();
        assert_eq!(testcase.get(), default_string_property());

        let set_value = "set_value";
        testcase.set(set_value);
        assert_eq!(testcase.get(), set_value);

        let toml_str = toml::to_string(&testcase).unwrap();
        assert!(toml_str.contains("string-field"));
        testcase = toml::from_str(toml_str.as_str()).unwrap();
        assert_eq!(testcase.get(), set_value);

        let testcase = TestProperty::default().with_default();
        assert_eq!(testcase.value.as_ref().unwrap(), &default_string_property());

        let toml_str = toml::to_string(&testcase).unwrap();
        assert!(toml_str.contains("string-field"));

        let testcase = toml::from_str::<TestProperty>(toml_str.as_str()).unwrap();
        assert_eq!(testcase.value.as_ref().unwrap(), &default_string_property());
    }

    #[test]
    fn bool_property() {
        fn default_bool_property() -> bool {
            false
        }
        bool_property!(TestProperty, "bool-field", default_bool_property(), "default_bool_property");
        let testcase = TestProperty::default();
        assert_eq!(testcase.get(), default_bool_property());

        let testcase = TestProperty { value: None, default_value: !default_bool_property() };
        assert_eq!(testcase.get(), !default_bool_property());
        let toml_str = toml::to_string(&testcase).unwrap();
        assert!(toml_str.is_empty());

        // after deserialization the default value comes from serde
        let mut testcase = toml::from_str::<TestProperty>(&toml_str).unwrap();
        assert_eq!(testcase.get(), default_bool_property());

        // change the value
        let set_value = !default_bool_property();
        testcase.set(set_value);
        assert_eq!(testcase.get(), set_value);

        let toml_str = toml::to_string(&testcase).unwrap();
        assert!(toml_str.contains("bool-field"));
        testcase = toml::from_str(toml_str.as_str()).unwrap();
        assert_eq!(testcase.get(), set_value);

        let testcase = TestProperty::default().with_default();
        assert_eq!(*testcase.value.as_ref().unwrap(), default_bool_property());

        let toml_str = toml::to_string(&testcase).unwrap();
        assert!(toml_str.contains("bool-field"));

        let testcase = toml::from_str::<TestProperty>(toml_str.as_str()).unwrap();
        assert_eq!(*testcase.value.as_ref().unwrap(), default_bool_property());
    }

    #[test]
    fn usize_property() {
        fn default_usize_property() -> usize {
            16
        }
        usize_property!(TestProperty, "usize-field", default_usize_property(), "default_usize_property");
        let testcase = TestProperty::default();
        assert_eq!(testcase.get(), default_usize_property());

        let testcase = TestProperty { value: None, default_value: 8 };
        assert_eq!(testcase.get(), 8);
        let toml_str = toml::to_string(&testcase).unwrap();
        assert!(toml_str.is_empty());

        // after deserialization the default value comes from serde
        let mut testcase = toml::from_str::<TestProperty>(&toml_str).unwrap();
        assert_eq!(testcase.get(), default_usize_property());

        // change the value
        let set_value = 32;
        testcase.set(set_value);
        assert_eq!(testcase.get(), set_value);

        let toml_str = toml::to_string(&testcase).unwrap();
        assert!(toml_str.contains("usize-field"));
        testcase = toml::from_str(toml_str.as_str()).unwrap();
        assert_eq!(testcase.get(), set_value);

        let testcase = TestProperty::default().with_default();
        assert_eq!(*testcase.value.as_ref().unwrap(), default_usize_property());

        let toml_str = toml::to_string(&testcase).unwrap();
        assert!(toml_str.contains("usize-field"));

        let testcase = toml::from_str::<TestProperty>(toml_str.as_str()).unwrap();
        assert_eq!(*testcase.value.as_ref().unwrap(), default_usize_property());
    }

    // #[test]
    #[allow(unused)]
    fn toml() {
        fn default_bool() -> bool {
            false
        }
        bool_property!(BoolProperty, "boolean", default_bool(), "default_bool");

        fn default_string() -> String {
            "default".into()
        }
        string_property!(StringProperty, "string", default_string(), "default_string");
        fn default_usize() -> usize {
            4
        }
        usize_property!(UsizeProperty, "usize", default_usize(), "default_usize");

        #[derive(Debug, Default, Serialize, Deserialize)]
        struct Testcase {
            #[serde(flatten)]
            string: StringProperty,
            #[serde(flatten)]
            boolean: BoolProperty,
            #[serde(flatten)]
            usize: UsizeProperty,
        }

        let testcase = Testcase::default();
        assert_eq!(testcase.string.get(), default_string());
        assert_eq!(testcase.boolean.get(), default_bool());
        assert_eq!(testcase.usize.get(), default_usize());

        let toml_str = toml::to_string(&testcase).unwrap();
        assert!(!toml_str.contains("boolean"));
        assert!(!toml_str.contains("string"));
        assert!(!toml_str.contains("usize"));

        let mut testcase = toml::from_str::<Testcase>(toml_str.as_str()).unwrap();
        assert_eq!(testcase.string.get(), default_string());
        assert_eq!(testcase.boolean.get(), default_bool());
        assert_eq!(testcase.usize.get(), default_usize());

        let set_string = "set value";
        let set_boolean = !default_bool();
        let set_usize = 64;
        testcase.string.set(set_string);
        testcase.boolean.set(set_boolean);
        testcase.usize.set(set_usize);
        let toml_str = toml::to_string(&testcase).unwrap();
        assert!(toml_str.contains("boolean"));
        assert!(toml_str.contains("string"));
        assert!(toml_str.contains("usize"));

        testcase = toml::from_str(toml_str.as_str()).unwrap();
        assert_eq!(testcase.string.get(), set_string);
        assert_eq!(testcase.boolean.get(), set_boolean);
        assert_eq!(testcase.usize.get(), set_usize);
    }
}
