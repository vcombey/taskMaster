#include <signal.h>

void    handle(int signum) {
    printf("signum: %d\n", signum);
}

int main() {
    for (int i = 0; i < 30; i++) {
        signal(i, handle);
    }
    while (42) {
        ;
    }
}
