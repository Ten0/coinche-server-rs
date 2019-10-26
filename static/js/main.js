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

	let match = window.location.href.match(/^http(?<secure>s?):\/\/(?<hostname>[^/]*)/)
	if (match) {
		let { secure, hostname } = match.groups
		socket = new WebSocket(`ws${secure}://${hostname}/ws/`);
		socket.onopen = function (event) {
			send("Init", { username: user });
		}
		socket.onmessage = onmessage;
	} else {
		alert("Could not parse url")
	}

});
