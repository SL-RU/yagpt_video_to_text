import subprocess

def convert_video_to_audio(video_input, audio_output):
    """
    Конвертирует видеофайл в аудиофайл с использованием ffmpeg.
    
    Args:
    video_input (str): Путь к входному видеофайлу.
    audio_output (str): Путь к выходному аудиофайлу.
    """
    # Команда для вызова ffmpeg
    command = ['ffmpeg', '-y', '-i', video_input, '-ac', '1', audio_output]
    
    try:
        # Выполнение команды
        subprocess.run(command, check=True)
        print(f"Файл успешно сконвертирован в {audio_output}")
    except subprocess.CalledProcessError as e:
        print(f"Ошибка при конвертации файла: {e}")
    except Exception as e:
        print(f"Произошла неожиданная ошибка: {e}")
