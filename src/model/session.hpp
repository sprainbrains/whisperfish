#pragma once

#include <QAbstractListModel>

class SessionModel : public QAbstractListModel
{
    Q_OBJECT

    Q_PROPERTY(int unread READ unread NOTIFY countChanged);
    Q_PROPERTY(int count READ count NOTIFY countChanged);

signals:
    void countChanged();

public:
    SessionModel(QObject *parent = nullptr): QAbstractListModel(parent) {
    }

    int unread() const;
    int count() const;

    int rowCount(const QModelIndex &parent=QModelIndex()) const;
    QVariant data(const QModelIndex &index, int role=Qt::DisplayRole) const;
};
