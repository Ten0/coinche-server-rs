const valuesHtml = {
	"Seven": "7",
	"Eight": "8",
	"Nine": "9",
	"Ten": "10",
	"Jack": "J",
	"Queen": "Q",
	"King": "K",
	"Ace": "A",
};

const colorsHtml = {
	"Spades": "&#9824;",
	"Hearts": "&hearts;",
	"Clubs": "&clubs;",
	"Diamonds": "&diams;",
	"NoTrump": "SA",
	"AllTrump": "TA"
}

/* DOM events */

function onCardClick(evt) {
	let data = JSON.parse($(this).attr("data"));
	let card = new Card(data.color, data.value);
	attemptPlay(card);
}

function onBidChange(evt) {
	let value = $('input:checked', '#bid-value-picker').val();
	let color = $('input:checked', '#bid-color-picker').val();
	if (color !== undefined && value !== undefined) {
		attemptBid(new Bid("bid", value, color));
	}
}

/* --- HTML generation --- */

class Vue {

	static sides = ["bottom", "right", "top", "left"];
	static clockwiseSides = ["bottom", "left", "top", "right"];

	constructor(clockwise) {
		this.clockwise = clockwise;
		$("#bid-picker input").change(onBidChange);
		this.freezed = false;
		this.stack = [];
		this.hideBidPicker();
	}

	freeze(ms) {
		this.freezed = true;
		window.setTimeout((function (vue) {
			return function () { vue.unfreeze(); }
		})(this), ms);
	}

	unfreeze() {
		this.freezed = false;
		while (!this.freezed && this.stack.length > 0) {
			const exec = this.stack.shift();
			this[exec[0]](...exec[1]);
		}
	}

	push(func_name, ...args) {
		this.stack.push([func_name, args]);
	}

	sideOfPlayer(player) {
		return (this.clockwise ? Vue.clockwiseSides : Vue.sides)[player];
	}

	handOfPlayer(player) {
		return $(`#${this.sideOfPlayer(player)}-hand`);
	}

	bidOfPlayer(player) {
		return $(`#${this.sideOfPlayer(player)}-bid`);
	}

	nameEltOfPlayer(player) {
		return $(`#${this.sideOfPlayer(player)}-name`);
	}

	genCard(player, card) {
		const side = this.sideOfPlayer(player);
		let elt;
		if (card === undefined) {
			elt = $('<div class="card hidden"><div></div></div>');
		}
		else {
			elt = $('<div class="card visible"><div><div></div>');
			elt.children().html("\n" + valuesHtml[card.value] + "<br>" + colorsHtml[card.color] + "\n");
			elt.attr("data", JSON.stringify(card.data));
			elt.attr("id", card.toString());
			if (card.trump) elt.addClass("trump");
			if (card.color == "Diamonds" || card.color == "Hearts") elt.css("color", "red");
		}
		elt.addClass(side);
		return elt;
	}

	drawOtherHand(player, nb_cards) {
		if (this.freezed) return this.push("drawOtherHand", player, nb_cards);
		let hand = this.handOfPlayer(player);
		hand.html("");
		for (let i = 0; i < nb_cards; i++) {
			hand.append(this.genCard(player));
		}
	}

	drawMyHand(cards) {
		if (this.freezed) return this.push("drawMyHand", cards);
		let hand = this.handOfPlayer(0);
		hand.html("");
		for (const card of cards) {
			hand.append(this.genCard(0, card));
		}
	}

	displayTrick(starting_player, cards) {
		if (this.freezed) return this.push("displayTrick", starting_player, cards);
		for (let i = 0; i < cards.length; i++) {
			this.playCard((starting_player + i) % 4, cards[i], true);
		}
	}

	playCard(player, card, forceCreate) {
		if (this.freezed) return this.push("playCard", player, card, forceCreate);
		let elt;
		if (player == 0 && !forceCreate) elt = $(".card#" + card.toString());
		else {
			elt = this.genCard(player, card);
			if (!forceCreate) $(this.handOfPlayer(player).children(".card")[0]).remove();
		}
		elt.addClass(this.sideOfPlayer(player));
		elt.addClass("visible");
		elt.addClass("played");
		elt.removeClass("playable");
		elt.appendTo("#current-trick");
		elt.unbind("click");
	}

	makeCardsPlayable(playableCards) {
		if (this.freezed) return this.push("makeCardsPlayable", playableCards);
		$(".card.bottom").unbind("click");
		for (const card of playableCards) {
			let elt = $(".card#" + card.toString());
			elt.addClass("playable");
			elt.click(onCardClick);
		}
	}

