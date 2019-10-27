
/* ---- bid class ---- */
type BidType = "bid" | "pass" | "double" | "doubled-double";
class Bid {
	// @arthur pas très clair ce qui représente la coinche, le type ou ces booléens
	public type: BidType;
	public value: number;
	public color: Color;
	public isDoubled = false;
	public isDoubleDoubled = false;

	constructor(type: BidType, value: number | undefined, color: Color) {
		this.type = type;
		if (value === undefined) this.value = 0;
		else this.value = value;
		this.color = color;
	}

	public doubleIt(): void {
		if (this.isDoubled) {
			this.isDoubled = false;
			this.isDoubleDoubled = true;
		} else this.isDoubled = true;
	}

	get isPass(): boolean { return this.type === "pass"; }
	get isDouble(): boolean { return this.type === "double"; }
	get isDoubledDouble(): boolean { return this.type === "doubled-double"; }
}

/* ---- card class ---- */

type Color = "Spades" | "Hearts" | "Clubs" | "Diamonds";
type Value = "Seven" | "Eight" | "Nine" | "Jack" | "Queen" | "King" | "Ten" | "Ace";
// known card
class Card {
	public static colors: Color[] = ["Spades", "Hearts", "Clubs", "Diamonds"];
	public static values: Value[] =
		["Seven", "Eight", "Nine", "Jack", "Queen", "King", "Ten", "Ace"];
	public static valuesTrump: Value[] =
		["Seven", "Eight", "Queen", "King", "Ten", "Ace", "Nine", "Jack"];

	constructor(public color: Color, public value: Value) { }

	get trump(): boolean {
		return (game.trumpColor && game.trumpColor === this.color);
	}

	public valueOf(): number {
		const colorIndex = Card.colors.indexOf(this.color);
		const valueIndex = (this.trump ? Card.valuesTrump : Card.values).indexOf(this.value);
		return colorIndex * 10 + valueIndex;
	}

	public toString(): string {
		return this.value + "-" + this.color;
	}

	get data(): { color: Color, value: Value } {
		return { color: this.color, value: this.value };
	}
}

class Game {
	constructor(public playerId: number) { }

	public loadState(data: GameData) {
		this.first_player = this.localPlayerId(data.dealer_id + 1);
		this.players = data.players;
		vue.showNames(this.players);

		// update scores
		if (this.player_id % 2 === 0) vue.updateScores(...data.points);
		else vue.updateScores(data.points[1], data.points[0]);

		const state = data.game_state;
		const type = serde.datatype(state);
		if (type === "Lobby") {
			vue.message("En attente d'autres joueurs...");
		}
		if (type === "Bidding") {
			this.bids = { };
			this.phase = 1;
			this.trumpColor = undefined;
			for (const pbid of state.Bidding.bids) {
				const player = this.localPlayerId(pbid.player_id);
				this.bids[player] = serde.playerBid(pbid, state.Bidding.coinche_state);
				this.turn = player;
			}
			this.turn = (this.first_player + state.Bidding.bids.length) % 4;
			vue.displayAllBids(this.bids);
			this.bidTurn();
		}
		if (type === "Running") {
			this.phase = 2;
			vue.hideBidPicker();
			this.bids = { };
			const bid = serde.bid(state.Running.bid, state.Running.coinche_state);
			this.bids[this.localPlayerId(state.Running.team ? 1 : 0)] = bid;
			this.bids[this.localPlayerId(state.Running.team ? 3 : 2)] = bid;
			this.trumpColor = bid.color;
			const board = state.Running.board;
			this.current_trick = board.cards.map(serde.card);
			this.starting_player = this.localPlayerId(board.starting_player_id);
			this.turn = (this.starting_player + this.current_trick.length) % 4;
			console.log(this.turn);
			vue.displayTrick(this.starting_player, this.current_trick);
			vue.displayAllBids(this.bids);
			this.cardTurn();
		}
	}

	public setCards(cards) {
		cards.sort(function(a, b) { return a - b; });
		this.cards = cards;
		vue.drawMyHand(this.cards);
		if (this.phase === 2) this.cardTurn();
	}

	public isPlayerInMyTeam(player) {
		return player % 2 === 0;
	}

	public bidTurn() {
		if (this.highestBid && this.highestBid.isDoubled) {
			vue.showTurn([this.turn, (this.turn + 2) % 4], 1);
			if (this.turn === 0 || this.turn === 2) vue.showDoubledDoubleOption();
		} else {
			vue.showTurn(this.turn, 1);
			const doubleAvail = this.highestBid && !this.isPlayerInMyTeam(this.highestBidPlayer);
			if (this.turn === 0) {
				const val = this.highestBid ? this.highestBid.value : 0;
				vue.showBidPicker(val, doubleAvail);
			}
			if (this.turn === 2 && doubleAvail) vue.showDoubleOption();
		}

	}

