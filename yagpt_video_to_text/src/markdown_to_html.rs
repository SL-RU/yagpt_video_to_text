use encoding::{self, Encoding};
use teloxide::types::InputFile;

pub fn markdown_to_html_bytes(input: &str) -> Vec<u8> {
    let md = markdown::to_html(input);
    encoding::all::UTF_8
        .encode(&md, encoding::EncoderTrap::Replace)
        .unwrap_or_else(|_| Vec::new())
}

pub fn markdown_to_tg(input: &str) -> InputFile {
    InputFile::memory(markdown_to_html_bytes(input)).file_name("result.html")
}
