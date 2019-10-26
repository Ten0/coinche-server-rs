var socket;
var game;
var vue;

window.addEventListener("load", function () {

	vue = new Vue();

	var url = new URL(location);
	var user = url.searchParams.get("user");
	if (!user) {
		alert("Please enter an username in the url : [...].html?user=<your name>");
	}

	socket = new WebSocket("ws://localhost:3000/ws/");
	socket.onopen = function (event) {
		send("Init", { username: user });
	}
	socket.onmessage = onmessage;


});