	public cardTurn() {
		vue.showTurn(this.turn, 2);
		if (this.turn === 0) {
			/*
			if(this.cards.length == 1){
				attemptPlay(this.cards[0]);
				this.cards = [];
			}
			else vue.makeCardsPlayable(this.getPlayableCards()); */
			vue.makeCardsPlayable(this.getPlayableCards());
		} else vue.makeCardsUnplayable();
	}

	public playCard(player, card) {
		this.turn = (this.turn + 1) % 4;
		this.current_trick.push(card);
		if (player === 0) this.removeCard(card);
		vue.playCard(player, card);
		this.cardTurn();
	}

	public removeCard(card) {
		for (let i = 0; i < this.cards.length; i++) {
			if (this.cards[i].valueOf() === card.valueOf()) this.cards.splice(i, 1);
		}
	}

	public doBid(player, bid) {
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
			console.log("BID FINISHED");
			return;
		} else this.turn = (this.turn + 1) % 4;
		vue.hideBidPicker();
		this.bidTurn();
	}

	public trickWon(winner) {
		this.current_trick = [];
		this.turn = winner;
		this.starting_player = winner;
		vue.showTrickWinner(winner);
		this.cardTurn();
	}

	public localPlayerId(player_id) {
		return (player_id + 4 - this.player_id) % 4;
	}

	get highestBidPlayer() {
		let max_v = 0;
		let max_p = -1;
		for (const player in this.bids) {
			const v = this.bids[player].value;
			if (v > max_v) {
				max_v = v;
				max_p = player;
			}
		}
		return max_p;
	}

	get highestBid() {
		return this.bids[this.highestBidPlayer];
	}

	public isBidPhaseFinished() {
		if (Object.keys(this.bids).length === 0) return false;
		if (this.highestBid && this.highestBid.isDoubled) {
			const p1 = (this.highestBidPlayer + 1) % 4;
			const p3 = (this.highestBidPlayer + 3) % 4;
			return (this.bids[p1] && this.bids[p3] && this.bids[p1].isPass && this.bids[p3].isPass);
		} else if (this.highestBid && this.highestBid.isDoubleDoubled) {
			return true;
		} else {
			let nbPass = 0;
			for (const p in this.bids) nbPass += this.bids[p].isPass;
			if (nbPass === 4) return true;
			if (this.highestBid && nbPass === 3 && ((this.turn + 1) % 4 === this.highestBidPlayer)) return true;
			return false;
		}
	}

	public getPlayableCards() {
		if (this.current_trick.length === 0) return this.cards;
		const firstColor = this.current_trick[0].color;
		const sameColorCards = this.cards.filter(c => c.color === firstColor);
		const maxCardFn = (a, b) => Math.max(a, b) === a ? a : b;

		let winningCard;
		if (this.trumpColor === "AllTrump" || this.trumpColor === "NoTrump" || this.trumpColor === firstColor) {
			winningCard = this.current_trick.filter(c => c.color === firstColor).reduce(maxCardFn);
		} else {
			const trumpCards = this.current_trick.filter(c => c.color === this.trumpColor);
			if (trumpCards.length) winningCard = trumpCards.reduce(maxCardFn);
			else winningCard = this.current_trick.filter(c => c.color === firstColor).reduce(maxCardFn);
		}

		const winningPlayer = this.current_trick.map(card => card.valueOf()).indexOf(winningCard.valueOf());

		if (sameColorCards.length) {
			if (this.trumpColor === "AllTrump" || this.trumpColor === firstColor) {
				const above = sameColorCards.filter(card => card.valueOf() > winningCard.valueOf());
				if (above.length) return above;
				else return sameColorCards;
			} else return sameColorCards;
		} else {
			if (this.trumpColor === "AllTrump" || this.trumpColor === "NoTrump") return this.cards;
			else {
				if ((this.starting_player + winningPlayer) % 4 === 2) {
					return this.cards;
				} else {
					const myTrumps = this.cards.filter(card => card.color === this.trumpColor);
					if (myTrumps.length) {
						if (this.trumpColor === firstColor) return this.cards;
						else {
							if (winningCard.color === this.trumpColor) {
								const above = myTrumps.filter(card => card.valueOf() > winningCard.valueOf());
								if (above.length) return above;
								else return myTrumps;
							} else return myTrumps;
						}
					} else return this.cards;
				}
				// winningPlayer in my team ? -> all
				// no -> atout supérieur à la coupe (si coupe il y a)
			}
		}
	}

}
