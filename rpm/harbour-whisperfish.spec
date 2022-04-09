%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

%bcond_with harbour

%if %{with harbour}
%define builddir target/sailfishos-harbour/%{_target_cpu}
%else
%define builddir target/sailfishos/%{_target_cpu}
%endif

Name: harbour-whisperfish
Summary: Private messaging using Signal for SailfishOS.

Version: 0.6.0
Release: 0%{?dist}
License: GPLv3+
Group: Qt/Qt
URL: https://gitlab.com/whisperfish/whisperfish/
Source0: %{name}-%{version}.tar.gz
Requires:   sailfishsilica-qt5 >= 0.10.9
Requires:   sailfish-components-contacts-qt5
Requires:   nemo-qml-plugin-contacts-qt5
Requires:   nemo-qml-plugin-configuration-qt5
Requires:   nemo-qml-plugin-notifications-qt5
Requires:   sailfish-components-webview-qt5
Requires:   openssl-libs
Requires:   dbus

Recommends:   sailjail
Recommends:   sailjail-permissions

# This comment lists SailfishOS-version specific code,
# for future reference, to track the reasoning behind the minimum SailfishOS version.
# We're aiming to support 3.4 as long as possible, since Jolla 1 will be stuck on that.
#
# - Contacts/contacts.db phoneNumbers.normalizedNumber: introduced in 3.3
Requires:   sailfish-version >= 3.3

%if "%{release}" == "0.sf3"
Requires: sailfish-version < 4.0
Requires: sailfish-version >= 3.0
%endif

%if "%{release}" == "0.sf4"
Requires: sailfish-version < 5.0
Requires: sailfish-version >= 4.0
%endif

%if "%{release}" == "0.sf5"
Requires: sailfish-version < 6.0
Requires: sailfish-version >= 5.0
%endif

BuildRequires:  rust >= 1.48
BuildRequires:  rust-std-static >= 1.48
BuildRequires:  cargo >= 1.52.1+git2-1
BuildRequires:  git
BuildRequires:  protobuf-compiler
BuildRequires:  nemo-qml-plugin-notifications-qt5-devel
BuildRequires:  qtmozembed-qt5-devel
BuildRequires:  qt5-qtwebsockets-devel
BuildRequires:  openssl-devel
BuildRequires:  dbus-devel
BuildRequires:  gcc-c++
BuildRequires:  zlib-devel
BuildRequires:  coreutils

# For vendored sqlcipher
BuildRequires:  tcl
BuildRequires:  automake

%if %{without harbour}
BuildRequires: pkgconfig(nemotransferengine-qt5)
Recommends:    %{name}-shareplugin
%endif

%{!?qtc_qmake5:%define qtc_qmake5 %qmake5}
%{!?qtc_make:%define qtc_make make}

%ifarch %arm
%define targetdir %{_sourcedir}/../target/armv7-unknown-linux-gnueabihf/release
%endif
%ifarch aarch64
%define targetdir %{_sourcedir}/../target/aarch64-unknown-linux-gnu/release
%endif
%ifarch %ix86
%define targetdir %{_sourcedir}/../target/i686-unknown-linux-gnu/release
%endif

%description
%{summary}

%prep
%setup -q -n %{?with_harbour:harbour-}whisperfish

# harbour-whisperfish-shareplugin
%if %{without harbour}
%package shareplugin
Summary: Share plugin for Whisperfish
%description shareplugin
%{summary}

Group: Qt/Qt

%endif
# end harbour-whisperfish-shareplugin

%build

# export CARGO_HOME=%{_sourcedir}/../target

rustc --version
cargo --version

# https://git.sailfishos.org/mer-core/gecko-dev/blob/master/rpm/xulrunner-qt5.spec#L224
# When cross-compiling under SB2 rust needs to know what arch to emit
# when nothing is specified on the command line. That usually defaults
# to "whatever rust was built as" but in SB2 rust is accelerated and
# would produce x86 so this is how it knows differently. Not needed
# for native x86 builds
%ifarch %arm
export SB2_RUST_TARGET_TRIPLE=armv7-unknown-linux-gnueabihf
export CFLAGS_armv7_unknown_linux_gnueabihf=$CFLAGS
export CXXFLAGS_armv7_unknown_linux_gnueabihf=$CXXFLAGS
%endif
%ifarch aarch64
export SB2_RUST_TARGET_TRIPLE=aarch64-unknown-linux-gnu
export CFLAGS_aarch64_unknown_linux_gnu=$CFLAGS
export CXXFLAGS_aarch64_unknown_linux_gnu=$CXXFLAGS
%endif
%ifarch %ix86
export SB2_RUST_TARGET_TRIPLE=i686-unknown-linux-gnu
export CFLAGS_i686_unknown_linux_gnu=$CFLAGS
export CXXFLAGS_i686_unknown_linux_gnu=$CXXFLAGS
%endif

