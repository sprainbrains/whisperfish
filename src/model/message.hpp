#pragma once

#include <QtCore>

class MessageModel : public QObject {
    Q_OBJECT
    Q_PROPERTY(int unsentCount READ unsentCount NOTIFY messageCountChanged);
    Q_PROPERTY(int total READ count NOTIFY messageCountChanged);

public:
    MessageModel(QObject *parent = nullptr): QObject(parent) {
    }

    int unsentCount() const;
    int count() const;

signals:
    void messageCountChanged();
};
