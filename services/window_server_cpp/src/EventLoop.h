/*
 * EventLoop Implementation
 * Reference: SerenityOS LibCore/EventLoop
 */

#ifndef EVENTLOOP_H
#define EVENTLOOP_H

#include <atomic>

namespace Core {

class EventLoop {
public:
    EventLoop();
    int exec();
    void quit(int code);

private:
    std::atomic<bool> m_running;
    int m_exit_code;
};

}

#endif
