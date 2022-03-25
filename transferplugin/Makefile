OBJS=WhisperfishTransfer.o WhisperfishTransferPlugin.o

all: libwhisperfishtransferplugin.so

libwhisperfishtransferplugin.so: $(OBJS)
	$(CXX) -shared -o $@ $^ -lnemotransferengine-qt5 -lQt5DBus -lQt5Core

%.o: %.cpp
	$(CXX) -fPIC -I /usr/include/qt5/ -I /usr/include/qt5/QtCore/ -I /usr/include/TransferEngine-qt5 -I /usr/include/qt5/QtDBus -I /usr/include/qt5/QtGui $$CXXFLAGS -c $<
