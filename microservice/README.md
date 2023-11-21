Microservice Instructions

Requesting Data: In order to request data from the microservice, all you have to do is make a POST request to the address "http://localhost:9000/RandomCard"

Example:
```
url = "http://localhost:9000/RandomCard"
response = requests.post(url)
```
Receiving Data: The data you get from the microservice will be the name of a random card, formatted in .json. It will look like the following example:
```
{'name': 'Performapal Barracuda'}
```
It only returns the name of the card, so it should be easy to parse out.

![image](https://github.com/crimpz/cs361/assets/18359937/52436f68-f853-43c1-9335-7d70665e777e)
