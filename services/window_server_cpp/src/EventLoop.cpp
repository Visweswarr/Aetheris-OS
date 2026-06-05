#include "EventLoop.h"
#include <iostream>
#include <thread>
#include <chrono>

namespace Core {

EventLoop::EventLoop() : m_running(false), m_exit_code(0) {}

int EventLoop::exec() {
    m_running = true;
    std::cout << "[EventLoop] Entered main loop.\n";

    while (m_running) {
        // Mock processing events
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
        
        // In real impl: poll() or select() on scheme FDs
    }

    return m_exit_code;
}

void EventLoop::quit(int code) {
    m_exit_code = code;
    m_running = false;
}

}
