let socket: WebSocket;
let game: Game;
let vue: Vue;

window.addEventListener("load", () => {

	vue = new Vue();

	const url = new URL(window.location.href);
	const user = url.searchParams.get("user");
	if (!user) {
		alert("Please enter an username in the url : [...].html?user=<your name>");
	}

	const match = window.location.href.match(/^http(?<secure>s?):\/\/(?<hostname>[^/]*)/);
	if (match) {
		const { secure, hostname } = match.groups!;
		socket = new WebSocket(`ws${secure}://${hostname}/ws/`);
		socket.onopen = _event => {
			send("Init", { username: user });
		};
		socket.onmessage = onMessage;
	} else {
		alert("Could not parse url");
	}

});
