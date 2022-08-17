import QtQuick 2.6
import Nemo.DBus 2.0

Item {
    id: container
    function queryService() { dbusManager.queryService() }
    function enableService() { dbusManager.enableService() }
    function disableService() { dbusManager.disableService() }

    property string serviceName

    property bool serviceEnabled: false

    property bool canAccessSystemd: true

    Component.onCompleted: {
        dbusManager.queryService()
    }

    DBusInterface {
        id: dbusManager
        service: 'org.freedesktop.systemd1'
        path: '/org/freedesktop/systemd1'
        iface: 'org.freedesktop.systemd1.Manager'

        function reload() {
            call('Reload')
        }
        function queryService() {
            typedCall('GetUnit',
                      { 'type': 's', 'value': serviceName },
                      function(result) {
                          dbusUnit.path = result
                          dbusUnit.queryEnabledState()
                      },
                      function(error, message) {
                          console.log('GetUnit failed:', error)
                          console.log('GetUnit message:', message)
                          container.canAccessSystemd = false 
                      })
        }
        function enableService() {
            typedCall('EnableUnitFiles',
                      [
                          { 'type': 'as', 'value': [serviceName] },
                          { 'type': 'b', 'value': false },
                          { 'type': 'b', 'value': false },
                      ],
                      function(install, changes) {
                          container.serviceEnabled = true
                          reload()
                      },
                      function(error, message) {
                          console.log("EnableUnitFiles failed:", error)
                          console.log("EnableUnitFiles message:", message)
                      })
        }
        function disableService() {
            typedCall('DisableUnitFiles',
                      [
                          { 'type': 'as', 'value': [serviceName] },
                          { 'type': 'b', 'value': false },
                      ],
                      function(install, changes) {
                          container.serviceEnabled = false
                          reload()
                      },
                      function(error, message) {
                          console.log("DisableUnitFiles failed:", error)
                          console.log("DisableUnitFiles message:", message)
                      })
        }
    }

    DBusInterface {
        id: dbusUnit
        service: 'org.freedesktop.systemd1'
        path: '/org/freedesktop/systemd1/unit/harbour_2dwhisperfish_2eservice'
        iface: 'org.freedesktop.systemd1.Unit'

        function queryEnabledState() {
            var result = getProperty('UnitFileState')
            container.serviceEnabled = (result == 'enabled')
        }
    }
}
