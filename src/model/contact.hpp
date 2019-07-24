#pragma once

#include <QtCore>

class ContactModel : public QObject {
    Q_OBJECT
    Q_PROPERTY(int total READ count NOTIFY countChanged);

public:
    ContactModel(QObject *parent = nullptr): QObject(parent) {
    }

    int count() const;

signals:
    void countChanged();
};