export CFLAGS="-O2 -g -pipe -Wall -Wp,-D_FORTIFY_SOURCE=2 -fexceptions -fstack-protector --param=ssp-buffer-size=4 -Wformat -Wformat-security -fmessage-length=0"
export CXXFLAGS=$CFLAGS
# This avoids a malloc hang in sb2 gated calls to execvp/dup2/chdir
# during fork/exec. It has no effect outside sb2 so doesn't hurt
# native builds.
export SB2_RUST_EXECVP_SHIM="/usr/bin/env LD_PRELOAD=/usr/lib/libsb2/libsb2.so.1 /usr/bin/env"
export SB2_RUST_USE_REAL_EXECVP=Yes
export SB2_RUST_USE_REAL_FN=Yes
export SB2_RUST_NO_SPAWNVP=Yes

%ifnarch %ix86
export HOST_CC=host-cc
export HOST_CXX=host-cxx
export CC_i686_unknown_linux_gnu=$HOST_CC
export CXX_i686_unknown_linux_gnu=$HOST_CXX
%endif

# Set meego cross compilers
export PATH=/opt/cross/bin/:$PATH
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=armv7hl-meego-linux-gnueabi-gcc
export CC_armv7_unknown_linux_gnueabihf=armv7hl-meego-linux-gnueabi-gcc
export CXX_armv7_unknown_linux_gnueabihf=armv7hl-meego-linux-gnueabi-g++
export AR_armv7_unknown_linux_gnueabihf=armv7hl-meego-linux-gnueabi-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-meego-linux-gnu-gcc
export CC_aarch64_unknown_linux_gnu=aarch64-meego-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-meego-linux-gnu-g++
export AR_aarch64_unknown_linux_gnu=aarch64-meego-linux-gnu-ar

# Hack for qmetaobject on QT_SELECT=5 platforms
# export QMAKE=%{_sourcedir}/../rpm/qmake-sailfish

# Hack for cross linking against dbus
export PKG_CONFIG_ALLOW_CROSS_i686_unknown_linux_gnu=1
export PKG_CONFIG_ALLOW_CROSS_armv7_unknown_linux_gnueabihf=1
export PKG_CONFIG_ALLOW_CROSS_aarch64_unknown_linux_gnu=1

%if %{without harbour}
FEATURES=sailfish
%endif
%if %{with harbour}
FEATURES="sailfish,harbour"
%endif

export RUSTFLAGS="%{?rustflags}"
export RPM_VERSION=%{version}-%{release}

# Configure Cargo.toml
%if 0%{?cargo_version:1}
sed -ie "s/^version\s\?=\s\?\".*\"/version = \"%{cargo_version}\"/" %{_sourcedir}/../Cargo.toml
export CARGO_PROFILE_RELEASE_LTO=fat
%endif
cat %{_sourcedir}/../Cargo.toml

cargo build \
          -j 1 \
          --verbose \
          --release \
          --no-default-features \
          --features $FEATURES \
          --manifest-path %{_sourcedir}/../Cargo.toml

%if %{without harbour}
mkdir -p %{targetdir}/shareplugin/
cd %{targetdir}/shareplugin/
rm -f *.so *.o moc_*

