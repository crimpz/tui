import requests

url = "http://localhost:9000/RandomCard"

response = requests.post(url)
data = response.json()

print(data)
