// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nemo.Thumbnailer 1.0

MouseArea {
    id: root
    property var attach: null
    property bool highlighted: containsPress
    property bool _hasAttach: attach !== null
    property bool _isAnimated: _hasAttach ? /\.(gif)$/i.test(attach.data) : false
    property bool _isVideo: _hasAttach ? /^video\//.test(attach.type) : false
    property bool _isAnimatedPaused: false

    onClicked: {
        if (!_hasAttach) {
            return
        } else if (_isVideo) {
            pageStack.push('../../pages/ViewVideoPage.qml', {
                               'title': MessageModel.peerName,
                               'path': attach.data,
                           })
        } else if (_isAnimatedPaused && animationLoader.item) {
            _isAnimatedPaused = false
            animationLoader.item.paused = false
        } else {
            pageStack.push('../../pages/ViewImagePage.qml', {
                               'title': MessageModel.peerName,
                               'path': attach.data,
                               'isAnimated': _isAnimated,
                           })
        }
    }

    // TODO handle missing files and failed thumbnails
    // TODO fix: there are no thumbnails for video files in Whisperfish, even though
    //      the thumbnailer supports videos
    Thumbnail {
        visible: !_isAnimated
        width: parent.width; height: parent.height
        source: (!_isAnimated && _hasAttach) ? attach.data : ''
        sourceSize { width: width; height: height }

        onStatusChanged: {
            if (status === Thumbnail.Error && _hasAttach) {
                console.warn("thumbnail failed for", attach.data)
            }
        }
    }

    Loader {
        id: animationLoader
        anchors.fill: parent
        sourceComponent: _isAnimated ? animatedComponent : null
    }

    HighlightImage {
        anchors.centerIn: parent
        width: Theme.iconSizeLarge; height: width
        source: (_isVideo || _isAnimatedPaused) ? 'image://theme/icon-l-play' : ''
    }

    Rectangle {
        anchors.fill: parent
        visible: highlighted
        color: Theme.highlightBackgroundColor
        opacity: Theme.opacityFaint
    }

    Component {
        id: animatedComponent
        AnimatedImage {
            // TODO Find the most intuitive way to show a gif, restart it,
            // and show it on a separate page. Is it ok if the inline view is cropped?
            property int rounds: 0
            property int maxRounds: 2
            asynchronous: true
            fillMode: Image.PreserveAspectCrop
            source: _hasAttach ? attach.data : ''
            onCurrentFrameChanged: if (currentFrame === 0) rounds++
            onRoundsChanged: {
                if (rounds <= maxRounds) return
                rounds = 0
                paused = true
                _isAnimatedPaused = true
            }
        }
    }
}
