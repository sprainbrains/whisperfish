#pragma once

#include <QAbstractListModel>

class SessionModel : public QAbstractListModel
{
    Q_OBJECT

public:
    SessionModel(QObject *parent = nullptr): QAbstractListModel(parent) {
    }
    int rowCount(const QModelIndex &parent=QModelIndex()) const;
    QVariant data(const QModelIndex &index, int role=Qt::DisplayRole) const;
};
