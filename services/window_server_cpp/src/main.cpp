/*
 * WindowServer Main
 * Reference: SerenityOS WindowServer
 */

#include "EventLoop.h"
#include <iostream>

int main() {
    std::cout << "[WindowServer] Starting C++ Window Server (Serenity Style)...\n";

    // Initialize Event Loop
    Core::EventLoop event_loop;

    // TODO: Connect to 'display:' scheme via kernel IPC
    std::cout << "[WindowServer] Connecting to 'display:' scheme...\n";

    return event_loop.exec();
}
