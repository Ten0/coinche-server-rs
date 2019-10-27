const valuesHtml = {
	Seven: "7",
	Eight: "8",
	Nine: "9",
	Ten: "10",
	Jack: "J",
	Queen: "Q",
	King: "K",
	Ace: "A",
};

const colorsHtml = {
	Spades: "&#9824;",
	Hearts: "&hearts;",
	Clubs: "&clubs;",
	Diamonds: "&diams;",
	NoTrump: "SA",
	AllTrump: "TA",
};

/* DOM events */

function onCardClick(): void {
	const data = JSON.parse($(this).attr("data")!);
	const card = new Card(data.color, data.value);
	attemptPlay(card);
}

function onBidChange(): void {
	const value = $("input:checked", "#bid-value-picker").val();
	const color = $("input:checked", "#bid-color-picker").val();
	if (color !== undefined && value !== undefined) {
		attemptBid(new Bid("bid", value, color));
	}
}

/* --- HTML generation --- */

type Side = "bottom" | "right" | "top" | "left";
type VuePlayerId = 0 | 1 | 2 | 3;
class Vue {
	public freezed = false;
	private stack: Array<() => void> = [];
	public static trigoSides: { [playerId: number]: Side | undefined } = ["bottom", "right", "top", "left"];
	public static clockWiseSides: { [playerId: number]: Side | undefined } = ["bottom", "left", "top", "right"];

	constructor(public clockwise = false) {
		$("#bid-picker input").change(onBidChange);
		this.hideBidPicker();
	}

	public freeze(ms: number): void {
		this.freezed = true;
		window.setTimeout(() => this.unfreeze(), ms);
	}

	public unfreeze(): void {
		this.freezed = false;
		while (!this.freezed) {
			const exec = this.stack.shift();
			if (exec) exec();
			else break;
		}
	}

	public push(fn: () => void): void {
		this.stack.push(fn);
	}

	public sideOfPlayer(player: VuePlayerId): Side {
		const side = (this.clockwise ? Vue.clockWiseSides : Vue.trigoSides)[player];
		if (!side) throw new Error("Invalid player id");
		return side;
	}

	public handOfPlayer(player: VuePlayerId): JQuery<HTMLElement> {
		return $("#" + this.sideOfPlayer(player) + "-hand");
	}

	public bidOfPlayer(player: VuePlayerId): JQuery<HTMLElement> {
		return $("#" + this.sideOfPlayer(player) + "-bid");
	}

	public nameEltOfPlayer(player: VuePlayerId): JQuery<HTMLElement> {
		return $("#" + this.sideOfPlayer(player) + "-name");
	}

	public genCard(player: VuePlayerId, card?: Card): JQuery<HTMLElement> {
		const side = this.sideOfPlayer(player);
		let elt;
		if (card === undefined) {
			elt = $("<div class=\"card hidden\"><div></div></div>");
		} else {
			elt = $("<div class=\"card visible\"><div><div></div>");
			elt.children().html("\n" + valuesHtml[card.value] + "<br>" + colorsHtml[card.color] + "\n");
			elt.attr("data", JSON.stringify(card.data));
			elt.attr("id", card.toString());
			if (card.trump) elt.addClass("trump");
			if (card.color === "Diamonds" || card.color === "Hearts") elt.css("color", "red");
		}
		elt.addClass(side);
		return elt;
	}

	public drawOtherHand(player: VuePlayerId, nbCards: number): void {
		if (this.freezed) return this.push(() => this.drawOtherHand(player, nbCards));
		const hand = this.handOfPlayer(player);
		hand.html("");
		for (let i = 0; i < nbCards; i++) {
			hand.append(this.genCard(player));
		}
	}

	public drawMyHand(cards: Card[]): void {
		if (this.freezed) return this.push(() => this.drawMyHand(cards));
		const hand = this.handOfPlayer(0);
		hand.html("");
		for (const card of cards) {
			hand.append(this.genCard(0, card));
		}
	}

	public displayTrick(startingPlayer: VuePlayerId, cards: Card[]): void {
		if (this.freezed) return this.push(() => this.displayTrick(startingPlayer, cards));
		for (let i = 0; i < cards.length; i++) {
			this.playCard((startingPlayer + i) % 4 as VuePlayerId, cards[i], true);
		}
	}

	public playCard(player: VuePlayerId, card: Card, forceCreate: boolean): void {
		if (this.freezed) return this.push(() => this.playCard(player, card, forceCreate));
		let elt;
		if (player === 0 && !forceCreate) elt = $(".card#" + card.toString());
		else {
			elt = this.genCard(player, card);
			if (!forceCreate) $(this.handOfPlayer(player).children(".card")[0]).remove();
		}
		elt.addClass(this.sideOfPlayer(player)); // @arthur il y avait [] ici avant
		elt.addClass("visible");
		elt.addClass("played");
		elt.removeClass("playable");
		elt.appendTo("#current-trick");
		elt.unbind("click");
	}

