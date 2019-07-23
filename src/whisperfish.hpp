#pragma once

#include <QString>

struct Version {
    int v1; int v2; int v3;
};

static const Version get_version();

struct Paths {
    QString data;
    QString config;
};

static const Paths get_paths();
