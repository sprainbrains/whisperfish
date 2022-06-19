import QtQuick 2.2
import Sailfish.Silica 1.0
import Sailfish.WebView 1.0
import Nemo.DBus 2.0

// Warning: Do not use this page within Whisperfish:
// Clashing sqlite and sqlcipher causes mozembedded to crash.

WebViewPage {
	id: page

	allowedOrientations: Orientation.PortraitMask

	backNavigation: false
	forwardNavigation: false
	showNavigationIndicator: false

    DBusInterface {
        id: whisperfishApp
        service: "be.rubdos.whisperfish"
        path: "/be/rubdos/whisperfish/captcha"
        iface: "be.rubdos.whisperfish.captcha"
    }

	Timer {
		id: closeTimer
		interval: 3000
		running: false
		repeat: false
		onTriggered: Qt.quit()
	}

	// XXX Maybe put a PageHeader and a short text here?

	WebView {
		id: webView

		anchors {
			verticalCenter: parent.verticalCenter
			horizontalCenter: parent.horizontalCenter
		}

		// Capcha Format: aprox. 300px x 500x: 300/500 = 0.6; 500/300 = 1.666
		viewportWidth: parent.width
		viewportHeight: Math.min(parent.width*1.666, parent.height)
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

		property bool captchaSent: false

		function complete(code) {
			if(!captchaSent) {
				captchaSent = true
				whisperfishApp.call(
					"handleCaptcha",
					[code],
					function () {
						console.log("Captcha code sent!")
						closeTimer.start()
					},
					function (error, message) {
						console.log('Sending captcha code failed: ' + error + ' message: ' + message)
						closeTimer.start()
					}
				)
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
