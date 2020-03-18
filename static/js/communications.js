
function send(type, data) {
	if (data === undefined) data = null;
	toBeSent = {}
	toBeSent[type] = data;
	socket.send(JSON.stringify(toBeSent));
}

function attemptBid(bid) {
	if (bid.isPass) {
		if (game.highestBid && game.highestBid.multiplier == 2) {
			send("SurCoinche", false);
			vue.hideBidPicker();
		}
		else send("Bid", null);
	}
	else if (bid.isDouble) {
		send("Coinche");
	}
	else if (bid.isDoubledDouble) {
		send("SurCoinche", true);
	}
	else {
		let score = bid.value.toString()
		if (score == "250") score = "Capot";
		let bid_obj = {
			trump: { Suit: bid.color },
			score: score
		}
		if (bid.color == "NoTrump" || bid.color == "AllTrump") bid_obj["trump"] = bid.color;
		send("Bid", bid_obj);
	}
}

function attemptPlay(card) {
	send("PlayCard", { "Card": { "suit": card.color, "value": card.value } });
}

/* ------ handlers ---- */

function onmessage(event) {
	try {
		const [type, data] = serde.datatype(JSON.parse(event.data));
		console.log(type, data);
		if (messageHandlers[type] === undefined) {
			console.error("unknow message type", data);
		}
		else {
			messageHandlers[type](data);
		}
	}
	catch (error) {
		console.error("This message raised an error:", event.data);
		console.error(error);
	}

}

const messageHandlers = {
	Game: function (data) {
		if (game === undefined) {
			game = new Game(data.player_id);
		}
		game.loadState(data.game);
	},

	Cards: function (data) {
		let cards = data.cards.map(serde.card);
		game.setCards(cards);
	},

	CardCount: function (data) {
		vue.drawOtherHand(game.localPlayerId(data.player_id), data.count);
	},

	PlayerBid: function (data) {
		let bid = serde.playerBid(data, "No");
		game.doBid(game.localPlayerId(data.player_id), bid);
	},

	Coinche: function (data) {
		game.doBid(game.localPlayerId(data.player_id), new Bid("double"));
	},

	SurCoinche: function (data) {
		game.doBid(game.localPlayerId(data.player_id), new Bid("doubled-double"));
	},

	PlayedCard: function (data) {
		const card = new Card(data.card.suit, data.card.value);
		const player = game.localPlayerId(data.player_id);
		game.playCard(player, card);
	},

	Trick: function (data) {
		const winner = game.localPlayerId(data.winner_id);
		game.trickWon(winner);
	},

	Error: function (data) {
		alert(data.message);
	},
}

const serde = {
	datatype: function (obj) {
		if (typeof(obj) == "string") return [obj, null];
		else{
			const type = Object.keys(obj)[0];
			return [type, obj[type]];
		} 
	},
	card: function (obj) {
		return new Card(obj.suit, obj.value);
	},
	bid: function (obj, cs) {
		if (obj === null) return new Bid("pass");
		const val = obj.score === "Capot" ? 250 : parseInt(obj.score);
		let [trump, color] = serde.datatype(obj.trump);
		if (trump == "Suit") trump = color;
		let bid = new Bid("bid", val, trump);
		if(cs !== undefined){
			[cs, ] = serde.datatype(cs);
			if (cs == "Coinche") bid.doubleIt();
			if (cs == "Surcoinche") {
				bid.doubleIt();
				bid.doubleIt();
			}
		}

		return bid;
	},
	playerBid: function (obj, cs) {
		if (obj) return serde.bid(obj.bid, cs);
		else return new Bid("pass");
	}
}