	public makeCardsPlayable(playableCards: Card[]): void {
		if (this.freezed) return this.push(() => this.makeCardsPlayable(playableCards));
		$(".card.bottom").unbind("click");
		for (const card of playableCards) {
			const elt = $(".card#" + card.toString());
			elt.addClass("playable");
			elt.click(onCardClick);
		}
	}

	public makeCardsUnplayable(): void {
		if (this.freezed) return this.push(() => this.makeCardsUnplayable());
		$(".card.bottom").removeClass("playable");
		$(".card.bottom").unbind("click");
	}

	public displayAllBids(bids: Bid[]): void {
		if (this.freezed) return this.push(() => this.displayAllBids(bids));
		$(".bid").html("");
		for (const player in bids) {
			this.displayBid(player, bids[player]);
		}
	}

	public displayBid(player: VuePlayerId, bid: Bid): void {
		if (this.freezed) return this.push(() => this.displayBid(player, bid));
		const elt = this.bidOfPlayer(player);
		elt.css("color", "black");
		if (bid.isPass || bid.isDouble || bid.isDoubledDouble) {
			if (bid.isPass) elt.html("-");
			if (bid.isDouble) elt.html("C");
			if (bid.isDoubleDoubled) elt.html("CC");
		} else {
			if (bid.color === "Diamonds" || bid.color === "Hearts") elt.css("color", "red");
			elt.html(bid.value + " " + colorsHtml[bid.color]);
			if (bid.isDoubled) elt.append("<span>C</span>");
			if (bid.isDoubleDoubled) elt.append("<span>CC</span>");
		}
		elt.show();
	}

	public showBidPicker(minimumBid: number, doubleAvail: boolean): void {
		if (this.freezed) return this.push(() => this.showBidPicker(minimumBid, doubleAvail));
		$("#bid-picker").show();
		$("#bid-picker input:checked").removeAttr("checked");
		$("#bid-doubled-double").hide();
		if (doubleAvail) $("#bid-double").show();
		else $("#bid-double").hide();
		$("#bid-pass").removeClass("disabled");
		$("#bid-picker label").removeClass("disabled");
		$("#bid-picker input").removeAttr("disabled");

		for (const elt of $("#bid-value-picker label")) {
			const elt2 = $(elt);
			const val = $("#" + elt2.attr("for")).val();
			if (typeof val !== "number") throw new Error("Val is not a number");
			if (val <= minimumBid) {
				$("#" + elt2.attr("for")).attr("disabled", "");
				elt2.addClass("disabled");
			}
		}
	}

	public hideBidPicker(): void {
		if (this.freezed) return this.push(() => this.hideBidPicker());
		$("#bid-picker").hide();
	}

	public disableAllBids(): void {
		if (this.freezed) return this.push(() => this.disableAllBids());
		$("#bid-picker label").addClass("disabled");
		$("#bid-picker label").attr("disabled", "");
	}

	public showDoubleOption(): void {
		if (this.freezed) return this.push(() => this.showDoubleOption());
		$("#bid-picker").show();
		this.disableAllBids();
		$("#bid-pass").addClass("disabled");
		$("#bid-doubled-double").hide();
		$("#bid-double").show();
	}

	public showDoubledDoubleOption(): void {
		if (this.freezed) return this.push(() => this.showDoubledDoubleOption());
		$("#bid-picker").show();
		$("#bid-pass").removeClass("disabled");
		this.disableAllBids();
		$("#bid-doubled-double").show();
		$("#bid-double").hide();
	}

	public showTrickWinner(winner: VuePlayerId): void {
		if (this.freezed) return this.push(() => this.showTrickWinner(winner));
		$(".played." + this.sideOfPlayer(winner)).addClass("winner");
		this.freeze(1500);
		this.cleanTrick();
	}

	public cleanTrick(): void {
		if (this.freezed) return this.push(() => this.cleanTrick());
		$("#last-trick").empty();
		$("#current-trick").children().appendTo("#last-trick");
	}

	public showTurn(turn: VuePlayerId, phase: 1 | 2): void {
		if (this.freezed) return this.push(() => this.showTurn(turn, phase));
		$(".name").removeClass("turn");
		$(".hand").removeClass("turn");
		if (phase === 1) {
			if (typeof turn === "number") this.nameEltOfPlayer(turn).addClass("turn");
			else {
				this.nameEltOfPlayer(turn[0]).addClass("turn");
				this.nameEltOfPlayer(turn[1]).addClass("turn");
			}
		}
		if (phase === 2) this.handOfPlayer(turn).addClass("turn");
	}

	public updateScores(ourScore: number, theirScore: number): void {
		if (this.freezed) return this.push(() => this.updateScores(ourScore, theirScore));
		$("#last-trick").empty();
		$("#us").html(ourScore.toFixed(0));
		$("#them").html(theirScore.toFixed(0));
	}

	public message(msg, ms): void {
		console.log("message de la vue :", msg);
	}

	public showNames(players: Array<{ localPlayerId: VuePlayerId, player: Player }>): void {
		for (const player of players) {
			this.nameEltOfPlayer(player.localPlayerId).text(player.player.username);
		}
	}

}
