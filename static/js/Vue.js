var valuesHtml = {
	"Seven": "7",
	"Eight": "8",
	"Nine": "9",
	"Ten": "10",
	"Jack": "J",
	"Queen": "Q",
	"King": "K",
	"Ace": "A",
};

var colorsHtml = {
	"Spades": "&#9824;",
	"Hearts": "&hearts;",
	"Clubs": "&clubs;",
	"Diamonds": "&diams;",
	"NoTrump": "SA",
	"AllTrump": "TA"
}

/* DOM events */

function onCardClick(evt){
	var data = JSON.parse($(this).attr("data"));
	var card = new Card(data.color, data.value);
	attemptPlay(card);
}

function onBidChange(evt){
	var value = $('input:checked', '#bid-value-picker').val();
	var color = $('input:checked', '#bid-color-picker').val();
	if(color !== undefined && value !== undefined){
		attemptBid(new Bid("bid", value, color));
	}
}

/* --- HTML generation --- */

class HtmlGenerator{

	constructor(clockwise){
		this.clockwise = clockwise;
		$("#bid-picker input").change(onBidChange);
		this.hideBidPicker();
	}
	
	timeout(callback, ms){
		window.setTimeout(function(){
			callback();
		}, ms);
	}
	
	sideOfPlayer(player){
		var sides = ["bottom", "right", "top", "left"];
		if(this.clockwise) sides = ["bottom", "left", "top", "right"];
		return sides[player];
	}
	
	handOfPlayer(player){
		return $("#" + this.sideOfPlayer(player) + "-hand");
	}
	
	bidOfPlayer(player){
		return $("#" + this.sideOfPlayer(player) + "-bid");
	}
	
	nameEltOfPlayer(player){
		return $("#" + this.sideOfPlayer(player) + "-name");
	}
	
	genCard(player, card){
		var side = this.sideOfPlayer(player);
		var elt;
		if(card === undefined){
			elt = $('<div class="card hidden"><div></div></div>');
		}
		else{
			elt = $('<div class="card visible"><div><div></div>');
			elt.children().html("\n" + valuesHtml[card.value] + "<br>" + colorsHtml[card.color] + "\n");
			elt.attr("data", JSON.stringify(card.data));
			elt.attr("id", card.toString());
			if(card.trump) elt.addClass("trump");
			if(card.color == "Diamonds" || card.color == "Hearts") elt.css("color", "red");
		}
		elt.addClass(side);
		return elt;
	}
	
	drawOtherHand(player, nb_cards){
		var hand = this.handOfPlayer(player);
		hand.html("");
		for(var i = 0; i < nb_cards; i++){
			hand.append(this.genCard(player));
		}
	}
	
	drawMyHand(cards){
		var hand = this.handOfPlayer(0);
		hand.html("");
		for(var card of cards){
			hand.append(this.genCard(0, card));
		}
	}
	
	displayTrick(starting_player, cards){
		for(var i = 0; i < cards.length; i++){
			this.playCard((starting_player + i) % 4, cards[i], true);
		}
	}
	
	playCard(player, card, forceCreate){
		var elt;
		if(player == 0 && !forceCreate) elt = $(".card#" + card.toString());
		else{
			elt = this.genCard(player, card);
			if(!forceCreate) $(this.handOfPlayer(player).children(".card")[0]).remove();
		}
		elt.addClass(this.sideOfPlayer[player]);
		elt.addClass("visible");
		elt.addClass("played");
		elt.removeClass("playable");
		elt.appendTo("#current-trick");
		elt.unbind("click");
	}
	
	makeCardsPlayable(playableCards){
		$(".card.bottom").unbind("click");
		for(var card of playableCards){
			var elt = $(".card#" + card.toString());
			elt.addClass("playable");
			elt.click(onCardClick);
		}
	}
	
	makeCardsUnplayable(){
		$(".card.bottom").removeClass("playable");
		$(".card.bottom").unbind("click");
	}
	
	displayAllBids(bids){
		for(var player in bids){
			this.displayBid(player, bids[player]);
		}
	}
	
	displayBid(player, bid){
		var elt = this.bidOfPlayer(player);
		elt.css("color", "black");
		if(bid.isPass || bid.isDouble || bid.isDoubleDoubled){
			if(bid.isPass) elt.html("-");
			if(bid.isDouble) elt.html("C");
			if(bid.isDoubleDoubled) elt.html("CC");
		}
		else{
			if(bid.color == "Diamonds" || bid.color == "Hearts") elt.css("color", "red");
			elt.html(bid.value + " " + colorsHtml[bid.color]);
			if(bid.isDoubled) elt.append("<span>C</span>");
			if(bid.isDoubleDoubled) elt.append("<span>CC</span>");
		}
		elt.show();
	}
	
	showBidPicker(minimumBid, doubleAvail){
		$("#bid-picker").show();
		$("#bid-picker input:checked").removeAttr("checked")
		$("#bid-doubled-double").hide();
		if(doubleAvail) $("#bid-double").show();
		else $("#bid-double").hide();
		$("#bid-pass").removeClass("disabled");
		$("#bid-picker label").removeClass("disabled");
		$("#bid-picker label").removeAttr("disabled");
		
		for(var elt of $("#bid-value-picker label")){
			elt = $(elt)
			var val = $("#" + elt.attr("for")).val();
			if(val <= minimumBid){
				$("#" + elt.attr("for")).attr("disabled", "");
				elt.addClass("disabled");
			}
		}
	}
	
	disableAllBids(){
		$("#bid-picker label").addClass("disabled");
		$("#bid-picker label").attr("disabled", "");
	}
	
	showDoubleOption(){
		$("#bid-picker").show();
		this.disableAllBids();
		$("#bid-pass").addClass("disabled");
		$("#bid-doubled-double").hide();
		$("#bid-double").show();
	}
	
	showDoubledDoubleOption(){
		$("#bid-picker").show();
		$("#bid-pass").removeClass("disabled");
		this.disableAllBids();
		$("#bid-doubled-double").show();
		$("#bid-double").hide();
	}
	
	showTrickWinner(winner){
		$(".played." + this.sideOfPlayer(winner)).addClass("winner");
		this.timeout(vue.cleanTrick, 1500);
	}
	
	cleanTrick(){
		$("#last-trick").empty();
		$("#current-trick").children().appendTo("#last-trick");
	}
	
	showTurn(turn, phase){
		$(".name").removeClass("turn");
		$(".hand").removeClass("turn");
		if(phase == 1){
			if(typeof(turn) == "number") this.nameEltOfPlayer(turn).addClass("turn");
			else{
				this.nameEltOfPlayer(turn[0]).addClass("turn");
				this.nameEltOfPlayer(turn[1]).addClass("turn");
			}
		}
		if(phase == 2) this.handOfPlayer(turn).addClass("turn");
	}
	
	message(msg, ms){
		console.log("message de la vue :", msg);
	}
	
	showNames(players){
		for(var player in players){
			this.nameEltOfPlayer(game.localPlayerId(parseInt(player))).text(players[player].username);
		}
	}
	
	hideBidPicker(){
		$("#bid-picker").hide();
	}
	
}