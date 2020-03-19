
/* ---- bid class ---- */
class Bid {

	static types = ["bid", "pass", "double", "doubled-double"];

	constructor(type, value, color, multiplier) {
		console.assert(Bid.types.includes(type), `Unrecognized type argument : ${type}`);
		this.type = type;
		this.value = value === undefined ? 0 : value;
		this.color = color === undefined ? null : color;
		this.multiplier = multiplier === undefined ? 1 : multiplier;
	}

	doubleIt() {
		console.assert(this.type == "bid", `Can't double a bid of type ${this.color}`);
		console.assert(this.multiplier == 1 || this.multiplier == 2, `Can't double a bid with a multiplier = ${this.multiplier}`);
		this.multiplier *= 2;
	}

	get valueRepr() { return this.value == 250 ? "Cpt" : this.value }
	get isPass() { return this.type == "pass"; }
	get isDouble() { return this.type == "double"; }
	get isDoubledDouble() { return this.type == "doubled-double"; }
}

/* ---- card class ---- */

// known card
class Card {

	static colors = ["Spades", "Hearts", "Clubs", "Diamonds"];
	static values = ["Seven", "Eight", "Nine", "Jack", "Queen", "King", "Ten", "Ace"];
	static valuesTrump = ["Seven", "Eight", "Queen", "King", "Ten", "Ace", "Nine", "Jack"];

	constructor(color, value) {
		console.assert(Card.colors.includes(color) && Card.values.includes(value), 
			`Unsupported Card argument : ${color}, ${value}`);
		this.color = color;
		this.value = value;
	}

	get trump() {
		return (game.trumpColor && (game.trumpColor == this.color || game.trumpColor == "AllTrump"));
	}

	valueOf() {
		const color_index = Card.colors.indexOf(this.color);
		const value_index = (this.trump ? Card.valuesTrump : Card.values).indexOf(this.value);
		return color_index * 10 + value_index;
	}

	toString() {
		return this.value + "-" + this.color;
	}

	get data() {
		return { color: this.color, value: this.value };
	}
}

class Game {

	constructor(player_id) {
		this.player_id = player_id;
	}

	loadState(data) {
		this.first_player = this.localPlayerId(data.dealer_id + 1);
		this.players = data.players;
		vue.showNames(this.players);

		// update scores
		if (this.player_id % 2 == 0) vue.updateScores(...data.points)
		else vue.updateScores(data.points[1], data.points[0]);

		const [type, state] = serde.datatype(data.game_state);
		if (type == "Lobby") {
			vue.message("En attente d'autres joueurs...");
		}
		if (type == "Bidding") {
			this.bids = {}
			this.phase = 1;
			this.trumpColor = undefined;
			for (const pbid of state.bids) {
				const player = this.localPlayerId(pbid.player_id);
				this.bids[player] = serde.playerBid(pbid);
			}
			const [coinche_state, coinche_infos] = serde.datatype(state.coinche_state);
			if(coinche_state == "Coinche"){
				const doubler = this.localPlayerId(coinche_infos.player_id);
				this.bids[doubler] = new Bid("double");
				this.highestBid.doubleIt();
				this.turn = (doubler + 1) % 4;
			}
			else{
				this.turn = (this.first_player + state.bids.length) % 4;
			}
			vue.displayAllBids(this.bids);
			this.bidTurn();
		}
		if (type == "Running") {
			this.phase = 2;
			vue.hideBidPicker();
			this.bids = {}
			let bid = serde.bid(state.bid, state.coinche_state);
			this.bids[this.localPlayerId(state.team ? 1 : 0)] = bid;
			this.bids[this.localPlayerId(state.team ? 3 : 2)] = bid;
			this.trumpColor = bid.color;
			const board = state.board;
			this.current_trick = board.cards.map(serde.card);
			this.starting_player = this.localPlayerId(board.starting_player_id);
			this.turn = (this.starting_player + this.current_trick.length) % 4;
			console.log(this.turn);
			vue.displayTrick(this.starting_player, this.current_trick);
			vue.displayAllBids(this.bids);
			if(this.cards !== undefined) this.cardTurn();
		}
	}

	setCards(cards) {
		cards.sort(function (a, b) { return a - b });
		this.cards = cards;
		vue.drawMyHand(this.cards);
		if (this.phase == 2) this.cardTurn();
	}

	isPlayerInMyTeam(player) {
		return player % 2 == 0
	}

	bidTurn() {
		if (this.highestBid && this.highestBid.multiplier == 2) {
			vue.showTurn([this.turn, (this.turn + 2) % 4], 1);
			if (this.turn == 0 || this.turn == 2) vue.showDoubledDoubleOption();
		}
		else {
			vue.showTurn(this.turn, 1);
			const doubleAvail = this.highestBid && !this.isPlayerInMyTeam(this.highestBidPlayer)
			if (this.turn == 0) {
				const val = this.highestBid ? this.highestBid.value : 0;
				vue.showBidPicker(val, doubleAvail);
			}
			if (this.turn == 2 && doubleAvail) vue.showDoubleOption();
		}

	}

