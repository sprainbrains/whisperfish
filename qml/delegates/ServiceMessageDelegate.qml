// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later

import QtQuick 2.6
import Sailfish.Silica 1.0
// import "../components"

ListItem {
    id: delegate
    contentHeight: Theme.itemSizeMedium
    width: parent.width

    property QtObject modelData
}
