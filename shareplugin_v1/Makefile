OBJS=WhisperfishPluginInfo.o WhisperfishTransfer.o WhisperfishTransferPlugin.o

all: libwhisperfishshareplugin.so

libwhisperfishshareplugin.so: $(OBJS)
	$(CXX) -shared -o $@ $^ -lnemotransferengine-qt5 -lQt5DBus -lQt5Core

%.o: %.cpp
	$(CXX) -fPIC -I /usr/include/qt5/ -I /usr/include/qt5/QtCore/ $$CXXFLAGS -c $<
