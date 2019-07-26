pub mod errors;
pub mod extensions;
pub mod parser;

use errors::LocaleError;
pub use extensions::{ExtensionType, ExtensionsMap, UnicodeExtensionKey};
use std::str::FromStr;
pub use unic_langid::LanguageIdentifier;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Locale {
    pub langid: LanguageIdentifier,
    pub extensions: extensions::ExtensionsMap,
}

impl Locale {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_parts<S: AsRef<str>>(
        language: Option<S>,
        script: Option<S>,
        region: Option<S>,
        variants: Option<&[S]>,
        extensions: Option<extensions::ExtensionsMap>,
    ) -> Result<Self, LocaleError> {
        let langid = LanguageIdentifier::from_parts(language, script, region, variants)?;
        Ok(Locale {
            langid,
            extensions: extensions.unwrap_or_default(),
        })
    }

    pub fn matches(&self, other: &Self, self_as_range: bool, other_as_range: bool) -> bool {
        if !self.extensions.get_private().is_empty() || !other.extensions.get_private().is_empty() {
            return false;
        }
        self.langid
            .matches(&other.langid, self_as_range, other_as_range)
    }

    pub fn get_language(&self) -> &str {
        self.langid.get_language()
    }

    pub fn set_language(&mut self, language: Option<&str>) -> Result<(), LocaleError> {
        self.langid
            .set_language(language)
            .map_err(std::convert::Into::into)
    }

    pub fn get_script(&self) -> Option<&str> {
        self.langid.get_script()
    }

    pub fn set_script(&mut self, script: Option<&str>) -> Result<(), LocaleError> {
        self.langid
            .set_script(script)
            .map_err(std::convert::Into::into)
    }

    pub fn get_region(&self) -> Option<&str> {
        self.langid.get_region()
    }

    pub fn set_region(&mut self, region: Option<&str>) -> Result<(), LocaleError> {
        self.langid
            .set_region(region)
            .map_err(std::convert::Into::into)
    }

    pub fn get_variants(&self) -> Vec<&str> {
        self.langid.get_variants()
    }

    pub fn set_variants(&mut self, variants: &[&str]) -> Result<(), LocaleError> {
        self.langid
            .set_variants(variants)
            .map_err(std::convert::Into::into)
    }

    pub fn set_extension(
        &mut self,
        extension: ExtensionType,
        key: &str,
        value: Option<&str>,
    ) -> Result<(), LocaleError> {
        match extension {
            ExtensionType::Unicode => {
                let k = UnicodeExtensionKey::from_str(key)?;
                self.extensions.set_unicode_value(k, value)
            }
            _ => unimplemented!(),
        }
    }

    pub fn get_extensions(&self) -> &extensions::ExtensionsMap {
        &self.extensions
    }
}

impl FromStr for Locale {
    type Err = LocaleError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        parser::parse_locale(source).map_err(std::convert::Into::into)
    }
}

impl From<LanguageIdentifier> for Locale {
    fn from(langid: LanguageIdentifier) -> Self {
        Locale {
            langid,
            extensions: ExtensionsMap::default(),
        }
    }
}

impl Into<LanguageIdentifier> for Locale {
    fn into(self) -> LanguageIdentifier {
        self.langid
    }
}

impl AsRef<LanguageIdentifier> for Locale {
    fn as_ref(&self) -> &LanguageIdentifier {
        &self.langid
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut subtags = vec![self.langid.to_string()];
        let ext = self.extensions.to_string();

        if !ext.is_empty() {
            subtags.push(ext);
        }
        write!(f, "{}", subtags.join("-"))
    }
}

pub fn canonicalize(input: &str) -> Result<String, LocaleError> {
    let locale: Locale = input.parse()?;
    Ok(locale.to_string())
}
