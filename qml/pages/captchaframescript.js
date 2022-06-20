addEventListener("DOMContentLoaded", function () {
	// Adjust scaling
	viewport = content.document.createElement("meta");
	viewport.name = "viewport"
	viewport.content = "width=device-width, initial-scale=1"
	content.document.head.appendChild(viewport);

	// Forward event & result to qml
	content.document.body.addEventListener('ccdone', function (event) {
		sendAsyncMessage("Whisperfish:CaptchaDone", { "code": content.document.body.dataset.wfResult });
	});

	// Extract recaptcha result
	var sc = content.document.createElement("script");
	sc.textContent = "var wf_cp_done = false;" +
		"function wf_cp_handler(c) {" +
		"if (wf_cp_done) return;" +
		"var token = grecaptcha.enterprise.getResponse();"+
		"if (token == '') return;"+
		"wf_cp_done = true;"+
		// Build the captcha string
		"var sitekey = '6LfBXs0bAAAAAAjkDyyI1Lk5gBAUWfhI_bIyox5W';"+
		"var result = 'signal-recaptcha-v2.' + sitekey + '.registration.' + token;"+
		// Make result accessible and notify frame script
		"document.body.dataset.wfResult = result;"+
		"document.body.dispatchEvent(new Event('ccdone'));"+
		"}"+
		// Watch for body class changes to detect when the capcha is done
		"var wf_observer = new MutationObserver(wf_cp_handler);"+
		"wf_observer.observe(document.body, {'attributes': true});";
	content.document.body.appendChild(sc);
})
