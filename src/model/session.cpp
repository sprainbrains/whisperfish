#include "model/session.hpp"

int SessionModel::rowCount(const QModelIndex &parent) const {
    // TODO
    return 0;
}

QVariant SessionModel::data(const QModelIndex &index, int role) const {
    // TODO
    return QVariant::fromValue(true);
}

int SessionModel::unread() const {
    // TODO
    return 0;
}

int SessionModel::count() const {
    // TODO
    return 0;
}
