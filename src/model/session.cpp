#include "model/session.hpp"

int SessionModel::rowCount(const QModelIndex &parent) const {
    return 0;
}

QVariant SessionModel::data(const QModelIndex &index, int role) const {
    return QVariant::fromValue(true);
}

int SessionModel::unread() const {
    return 0;
}
