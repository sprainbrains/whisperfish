%bcond_with harbour
%bcond_with lto
%bcond_with sccache
%bcond_with tools

# Targets 4.5 and newer default to Zstd RPM compression,
# which is not supported on 4.4 and older
%define _source_payload w6.xzdio
%define _binary_payload w6.xzdio

%if %{with harbour}
%define builddir target/sailfishos-harbour/%{_target_cpu}
%else
%define builddir target/sailfishos/%{_target_cpu}
%endif

%global __provides_exclude_from ^%{_datadir}/%{name}/lib/.*\\.so.*$
%global __requires_exclude_from ^%{_datadir}/%{name}/lib/.*$


Name: be.rubdos.harbour.whisperfish
Summary: Private messaging using Signal for SailfishOS/AuroraOS.

Version: 0.6.0
Release: 0
License: GPLv3+
Group: Qt/Qt
URL: https://github.com/sprainbrains/whisperfish/
Source0: %{name}-%{version}.tar.gz
#AutoReqProv: no
Requires:   sailfishsilica-qt5 >= 0.10.9
#Requires:   libauroraapp-launcher
#Requires:   sailfish-components-contacts-qt5
#Requires:   nemo-qml-plugin-contacts-qt5
#Requires:   nemo-qml-plugin-configuration-qt5
#Requires:   nemo-qml-plugin-notifications-qt5


# For the captcha QML application
#Requires:   qtmozembed-qt5

Requires:   sailfish-components-webview-qt5
#Requires:   openssl-libs
#Requires:   dbus
Requires: nemo-qml-plugin-dbus-qt5
Requires: sailfish-components-webview-qt5
Requires: sailfish-components-webview-qt5-popups
Requires: sailfish-components-webview-qt5-pickers


#Recommends:   sailjail
#Recommends:   sailjail-permissions
#Recommends:   harbour-whisperfish-shareplugin


# This comment lists SailfishOS-version specific code,
# for future reference, to track the reasoning behind the minimum SailfishOS version.
# We're aiming to support 3.4 as long as possible, since Jolla 1 will be stuck on that.
#
# - Contacts/contacts.db phoneNumbers.normalizedNumber: introduced in 3.3

















BuildRequires:  rust == 1.61.0+git1-1
BuildRequires:  rust-std-static == 1.61.0+git1-1
BuildRequires:  cargo == 1.61.0+git1-1
BuildRequires:  git
BuildRequires:  protobuf-compiler
BuildRequires:  nemo-qml-plugin-notifications-qt5-devel
BuildRequires:  qt5-qtwebsockets-devel
BuildRequires:  dbus-devel
BuildRequires:  gcc-c++
BuildRequires:  zlib-devel
BuildRequires:  coreutils
BuildRequires:  pkgconfig(auroraapp)
BuildRequires:  desktop-file-utils
BuildRequires:  pkgconfig(openssl) >= 1.1.1k
BuildRequires:  dbus-devel
BuildRequires:  pkgconfig(nemonotifications-qt5)
BuildRequires:  pkgconfig(Qt5Core)
BuildRequires:  pkgconfig(Qt5Qml)
BuildRequires:  pkgconfig(Qt5Quick)
BuildRequires:  pkgconfig(Qt5DBus)
BuildRequires:  pkgconfig(Qt5Sql)
BuildRequires:  pkgconfig(Qt5Multimedia)

BuildRequires:  meego-rpm-config

# For vendored sqlcipher
BuildRequires:  tcl
BuildRequires:  automake

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
%setup -q -n %{?with_harbour:harbour.}whisperfish

%build

# export CARGO_HOME=%{_sourcedir}/../target

rustc --version
cargo --version

