use std::path::PathBuf;

use teloxide::types::InputFile;

use crate::config;

pub fn markdown_to_tg(config: &config::Config, input: String) -> InputFile {
    let mut pandoc = pandoc::new();
    let path = PathBuf::from(config.refactored_html.clone() + ".docx");
    pandoc.set_input_format(pandoc::InputFormat::Markdown, Vec::new());
    pandoc.set_input(pandoc::InputKind::Pipe(input));
    //pandoc.set_input(pandoc::InputKind::Files(vec![input]));
    pandoc.set_output(pandoc::OutputKind::File(path.clone()));
    pandoc.set_output_format(pandoc::OutputFormat::Docx, Vec::new());
    match pandoc.execute() {
        Ok(res) => match res {
            pandoc::PandocOutput::ToFile(_) => {
                InputFile::file(&path).file_name("result.docx")
            }
            pandoc::PandocOutput::ToBuffer(buff) => {
                InputFile::memory(buff).file_name("result.docx")
            }
            pandoc::PandocOutput::ToBufferRaw(buff) => {
                InputFile::memory(buff).file_name("result.docx")
            }
        },
        Err(err) => InputFile::memory(format!("Pandoc error: {}", err)).file_name("error.txt"),
    }
}
