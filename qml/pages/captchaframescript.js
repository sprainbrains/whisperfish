addEventListener("DOMContentLoaded", function () {
	// Adjust scaling
	viewport = content.document.createElement("meta");
	viewport.name = "viewport"
	viewport.content = "width=device-width, initial-scale=1"
	content.document.head.appendChild(viewport);

	// Forward events to qml
	content.document.body.addEventListener('ccdone', function (event) {
		sendAsyncMessage("Whisperfish:CaptchaDone", { "code": content.document.body.dataset.wfResult });
	});

	content.document.body.addEventListener('beforeunload', function (event) {
		sendAsyncMessage("Whisperfish:CaptchaUnload", { "url": content.window.location.href });
		return false;
	})

	// Insert custom captcha callback to extract the result.
	var gc = content.document.getElementsByClassName("g-recaptcha")[0];
	gc.dataset.callback = "wf_cp_handler";

	var sc = content.document.createElement("script");
	sc.textContent = "function wf_cp_handler(c) {"+
		"document.body.dataset.wfResult = c;"+
		"document.body.dispatchEvent(new Event('ccdone'));"+
		"}";
	content.document.body.appendChild(sc);
})