	cardTurn() {
		vue.showTurn(this.turn, 2);
		if (this.turn == 0) {
			/*
			if(this.cards.length == 1){
				attemptPlay(this.cards[0]);
				this.cards = [];
			}
			else vue.makeCardsPlayable(this.getPlayableCards()); */
			vue.makeCardsPlayable(this.getPlayableCards());
		}
		else vue.makeCardsUnplayable();
	}

	playCard(player, card) {
		this.turn = (this.turn + 1) % 4;
		this.current_trick.push(card);
		if (player == 0) this.removeCard(card);
		vue.playCard(player, card);
		if(this.current_trick.length < 4) this.cardTurn();
	}

	removeCard(card) {
		for (var i = 0; i < this.cards.length; i++) {
			if (this.cards[i].valueOf() == card.valueOf()) this.cards.splice(i, 1);
		}
	}

	doBid(player, bid) {
		if (!bid.isDoubledDouble) this.bids[player] = bid;
		vue.displayBid(player, bid);
		if (bid.isDouble || bid.isDoubledDouble) {
			this.highestBid.doubleIt();
			vue.displayBid(this.highestBidPlayer, this.highestBid);
		}
		if (this.isBidPhaseFinished()) {
			vue.hideBidPicker();
			if (this.highestBid) {
				this.trumpColor = this.highestBid.color;
				this.setCards(this.cards);
			}
			return;
		}
		else{
			this.turn = (this.turn + 1) % 4;
			vue.hideBidPicker();
			this.bidTurn();
		}
	}

	trickWon(winner) {
		this.current_trick = [];
		this.turn = winner;
		this.starting_player = winner;
		vue.showTrickWinner(winner);
		this.cardTurn();
	}

	localPlayerId(player_id) {
		return (player_id + 4 - this.player_id) % 4;
	}

	get highestBidPlayer() {
		let max_v = 0;
		let max_p = null;
		for (const player in this.bids) {
			const v = this.bids[player].value;
			if (v > max_v) {
				max_v = v;
				max_p = player;
			}
		}
		return max_p
	}

	get highestBid() {
		const highestBidPlayer = this.highestBidPlayer;
		return highestBidPlayer === null ? null : this.bids[highestBidPlayer];
	}

	isBidPhaseFinished() {
		if (Object.keys(this.bids).length == 0) return false;
		if (this.highestBid && this.highestBid.multiplier == 2) {
			const p1 = (this.highestBidPlayer + 1) % 4;
			const p3 = (this.highestBidPlayer + 3) % 4;
			return (this.bids[p1] && this.bids[p3] && this.bids[p1].isPass && this.bids[p3].isPass);
		}
		else if (this.highestBid && this.highestBid.multiplier == 4) {
			return true;
		}
		else {
			let nbPass = 0;
			for (const p in this.bids) nbPass += this.bids[p].isPass;
			if (nbPass == 4) return true;
			if (this.highestBid && nbPass == 3 && ((this.turn + 1) % 4 == this.highestBidPlayer)) return true;
			return false;
		}
	}

	getPlayableCards() {
		console.assert(this.cards !== undefined, "Can't get playable cards when cards === undefined.");
		if (this.current_trick.length == 0) return this.cards;
		const firstColor = this.current_trick[0].color;
		const sameColorCards = this.cards.filter(c => c.color == firstColor);
		const maxCardFunc = (a, b) => Math.max(a, b) == a ? a : b;

		let winningCard = null;
		if (this.trumpColor == "AllTrump" || this.trumpColor == "NoTrump" || this.trumpColor == firstColor) {
			winningCard = this.current_trick.filter(c => c.color == firstColor).reduce(maxCardFunc);
		}
		else {
			const trumpCards = this.current_trick.filter(c => c.color == this.trumpColor);
			if (trumpCards.length) winningCard = trumpCards.reduce(maxCardFunc);
			else winningCard = this.current_trick.filter(c => c.color == firstColor).reduce(maxCardFunc);
		}

		const winningPlayer = this.current_trick.map((card) => card.valueOf()).indexOf(winningCard.valueOf());

		if (sameColorCards.length) {
			if (this.trumpColor == "AllTrump" || this.trumpColor == firstColor) {
				const above = sameColorCards.filter((card) => card.valueOf() > winningCard.valueOf());
				if (above.length) return above;
				else return sameColorCards;
			}
			else return sameColorCards;
		}
		else {
			if (this.trumpColor == "AllTrump" || this.trumpColor == "NoTrump") return this.cards
			else {
				if ((this.starting_player + winningPlayer) % 4 == 2) {
					return this.cards;
				}
				else {
					const myTrumps = this.cards.filter((card) => card.color == this.trumpColor);
					if (myTrumps.length) {
						if (this.trumpColor == firstColor) return this.cards;
						else {
							if (winningCard.color == this.trumpColor) {
								const above = myTrumps.filter((card) => card.valueOf() > winningCard.valueOf());
								if (above.length) return above;
								else return myTrumps;
							}
							else return myTrumps;
						}
					}
					else return this.cards;
				}
			}
		}
	}

}