	makeCardsUnplayable() {
		if (this.freezed) return this.push("makeCardsUnplayable");
		$(".card.bottom").removeClass("playable");
		$(".card.bottom").unbind("click");
	}

	displayAllBids(bids) {
		if (this.freezed) return this.push("displayAllBids", bids);
		$(".bid").html("");
		for (const player in bids) {
			this.displayBid(player, bids[player]);
		}
	}

	displayBid(player, bid) {
		if (this.freezed) return this.push("displayBid", player, bid);
		let elt = this.bidOfPlayer(player);
		elt.css("color", "black");
		if (bid.isPass || bid.isDouble || bid.isDoubledDouble) {
			if (bid.isPass) elt.html("-");
			if (bid.isDouble) elt.html("C");
			if (bid.isDoubleDoubled) elt.html("CC");
		}
		else {
			if (bid.color == "Diamonds" || bid.color == "Hearts") elt.css("color", "red");
			elt.html(bid.value + " " + colorsHtml[bid.color]);
			if (bid.multiplier == 2) elt.append("<span>C</span>");
			if (bid.multiplier == 4) elt.append("<span>CC</span>");
		}
		elt.show();
	}

	showBidPicker(minimumBid, doubleAvail) {
		if (this.freezed) return this.push("showBidPicker", minimumBid, doubleAvail);
		$("#bid-picker").show();
		$("#bid-picker input:checked").removeAttr("checked")
		$("#bid-doubled-double").hide();
		if (doubleAvail) $("#bid-double").show();
		else $("#bid-double").hide();
		$("#bid-pass").removeClass("disabled");
		$("#bid-picker label").removeClass("disabled");
		$("#bid-picker input").removeAttr("disabled");

		for (let elt of $("#bid-value-picker label")) {
			elt = $(elt)
			const val = $("#" + elt.attr("for")).val();
			if (val <= minimumBid) {
				$("#" + elt.attr("for")).attr("disabled", "");
				elt.addClass("disabled");
			}
		}
	}

	hideBidPicker() {
		if (this.freezed) return this.push("hideBidPicker");
		$("#bid-picker").hide();
	}

	disableAllBids() {
		if (this.freezed) return this.push("disableAllBids");
		$("#bid-picker label").addClass("disabled");
		$("#bid-picker label").attr("disabled", "");
	}

	showDoubleOption() {
		if (this.freezed) return this.push("showDoubleOption");
		$("#bid-picker").show();
		this.disableAllBids();
		$("#bid-pass").addClass("disabled");
		$("#bid-doubled-double").hide();
		$("#bid-double").show();
	}

	showDoubledDoubleOption() {
		if (this.freezed) return this.push("showDoubledDoubleOption");
		$("#bid-picker").show();
		$("#bid-pass").removeClass("disabled");
		this.disableAllBids();
		$("#bid-doubled-double").show();
		$("#bid-double").hide();
	}

	showTrickWinner(winner) {
		if (this.freezed) return this.push("showTrickWinner", winner);
		$(".played." + this.sideOfPlayer(winner)).addClass("winner");
		this.freeze(1500);
		this.cleanTrick();
	}

	cleanTrick() {
		if (this.freezed) return this.push("cleanTrick");
		$("#last-trick").empty();
		$("#current-trick").children().appendTo("#last-trick");
	}

	showTurn(turn, phase) {
		if (this.freezed) return this.push("showTurn", turn, phase);
		$(".name").removeClass("turn");
		$(".hand").removeClass("turn");
		if (phase == 1) {
			if (typeof (turn) == "number") this.nameEltOfPlayer(turn).addClass("turn");
			else {
				this.nameEltOfPlayer(turn[0]).addClass("turn");
				this.nameEltOfPlayer(turn[1]).addClass("turn");
			}
		}
		if (phase == 2) this.handOfPlayer(turn).addClass("turn");
	}

	updateScores(our_score, their_score) {
		if (this.freezed) return this.push("updateScores", our_score, their_score);
		$("#last-trick").empty();
		$("#us").html(our_score);
		$("#them").html(their_score);
	}

	message(msg, ms) {
		console.log("message de la vue :", msg);
	}

	showNames(players) {
		for (const player in players) {
			this.nameEltOfPlayer(game.localPlayerId(parseInt(player))).text(players[player].username);
		}
	}

}
