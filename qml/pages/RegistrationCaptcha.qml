import QtQuick 2.2
import Sailfish.Silica 1.0
import Sailfish.WebView 1.0

WebViewPage {
	id: page

	allowedOrientations: Orientation.PortraitMask

	backNavigation: false
	forwardNavigation: false
	showNavigationIndicator: false

	WebView {
		id: webView

		anchors {
			verticalCenter: parent.verticalCenter
			horizontalCenter: parent.horizontalCenter
		}

		// Capcha Format: aprox. 300px x 481px: 300/481 = 0.6237006237006237; 481/300 = 1.6033333333333333
		viewportWidth: parent.width
		viewportHeight: Math.min(parent.width*1.5, parent.height)
		width: viewportWidth
		height: viewportHeight

		active: true
		url: "https://signalcaptchas.org/registration/generate.html"

		onViewInitialized: {
			webView.loadFrameScript(Qt.resolvedUrl("captchaframescript.js"));
			webView.addMessageListener("Whisperfish:CaptchaDone");
		}

		function filterUrl(uri) {
			var codeMatch = /^signalcaptcha:\/\/(.*)$/.exec(uri);
			if (codeMatch !== null && codeMatch[1] != '') {
				console.log("Captcha Code Received", codeMatch[1]);
				complete(codeMatch[1]);
				return true;
			}
			return false;
		}

		function complete(code) {
			Prompt.captcha(code);
			if (!pageStack.busy) {
				pageStack.pop();
			}
		}

		onUrlChanged: {
			console.log("Url changed to: " + webView.url);
            if (filterUrl(url)) {
                webView.loadHtml("<html><head></head><body></body></html>");
            }
		}

		onRecvAsyncMessage: {
			console.log(message);
			if (message == "Whisperfish:CaptchaDone") {
				console.log("Captcha Code Received:", data.code);
				complete(data.code);
			} else if (message == "Whisperfish:CaptchaUnload") {
				console.log("Captcha Page Unloading: ", data.url);
				filterUrl(data.url);
			}
		}
	}
}
