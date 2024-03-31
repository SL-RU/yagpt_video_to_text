import json

def extract_text_to_file(input_json_path, output_text_path):
    """
    Reads text data from a JSON file and writes each text entry to a new line in a text file.

    :param input_json_path: Path to the input JSON file containing the text data.
    :param output_text_path: Path where the output text file will be saved.
    """
    # Reading the JSON data from the input file
    try:
        with open(input_json_path, 'r', encoding='utf-8') as file:
            data_from_file = json.load(file)
    except FileNotFoundError:
        print(f"File not found: {input_json_path}")
        return
    except json.JSONDecodeError:
        print(f"Error decoding JSON from file: {input_json_path}")
        return

    # Extracting the text data
    text_lines_from_file = [chunk['alternatives'][0]['text'] for chunk in data_from_file['response']['chunks']]

    # Writing the extracted text to the output text file
    with open(output_text_path, 'w', encoding='utf-8') as file:
        for line in text_lines_from_file:
            file.write(line + '\n')

    print(f"Text data has been written to {output_text_path}")