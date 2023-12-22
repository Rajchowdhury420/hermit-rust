#include <windows.h>

int main() {
    MessageBox(
        NULL,
        (LPCSTR)"This is test box.",
        (LPCSTR)"Hello;)",
        MB_ICONWARNING | MB_CANCELTRYCONTINUE | MB_DEFBUTTON2
    );
}