%if %{with sccache}
%ifnarch %ix86
export RUSTC_WRAPPER=sccache
sccache --start-server
sccache -s
%endif
%endif

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
# export SB2_RUST_EXECVP_SHIM="/usr/bin/env LD_PRELOAD=/usr/lib/libsb2/libsb2.so.1 /usr/bin/env"
# export SB2_RUST_USE_REAL_EXECVP=Yes
# export SB2_RUST_USE_REAL_FN=Yes
# export SB2_RUST_NO_SPAWNVP=Yes

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
# We could use the %(version) and %(release), but SFDK will include a datetime stamp,
# ordering Cargo to recompile literally every second when the workspace is dirty.
# git describe is a lot stabler, because it only uses the commit number and potentially a -dirty flag
export GIT_VERSION=$(git describe  --exclude release,tag --dirty=-dirty)

# Configure Cargo.toml
# https://blog.rust-lang.org/2022/09/22/Rust-1.64.0.html#cargo-improvements-workspace-inheritance-and-multi-target-builds
%if 0%{?cargo_version:1}
for TOML in $(ls %{_sourcedir}/../Cargo.toml %{_sourcedir}/../*/Cargo.toml) ; do
  sed -i.bak "s/^version\s*=\s*\"[-\.0-9a-zA-Z]*\"$/version = \"%{cargo_version}\"/" "$TOML"
done
export CARGO_PROFILE_RELEASE_LTO=thin
%endif
cat %{_sourcedir}/../Cargo.toml

%if %{with lto}
export CARGO_PROFILE_RELEASE_LTO=thin
%endif

%if %{with tools}
BINS="--bins"
%else
BINS="--bin harbour-whisperfish"
%endif

if [ -z "$TARGET_VERSION" ]
then
TARGET_VERSION=$(grep VERSION_ID /etc/aurora-release | cut -d "=" -f2)
fi

# To make comparing easier: 4.4.0.58 >> 4.4
MAJOR_VERSION=$(echo $TARGET_VERSION | awk -F. '{print $1 FS $2}')

%if %{with shareplugin_v1} && %{with shareplugin_v2}
echo "Error: only give shareplugin_v1 or shareplugin_v2"
exit 1
%endif

%if %{with shareplugin_v2}
if [[ "$MAJOR_VERSION" < "4.4" ]]
then
    echo "Error: trying to compile shareplugin v2 for SFOS < 4.4"
    exit 1
fi
%define sharingsubdir sharing
%endif

%if %{with shareplugin_v1}
if [[ ! "$MAJOR_VERSION" < "4.4" ]]
then
    echo "Error: trying to compile shareplugin v1 for SFOS >= 4.4"
    exit 1
fi
%define sharingsubdir .
%endif

#cargo update
cargo build \
          -j 1 \
          -vv \
          --release \
          --no-default-features \
          $BINS \
          --features $FEATURES \
          --manifest-path %{_sourcedir}/../Cargo.toml

%if %{with sccache}
sccache -s
%endif

lrelease -idbased %{_sourcedir}/../translations/*.ts

%install

%{__mkdir_p} %{_sourcedir}/../translations_new/
%{__cp} -r %{_sourcedir}/../translations/*.qm %{_sourcedir}/../translations_new/

rename 'harbour-whisperfish' '%{name}' %{_sourcedir}/../translations_new/*.qm

install -d %{buildroot}%{_datadir}/%{name}/translations

install -Dm 644 %{_sourcedir}/../translations_new/*.qm \
        %{buildroot}%{_datadir}/%{name}/translations

install -Dm 644 %{_sourcedir}/../translations/harbour-whisperfish.qm \
        %{buildroot}%{_datadir}/%{name}/translations/harbour.whisperfish.qm

install -Dm 644 %{_sourcedir}/../translations/harbour-whisperfish-ru.qm \
        %{buildroot}%{_datadir}/%{name}/translations/harbour.whisperfish-ru.qm



%{__rm} -rf %{_sourcedir}/../translations_new

install -D %{targetdir}/harbour-whisperfish %{buildroot}%{_bindir}/%{name}

%if %{without harbour}
%if %{with tools}
install -D %{targetdir}/fetch-signal-attachment %{buildroot}%{_bindir}/fetch-signal-attachment
install -D %{targetdir}/whisperfish-migration-dry-run %{buildroot}%{_bindir}/whisperfish-migration-dry-run
%endif
%endif



install -D %{_sourcedir}/../harbour-whisperfish.desktop %{_sourcedir}/../%{name}.desktop

desktop-file-install --delete-original \
 --dir %{buildroot}%{_datadir}/applications \
   %{_sourcedir}/../%{name}.desktop


#
#
#
#
#
#

# Application icons
install -Dm 644 %{_sourcedir}/../icons/86x86/harbour-whisperfish.png \
    %{buildroot}%{_datadir}/icons/hicolor/86x86/apps/%{name}.png
install -Dm 644 %{_sourcedir}/../icons/108x108/harbour-whisperfish.png \
    %{buildroot}%{_datadir}/icons/hicolor/108x108/apps/%{name}.png
install -Dm 644 %{_sourcedir}/../icons/128x128/harbour-whisperfish.png \
    %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/%{name}.png
install -Dm 644 %{_sourcedir}/../icons/172x172/harbour-whisperfish.png \
    %{buildroot}%{_datadir}/icons/hicolor/172x172/apps/%{name}.png

# Libs
#%{__mkdir_p} %{buildroot}%{_datadir}/%{name}/lib/
#%{__cp} %{_sourcedir}/../lib/program/* %{buildroot}%{_datadir}/%{name}/lib/program

install -Dm 777 %{_sourcedir}/../lib/program/aurora-qml %{buildroot}%{_libexecdir}/%{name}/%{name}
#chmod +x %{buildroot}%{_datadir}/%{name}/lib/program/aurora-qml

# QML & icons
(cd %{_sourcedir}/../ && find ./qml ./icons \
    -type f \
    -exec \
        install -Dm 644 "{}" "%{buildroot}%{_datadir}/%{name}/{}" \; )

rename 'harbour-whisperfish' '%{name}' %{buildroot}%{_datadir}/%{name}/qml/*.qml

# Set the build date to the update notification
CURR_DATE=$(date "+%Y-%m-%d")
sed -i -r "s/buildDate: \"[0-9\-]{10}\".*/buildDate: \"${CURR_DATE}\"/g" "%{buildroot}%{_datadir}/%{name}/qml/pages/MainPage.qml"

%if %{without harbour}
# Dbus service
#
#
#
#

# Share plugin
%if %{with shareplugin_v1} || %{with shareplugin_v2}
install -Dm 644 %{targetdir}/shareplugin/WhisperfishShare.qml \
    %{buildroot}%{_datadir}/nemo-transferengine/plugins/%{sharingsubdir}/WhisperfishShare.qml
install -Dm 755 %{targetdir}/shareplugin/libwhisperfishshareplugin.so \
    %{buildroot}%{_libdir}/nemo-transferengine/plugins/%{sharingsubdir}/libwhisperfishshareplugin.so
%endif

%endif

%clean
rm -rf %{buildroot}

%if %{without harbour}





%endif

%if %{without harbour}



%endif

%files
%defattr(-,root,root,-)
%{_bindir}/%{name}
%{_libexecdir}/%{name}
%defattr(644,root,root,-)
%{_datadir}/%{name}
%{_datadir}/applications/%{name}.desktop

%{_datadir}/icons/hicolor/*/apps/%{name}.png





%if %{without harbour}




%if %{with shareplugin_v1} || %{with shareplugin_v2}
%files shareplugin
%{_datadir}/nemo-transferengine/plugins/%{sharingsubdir}/WhisperfishShare.qml
%{_libdir}/nemo-transferengine/plugins/%{sharingsubdir}/libwhisperfishshareplugin.so
%endif

%endif
