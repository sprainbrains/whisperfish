#include "model/prompt.hpp"

void Prompt::password(const QString pw) {
    emit receivePassword(pw);
}
