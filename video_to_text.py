import requests
import json
import time

class YandexSpeechToText:
    def __init__(self, iam_token, audio_uri):
        self.iam_token = iam_token
        self.audio_uri = audio_uri
        self.headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {iam_token}",
        }
        self.base_url = "https://transcribe.api.cloud.yandex.net/speech/stt/v2/longRunningRecognize"

    def transcribe_audio(self, language_code="ru-RU", model="general", profanity_filter=False,
                         literature_text=True, audio_encoding="LINEAR16_PCM", sample_rate_hertz=48000,
                         audio_channel_count=1, raw_results=False, output_file_path='out.json'):
        # Prepare the request body
        data = {
            "config": {
                "specification": {
                    "languageCode": language_code,
                    "model": model,
                    "profanityFilter": profanity_filter,
                    "literature_text": literature_text,
                    "audioEncoding": audio_encoding,
                    "sampleRateHertz": sample_rate_hertz,
                    "audioChannelCount": audio_channel_count,
                    "rawResults": raw_results
                }
            },
            "audio": {
                "uri": self.audio_uri
            }
        }

        # Send the initial POST request to start the transcription process
        response = requests.post(self.base_url, headers=self.headers, json=data)
        if response.status_code != 200:
            print(f"Request failed with status code {response.status_code}")
            print(response.text)
            return None

        print("Request successful.")
        operation_response = response.json()
        print(json.dumps(operation_response, indent=2, ensure_ascii=False))
        operation_id = operation_response['id']

        return self._wait_for_operation(operation_id, output_file_path)

    def _wait_for_operation(self, operation_id, output_file_path):
        operation_url = f"https://operation.api.cloud.yandex.net/operations/{operation_id}"
        while True:
            response = requests.get(operation_url, headers=self.headers)
            if response.status_code != 200:
                print(f"Request failed with status code {response.status_code}")
                print(response.text)
                break

            operation_status = response.json()
            if operation_status.get('done', False):
                with open(output_file_path, 'w', encoding='utf8') as f:
                    json.dump(operation_status, f, ensure_ascii=False)
                print(f"Transcription completed and saved to '{output_file_path}'.")
                break
            time.sleep(1)
