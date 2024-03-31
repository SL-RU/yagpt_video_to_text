import requests
import time
from itertools import islice


class YandexGPTAsyncV3:
    def __init__(self, iam_token):
        self.headers = {
            "Authorization": f"Bearer {iam_token}",
            "Content-Type": "application/json",
        }
        self.base_url = "https://llm.api.cloud.yandex.net/foundationModels/v1/completionAsync"

    def start_async_request(self, model_uri, messages, temperature=0.6, max_tokens=2000, stream=False):
        """
        Отправляет асинхронный запрос на генерацию текста.
        """
        data = {
            "modelUri": model_uri,
            "completionOptions": {
                "stream": stream,
                "temperature": temperature,
                "maxTokens": max_tokens
            },
            "messages": messages
        }
        response = requests.post(self.base_url, json=data, headers=self.headers)
        if response.status_code == 200:
            operation_id = response.json().get('id')
            print(f"Operation started successfully with ID: {operation_id}")
            return operation_id
        else:
            print(f"Failed to start operation: {response.text}")
            return None

    def check_operation_status(self, operation_id):
        """
        Проверяет статус выполнения асинхронной задачи.
        """
        response = requests.get(f"https://operation.api.cloud.yandex.net/operations/{operation_id}", headers=self.headers)
        if response.status_code == 200:
            operation_status = response.json()
            return operation_status.get('done', False), operation_status
        else:
            print(f"Failed to check operation status: {response.text}")
            return False, None

    def wait_for_completion(self, operation_id, check_interval=5):
        """
        Ожидает завершения задачи и возвращает результат.
        """
        while True:
            done, operation_status = self.check_operation_status(operation_id)
            if done:
                if 'response' in operation_status:
                    # Новый код для обработки структуры ответа
                    response = operation_status['response']
                    alternatives = response.get('alternatives', [])
                    if alternatives:
                        # Пример выводит только первую альтернативу. Можно адаптировать под свои нужды.
                        first_alternative = alternatives[0]
                        message = first_alternative.get('message', {})
                        text = message.get('text', 'No text provided')
                        return text
                    else:
                        return "No alternatives provided in response."
                else:
                    return operation_status.get('error', {})
            else:
                time.sleep(check_interval)


def process_text_with_gpt(input_filename, output_filename, iam_token, model_uri):
    gpt = YandexGPTAsyncV3(iam_token)
    line_count = 0

    with open(input_filename, "r", encoding="utf8") as input_file:
        # Подсчитываем общее количество строк в файле для логгирования прогресса
        total_lines = sum(1 for _ in input_file)
    
    with open(input_filename, "r", encoding="utf8") as input_file, open(output_filename, "w", encoding="utf8") as output_file:
        while True:
            lines = list(islice(input_file, 100))
            if not lines:
                break  # Прекращаем цикл, если строки закончились
            
            text_to_process = "".join(lines)
            line_count += len(lines)

            # Формирование запроса
            messages = [
                {
                    "role": "system",
                    "text": "Это траскрибирование лекции в университете, сделай текст читаемым, исправь знаки, повторы и ошибки и сформируй абзацы. Переформулируй, чтобы это были не обрывистые высказывания преподавателя, а текст, который удобно читать, а большие связанные абзацы, без обращений к аудитории только суть лекции. Не сокращай, исходный и конечный объём должен быть примерно одинаковый"
                },
                {
                    "role": "user",
                    "text": text_to_process
                }
            ]

            print(f"Обработка строк {line_count-len(lines)+1} до {line_count} из {total_lines}...")
            operation_id = gpt.start_async_request(model_uri, messages, temperature=0.9, max_tokens=70000, stream=False)
            if operation_id:
                result = gpt.wait_for_completion(operation_id)
                # Запись результата в выходной файл
                output_file.write(result + '\n\n')
    
    print("Обработка завершена, результат записан в", output_filename)