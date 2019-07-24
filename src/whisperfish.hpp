#pragma once

#include <QString>

struct Version {
    int v1; int v2; int v3;
};

Version get_version();

struct Paths {
    QString data;
    QString config;
};

Paths get_paths();
