use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontGlyphSet {
    pub preset: FontGlyphPreset,
    pub extra: String,
}

impl Default for FontGlyphSet {
    fn default() -> Self {
        Self {
            preset: FontGlyphPreset::ConsoleLatinExt,
            extra: String::new(),
        }
    }
}

impl FontGlyphSet {
    pub fn characters(&self, missing_glyph: char) -> Vec<char> {
        let mut chars = BTreeSet::new();

        for ch in self.preset.characters().chars() {
            chars.insert(ch);
        }

        for ch in self.extra.chars() {
            chars.insert(ch);
        }

        chars.insert(' ');
        chars.insert('\t');
        chars.insert(missing_glyph);
        chars.into_iter().collect()
    }

    pub fn cache_key(&self) -> String {
        let mut value = self.preset.as_str().to_owned();
        if !self.extra.is_empty() {
            value.push(':');
            value.push_str(&self.extra);
        }
        value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontGlyphPreset {
    Ascii,
    Latin,
    LatinExt,
    Polish,
    ConsoleLatinExt,
    Custom,
}

impl FontGlyphPreset {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "ascii" => Self::Ascii,
            "latin" => Self::Latin,
            "latin-ext" | "latin_ext" => Self::LatinExt,
            "polish" | "pl" => Self::Polish,
            "console" | "console-latin" | "console-latin-ext" => Self::ConsoleLatinExt,
            "custom" => Self::Custom,
            _ => Self::ConsoleLatinExt,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ascii => "ascii",
            Self::Latin => "latin",
            Self::LatinExt => "latin-ext",
            Self::Polish => "polish",
            Self::ConsoleLatinExt => "console-latin-ext",
            Self::Custom => "custom",
        }
    }

    pub fn characters(self) -> &'static str {
        match self {
            Self::Ascii => ASCII,
            Self::Latin => LATIN,
            Self::LatinExt => LATIN_EXT,
            Self::Polish => POLISH,
            Self::ConsoleLatinExt => CONSOLE_LATIN_EXT,
            Self::Custom => "",
        }
    }
}

const ASCII: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,;:!?\"'`~@#$%^&*()[]{}<>/\\|+-_=0123456789";

const LATIN: &str = concat!(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789",
    " .,;:!?\"'`~@#$%^&*()[]{}<>/\\|+-_=",
    "ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞß",
    "àáâãäåæçèéêëìíîïðñòóôõöøùúûüýþÿ",
);

const POLISH: &str = concat!(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789",
    "ąćęłńóśżźĄĆĘŁŃÓŚŻŹ",
    " .,;:!?\"'`~@#$%^&*()[]{}<>/\\|+-_=",
);

const LATIN_EXT: &str = concat!(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789",
    " .,;:!?\"'`~@#$%^&*()[]{}<>/\\|+-_=",
    "ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞß",
    "àáâãäåæçèéêëìíîïðñòóôõöøùúûüýþÿ",
    "ĀāĂăĄąĆćĈĉĊċČčĎďĐđĒēĔĕĖėĘęĚě",
    "ĜĝĞğĠġĢģĤĥĦħĨĩĪīĬĭĮįİıĲĳĴĵ",
    "ĶķĸĹĺĻļĽľĿŀŁłŃńŅņŇňŉŊŋŌōŎŏŐő",
    "ŒœŔŕŖŗŘřŚśŜŝŞşŠšŢţŤťŦŧŨũŪūŬŭ",
    "ŮůŰűŲųŴŵŶŷŸŹźŻżŽž",
);

const CONSOLE_LATIN_EXT: &str = concat!(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789",
    " .,;:!?\"'`~@#$%^&*()[]{}<>/\\|+-_=",
    "ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞß",
    "àáâãäåæçèéêëìíîïðñòóôõöøùúûüýþÿ",
    "ĀāĂăĄąĆćĈĉĊċČčĎďĐđĒēĔĕĖėĘęĚě",
    "ĜĝĞğĠġĢģĤĥĦħĨĩĪīĬĭĮįİıĲĳĴĵ",
    "ĶķĸĹĺĻļĽľĿŀŁłŃńŅņŇňŉŊŋŌōŎŏŐő",
    "ŒœŔŕŖŗŘřŚśŜŝŞşŠšŢţŤťŦŧŨũŪūŬŭ",
    "ŮůŰűŲųŴŵŶŷŸŹźŻżŽž",
    "\n\r\t",
    "→←↑↓↔↕",
    "✓✗×•·…",
    "█▓▒░",
);
