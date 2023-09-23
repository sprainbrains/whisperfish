Name: harbour-whisperfish-shareplugin-v1
Summary: Share plugin v1 for Whisperfish

Version: 1.0.0
Release: 0%{?dist}
License: GPLv3+
Group: Qt/Qt
URL: https://gitlab.com/whisperfish/whisperfish/
Source0: %{name}-%{version}.tar.gz
Requires:  harbour-whisperfish
Requires:  nemo-transferengine-qt5
Requires:  declarative-transferengine-qt5 >= 0.0.44
Requires:  qt5-qtdeclarative-import-sensors >= 5.2

Requires:   sailfish-version <= 4.2

BuildRequires: pkgconfig(Qt5Core)
BuildRequires: pkgconfig(Qt5Qml)
BuildRequires: pkgconfig(nemotransferengine-qt5)
BuildRequires: qt5-qttools
BuildRequires: qt5-qttools-linguist

%{!?qtc_qmake5:%define qtc_qmake5 %qmake5}
%{!?qtc_make:%define qtc_make make}

%description
Whisperfish sharing plugin for Sailfish OS

%prep
%setup -q -n %{name}-%{version}

%build

%qmake5
make %{?_smp_mflags}

%install

install -Dm 644 %{_sourcedir}/../WhisperfishShare.qml \
    %{buildroot}%{_datadir}/nemo-transferengine/plugins/WhisperfishShare.qml
install -Dm 755 %{_sourcedir}/../libwhisperfishshareplugin.so \
    %{buildroot}%{_libdir}/nemo-transferengine/plugins/libwhisperfishshareplugin.so

%clean
rm -rf %{buildroot}
make clean

%files
%{_datadir}/nemo-transferengine/plugins/WhisperfishShare.qml
%{_libdir}/nemo-transferengine/plugins/libwhisperfishshareplugin.so
