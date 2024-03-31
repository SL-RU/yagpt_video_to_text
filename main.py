import argparse
import json
import markdown
import os

from dl import DL
from video_to_audio import convert_video_to_audio
from video_to_text import YandexSpeechToText
from upload import upload
from iam_gen import IAMTokenGenerator
from json_to_text import extract_text_to_file
from gpt import process_text_with_gpt


def remove_files(config):
    files_to_remove = [
        config.get("video_path"),
        config.get("audio_path"),
        config.get("transcribed_json"),
        config.get("transcribed_text"),
        config.get("refactored_md"),
        config.get("refactored_html"),
    ]
    for file_path in files_to_remove:
        if file_path and os.path.exists(file_path):
            os.remove(file_path)
            print(f"Файл {file_path} удален.")


def main(config_path, video_url=None, video_file=None):
    with open(config_path, "r") as config_file:
        config = json.load(config_file)

    remove_files(config)

    token_generator = IAMTokenGenerator(config["token_path"])
    iam_token = token_generator.generate_iam_token()
    print("IAM Token:", iam_token)

    if video_url:
        res = DL(video_url, config["video_path"])
        video_title = res["title"]
        print("Скачано", video_title)
    elif video_file:
        config["video_path"] = video_file
        video_title = os.path.splitext(os.path.basename(video_file))[0]
        print("Используется локальный файл", video_title)
    else:
        print("Необходимо указать либо URL видео, либо путь к локальному файлу.")
        return

    final_file = f"./data/{video_title}.html"

    convert_video_to_audio(config["video_path"], config["audio_path"])

    audio_uri = upload(config["audio_path"], config["aws_access_key_id"], config["aws_secret_access_key"], config["bucket_name"])
    print("Загружено в бакет", audio_uri)

    print("Распознавание...")
    transcriber = YandexSpeechToText(iam_token, audio_uri)
    transcriber.transcribe_audio(output_file_path=config["transcribed_json"], audio_encoding="MP3")

    print("JSON в текст")
    extract_text_to_file(config["transcribed_json"], config["transcribed_text"])

    print("Обработка GPT")
    process_text_with_gpt(config["transcribed_text"], config["refactored_md"], iam_token, config["model_uri"])

    print("Markdown")
    markdown.markdownFromFile(
        input=config["refactored_md"],
        output=config["refactored_html"],
        encoding="utf8",
    )

    print("Markdown", config["refactored_html"], "->", final_file)
    os.rename(config["refactored_html"], final_file)
    print("Итог:", final_file)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Процесс обработки видео.")
    parser.add_argument("config_path", type=str, help="Путь к файлу конфигурации.")
    parser.add_argument("--link", type=str, help="URL видео на YouTube для скачивания.")
    parser.add_argument("--file", type=str, help="Путь к локальному видеофайлу.")

    args = parser.parse_args()

    if args.link:
        main(args.config_path, video_url=args.link)
    elif args.file:
        main(args.config_path, video_file=args.file)
    else:
        print("Ошибка: необходимо указать либо --link URL, либо --file PATH.")
