import boto3

import hashlib
import os

def get_md5(file_path):
    """
    Calculates the MD5 hash of the file.
    
    Args:
    file_path (str): The path to the file.
    
    Returns:
    str: md5.
    """
    # Calculate MD5
    hasher = hashlib.md5()
    with open(file_path, 'rb') as file:
        buf = file.read()
        hasher.update(buf)
    md5_hash = hasher.hexdigest()
    
    return md5_hash


def upload(original_file_path, aws_access_key_id, aws_secret_access_key, bucket_name):
    # Переименование файла в его MD5 хеш
    filename = original_file_path
    print(f"Upload {original_file_path}")
    objname = get_md5(original_file_path)
    print(f"MD5 {objname}")
    
    # Создание сессии и клиента S3 с указанными учетными данными
    session = boto3.session.Session()
    s3 = session.client(
        service_name='s3',
        endpoint_url='https://storage.yandexcloud.net',
        region_name='ru-central1',
        aws_access_key_id=aws_access_key_id,
        aws_secret_access_key=aws_secret_access_key
    )
    
    # Загрузка файла в указанный бакет
    s3.upload_file(filename, bucket_name, objname)
    print("File uploaded successfully.")
    
    # Генерация пре-сигнатурного URL для доступа к файлу
    presigned_url = s3.generate_presigned_url('get_object', Params={'Bucket': bucket_name, 'Key': objname}, ExpiresIn=3600)
    print("Presigned URL:", presigned_url)
    return presigned_url
