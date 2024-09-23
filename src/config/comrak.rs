use comrak::Options;
use serde::Deserialize;
use spring::config::Configurable;

#[derive(Deserialize, Configurable)]
#[config_prefix = "comrak"]
pub struct ComrakConfig {
    #[serde(default = "default_true")]
    pub strikethrough: bool,
    #[serde(default = "default_true")]
    pub tagfilter: bool,
    #[serde(default = "default_true")]
    pub table: bool,
    #[serde(default = "default_true")]
    pub autolink: bool,
    #[serde(default = "default_true")]
    pub tasklist: bool,
    #[serde(default = "default_true")]
    pub superscript: bool,
    pub header_ids: Option<String>,
    #[serde(default = "default_true")]
    pub footnotes: bool,
    #[serde(default = "default_true")]
    pub description_lists: bool,
    pub front_matter_delimiter: Option<String>,
    #[serde(default = "default_true")]
    pub multiline_block_quotes: bool,
    #[serde(default = "default_true")]
    pub math_dollars: bool,
    #[serde(default = "default_true")]
    pub math_code: bool,
    #[serde(default = "default_true")]
    pub shortcodes: bool,
    #[serde(default = "default_true")]
    pub wikilinks_title_after_pipe: bool,
    #[serde(default = "default_true")]
    pub wikilinks_title_before_pipe: bool,
    #[serde(default = "default_true")]
    pub underline: bool,
    #[serde(default = "default_true")]
    pub spoiler: bool,
    #[serde(default = "default_true")]
    pub greentext: bool,
}

fn default_true() -> bool {
    true
}

impl<'c> From<ComrakConfig> for Options<'c> {
    fn from(value: ComrakConfig) -> Self {
        let mut opts = Options::default();
        opts.extension.strikethrough = value.strikethrough;
        opts.extension.tagfilter = value.tagfilter;
        opts.extension.table = value.table;
        opts.extension.autolink = value.autolink;
        opts.extension.tasklist = value.tasklist;
        opts.extension.superscript = value.superscript;
        opts.extension.header_ids = value.header_ids;
        opts.extension.footnotes = value.footnotes;
        opts.extension.description_lists = value.description_lists;
        opts.extension.front_matter_delimiter = value.front_matter_delimiter;
        opts.extension.multiline_block_quotes = value.multiline_block_quotes;
        opts.extension.math_dollars = value.math_dollars;
        opts.extension.math_code = value.math_code;
        opts.extension.shortcodes = value.shortcodes;
        opts.extension.wikilinks_title_after_pipe = value.wikilinks_title_after_pipe;
        opts.extension.wikilinks_title_before_pipe = value.wikilinks_title_before_pipe;
        opts.extension.underline = value.underline;
        opts.extension.spoiler = value.spoiler;
        opts.extension.greentext = value.greentext;
        opts
    }
}