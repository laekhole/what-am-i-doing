//! Product text only. Session content and machine-readable identifiers stay intact.
use std::sync::{atomic::{AtomicBool, Ordering}, LazyLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Korean,
    English,
}

impl Language {
    pub fn from_code(code: &str) -> Option<Self> {
        match code.split(['-', '_', '.', '@']).next()?.to_ascii_lowercase().as_str() {
            "ko" => Some(Self::Korean),
            "en" => Some(Self::English),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self { Self::Korean => "ko", Self::English => "en" }
    }

    pub fn text<'a>(self, ko: &'a str, en: &'a str) -> &'a str {
        match self { Self::Korean => ko, Self::English => en }
    }

    pub fn detect() -> Self {
        if let Some(lang) = std::env::var("WAID_LANG").ok().as_deref().and_then(Self::from_code) {
            return lang;
        }
        #[cfg(windows)]
        {
            #[link(name = "kernel32")]
            extern "system" { fn GetUserDefaultUILanguage() -> u16; }
            // Primary LANG_KOREAN, independent of the region sublanguage.
            return if unsafe { GetUserDefaultUILanguage() } & 0x3ff == 0x12 {
                Self::Korean
            } else { Self::English };
        }
        #[cfg(not(windows))]
        {
            for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
                if let Ok(code) = std::env::var(key) {
                    if !code.is_empty() { return Self::from_code(&code).unwrap_or(Self::English); }
                }
            }
            #[cfg(target_os = "macos")]
            if let Ok(output) = std::process::Command::new("/usr/bin/defaults")
                .args(["read", "-g", "AppleLanguages"]).output()
            {
                if output.status.success() {
                    if let Some(code) = String::from_utf8_lossy(&output.stdout)
                        .split(|c: char| !c.is_ascii_alphabetic() && c != '-')
                        .find(|s| !s.is_empty())
                    {
                        return Self::from_code(code).unwrap_or(Self::English);
                    }
                }
            }
            Self::English
        }
    }
}

// UI events change one application-wide preference; collector threads read it.
static ENGLISH: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(Language::detect() == Language::English));

pub fn language() -> Language {
    if ENGLISH.load(Ordering::Relaxed) { Language::English } else { Language::Korean }
}

pub fn set_language(language: Language) {
    ENGLISH.store(language == Language::English, Ordering::Relaxed);
}

pub fn tr<'a>(ko: &'a str, en: &'a str) -> &'a str { language().text(ko, en) }

/// Each branch is a literal so Rust checks format arguments in both languages.
#[macro_export]
macro_rules! trf {
    ($ko:literal, $en:literal $(, $($args:tt)*)?) => {
        match $crate::i18n::language() {
            $crate::i18n::Language::Korean => format!($ko $(, $($args)*)?),
            $crate::i18n::Language::English => format!($en $(, $($args)*)?),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locales_and_product_text_preserve_content() {
        for code in ["ko", "ko-KR", "ko_KR.UTF-8", "KO"] {
            assert_eq!(Language::from_code(code), Some(Language::Korean));
        }
        for code in ["en", "en-US", "en_GB.UTF-8", "EN"] {
            assert_eq!(Language::from_code(code), Some(Language::English));
        }
        for code in ["", "korean", "english", "fr", "<script>"] {
            assert_eq!(Language::from_code(code), None);
        }
        assert_eq!(Language::Korean.text("내 차례", "Waiting"), "내 차례");
        assert_eq!(Language::English.text("내 차례", "Waiting"), "Waiting");
        for lang in [Language::Korean, Language::English] {
            assert_eq!(Language::from_code(lang.code()), Some(lang));
            assert_eq!(lang.text("원문 그대로", "원문 그대로"), "원문 그대로");
        }
    }
}
