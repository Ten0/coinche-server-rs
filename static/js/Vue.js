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

	beloteOfPlayer(player){
		return $(`#${this.sideOfPlayer(player)}-belote`);
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
			this.playCard((starting_player + i) % 4, cards[i], null, true);
		}
	}

	playCard(player, card, belote, forceCreate) {
		if (this.freezed) return this.push("playCard", player, card, belote, forceCreate);
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
		if(belote !== null){
			this.beloteOfPlayer(player).html(belote);
			this.beloteOfPlayer(player).show();
			this.beloteOfPlayer(player).fadeOut(3000);
		}
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
		let [html, color] = this.bidHtml(bid);
		elt.css("color", color);
		elt.html(html);
		elt.show();
	}

	bidHtml(bid){
		var color = "black";
		var html = "";
		if (bid.isPass || bid.isDouble || bid.isDoubledDouble) {
			if (bid.isPass) html = "-";
			if (bid.isDouble) html = "C";
			if (bid.isDoubleDoubled) html = "CC";
		}
		else {
			if (bid.color == "Diamonds" || bid.color == "Hearts") color = "red";
			html = bid.valueRepr + " " + colorsHtml[bid.color];
			if (bid.multiplier == 2) html += "<span>C</span>";
			if (bid.multiplier == 4) html += "<span>CC</span>";
		}
		return [html, color];
	}

	roundResultHtml(round_result, team, extended){
		let bid = serde.bid(round_result.bid);
		let [bid_html, bid_color] = this.bidHtml(bid);
		let points = round_result.points
		let bid_team = round_result.team ? 1 : 0;
		let won = points[team] > points[1 - team];

		let tds = [];
		tds.push(createElt("td", bid_html, {
			color : bid_color,
			fontStyle: bid_team == team ? "normal" : "italic"
		}));
		tds.push(createElt("td", points[team]));
		tds.push(createElt("td", points[1 - team]));
		
		let tr_class = won ? "won" : "lost";
		let main_tr = createElt("tr", tds, {cursor: "pointer"}, {class: tr_class});
		
		$(main_tr).click(function(){
			let elt = $(this);
			if(elt.hasClass("extended-first")){
				elt.removeClass("extended-first");
				elt.next().hide();
				elt.next().next().hide();
			}
			else{
				elt.addClass("extended-first")
				elt.next().show();
				elt.next().next().show();
			}
		})
		let tr_bid = createElt("tr", [
			createElt("td", "Annonce"),
			createElt("td", bid_team == team ? bid_html : "", {color: bid_color}),
			createElt("td", bid_team != team ? bid_html : "", {color: bid_color})
		], {display: "none"}, {class: "extented"});
		let tr_scored = createElt("tr", [
			createElt("td", "Faits"),
			createElt("td", round_result.scored_points[team], {fontWeight: won ? "bold" : "normal"}),
			createElt("td", round_result.scored_points[1 - team], {fontWeight: won ? "normal" : "bold"})
		], {display: "none"}, {class: "extented-last"});
		return [main_tr, tr_bid, tr_scored];
	}

	updateScoreboard(points, round_results, team){
		if (this.freezed) return this.push("updateScoreboard", points, round_results, team);
		$("#last-trick").empty();
		$($("th")[1]).html(points[team]);
		$($("th")[2]).html(points[1 - team]);
		$("tbody").html("");
		for(let round_result of round_results){
			$("tbody").append(this.roundResultHtml(round_result, team));
		}
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

		if(minimumBid == 250){
			this.disableAllBids();
		}
		else{
			for (let elt of $("#bid-value-picker label")) {
				elt = $(elt)
				const val = $("#" + elt.attr("for")).val();
				if (val <= minimumBid) {
					$("#" + elt.attr("for")).attr("disabled", "");
					elt.addClass("disabled");
				}
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
		document.title = "Coinche";
		$(".name").removeClass("turn");
		$(".hand").removeClass("turn");
		if (phase == 1) {
			if (typeof (turn) == "number") this.nameEltOfPlayer(turn).addClass("turn");
			else {
				this.nameEltOfPlayer(turn[0]).addClass("turn");
				this.nameEltOfPlayer(turn[1]).addClass("turn");
				if(turn[0] == 0 || turn[1] == 0) this.notifyMyTurn();
			}
		}
		if (phase == 2) this.handOfPlayer(turn).addClass("turn");
		if (turn == 0) this.notifyMyTurn();
	}

	notifyMyTurn(){
		$("#turn_sound")[0].play();
		document.title = "Coinche - A toi de jouer !"
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

function createElt(tag, content, css, attrs){
	if(content === undefined) content = "";
	var elt = $("<" + tag + "></" + tag + ">");
	if(Array.isArray(content) == "string"){
		for(e of content) elt.append(e);
	}
	else elt.append(content);
	if (css !== undefined) elt.css(css);
	if (attrs !== undefined) elt.attr(attrs);
	return elt;
}