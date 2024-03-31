import yt_dlp
import json


def DL(url, outfile):
    URLS = [url]
    ydl_opts = {
        "outtmpl": outfile,
        "format": "m4a/bestaudio/best",
        # ℹ️ See help(yt_dlp.postprocessor) for a list of available Postprocessors and their arguments
        "postprocessors": [
            {  # Extract audio using ffmpeg
                "key": "FFmpegExtractAudio",
                "preferredcodec": "m4a",
            }
        ],
    }

    with yt_dlp.YoutubeDL(ydl_opts) as ydl:
        info = ydl.extract_info(URLS[0], download=True)

        # ℹ️ ydl.sanitize_info makes the info json-serializable
        return ydl.sanitize_info(info)
