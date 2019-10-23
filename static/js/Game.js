
/* ---- bid class ---- */
class Bid{
	
	constructor(type, value, color){
		this.type = type;
		if(value === undefined) this.value = 0;
		else this.value = value;
		this.color = color;
		this.isDoubled = false;
		this.isDoubleDoubled = false;
	}
	
	doubleIt(){
		if(this.isDoubled){
			this.isDoubled = false;
			this.isDoubleDoubled  = true;
		}
		else this.isDoubled = true;
	}
	
	get isPass(){ return this.type == "pass"; }
	get isDouble(){ return this.type == "double"; }
	get isDoubledDouble(){ return this.type == "doubled-double"; }
}

/* ---- card class ---- */

// known card
class Card{
	
	colors = ["Spades", "Hearts", "Clubs", "Diamonds"];
	values = ["Seven", "Eight", "Nine", "Jack", "Queen", "King", "Ten", "Ace"];
	valuesTrump = ["Seven", "Eight", "Queen", "King", "Ten", "Ace", "Nine", "Jack"];
	
	constructor(color, value){
		this.color = color;
		this.value = value;
	}
	
	get trump(){
		return (game.trumpColor !== undefined && game.trumpColor == this.color);
	}
	
	valueOf(){
		var color_index = this.colors.indexOf(this.color);
		var value_index = (this.trump ? this.valuesTrump : this.values).indexOf(this.value);
		return color_index * 10 + value_index;
	}
	
	toString(){
		return this.value + "-" + this.color;
	}

	get data(){
		return {color: this.color, value: this.value};
	}
}

class Game{
	
	constructor(player_id){
		this.player_id = player_id;
	}
	
	loadState(data){
		this.first_player = this.localPlayerId(data.dealer_id + 1);
		this.players = data.players;
		vue.showNames(this.players);
		var state = data.game_state;
		var type = serde.datatype(state);
		if(type == "Lobby"){
			vue.message("En attente d'autres joueurs...");
		}
		if(type == "Bidding"){
			this.bids = {}
			this.phase = 1;
			for(var pbid of state.Bidding.bids){
				var player = this.localPlayerId(pbid.player_id);
				this.bids[player] = serde.playerBid(pbid, state.Bidding.coinche_state);
				this.turn = player;
			}
			this.turn = (this.first_player + state.Bidding.bids.length) % 4;
			vue.displayAllBids(this.bids);
			this.bidTurn();
		}
		if(type == "Running"){
			this.phase = 2;
			if(!this.trumpColor){
				this.bids = {}
				var bid = serde.bid(state.Running.bid, state.Running.coinche_state);
				// NOT GOOD : 
				this.bids[state.Running.team ? 0 : 1] = bid
				this.bids[state.Running.team ? 2 : 3] = bid
				this.trumpColor = bid.color;
			}
			var board = state.Running.board;
			this.current_trick = board.cards.map(serde.card);
			this.starting_player = this.localPlayerId(board.starting_player_id);
			this.turn = (this.starting_player + this.current_trick.length) % 4;
			console.log(this.turn);
			vue.displayTrick(this.starting_player, this.current_trick);
			vue.displayAllBids(this.bids);
			this.cardTurn();
		}
	}
	
	setCards(cards){
		cards.sort(function(a, b){ return a - b });
		this.cards = cards;
		vue.drawMyHand(this.cards);
		if(this.phase == 2) this.cardTurn();
	}
	
	isPlayerInMyTeam(player){
		return player % 2 == 0
	}
	
	bidTurn(){
		if(this.highestBid && this.highestBid.isDoubled){
			vue.showTurn([this.turn, (this.turn + 2)%4], 1);
			if(this.turn == 0 || this.turn == 2) vue.showDoubledDoubleOption();
		}
		else{
			vue.showTurn(this.turn, 1);
			var doubleAvail = this.highestBid && !this.isPlayerInMyTeam(this.highestBidPlayer)		
			if(this.turn == 0){
				var val = this.highestBid ? this.highestBid.value : 0;
				vue.showBidPicker(val, doubleAvail);
			}
			if(this.turn == 2 && doubleAvail) vue.showDoubleOption();
		}

	}
	
	cardTurn(){
		vue.showTurn(this.turn, 2);
		if(this.turn == 0) vue.makeCardsPlayable(this.cards);
		else vue.makeCardsUnplayable();
	}
	
	playCard(player, card){
		this.turn = (this.turn + 1) % 4;
		this.current_trick.push(card);
		if(player == 0) this.removeCard(card);
		vue.playCard(player, card);
		this.cardTurn();
	}
	
	removeCard(card){
		for(var i = 0; i < this.cards.length; i++){
			if(this.cards[i].valueOf() == card.valueOf()) this.cards.splice(i, 1);
		}
	}
	
	doBid(player, bid){
		this.bids[player] = bid;
		vue.displayBid(player, bid);
		if(bid.isDouble || bid.isDoubledDouble){
			this.highestBid.doubleIt();
			vue.displayBid(this.highestBidPlayer, this.highestBid);
		}
		if(this.isBidPhaseFinished()){
			vue.hideBidPicker();
			if(this.highestBid){
				this.trumpColor = this.highestBid.color;
				this.setCards(this.cards);
			}
			console.log("BID FINISHED");
			return;
		}
		else this.turn = (this.turn + 1) % 4;
		vue.hideBidPicker();
		this.bidTurn();
	}
	
	trickWon(winner){
		this.current_trick = [];
		this.turn = winner
		vue.showTrickWinner(winner);
		this.cardTurn();
	}
	
	localPlayerId(player_id){
		return (player_id + 4 - this.player_id) % 4;
	}
	
	get highestBidPlayer(){
		var max_v = 0;
		var max_p = -1;
		for(var player in this.bids){
			var v = this.bids[player].value;
			if(v > max_v){
				max_v = v;
				max_p = player;
			}
		}
		return max_p
	}
	
	get highestBid(){
		return this.bids[this.highestBidPlayer];
	}
	
	isBidPhaseFinished(){
		if(Object.keys(this.bids).length == 0) return false;
		if(this.highestBid && this.highestBid.isDoubled){
			var p1 = (this.highestBidPlayer + 1) % 4;
			var p3 = (this.highestBidPlayer + 3) % 4;
			return (this.bids[p1] && this.bids[p3] && this.bids[p1].isPass && this.bids[p3].isPass);
		}
		else if(this.highestBid && this.highestBid.isDoubleDoubled){
			return true;
		}
		else{
			var nbPass = 0;
			for(var p in this.bids) nbPass += this.bids[p].isPass;
			if(nbPass == 4) return true;
			if(this.highestBid && nbPass == 3 && ((this.turn + 1) % 4 == this.highestBidPlayer)) return true;
			return false;
		}
	}
	
}