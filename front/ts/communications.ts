
function send(type, data) {
	if (data === undefined) data = null;
	toBeSent = { };
	toBeSent[type] = data;
	socket.send(JSON.stringify(toBeSent));
}

function attemptBid(bid) {
	if (bid.isPass) {
		if (game.highestBid && game.highestBid.isDoubled) {
			send("SurCoinche", false);
			vue.hideBidPicker();
		} else send("Bid", null);
	} else if (bid.isDouble) {
		send("Coinche");
	} else if (bid.isDoubledDouble) {
		send("SurCoinche", true);
	} else {
		let score = bid.value.toString();
		if (score === "250") score = "Capot";
		const bid_obj = {
			trump: { Suit: bid.color },
			score,
		};
		if (bid.color === "NoTrump" || bid.color === "AllTrump") bid_obj.trump = bid.color;
		send("Bid", bid_obj);
	}
}

function attemptPlay(card) {
	send("PlayCard", { Card: { suit: card.color, value: card.value } });
}

/* ------ handlers ---- */

function onMessage(this: WebSocket, event: MessageEvent) {
	try {
		let data = JSON.parse(event.data);
		const type = Object.keys(data)[0];
		data = data[type];
		console.log(type, data);
		if (messageHandlers[type] === undefined) {
			console.error("unknow message type", data);
		} else {
			messageHandlers[type](data);
		}
	} catch (error) {
		console.error("This message raised an error:", event.data);
		console.error(error);
	}

}

let messageHandlers = {
	Game(data) {
		if (game === undefined) {
			game = new Game(data.player_id);
		}
		game.loadState(data.game);
	},

	Cards(data) {
		const cards = data.cards.map(serde.card);
		game.setCards(cards);
	},

	CardCount(data) {
		vue.drawOtherHand(game.localPlayerId(data.player_id), data.count);
	},

	PlayerBid(data) {
		const bid = serde.playerBid(data, "No");
		game.doBid(game.localPlayerId(data.player_id), bid);
	},

	Coinche(data) {
		game.doBid(game.localPlayerId(data.player_id), new Bid("double"));
	},

	SurCoinche(data) {
		game.doBid(game.localPlayerId(data.player_id), new Bid("doubled-double"));
	},

	PlayedCard(data) {
		const card = new Card(data.card.suit, data.card.value);
		const player = game.localPlayerId(data.player_id);
		game.playCard(player, card);
	},

	Trick(data) {
		const winner = game.localPlayerId(data.winner_id);
		game.trickWon(winner);
	},

	Error(data) {
		alert(data.message);
	},
};

/* ---- game logic --- */

let serde = {
	datatype(obj) {
		if (typeof (obj) === "string") return obj;
		else return Object.keys(obj)[0];
	},
	card(obj) {
		return new Card(obj.suit, obj.value);
	},
	bid(obj, cs) {
		if (obj === null) return new Bid("pass");
		let val = obj.score;
		if (val === "C") val = "250";
		val = parseInt(val);
		let trump = serde.datatype(obj.trump);
		if (trump === "Suit") trump = obj.trump.Suit;
		const bid = new Bid("bid", val, trump);
		cs = serde.datatype(cs);
		if (cs === "Coinche") bid.doubleIt();
		if (cs === "Surcoinche") {
			bid.doubleIt();
			bid.doubleIt();
		}
		return bid;
	},
	playerBid(obj, cs) {
		if (obj) return serde.bid(obj.bid, cs);
		else return new Bid("pass");
	},
};