if [[ "$(bash %{_sourcedir}/../target_at_least.sh 4.4.0.58)" == "1" ]]
then
    # Share plugin API v2
    cp -ar %{_sourcedir}/../shareplugin_v2/* .
    %qmake5
    make %{?_smp_mflags}
else
    # Share plugin API v1
    cp -ar %{_sourcedir}/../shareplugin_v1/* .
    make %{?_smp_mflags}
fi
%endif

%install

install -d %{buildroot}%{_datadir}/harbour-whisperfish/translations
for filename in %{_sourcedir}/../translations/*.ts; do
    base=$(basename -s .ts $filename)
    lrelease \
        -idbased "%{_sourcedir}/../translations/$base.ts" \
        -qm "%{buildroot}%{_datadir}/harbour-whisperfish/translations/$base.qm";
done

install -D %{targetdir}/harbour-whisperfish %{buildroot}%{_bindir}/harbour-whisperfish
%if %{without harbour}
install -D %{targetdir}/fetch-signal-attachment %{buildroot}%{_bindir}/fetch-signal-attachment
install -D %{targetdir}/whisperfish-migration-dry-run %{buildroot}%{_bindir}/whisperfish-migration-dry-run
%endif

desktop-file-install \
  --dir %{buildroot}%{_datadir}/applications \
   %{_sourcedir}/../harbour-whisperfish.desktop

install -Dm 644 %{_sourcedir}/../harbour-whisperfish.profile \
    %{buildroot}%{_sysconfdir}/sailjail/permissions/harbour-whisperfish.profile
install -Dm 644 %{_sourcedir}/../harbour-whisperfish.privileges \
    %{buildroot}%{_datadir}/mapplauncherd/privileges.d/harbour-whisperfish.privileges
install -Dm 644 %{_sourcedir}/../harbour-whisperfish-message.conf \
    %{buildroot}%{_datadir}/lipstick/notificationcategories/harbour-whisperfish-message.conf

# Application icons
install -Dm 644 %{_sourcedir}/../icons/86x86/harbour-whisperfish.png \
    %{buildroot}%{_datadir}/icons/hicolor/86x86/apps/harbour-whisperfish.png
install -Dm 644 %{_sourcedir}/../icons/108x108/harbour-whisperfish.png \
    %{buildroot}%{_datadir}/icons/hicolor/108x108/apps/harbour-whisperfish.png
install -Dm 644 %{_sourcedir}/../icons/128x128/harbour-whisperfish.png \
    %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/harbour-whisperfish.png
install -Dm 644 %{_sourcedir}/../icons/172x172/harbour-whisperfish.png \
    %{buildroot}%{_datadir}/icons/hicolor/172x172/apps/harbour-whisperfish.png

# QML & icons
(cd %{_sourcedir}/../ && find ./qml ./icons \
    -type f \
    -exec \
        install -Dm 644 "{}" "%{buildroot}%{_datadir}/harbour-whisperfish/{}" \; )

%if %{without harbour}
# Dbus service
install -Dm 644 %{_sourcedir}/../be.rubdos.whisperfish.service \
    %{buildroot}%{_datadir}/dbus-1/services/be.rubdos.whisperfish.service
install -Dm 644 %{_sourcedir}/../harbour-whisperfish.service \
    %{buildroot}%{_userunitdir}/harbour-whisperfish.service

# Share plugin
install -Dm 644 %{targetdir}/shareplugin/WhisperfishShare.qml \
    %{buildroot}%{_datadir}/nemo-transferengine/plugins/sharing/WhisperfishShare.qml
install -Dm 755 %{targetdir}/shareplugin/libwhisperfishshareplugin.so \
    %{buildroot}%{_libdir}/nemo-transferengine/plugins/sharing/libwhisperfishshareplugin.so
%endif

%clean
rm -rf %{buildroot}

%if %{without harbour}
%post
systemctl-user daemon-reload
%endif

%if %{without harbour}
%preun
systemctl-user disable harbour-whisperfish.service || true
%endif

%files
%defattr(-,root,root,-)
%{_bindir}/*
%{_datadir}/%{name}
%{_datadir}/applications/%{name}.desktop
%{_datadir}/mapplauncherd/privileges.d/%{name}.privileges
%{_datadir}/icons/hicolor/*/apps/%{name}.png
%{_datadir}/lipstick/notificationcategories/%{name}-message.conf

%{_sysconfdir}/sailjail/permissions/harbour-whisperfish.profile

%if %{without harbour}
%{_exec_prefix}/lib/systemd/user/%{name}.service
%{_datadir}/dbus-1/services/be.rubdos.whisperfish.service
%endif

%if %{without harbour}
%files shareplugin
%{_datadir}/nemo-transferengine/plugins/sharing/WhisperfishShare.qml
%{_libdir}/nemo-transferengine/plugins/sharing/libwhisperfishshareplugin.so
%endif
