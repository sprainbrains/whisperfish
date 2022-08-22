import QtQuick 2.6
import Nemo.DBus 2.0

Item {
    id: container
    function queryService() { dbusManager.queryService() }
    function enableService() { dbusManager.enableService() }
    function disableService() { dbusManager.disableService() }

    property string serviceName

    property bool serviceEnabled: false

    property bool serviceExists: false

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
            typedCall('GetUnitFileState',
                      { 'type': 's', 'value': serviceName },
                      function(result) {
                          container.serviceExists = true
                          dbusUnit.queryEnabledState()
                      },
                      function(error, message) {
                          console.log('GetUnitFileStatus failed:', error)
                          console.log('GetUnitFileStatus message:', message)
                          container.serviceExists = false 
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
