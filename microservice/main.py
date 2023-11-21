from flask import Flask, jsonify
import requests

app = Flask(__name__)
url = "http://db.ygoprodeck.com/api/v7/randomcard.php"


@app.route('/RandomCard', methods=['POST'])
def get_card():
    try:
        response = requests.get(url)
        data = response.json()

        card_name = data.get('name', None)

        if card_name:
            name = card_name

            return jsonify({'name': name})

        else:
            error = {'error': 'Error'}
            return jsonify(error)

    except Exception as e:
        error = {'error': str(e)}
        return jsonify(error)


if __name__ == '__main__':
    app.run(port=9000)
