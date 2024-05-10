use encoding::{self, Encoding};
use teloxide::types::InputFile;

pub fn markdown_to_tg(input: &str) -> InputFile {
    let md = markdown::to_html(input);
    let html = encoding::all::UTF_8
        .encode(&md, encoding::EncoderTrap::Replace)
        .unwrap_or_else(|_| Vec::new());

    InputFile::memory(html).file_name("result.html")
}
