import jwt
import requests
import time
import json
from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives import serialization


class IAMTokenGenerator:
    def __init__(self, key_file_path):
        self.key_file_path = key_file_path
        self.iam_url = "https://iam.api.cloud.yandex.net/iam/v1/tokens"

    def _load_key_data(self):
        with open(self.key_file_path) as key_file:
            return json.load(key_file)

    def _load_private_key(self, private_key_string):
        private_key = serialization.load_pem_private_key(private_key_string.encode(), password=None, backend=default_backend())
        return private_key

    def generate_iam_token(self):
        key_data = self._load_key_data()
        service_account_id = key_data["service_account_id"]
        key_id = key_data["id"]
        private_key_string = key_data["private_key"]

        private_key = self._load_private_key(private_key_string)

        # Prepare JWT payload
        now = int(time.time())
        payload = {
            "aud": self.iam_url,
            "iss": service_account_id,
            "iat": now,
            "exp": now + 360,
        }

        # Sign the JWT with the RSA private key
        signed_jwt = jwt.encode(payload, private_key, algorithm="PS256", headers={"kid": key_id})

        # Request IAM token
        response = requests.post(self.iam_url, json={"jwt": signed_jwt})
        if response.status_code == 200:
            return response.json().get("iamToken")
        else:
            raise Exception(f"Failed to obtain IAM token: {response.text}")
